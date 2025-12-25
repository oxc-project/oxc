use oxc_ast::{
    AstKind,
    ast::{Argument, StringLiteral},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, parse_general_jest_fn_call},
};

fn prefer_describe_function_title_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "A function should be used as the describe title instead of an equivalent string",
    )
    .with_help("Use the function itself instead of a string")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferDescribeFunctionTitle;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces the use of a named function as the title of the describe block instead of a hardcoded string.
    ///
    /// ### Why is this bad?
    ///
    /// Using a hardcoded string risks having the function name go out of sync with the function name being tested if it is renamed.
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
);

impl Rule for PreferDescribeFunctionTitle {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, jest_node, ctx) else {
            return;
        };
        if !matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Describe)) {
            return;
        };
        if call_expr.arguments.len() < 2 {
            return;
        }
        let Some(arg) = call_expr.arguments.first() else {
            return;
        };

        if let Argument::StringLiteral(string_literal) = arg {
            validate_title(&string_literal, jest_node.node, ctx);
        }
    }
}

fn validate_title(string_literal: &StringLiteral, node: &AstNode, ctx: &LintContext) {
    let scope = ctx.scoping();
    if let Some(symbol_id) = scope.find_binding(node.scope_id(), string_literal.value.as_str()) {
        let flags = ctx.scoping().symbol_flags(symbol_id);
        if !flags.is_import() {
            return;
        }

        let replacement = string_literal.value.to_string();
        ctx.diagnostic_with_fix(prefer_describe_function_title_diagnostic(string_literal.span), |fixer| {
            fixer.replace(string_literal.span, replacement)
        });
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
			        import { myFunction } from "./myFunction"
			        describe("myFunction", () => {})
			      "#,
            None,
            Some(serde_json::json!({ "settings": {  "vitest": {  "typecheck": true,  },  } })),
            Some(PathBuf::from("myFunction.test.ts")),
        ),
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
    ];
    Tester::new(PreferDescribeFunctionTitle::NAME, PreferDescribeFunctionTitle::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
