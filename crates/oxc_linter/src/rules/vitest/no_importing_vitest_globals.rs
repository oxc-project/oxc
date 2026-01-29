use itertools::Itertools;
use oxc_ast::{
    AstKind,
    ast::{
        Argument, BindingPattern, Expression, ImportDeclarationSpecifier, ImportOrExportKind,
        VariableDeclarationKind, VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

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

fn is_vitest_require_declaration(declaration: &VariableDeclarator<'_>) -> bool {
    let Some(Expression::CallExpression(call_expr)) = &declaration.init else {
        return false;
    };

    if !call_expr.is_require_call() {
        return false;
    }

    let Some(Argument::StringLiteral(require_import)) = call_expr.arguments.first() else {
        return false;
    };

    if require_import.value.as_str() != "vitest" {
        return false;
    }

    if declaration.id.is_binding_identifier() {
        return false;
    }

    true
}

impl Rule for NoImportingVitestGlobals {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let mut spans: Vec<Span> = vec![];
        let mut new_imports: Vec<String> = vec![];
        let mut span_start_specifiers: u32 = u32::MAX;
        let mut span_end_specifiers: u32 = u32::MIN;

        match node.kind() {
            AstKind::VariableDeclaration(variable_declarations) => {
                let mut new_declarations: Vec<DeclarationRenderType> = vec![];
                for declaration in &variable_declarations.declarations {
                    if !is_vitest_require_declaration(declaration) {
                        new_declarations.push(DeclarationRenderType::NoVitest(declaration.span));
                        continue;
                    }

                    let BindingPattern::ObjectPattern(obj) = &declaration.id else {
                        new_declarations.push(DeclarationRenderType::NoVitest(declaration.span));
                        continue;
                    };

                    if obj.rest.is_some() {
                        new_declarations.push(DeclarationRenderType::NoVitest(declaration.span));
                        continue;
                    }

                    if obj
                        .properties
                        .iter()
                        .any(|property| property.key.is_specific_static_name("default"))
                    {
                        new_declarations.push(DeclarationRenderType::NoVitest(declaration.span));

                        continue;
                    }

                    let mut global_vitest_spans: Vec<Span> = vec![];
                    let mut non_global_vitest_import: Vec<String> = vec![];

                    for property in &obj.properties {
                        let Some(property_name) = property.key.static_name() else {
                            continue;
                        };

                        if VITEST_GLOBALS.contains(&property_name.as_ref()) {
                            global_vitest_spans.push(property.span);
                        } else {
                            non_global_vitest_import
                                .push(ctx.source_range(property.span).to_string());
                        }
                    }

                    let remove_declaration = non_global_vitest_import.len() == 0;

                    new_declarations.push(DeclarationRenderType::Vitest(VitestRequire {
                        remove_fully: remove_declaration,
                        global_vitest_spans,
                        non_global_vitest_import,
                    }));
                }

                let Some(DeclarationRenderType::Vitest(vitest_require)) = &new_declarations
                    .iter()
                    .find(|value| matches!(value, DeclarationRenderType::Vitest(_)))
                else {
                    return;
                };

                ctx.diagnostic_with_fix(
                    no_importing_vitest_globals_diagnostic(&vitest_require.global_vitest_spans),
                    |fixer| {
                        let variable_modifier = match variable_declarations.kind {
                            VariableDeclarationKind::Const => "const",
                            VariableDeclarationKind::Let => "let",
                            VariableDeclarationKind::Var => "var",
                            _ => return fixer.noop(),
                        };

                        let declarations = new_declarations
                            .iter()
                            .filter_map(|declaration| match declaration {
                                DeclarationRenderType::NoVitest(span) => {
                                    let source_declaration = ctx.source_range(*span);
                                    Some(source_declaration.to_string())
                                }
                                DeclarationRenderType::Vitest(vitest_require) => {
                                    if vitest_require.remove_fully {
                                        return None;
                                    }

                                    let new_vitest_declaration = format!(
                                        "{{ {} }} = require('vitest')",
                                        vitest_require.non_global_vitest_import.join(", ")
                                    );
                                    Some(new_vitest_declaration)
                                }
                            })
                            .join(", ");

                        if declarations.is_empty() {
                            return fixer.delete(node);
                        }

                        let new_declaration = format!("{variable_modifier} {declarations};");

                        fixer.replace(variable_declarations.span, new_declaration)
                    },
                );
            }
            AstKind::ImportDeclaration(import_decl) => {
                if import_decl.source.value.as_str() != "vitest" {
                    return;
                }

                let Some(specifiers) = &import_decl.specifiers else {
                    return;
                };

                for import_specifier in specifiers {
                    match import_specifier {
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(_)
                        | ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                            continue;
                        }
                        ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                            span_start_specifiers = span_start_specifiers.min(specifier.span.start);
                            span_end_specifiers = span_end_specifiers.max(specifier.span.end);

                            if specifier.import_kind == ImportOrExportKind::Type {
                                new_imports.push(ctx.source_range(specifier.span).to_string());
                                continue;
                            }

                            if !specifier.imported.is_identifier() {
                                continue;
                            }

                            if VITEST_GLOBALS.contains(&specifier.local.name.as_ref()) {
                                spans.push(specifier.span);
                            } else {
                                new_imports.push(ctx.source_range(specifier.span).to_string());
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
            _ => return,
        }
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

#[derive(Debug, PartialEq, Eq)]
struct VitestRequire {
    remove_fully: bool,
    global_vitest_spans: Vec<Span>,
    non_global_vitest_import: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
enum DeclarationRenderType {
    NoVitest(Span),
    Vitest(VitestRequire),
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
