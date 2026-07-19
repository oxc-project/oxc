use oxc_ast::{
    AstKind,
    ast::{Expression, ExpressionKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_number_coercion_diagnostic(span: Span, replacement: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `{replacement}`."))
        .with_help(format!("Replace this call with `{replacement}`."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferNumberCoercion;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `Number()` over `parseFloat()` and base-10 `parseInt()`.
    ///
    /// ### Why is this bad?
    ///
    /// `parseFloat()` and `parseInt()` parse numeric prefixes and ignore trailing text.
    /// `Number()` parses the full input, which better matches intent when coercing values.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const value = parseFloat(input);
    /// const integer = parseInt(input, 10);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const value = Number(input);
    /// const integer = Math.trunc(Number(input));
    /// ```
    PreferNumberCoercion,
    unicorn,
    pedantic,
    suggestion,
    version = "1.71.0",
    short_description = "Prefer `Number()` over `parseFloat()` and base-10 `parseInt()`.",
);

impl Rule for PreferNumberCoercion {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional {
            return;
        }

        let Some(target) = parse_target(&call_expr.callee, ctx) else {
            return;
        };

        let Some(first_argument) = call_expr.arguments.first() else {
            return;
        };

        match target {
            ParseTarget::ParseFloat if call_expr.arguments.len() != 1 => return,
            ParseTarget::ParseInt => {
                if call_expr.arguments.len() != 2 {
                    return;
                }
                let Some(second_argument) = call_expr.arguments.get(1) else {
                    return;
                };
                if !second_argument
                    .as_expression()
                    .is_some_and(|expression| expression.is_number_value(10.0))
                {
                    return;
                }
            }
            ParseTarget::ParseFloat => {}
        }

        if first_argument.as_expression().is_none() {
            return;
        }

        let first_argument_text = ctx.source_range(first_argument.span());
        let replacement_text = match target {
            ParseTarget::ParseFloat => format!("Number({first_argument_text})"),
            ParseTarget::ParseInt => format!("Math.trunc(Number({first_argument_text}))"),
        };

        ctx.diagnostic_with_suggestion(
            prefer_number_coercion_diagnostic(call_expr.span, &replacement_text),
            |fixer| fixer.replace(call_expr.span, replacement_text),
        );
    }
}

#[derive(Clone, Copy)]
enum ParseTarget {
    ParseFloat,
    ParseInt,
}

fn parse_target(callee: &Expression, ctx: &LintContext) -> Option<ParseTarget> {
    let callee = callee.without_parentheses();
    match callee.kind() {
        ExpressionKind::Identifier(ident) => {
            if !ctx.is_reference_to_global_variable(ident) {
                return None;
            }
            match ident.name.as_str() {
                "parseFloat" => Some(ParseTarget::ParseFloat),
                "parseInt" => Some(ParseTarget::ParseInt),
                _ => None,
            }
        }
        _ if callee.is_member_expression() => {
            let member_expr = callee.to_member_expression();
            let ExpressionKind::Identifier(object) = member_expr.object().kind() else {
                return None;
            };
            if object.name != "Number" {
                return None;
            }
            if !ctx.is_reference_to_global_variable(object) {
                return None;
            }
            match member_expr.static_property_name()? {
                "parseFloat" => Some(ParseTarget::ParseFloat),
                "parseInt" => Some(ParseTarget::ParseInt),
                _ => None,
            }
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Number(value);",
        "Number.parseInt(value);",
        "Number.parseInt(value, 16);",
        "parseInt(value);",
        "parseInt(value, 16);",
        "parseFloat();",
        "parseInt(value, radix);",
        "parseFloat(value, sideEffect());",
        "parseInt(value, 10, sideEffect());",
        "foo.parseFloat(value);",
        "foo.parseInt(value, 10);",
        "const parseFloat = () => {}; parseFloat(value);",
        "const Number = { parseFloat() {} }; Number.parseFloat(value);",
    ];

    let fail = vec![
        "parseFloat(value);",
        "parseFloat(a + b);",
        "Number.parseFloat(value);",
        "Number['parseFloat'](value);",
        "parseInt(value, 10);",
        "parseInt(getValue(), 10);",
        "parseInt(parseInt(value, 10), 10);",
        "parseInt(value as string, 10);",
        "Number.parseInt(value, 10);",
        "parseInt(value, 10.0);",
        "(parseInt)(value, 10);",
        "Number.parseInt((value), 10);",
    ];

    let fix = vec![
        ("parseFloat(value);", "Number(value);"),
        ("parseFloat(a + b);", "Number(a + b);"),
        ("Number.parseFloat(value);", "Number(value);"),
        ("Number['parseFloat'](value);", "Number(value);"),
        ("parseInt(value, 10);", "Math.trunc(Number(value));"),
        ("parseInt(getValue(), 10);", "Math.trunc(Number(getValue()));"),
        ("parseInt(parseInt(value, 10), 10);", "Math.trunc(Number(parseInt(value, 10)));"),
        ("parseInt(value as string, 10);", "Math.trunc(Number(value as string));"),
        ("Number.parseInt(value, 10);", "Math.trunc(Number(value));"),
        ("parseInt(value, 10.0);", "Math.trunc(Number(value));"),
        ("(parseInt)(value, 10);", "Math.trunc(Number(value));"),
        ("Number.parseInt((value), 10);", "Math.trunc(Number((value)));"),
    ];

    Tester::new(PreferNumberCoercion::NAME, PreferNumberCoercion::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
