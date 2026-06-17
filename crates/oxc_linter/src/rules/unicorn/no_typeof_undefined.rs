use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    ast_util::{could_be_asi_hazard, get_declaration_of_variable},
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::{DefaultRuleConfig, Rule},
};

fn no_typeof_undefined_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Compare with `undefined` directly instead of using `typeof`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoTypeofUndefined {
    /// If set to `true`, also report `typeof x === "undefined"` when `x` may be a global
    /// variable that is not declared (commonly checked via `typeof foo === "undefined"`).
    check_global_variables: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `typeof` comparisons with `undefined`.
    ///
    /// ### Why is this bad?
    ///
    /// Checking if a value is `undefined` by using `typeof value === 'undefined'` is needlessly verbose. It's generally better to compare against `undefined` directly. The only time `typeof` is needed is when a global variable potentially does not exists, in which case, using `globalThis.value === undefined` may be better.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// typeof foo === 'undefined';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// foo === undefined;
    /// ```
    NoTypeofUndefined,
    unicorn,
    pedantic,
    fix_or_suggestion,
    config = NoTypeofUndefined,
    version = "0.0.18",
    short_description = "Disallow `typeof` comparisons with `undefined`.",
);

impl Rule for NoTypeofUndefined {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(bin_expr) = node.kind() else {
            return;
        };

        if !matches!(
            bin_expr.operator,
            BinaryOperator::StrictEquality
                | BinaryOperator::StrictInequality
                | BinaryOperator::Equality
                | BinaryOperator::Inequality,
        ) {
            return;
        }

        let Expression::UnaryExpression(unary_expr) = &bin_expr.left else {
            return;
        };

        if unary_expr.operator != UnaryOperator::Typeof {
            return;
        }

        if !bin_expr.right.is_specific_string_literal("undefined") {
            return;
        }

        let is_global_variable = is_global_variable(&unary_expr.argument, ctx);

        if !self.check_global_variables && is_global_variable {
            return;
        }

        if is_global_variable {
            ctx.diagnostic_with_suggestion(
                no_typeof_undefined_diagnostic(bin_expr.span),
                |fixer| {
                    generate_fix(
                        fixer,
                        node,
                        &unary_expr.argument,
                        bin_expr.operator,
                        bin_expr.span,
                        ctx,
                    )
                },
            );
        } else {
            ctx.diagnostic_with_fix(no_typeof_undefined_diagnostic(bin_expr.span), |fixer| {
                generate_fix(
                    fixer,
                    node,
                    &unary_expr.argument,
                    bin_expr.operator,
                    bin_expr.span,
                    ctx,
                )
            });
        }
    }

    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }
}

fn is_global_variable<'a>(ident: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    let Expression::Identifier(ident) = ident else {
        return false;
    };

    get_declaration_of_variable(ident, ctx).is_none()
}

fn generate_fix<'a>(
    fixer: RuleFixer<'_, 'a>,
    node: &AstNode<'a>,
    argument: &Expression<'a>,
    operator: BinaryOperator,
    span: Span,
    ctx: &LintContext<'a>,
) -> RuleFix {
    let argument_text = ctx.source_range(argument.span());
    let op = match operator {
        BinaryOperator::StrictEquality | BinaryOperator::Equality => "===",
        BinaryOperator::StrictInequality | BinaryOperator::Inequality => "!==",
        _ => unreachable!(),
    };
    // Removing the `typeof` keyword can create an ASI hazard when the argument starts with `(`/`[`
    // and the expression is at statement start: `foo\ntypeof [] === "undefined"` would otherwise
    // become `foo\n[] === undefined` (parsed as `foo[]`). Prepend `;` in that case, like upstream.
    let prefix = if could_be_asi_hazard(node, ctx)
        && matches!(argument_text.chars().next(), Some('(' | '['))
    {
        ";"
    } else {
        ""
    };
    fixer.replace(span, format!("{prefix}{argument_text} {op} undefined"))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"typeof a.b", None),
        (r#"typeof a.b > "undefined""#, None),
        (r#"a.b === "undefined""#, None),
        (r#"void a.b === "undefined""#, None),
        (r#"+a.b === "undefined""#, None),
        (r#"++a.b === "undefined""#, None),
        (r#"a.b++ === "undefined""#, None),
        (r"foo === undefined", None),
        (r#"typeof a.b === "string""#, None),
        (r#"typeof foo === "undefined""#, None),
        (r#"foo = 2; typeof foo === "undefined""#, None),
        (r#"/* globals foo: readonly */ typeof foo === "undefined""#, None),
        (r#"/* globals globalThis: readonly */ typeof globalThis === "undefined""#, None),
        (r#""undefined" === typeof a.b"#, None),
        (r#"const UNDEFINED = "undefined"; typeof a.b === UNDEFINED"#, None),
        (r"typeof a.b === `undefined`", None),
    ];

    let fail = vec![
        (r#"typeof a.b === "undefined""#, None),
        (r#"typeof a.b !== "undefined""#, None),
        (r#"typeof a.b == "undefined""#, None),
        (r#"typeof a.b != "undefined""#, None),
        (r"typeof a.b == 'undefined'", None),
        (r#"let foo; typeof foo === "undefined""#, None),
        (r#"const foo = 1; typeof foo === "undefined""#, None),
        (r#"var foo; typeof foo === "undefined""#, None),
        (r#"var foo; var foo; typeof foo === "undefined""#, None),
        (r#"for (const foo of bar) typeof foo === "undefined";"#, None),
        (r#"function foo() {typeof foo === "undefined"}"#, None),
        (r#"function foo(bar) {typeof bar === "undefined"}"#, None),
        (r#"function foo({bar}) {typeof bar === "undefined"}"#, None),
        (r#"function foo([bar]) {typeof bar === "undefined"}"#, None),
        (r#"typeof foo.bar === "undefined""#, None),
        (
            r#"let foo; typeof foo === "undefined""#,
            Some(serde_json::json!([{ "checkGlobalVariables": false }])),
        ),
        (
            r#"typeof foo === "undefined""#,
            Some(serde_json::json!([{ "checkGlobalVariables": true }])),
        ),
    ];

    let fix = vec![
        (r#"typeof a.b === "undefined""#, r"a.b === undefined", None),
        (r#"typeof a.b !== "undefined""#, r"a.b !== undefined", None),
        (r#"typeof a.b == "undefined""#, r"a.b === undefined", None),
        (r#"typeof a.b != "undefined""#, r"a.b !== undefined", None),
        (r"typeof a.b == 'undefined'", r"a.b === undefined", None),
        (r#"let foo; typeof foo === "undefined""#, r"let foo; foo === undefined", None),
        (r#"const foo = 1; typeof foo === "undefined""#, r"const foo = 1; foo === undefined", None),
        (r#"var foo; typeof foo === "undefined""#, r"var foo; foo === undefined", None),
        (r#"typeof foo.bar === "undefined""#, r"foo.bar === undefined", None),
        (
            r#"typeof foo === "undefined""#,
            r"foo === undefined",
            Some(serde_json::json!([{ "checkGlobalVariables": true }])),
        ),
        // ASI: dropping `typeof` must not let the next line continue the previous statement
        // (`foo\n[] === undefined` would parse as `foo[]`).
        ("foo\ntypeof [] === \"undefined\"", "foo\n;[] === undefined", None),
        ("foo\ntypeof (a ? b : c) === \"undefined\"", "foo\n;(a ? b : c) === undefined", None),
    ];

    Tester::new(NoTypeofUndefined::NAME, NoTypeofUndefined::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
