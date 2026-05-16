use itertools::Itertools;
use oxc_ast::{
    AstKind,
    ast::{Argument, BindingPattern, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    module_record::ImportImportName,
    rule::{DefaultRuleConfig, Rule},
    utils::{
        JestFnKind, JestGeneralFnKind, ParsedJestFnCallNew, collect_possible_jest_call_node,
        parse_jest_fn_call,
    },
};

fn prefer_importing_jest_globals_diagnostic(span: Span, globals: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Import the following Jest functions from `@jest/globals`: {globals}"
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PreferImportingJestGlobals(Box<PreferImportingJestGlobalsConfig>);

impl std::ops::Deref for PreferImportingJestGlobals {
    type Target = PreferImportingJestGlobalsConfig;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct PreferImportingJestGlobalsConfig {
    /// Jest function types to enforce importing for.
    types: Vec<JestFnType>,
}

impl Default for PreferImportingJestGlobalsConfig {
    fn default() -> Self {
        Self {
            types: vec![
                JestFnType::Hook,
                JestFnType::Describe,
                JestFnType::Test,
                JestFnType::Expect,
                JestFnType::Jest,
                JestFnType::Unknown,
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum JestFnType {
    Hook,
    Describe,
    Test,
    Expect,
    Jest,
    Unknown,
}

impl JestFnType {
    fn matches(self, kind: JestFnKind) -> bool {
        matches!(
            (self, kind),
            (Self::Hook, JestFnKind::General(JestGeneralFnKind::Hook))
                | (Self::Describe, JestFnKind::General(JestGeneralFnKind::Describe))
                | (Self::Test, JestFnKind::General(JestGeneralFnKind::Test))
                | (Self::Expect, JestFnKind::Expect | JestFnKind::ExpectTypeOf)
                | (
                    Self::Jest,
                    JestFnKind::General(JestGeneralFnKind::Jest | JestGeneralFnKind::Vitest)
                )
                | (Self::Unknown, JestFnKind::Unknown)
        )
    }
}

const IMPORT_SOURCE: &str = "@jest/globals";

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer importing Jest globals (`describe`, `test`, `expect`, etc.) from
    /// `@jest/globals` rather than relying on ambient globals.
    ///
    /// ### Why is this bad?
    ///
    /// Using global Jest functions without explicit imports makes dependencies
    /// implicit and can cause issues with type checking, editor tooling, and
    /// when migrating between test runners.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// describe("suite", () => {
    ///   test("foo");
    ///   expect(true).toBeDefined();
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import { describe, expect, test } from '@jest/globals';
    /// describe("suite", () => {
    ///   test("foo");
    ///   expect(true).toBeDefined();
    /// });
    /// ```
    PreferImportingJestGlobals,
    jest,
    style,
    fix,
    config = PreferImportingJestGlobalsConfig,
    version = "1.60.0"
);

impl Rule for PreferImportingJestGlobals {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        let mut functions_to_import: FxHashSet<String> = FxHashSet::default();
        let mut reporting_span: Option<Span> = None;

        for jest_node in &collect_possible_jest_call_node(ctx) {
            if jest_node.original.is_some() {
                continue;
            }

            let AstKind::CallExpression(call_expr) = jest_node.node.kind() else {
                continue;
            };

            let Some(jest_fn_call) = parse_jest_fn_call(call_expr, jest_node, ctx) else {
                continue;
            };

            if !self.types.iter().any(|t| t.matches(jest_fn_call.kind())) {
                continue;
            }

            let name = match &jest_fn_call {
                // Fixture is from vitest
                ParsedJestFnCallNew::GeneralJest(c) | ParsedJestFnCallNew::Fixture(c) => {
                    c.name.to_string()
                }
                ParsedJestFnCallNew::Expect(c) | ParsedJestFnCallNew::ExpectTypeOf(c) => {
                    c.name.to_string()
                }
            };
            functions_to_import.insert(name);
            reporting_span.get_or_insert(call_expr.callee.span());
        }

        let Some(span) = reporting_span else { return };

        ctx.diagnostic_with_fix(
            prefer_importing_jest_globals_diagnostic(
                span,
                &functions_to_import.iter().sorted().join(", "),
            ),
            |fixer| build_fix(ctx, &fixer, &mut functions_to_import),
        );
    }
}

fn build_fix<'a>(
    ctx: &LintContext<'a>,
    fixer: &RuleFixer<'_, 'a>,
    functions_to_import: &mut FxHashSet<String>,
) -> RuleFix {
    let program = ctx.nodes().program();
    let is_module = ctx.source_type().is_module();

    // 1. Merge with existing `import ... from '@jest/globals'`
    if let Some(fix) = try_merge_esm_import(ctx, fixer, functions_to_import) {
        return fix;
    }

    // 2. Merge with existing `const { ... } = require('@jest/globals')`
    if let Some(fix) = try_merge_cjs_require(ctx, fixer, functions_to_import) {
        return fix;
    }

    // 3. Create a new import/require
    let text = create_import_text(is_module, functions_to_import);

    if let Some(directive) = program.directives.last() {
        return fixer.insert_text_after_range(directive.span, format!("\n{text}"));
    }
    if let Some(hashbang) = &program.hashbang {
        return fixer.insert_text_after_range(hashbang.span, format!("\n{text}"));
    }
    fixer.insert_text_before_range(Span::empty(0), format!("{text}\n"))
}

fn create_import_text(is_module: bool, functions: &FxHashSet<String>) -> String {
    let sorted = functions.iter().sorted().join(", ");
    if is_module {
        format!("import {{ {sorted} }} from '{IMPORT_SOURCE}';")
    } else {
        format!("const {{ {sorted} }} = require('{IMPORT_SOURCE}');")
    }
}

/// Merge with existing `import ... from '@jest/globals'` and replace it entirely.
/// Uses `module_record` to find import entries and their statement span.
fn try_merge_esm_import<'a>(
    ctx: &LintContext<'a>,
    fixer: &RuleFixer<'_, 'a>,
    functions_to_import: &mut FxHashSet<String>,
) -> Option<RuleFix> {
    let module_record = ctx.module_record();

    // Find the first `@jest/globals` import statement span
    let first_span = module_record
        .import_entries
        .iter()
        .find(|e| e.module_request.name() == IMPORT_SOURCE && !e.is_type)?
        .statement_span;

    // Merge only entries belonging to that same import statement
    for entry in &module_record.import_entries {
        if entry.statement_span != first_span || entry.is_type {
            continue;
        }

        match &entry.import_name {
            ImportImportName::Name(name_span) => {
                let imported = ctx.source_range(name_span.span);
                let local = entry.local_name.name.as_str();
                if imported == local {
                    functions_to_import.insert(local.to_string());
                } else {
                    functions_to_import.insert(format!("{imported} as {local}"));
                }
            }
            ImportImportName::Default(_) => {
                functions_to_import.insert(entry.local_name.name.to_string());
            }
            ImportImportName::NamespaceObject => {}
        }
    }

    Some(fixer.replace(first_span, create_import_text(true, functions_to_import)))
}

/// Merge with existing `const { ... } = require('@jest/globals')` and replace it entirely.
/// Uses semantic analysis to find `require` references instead of iterating the AST.
fn try_merge_cjs_require<'a>(
    ctx: &LintContext<'a>,
    fixer: &RuleFixer<'_, 'a>,
    functions_to_import: &mut FxHashSet<String>,
) -> Option<RuleFix> {
    let is_module = ctx.source_type().is_module();
    let alias_sep = if is_module { " as " } else { ": " };

    let require_refs = ctx.scoping().root_unresolved_references().get("require")?;

    for &ref_id in require_refs {
        let reference = ctx.scoping().get_reference(ref_id);
        let call_node = ctx.nodes().parent_node(reference.node_id());
        let AstKind::CallExpression(call) = call_node.kind() else { continue };

        let is_jest_require =
            call.arguments.len() == 1 && is_string_arg_matching(&call.arguments[0], IMPORT_SOURCE);

        if !is_jest_require {
            continue;
        }

        // Walk up to the VariableDeclarator and VariableDeclaration
        let Some(var_declarator_node) = ctx
            .nodes()
            .ancestors(call_node.id())
            .find(|n| matches!(n.kind(), AstKind::VariableDeclarator(_)))
        else {
            continue;
        };
        let AstKind::VariableDeclarator(declarator) = var_declarator_node.kind() else {
            continue;
        };

        // Merge existing destructured properties
        if let BindingPattern::ObjectPattern(pattern) = &declarator.id {
            for prop in &pattern.properties {
                if prop.computed {
                    continue;
                }
                let Some(key_name) = prop.key.static_name() else { continue };
                let BindingPattern::BindingIdentifier(value_ident) = &prop.value else {
                    continue;
                };
                let value_name = value_ident.name.as_str();

                if key_name == value_name {
                    functions_to_import.insert(key_name.to_string());
                } else {
                    functions_to_import.insert(format!("{key_name}{alias_sep}{value_name}"));
                }
            }
        }

        // VariableDeclaration is the direct parent of VariableDeclarator
        let var_decl_node = ctx.nodes().parent_node(var_declarator_node.id());

        return Some(
            fixer.replace(var_decl_node.span(), create_import_text(is_module, functions_to_import)),
        );
    }

    None
}

fn is_string_arg_matching(arg: &Argument, value: &str) -> bool {
    arg.as_expression().is_some_and(|expr| match expr {
        Expression::StringLiteral(lit) => lit.value == value,
        Expression::TemplateLiteral(tpl) => {
            tpl.quasis.len() == 1
                && tpl.expressions.is_empty()
                && tpl.quasis.first().is_some_and(|q| q.value.raw == value)
        }
        _ => false,
    })
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            "// with import
            import { test, expect } from '@jest/globals';
            test('should pass', () => {
                expect(true).toBeDefined();
            });",
            None,
        ),
        (
            "// with import
            import { 'test' as test, expect } from '@jest/globals';
            test('should pass', () => {
                expect(true).toBeDefined();
            });",
            None,
        ),
        (
            "test('should pass', () => {
                expect(true).toBeDefined();
            });",
            Some(serde_json::json!([{ "types": ["jest"] }])),
        ),
        (
            "const { it } = require('@jest/globals');
            it('should pass', () => {
                expect(true).toBeDefined();
            });",
            Some(serde_json::json!([{ "types": ["test"] }])),
        ),
        (
            "// with require
            const { test, expect } = require('@jest/globals');
            test('should pass', () => {
                expect(true).toBeDefined();
            });",
            None,
        ),
        (
            r"const { test, expect } = require(`@jest/globals`);
            test('should pass', () => {
                expect(true).toBeDefined();
            });",
            None,
        ),
        (
            r#"import { it as itChecks } from '@jest/globals';
            itChecks("foo");"#,
            None,
        ),
        (
            r#"import { 'it' as itChecks } from '@jest/globals';
            itChecks("foo");"#,
            None,
        ),
        (
            r#"const { test } = require('@jest/globals');
            test("foo");"#,
            None,
        ),
        (
            r#"const { test } = require('my-test-library');
            test("foo");"#,
            None,
        ),
    ];

    let fail = vec![
        (
            r#"import describe from '@jest/globals';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"import { describe as context } from '@jest/globals';
            context("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"import { describe as context } from '@jest/globals';
            describe("something", () => {
              context("suite", () => {
                test("foo");
                expect(true).toBeDefined();
              })
            })"#,
            None,
        ),
        (
            r#"import { 'describe' as describe } from '@jest/globals';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"import { 'describe' as context } from '@jest/globals';
            context("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"jest.useFakeTimers();
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            Some(serde_json::json!([{ "types": ["jest"] }])),
        ),
        (
            r#"import React from 'react';
            import { yourFunction } from './yourFile';
            import something from "something";
            import { test } from '@jest/globals';
            import { xit } from '@jest/globals';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"console.log('hello');
            import * as fs from 'fs';
            const { test, 'describe': describe } = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"console.log('hello');
            import jestGlobals from '@jest/globals';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"import { pending } from 'actions';
            describe('foo', () => {
              test.each(['hello', 'world'])("%s", (a) => {});
            });"#,
            None,
        ),
        (
            r#"const {describe} = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"const {describe: context} = require('@jest/globals');
            context("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"const {describe: context} = require('@jest/globals');
            describe("something", () => {
              context("suite", () => {
                test("foo");
                expect(true).toBeDefined();
              })
            })"#,
            None,
        ),
        (
            r#"const {describe: []} = require('@jest/globals');
            describe("something", () => {
              context("suite", () => {
                test("foo");
                expect(true).toBeDefined();
              })
            })"#,
            None,
        ),
        (
            r#"const {describe} = require(`@jest/globals`);
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"const source = 'globals';
            const {describe} = require(`@jest/${source}`);
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"const { [() => {}]: it } = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"console.log('hello');
            const fs = require('fs');
            const { test, 'describe': describe } = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"console.log('hello');
            const jestGlobals = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"const { pending } = require('actions');
            describe('foo', () => {
              test.each(['hello', 'world'])("%s", (a) => {});
            });"#,
            None,
        ),
        (
            r#"describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"#!/usr/bin/env node
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"// with comment above
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"'use strict';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"`use strict`;
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"console.log('hello');
            const onClick = jest.fn();
            describe("suite", () => {
              test("foo");
              expect(onClick).toHaveBeenCalled();
            })"#,
            None,
        ),
        (
            r#"console.log('hello');
            const onClick = jest.fn();
            describe("suite", () => {
              test("foo");
              expect(onClick).toHaveBeenCalled();
            })"#,
            None,
        ),
        (
            r#"import describe from '@jest/globals';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
        (
            r#"const {describe} = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
        ),
    ];

    let fix = vec![
        (
            r#"import describe from '@jest/globals';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            r#"import { describe, expect, test } from '@jest/globals';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
            None,
        ),
        (
            r#"import { describe as context } from '@jest/globals';
            context("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            r#"import { describe as context, expect, test } from '@jest/globals';
            context("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
            None,
        ),
        (
            r#"import { describe as context } from '@jest/globals';
            describe("something", () => {
              context("suite", () => {
                test("foo");
                expect(true).toBeDefined();
              })
            })"#,
            r#"import { describe, describe as context, expect, test } from '@jest/globals';
            describe("something", () => {
              context("suite", () => {
                test("foo");
                expect(true).toBeDefined();
              })
            })"#,
            None,
            None,
        ),
        (
            r#"import { 'describe' as describe } from '@jest/globals';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            r#"import { 'describe' as describe, expect, test } from '@jest/globals';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
            None,
        ),
        (
            r#"import { 'describe' as context } from '@jest/globals';
            context("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            r#"import { 'describe' as context, expect, test } from '@jest/globals';
            context("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
            None,
        ),
        (
            r#"jest.useFakeTimers();
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            "import { jest } from '@jest/globals';\njest.useFakeTimers();\n            describe(\"suite\", () => {\n              test(\"foo\");\n              expect(true).toBeDefined();\n            })",
            Some(serde_json::json!([{ "types": ["jest"] }])),
            Some(PathBuf::from("test.mjs")),
        ),
        (
            r#"import React from 'react';
            import { yourFunction } from './yourFile';
            import something from "something";
            import { test } from '@jest/globals';
            import { xit } from '@jest/globals';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            r#"import React from 'react';
            import { yourFunction } from './yourFile';
            import something from "something";
            import { describe, expect, test } from '@jest/globals';
            import { xit } from '@jest/globals';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
            None,
        ),
        (
            r#"console.log('hello');
            import * as fs from 'fs';
            const { test, 'describe': describe } = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            r#"console.log('hello');
            import * as fs from 'fs';
            import { describe, expect, test } from '@jest/globals';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
            None,
        ),
        (
            r#"console.log('hello');
            import jestGlobals from '@jest/globals';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            r#"console.log('hello');
            import { describe, expect, jestGlobals, test } from '@jest/globals';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
            None,
        ),
        (
            r#"import { pending } from 'actions';
            describe('foo', () => {
              test.each(['hello', 'world'])("%s", (a) => {});
            });"#,
            "import { describe, test } from '@jest/globals';\nimport { pending } from 'actions';\n            describe('foo', () => {\n              test.each(['hello', 'world'])(\"%s\", (a) => {});\n            });",
            None,
            None,
        ),
        (
            r#"const {describe} = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            r#"const { describe, expect, test } = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
            None,
        ),
        (
            r#"const {describe: context} = require('@jest/globals');
            context("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            r#"const { describe: context, expect, test } = require('@jest/globals');
            context("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
            None,
        ),
        (
            r#"const {describe: context} = require('@jest/globals');
            describe("something", () => {
              context("suite", () => {
                test("foo");
                expect(true).toBeDefined();
              })
            })"#,
            r#"const { describe, describe: context, expect, test } = require('@jest/globals');
            describe("something", () => {
              context("suite", () => {
                test("foo");
                expect(true).toBeDefined();
              })
            })"#,
            None,
            None,
        ),
        (
            r#"const {describe: []} = require('@jest/globals');
            describe("something", () => {
              context("suite", () => {
                test("foo");
                expect(true).toBeDefined();
              })
            })"#,
            r#"const { describe, expect, test } = require('@jest/globals');
            describe("something", () => {
              context("suite", () => {
                test("foo");
                expect(true).toBeDefined();
              })
            })"#,
            None,
            None,
        ),
        (
            r#"const {describe} = require(`@jest/globals`);
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            r#"const { describe, expect, test } = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
            None,
        ),
        (
            r#"const source = 'globals';
            const {describe} = require(`@jest/${source}`);
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            "const { expect, test } = require('@jest/globals');\nconst source = 'globals';\n            const {describe} = require(`@jest/${source}`);\n            describe(\"suite\", () => {\n              test(\"foo\");\n              expect(true).toBeDefined();\n            })",
            None,
            None,
        ),
        (
            r#"const { [() => {}]: it } = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            r#"const { describe, expect, test } = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
            None,
        ),
        (
            r#"console.log('hello');
            const fs = require('fs');
            const { test, 'describe': describe } = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            r#"console.log('hello');
            const fs = require('fs');
            const { describe, expect, test } = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
            None,
        ),
        (
            r#"console.log('hello');
            const jestGlobals = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            r#"console.log('hello');
            const { describe, expect, test } = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
            None,
        ),
        (
            r#"const { pending } = require('actions');
            describe('foo', () => {
              test.each(['hello', 'world'])("%s", (a) => {});
            });"#,
            "const { describe, test } = require('@jest/globals');\nconst { pending } = require('actions');\n            describe('foo', () => {\n              test.each(['hello', 'world'])(\"%s\", (a) => {});\n            });",
            None,
            None,
        ),
        (
            r#"describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            "const { describe, expect, test } = require('@jest/globals');\ndescribe(\"suite\", () => {\n              test(\"foo\");\n              expect(true).toBeDefined();\n            })",
            None,
            None,
        ),
        (
            r#"#!/usr/bin/env node
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            "#!/usr/bin/env node\nconst { describe, expect, test } = require('@jest/globals');\n            describe(\"suite\", () => {\n              test(\"foo\");\n              expect(true).toBeDefined();\n            })",
            None,
            None,
        ),
        (
            r#"// with comment above
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            "const { describe, expect, test } = require('@jest/globals');\n// with comment above\n            describe(\"suite\", () => {\n              test(\"foo\");\n              expect(true).toBeDefined();\n            })",
            None,
            None,
        ),
        (
            r#"'use strict';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            "'use strict';\nconst { describe, expect, test } = require('@jest/globals');\n            describe(\"suite\", () => {\n              test(\"foo\");\n              expect(true).toBeDefined();\n            })",
            None,
            None,
        ),
        (
            r#"`use strict`;
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            "const { describe, expect, test } = require('@jest/globals');\n`use strict`;\n            describe(\"suite\", () => {\n              test(\"foo\");\n              expect(true).toBeDefined();\n            })",
            None,
            None,
        ),
        (
            r#"console.log('hello');
            const onClick = jest.fn();
            describe("suite", () => {
              test("foo");
              expect(onClick).toHaveBeenCalled();
            })"#,
            "const { describe, expect, jest, test } = require('@jest/globals');\nconsole.log('hello');\n            const onClick = jest.fn();\n            describe(\"suite\", () => {\n              test(\"foo\");\n              expect(onClick).toHaveBeenCalled();\n            })",
            None,
            None,
        ),
        (
            r#"console.log('hello');
            const onClick = jest.fn();
            describe("suite", () => {
              test("foo");
              expect(onClick).toHaveBeenCalled();
            })"#,
            "import { describe, expect, jest, test } from '@jest/globals';\nconsole.log('hello');\n            const onClick = jest.fn();\n            describe(\"suite\", () => {\n              test(\"foo\");\n              expect(onClick).toHaveBeenCalled();\n            })",
            None,
            Some(PathBuf::from("test.mjs")),
        ),
        (
            r#"import describe from '@jest/globals';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            r#"import { describe, expect, test } from '@jest/globals';
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
            None,
        ),
        (
            r#"const {describe} = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            r#"const { describe, expect, test } = require('@jest/globals');
            describe("suite", () => {
              test("foo");
              expect(true).toBeDefined();
            })"#,
            None,
            None,
        ),
    ];

    Tester::new(PreferImportingJestGlobals::NAME, PreferImportingJestGlobals::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
