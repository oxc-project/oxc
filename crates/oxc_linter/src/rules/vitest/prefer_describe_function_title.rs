use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, parse_general_jest_fn_call},
};

fn prefer_describe_function_title_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Title description can not have the same content as a imported function name.")
        .with_help("Pass the function as a description title argument or modify the description title to not match any imported function name.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferDescribeFunctionTitle;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When testing a specific function, this rule aims to enforce passing a named function to `describe()`
    /// instead of an equivalent hardcoded string.
    ///
    /// ### Why is this bad?
    ///
    /// For tests that are related to a specific function, if the function being tested is renamed,
    /// the describe title will no longer match and can cause confusion in the future. Using the function
    /// directly ensures consistency even if the function is renamed.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // myFunction.test.js
    /// import { myFunction } from './myFunction'
    ///
    /// describe('myFunction', () => {
    ///   // ...
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// // myFunction.test.js
    /// import { myFunction } from './myFunction'
    ///
    /// describe(myFunction, () => {
    ///   // ...
    /// })
    /// ```
    PreferDescribeFunctionTitle,
    vitest,
    style,
    fix,
    version = "1.39.0",
    short_description = "When testing a specific function, this rule aims to enforce passing a named function to `describe()` instead of an equivalent hardcoded string.",
);

impl Rule for PreferDescribeFunctionTitle {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(_) = node.kind() else { return };
        Self::run(node, ctx);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.frameworks().is_test()
    }
}

impl PreferDescribeFunctionTitle {
    fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.arguments.len() < 2 {
            return;
        }

        let Some(possible_jest_node) = PossibleJestNode::from_call_expression(node, call_expr, ctx)
        else {
            return;
        };
        let Some(test_vitest_fn) = parse_general_jest_fn_call(call_expr, &possible_jest_node, ctx)
        else {
            return;
        };

        if test_vitest_fn.kind != JestFnKind::General(JestGeneralFnKind::Describe) {
            return;
        }

        let is_imported_name = |name: &str| {
            ctx.module_record()
                .import_entries
                .iter()
                .any(|entry| entry.local_name.name.as_ref() == name)
        };

        let is_value_imported_name = |name: &str| {
            ctx.module_record()
                .import_entries
                .iter()
                .any(|entry| !entry.is_type && entry.local_name.name.as_ref() == name)
        };

        let Some(title_arg) = call_expr.arguments.first() else {
            return;
        };

        match title_arg {
            Argument::StaticMemberExpression(title_expression) => {
                let Expression::Identifier(identifier) = &title_expression.object else {
                    return;
                };

                if title_expression.property.name != "name"
                    || !is_imported_name(identifier.name.as_ref())
                {
                    return;
                }

                if !is_value_imported_name(identifier.name.as_ref()) {
                    ctx.diagnostic(prefer_describe_function_title_diagnostic(
                        title_expression.span,
                    ));
                    return;
                }

                ctx.diagnostic_with_fix(
                    prefer_describe_function_title_diagnostic(title_expression.span),
                    |fixer| {
                        let variable = identifier.name.to_string();

                        fixer.replace(title_expression.span, variable)
                    },
                );
            }
            Argument::StringLiteral(string_title) => {
                let Some(title) = string_title.value.as_str() else {
                    return;
                };
                if !is_imported_name(title) {
                    return;
                }

                if ctx.settings().vitest.typecheck {
                    // TODO https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/src/rules/prefer-describe-function-title.ts#L85C9-L92C10
                    return;
                }

                if !is_value_imported_name(title) {
                    ctx.diagnostic(prefer_describe_function_title_diagnostic(string_title.span));
                    return;
                }

                ctx.diagnostic_with_fix(
                    prefer_describe_function_title_diagnostic(string_title.span),
                    |fixer| {
                        let span_without_quotes =
                            Span::new(string_title.span.start + 1, string_title.span.end - 1);

                        let variable = ctx.source_range(span_without_quotes).to_string();

                        fixer.replace(string_title.span, variable)
                    },
                );
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            r#"
			        import { myFunction } from "./myFunction"
			        describe()
			      "#,
            None,
            None,
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        (
            r#"
			        import { myFunction } from "./myFunction"
			        describe("myFunction")
			      "#,
            None,
            None,
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        (
            r#"
			        import { myFunction } from "./myFunction"
			        describe.todo("myFunction")
			      "#,
            None,
            None,
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        (
            r#"
			        import { myFunction } from "./myFunction"
			        describe.each("myFunction", () => {})
			      "#,
            None,
            None,
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        (
            r#"
			        import { myFunction } from "./myFunction"
			        describe(() => {})
			      "#,
            None,
            None,
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        (
            r#"
			        import { myFunction } from "./myFunction"
			        describe("other", () => {})
			      "#,
            None,
            None,
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        (
            r#"
			        import { myFunction } from "./myFunction"
			        it("myFunction", () => {})
			      "#,
            None,
            None,
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        (
            r#"
			        import { myFunction } from "./myFunction"
			        test("myFunction", () => {})
			      "#,
            None,
            None,
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        (
            r#"
			        import { other } from "./other.js"
			        describe("myFunction", () => {})
			      "#,
            None,
            None,
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        (
            r#"
				        import { myFunction } from "./myFunction.js"
				        describe(otherFunction.name, () => {})
				      "#,
            None,
            None,
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        (
            r#"
				        import { myFunction } from "./myFunction.js"
				        describe(myFunction.title, () => {})
				      "#,
            None,
            None,
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        (
            r#"
				        declare const myFunction: () => unknown
				        describe("myFunction", () => {})
				      "#,
            None,
            None,
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        (
            r#"
			        const myFunction = ""
			        describe("myFunction", () => {})
			      "#,
            None,
            Some(serde_json::json!({ "settings": {  "vitest": {  "typecheck": true,  },  } })),
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        (
            r#"
			        import { DocumentBuilder } from "./DocumentBuilder"
			        describe("Swagger Helper", () => {
			          beforeEach(() => {
			            vi.spyOn(DocumentBuilder.prototype, "setTitle")
			          })
			        })
			      "#,
            None,
            None,
            Some(PathBuf::from("swagger.helpers.spec.ts")),
        ),
        (
            r#"
			        import { myFunction } from "./myFunction"
			        describe("Test Suite", () => {
			          beforeEach(() => {
			            vi.spyOn(myFunction, "name")
			          })
			        })
			      "#,
            None,
            None,
            Some(PathBuf::from("myFunction.test.ts")),
        ),
    ];

    let fail = vec![
        (
            r#"
			        import { myFunction } from "./myFunction"
			        describe("myFunction", () => {})
			      "#,
            None,
            None,
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        (
            r#"
			        import { myFunction } from "./myFunction"
			        describe(myFunction.name, () => {})
			      "#,
            None,
            None,
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        (
            r#"
			        import { myFunction } from "./myFunction"
			        if (someProcessEnvCheck) {
			          describe("myFunction", () => {})
			        }
			      "#,
            None,
            None,
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        (
            r#"
			        import type { Button } from "./button"
			        describe("Button", () => {})
			      "#,
            None,
            None,
            Some(PathBuf::from("button.test.ts")),
        ),
        /*
        (
            r#"
			        import { myFunction } from "./myFunction"
			        describe("myFunction", () => {})
			      "#,
            None,
            Some(serde_json::json!({ "settings": {  "vitest": {  "typecheck": true,  },  } })),
            Some(PathBuf::from("myFunction.test.ts")),
        ),
        */
    ];

    let fix = vec![
        (
            r#"
			        import { myFunction } from "./myFunction"
			        describe("myFunction", () => {})
			      "#,
            r#"
			        import { myFunction } from "./myFunction"
			        describe(myFunction, () => {})
			      "#,
            None,
        ),
        (
            r#"
			        import { myFunction } from "./myFunction"
			        describe(myFunction.name, () => {})
			      "#,
            r#"
			        import { myFunction } from "./myFunction"
			        describe(myFunction, () => {})
			      "#,
            None,
        ),
        (
            r#"
			        import { myFunction } from "./myFunction"
			        if (someProcessEnvCheck) {
			          describe("myFunction", () => {})
			        }
			      "#,
            r#"
			        import { myFunction } from "./myFunction"
			        if (someProcessEnvCheck) {
			          describe(myFunction, () => {})
			        }
			      "#,
            None,
        ),
        (
            r#"
			        import { myFunction } from "./myFunction"
			        describe("myFunction", () => {})
			      "#,
            r#"
			        import { myFunction } from "./myFunction"
			        describe(myFunction, () => {})
			      "#,
            None,
        ),
        (
            r#"
			        import type { Button } from "./button"
			        describe("Button", () => {})
			      "#,
            r#"
			        import type { Button } from "./button"
			        describe("Button", () => {})
			      "#,
            None,
        ),
    ];
    Tester::new(PreferDescribeFunctionTitle::NAME, PreferDescribeFunctionTitle::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
