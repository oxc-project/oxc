use std::fmt::format;

use itertools::Itertools;
use oxc_ast::{
    AstKind,
    ast::{
        Argument, BindingPattern, Expression, ImportDeclarationSpecifier, ImportOrExportKind,
        VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNodes;
use oxc_span::Span;
use oxc_syntax::module_record::RequestedModule;

use crate::{
    AstNode,
    ast_util::is_global_require_call,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    rules::ImportNoDefaultExport,
};

fn no_importing_vitest_globals_diagnostic(spans: &[Span]) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong.")
        .with_help("Should be a command-like statement that tells the user how to fix the issue.")
        .with_labels(spans.iter().map(|span| span.label("Remove this global vitest import")))
}

#[derive(Debug, Default, Clone)]
pub struct NoImportingVitestGlobals;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoImportingVitestGlobals,
    vitest,
    style,
    fix,
);

impl Rule for NoImportingVitestGlobals {
    fn run_once(&self, ctx: &LintContext) {
        NoImportingVitestGlobals::run(ctx);
    }
}

const VITEST_GLOBALS: [&str; 17] = [
    "suite",
    "test",
    "chai",
    "describe",
    "it",
    "expectTypeOf",
    "assertType",
    "expect",
    "assert",
    "vitest",
    "vi",
    "beforeAll",
    "afterAll",
    "beforeEach",
    "afterEach",
    "onTestFailed",
    "onTestFinished",
];

impl NoImportingVitestGlobals {
    fn run(ctx: &LintContext) {
        for node in ctx.nodes() {
            let mut spans: Vec<Span> = vec![];
            let mut new_imports: Vec<&str> = vec![];
            let mut span_start_specifiers: u32 = u32::MAX;
            let mut span_end_specifiers: u32 = u32::MIN;

            match node.kind() {
                AstKind::VariableDeclaration(variable_declarations) => {
                    let mut new_declarations: Vec<Span> = vec![];
                    for declaration in &variable_declarations.declarations {
                        let Some(Expression::CallExpression(call_expr)) = &declaration.init else {
                            let start_span = {
                                if new_declarations.len() == 0 {
                                    variable_declarations.span.start
                                } else {
                                    declaration.span.start
                                }
                            };

                            new_declarations.push(Span::new(start_span, declaration.span.end));
                            continue;
                        };

                        if !call_expr.is_require_call() {
                            let start_span = {
                                if new_declarations.len() == 0 {
                                    variable_declarations.span.start
                                } else {
                                    declaration.span.start
                                }
                            };

                            new_declarations.push(Span::new(start_span, declaration.span.end));

                            continue;
                        }

                        let Some(Argument::StringLiteral(require_import)) =
                            call_expr.arguments.first()
                        else {
                            let start_span = {
                                if new_declarations.len() == 0 {
                                    variable_declarations.span.start
                                } else {
                                    declaration.span.start
                                }
                            };

                            new_declarations.push(Span::new(start_span, declaration.span.end));
                            continue;
                        };

                        if require_import.value.as_str() != "vitest" {
                            let start_span = {
                                if new_declarations.len() == 0 {
                                    variable_declarations.span.start
                                } else {
                                    declaration.span.start
                                }
                            };

                            new_declarations.push(Span::new(start_span, declaration.span.end));
                            continue;
                        }

                        if declaration.id.is_binding_identifier() {
                            let start_span = {
                                if new_declarations.len() == 0 {
                                    variable_declarations.span.start
                                } else {
                                    declaration.span.start
                                }
                            };

                            new_declarations.push(Span::new(start_span, declaration.span.end));
                            continue;
                        }

                        match &declaration.id {
                            BindingPattern::ObjectPattern(obj) => {
                                if obj.rest.is_some() {
                                    let start_span = {
                                        if new_declarations.len() == 0 {
                                            variable_declarations.span.start
                                        } else {
                                            declaration.span.start
                                        }
                                    };

                                    new_declarations
                                        .push(Span::new(start_span, declaration.span.end));
                                    continue;
                                }

                                if obj
                                    .properties
                                    .iter()
                                    .any(|property| property.key.is_specific_static_name("default"))
                                {
                                    let start_span = {
                                        if new_declarations.len() == 0 {
                                            variable_declarations.span.start
                                        } else {
                                            declaration.span.start
                                        }
                                    };

                                    new_declarations
                                        .push(Span::new(start_span, declaration.span.end));
                                    continue;
                                }

                                for property in &obj.properties {
                                    span_start_specifiers =
                                        span_start_specifiers.min(property.span.start);
                                    span_end_specifiers =
                                        span_end_specifiers.max(property.span.end);

                                    let Some(property_name) = property.key.static_name() else {
                                        continue;
                                    };

                                    if VITEST_GLOBALS.contains(&property_name.as_ref()) {
                                        spans.push(property.span);
                                    } else {
                                        new_imports.push(ctx.source_range(property.span));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if spans.len() > 0 {
                        ctx.diagnostic_with_fix(
                            no_importing_vitest_globals_diagnostic(spans.as_ref()),
                            |fixer| {
                                if new_imports.len() == 0 {
                                    let suffix_string = {
                                        if let Some(last_span) = new_declarations.last() {
                                            if last_span.end == variable_declarations.span.end {
                                                ""
                                            } else {
                                                ctx.source_range(Span::new(
                                                    variable_declarations.span.end - 1,
                                                    variable_declarations.span.end,
                                                ))
                                            }
                                        } else {
                                            ""
                                        }
                                    };

                                    let new_declaration_string = new_declarations
                                        .iter()
                                        .map(|span_declaration| ctx.source_range(*span_declaration))
                                        .join(", ");

                                    return fixer.replace(
                                        variable_declarations.span,
                                        format!("{new_declaration_string}{suffix_string}"),
                                    );
                                }

                                let import_text = new_imports.join(", ");

                                let specifiers_span =
                                    Span::new(span_start_specifiers, span_end_specifiers);

                                fixer.replace(specifiers_span, import_text)
                            },
                        );
                    }
                }
                AstKind::ImportDeclaration(import_decl) => {
                    if import_decl.source.value.as_str() != "vitest" {
                        continue;
                    }

                    let Some(specifiers) = &import_decl.specifiers else {
                        continue;
                    };

                    for import_specifier in specifiers {
                        match import_specifier {
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(_)
                            | ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                                continue;
                            }
                            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                                span_start_specifiers =
                                    span_start_specifiers.min(specifier.span.start);
                                span_end_specifiers = span_end_specifiers.max(specifier.span.end);

                                if specifier.import_kind == ImportOrExportKind::Type {
                                    new_imports.push(ctx.source_range(specifier.span));
                                    continue;
                                }

                                if !specifier.imported.is_identifier() {
                                    continue;
                                }

                                if VITEST_GLOBALS.contains(&specifier.local.name.as_ref()) {
                                    spans.push(specifier.span);
                                } else {
                                    new_imports.push(ctx.source_range(specifier.span));
                                }
                            }
                        }
                    }

                    if spans.len() > 0 {
                        ctx.diagnostic_with_fix(
                            no_importing_vitest_globals_diagnostic(spans.as_ref()),
                            |fixer| {
                                if spans.len() == specifiers.len() {
                                    return fixer.delete(node);
                                }

                                let import_text = new_imports.join(", ");

                                let specifiers_span =
                                    Span::new(span_start_specifiers, span_end_specifiers);

                                fixer.replace(specifiers_span, import_text)
                            },
                        );
                    }
                }
                _ => continue,
            }
        }
    }
}

/*
 * TODO
 * Test: multiples require
 */

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "import { describe } from 'jest';",
        "import vitest from 'vitest';",
        "import * as vitest from 'vitest';",
        r#"import { "default" as vitest } from 'vitest';"#,
        "import { BenchFactory } from 'vitest';",
        "import type { TestArtifactBase, TestAttachment } from 'vitest'",
        "let x;",
        "let x = 1;",
        "const x = console.log('hello');",
        "const x = print('hello');",
        "const x = require('something', 'wrong');",
        "const x = require(a_variable);",
        "const x = require('jest');",
        "const x = require('vitest');",
        "const { ...rest } = require('vitest');",
        r#"const { "default": vitest } = require('vitest');"#,
    ];

    let fail = vec![
        "import { describe } from 'vitest';",
        "import { describe, it } from 'vitest';",
        "import { describe, BenchFactory } from 'vitest';",
        "import { BenchFactory, describe } from 'vitest';",
        "import { describe, BenchFactory, it } from 'vitest';",
        "import { BenchTask, describe, BenchFactory, it } from 'vitest';",
        "import type { TestArtifactBase, TestAttachment } from 'vitest'
         import { it, describe } from 'vitest'",
        "import { type TestArtifactBase, BenchTask, describe, type TestAttachment, BenchFactory, it } from 'vitest';",
        "const x = 1, { describe } = require('vitest');",
        "const x = 1, { describe } = require('vitest'), y = 2;",
        "const { describe, it } = require('vitest');",
        "const { describe, BenchFactory } = require('vitest');",
        "const { BenchFactory, describe } = require('vitest');",
        "const { describe, BenchFactory, it } = require('vitest');",
        "const { BenchTask, describe, BenchFactory, it } = require('vitest');",
    ];

    let fix = vec![
        ("import { describe } from 'vitest';", "", None),
        ("import { describe, it } from 'vitest';", "", None),
        (
            "import { describe, BenchFactory } from 'vitest';",
            "import { BenchFactory } from 'vitest';",
            None,
        ),
        (
            "import { BenchFactory, describe } from 'vitest';",
            "import { BenchFactory } from 'vitest';",
            None,
        ),
        (
            "import { describe, BenchFactory, it } from 'vitest';",
            "import { BenchFactory } from 'vitest';",
            None,
        ),
        (
            "import { BenchTask, describe, BenchFactory, it } from 'vitest';",
            "import { BenchTask, BenchFactory } from 'vitest';",
            None,
        ),
        (
            "import type { TestArtifactBase, TestAttachment } from 'vitest'
import { it, describe } from 'vitest'",
            "import type { TestArtifactBase, TestAttachment } from 'vitest'
",
            None,
        ),
        (
            "import { type TestArtifactBase, BenchTask, describe, type TestAttachment, BenchFactory, it } from 'vitest';",
            "import { type TestArtifactBase, BenchTask, type TestAttachment, BenchFactory } from 'vitest';",
            None,
        ),
        ("const { describe } = require('vitest');", "", None),
        ("const { describe } = require('vitest'), x = 1;", "const x = 1;", None),
        ("const x = 1, { describe } = require('vitest');", "const x = 1;", None),
        ("const x = 1, { describe } = require('vitest'), y = 2;", "const x = 1, y = 2;", None),
        ("const { describe, it } = require('vitest');", "", None),
        (
            "const { describe, BenchFactory } = require('vitest');",
            "const { BenchFactory } = require('vitest');",
            None,
        ),
        (
            "const { BenchFactory, describe } = require('vitest');",
            "const { BenchFactory } = require('vitest');",
            None,
        ),
        (
            "const { describe, BenchFactory, it } = require('vitest');",
            "const { BenchFactory } = require('vitest');",
            None,
        ),
        (
            "const { BenchTask, describe, BenchFactory, it } = require('vitest');",
            "const { BenchTask, BenchFactory } = require('vitest');",
            None,
        ),
    ];
    Tester::new(NoImportingVitestGlobals::NAME, NoImportingVitestGlobals::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
