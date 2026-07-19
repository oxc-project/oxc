use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression, ExpressionKind, UnaryOperator},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn missing_delay_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{name}` should have an explicit delay argument."))
        .with_help("Add an explicit delay argument.")
        .with_label(span)
}

fn redundant_delay_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{name}` should not have an explicit delay of `0`."))
        .with_help("Remove the explicit `0` delay argument.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ExplicitTimerDelayMode {
    #[default]
    /// Require explicit `delay` argument for clarity.
    Always,
    /// Disallow explicit `0` delay, prefer implicit default.
    Never,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ExplicitTimerDelay(ExplicitTimerDelayMode);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce or disallow explicit `delay` argument for `setTimeout()` and
    /// `setInterval()`.
    ///
    /// ### Why is this bad?
    ///
    /// When using `setTimeout()` or `setInterval()`, the `delay` parameter is
    /// optional and defaults to `0`. This rule allows you to enforce whether the
    /// `delay` argument should always be explicitly provided or omitted when it's
    /// `0`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// setTimeout(() => console.log('Hello'));
    /// setInterval(callback);
    /// window.setTimeout(() => console.log('Hello'));
    /// globalThis.setInterval(callback);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// setTimeout(() => console.log('Hello'), 0);
    /// setInterval(callback, 0);
    /// window.setTimeout(() => console.log('Hello'), 0);
    /// globalThis.setInterval(callback, 0);
    /// setTimeout(() => console.log('Hello'), 1000);
    /// setInterval(callback, 100);
    /// ```
    ///
    /// With the `"never"` option, explicit `0` delays are disallowed and
    /// non-zero delays are still allowed.
    ///
    /// Examples of **incorrect** code for the `"never"` option:
    /// ```javascript
    /// setTimeout(() => console.log('Hello'), 0);
    /// setInterval(callback, 0);
    /// window.setTimeout(() => console.log('Hello'), 0);
    /// globalThis.setInterval(callback, 0);
    /// ```
    ///
    /// Examples of **correct** code for the `"never"` option:
    /// ```javascript
    /// setTimeout(() => console.log('Hello'));
    /// setInterval(callback);
    /// window.setTimeout(() => console.log('Hello'));
    /// globalThis.setInterval(callback);
    /// setTimeout(() => console.log('Hello'), 1000);
    /// globalThis.setInterval(callback, 100);
    /// ```
    ExplicitTimerDelay,
    unicorn,
    style,
    fix,
    config = ExplicitTimerDelayMode,
    version = "1.73.0",
    short_description = "Enforce or disallow explicit `delay` argument for `setTimeout()` and `setInterval()`.",
);

impl Rule for ExplicitTimerDelay {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional {
            return;
        }

        let Some(name) = timer_name(call_expr, ctx) else {
            return;
        };

        match self.0 {
            ExplicitTimerDelayMode::Always => check_always(call_expr, ctx, name),
            ExplicitTimerDelayMode::Never => check_never(call_expr, ctx, name),
        }
    }
}

fn check_always(call_expr: &CallExpression, ctx: &LintContext, name: &str) {
    if call_expr.arguments.len() != 1 || call_expr.arguments[0].is_spread() {
        return;
    }

    let first_argument = &call_expr.arguments[0];
    ctx.diagnostic_with_fix(missing_delay_diagnostic(call_expr.span, name), |fixer| {
        fixer.insert_text_after_range(Span::empty(first_argument.span().end), ", 0")
    });
}

fn check_never(call_expr: &CallExpression, ctx: &LintContext, name: &str) {
    if call_expr.arguments.len() != 2 {
        return;
    }

    let first_argument = &call_expr.arguments[0];
    let delay_argument = &call_expr.arguments[1];

    if !is_zero_delay(delay_argument) {
        return;
    }

    ctx.diagnostic_with_fix(redundant_delay_diagnostic(delay_argument.span(), name), |fixer| {
        let delete_span = Span::new(first_argument.span().end, delay_argument.span().end);
        fixer.delete_range(delete_span)
    });
}

fn timer_name<'a>(call_expr: &CallExpression<'a>, ctx: &LintContext<'a>) -> Option<&'a str> {
    let inner = call_expr.callee.get_inner_expression();
    match inner.kind() {
        ExpressionKind::Identifier(ident) if is_timer_function_name(ident.name.as_str()) => {
            ctx.is_reference_to_global_variable(ident).then_some(ident.name.as_str())
        }
        _ => {
            let member_expr = inner.as_member_expression()?;
            if member_expr.is_computed() {
                return None;
            }

            let name = member_expr.static_property_name()?;

            if !is_timer_function_name(name) {
                return None;
            }

            let ExpressionKind::Identifier(object) =
                member_expr.object().get_inner_expression().kind()
            else {
                return None;
            };

            (is_global_object_name(object.name.as_str())
                && ctx.is_reference_to_global_variable(object))
            .then_some(name)
        }
    }
}

fn is_timer_function_name(name: &str) -> bool {
    matches!(name, "setTimeout" | "setInterval")
}

fn is_global_object_name(name: &str) -> bool {
    matches!(name, "window" | "globalThis" | "global" | "self")
}

fn is_zero_delay(argument: &Argument<'_>) -> bool {
    let Some(expression) = argument.as_expression() else {
        return false;
    };

    is_zero_expression(expression.without_parentheses())
}

fn is_zero_expression(expression: &Expression<'_>) -> bool {
    match expression.kind() {
        ExpressionKind::NumericLiteral(literal) => literal.value == 0.0,
        ExpressionKind::UnaryExpression(unary)
            if matches!(
                unary.operator,
                UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation
            ) =>
        {
            is_zero_expression(unary.argument.without_parentheses())
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"setTimeout(() => console.log("Hello"), 0);"#, None),
        ("setInterval(callback, 0);", None),
        (r#"setTimeout(() => console.log("Hello"), 1000);"#, None),
        ("setInterval(callback, 100);", None),
        (r#"window.setTimeout(() => console.log("Hello"), 0);"#, None),
        ("globalThis.setInterval(callback, 0);", None),
        ("global.setTimeout(() => {}, 0);", None),
        ("self.setTimeout(() => {}, 0);", None),
        ("setTimeout(callback, 0, arg1, arg2);", None),
        ("setInterval(callback, 100, arg1);", None),
        (
            "import {setTimeout as delay} from 'node:timers/promises';
            await delay(100);",
            None,
        ),
        ("setTimeout?.(() => {});", None),
        ("window.setTimeout?.(callback);", None),
        ("setTimeout();", None),
        ("setTimeout(...args);", None),
        ("customSetTimeout(callback);", None),
        ("obj.customSetTimeout(callback);", None),
        (r#"globalThis["setTimeout"](callback);"#, None),
        ("Math.setTimeout(callback);", None),
        ("foo.setTimeout(callback);", None),
        (
            "const window = foo;
            window.setTimeout(callback);",
            None,
        ),
        (
            "const self = foo;
            self.setInterval(callback);",
            None,
        ),
        (r#"setTimeout(() => console.log("Hello"));"#, Some(serde_json::json!(["never"]))),
        ("setInterval(callback);", Some(serde_json::json!(["never"]))),
        (r#"window.setTimeout(() => console.log("Hello"));"#, Some(serde_json::json!(["never"]))),
        ("globalThis.setInterval(callback);", Some(serde_json::json!(["never"]))),
        ("setTimeout((callback), 0);", None),
        (r#"setTimeout(() => console.log("Hello"), 1000);"#, Some(serde_json::json!(["never"]))),
        ("setTimeout(callback, (1000));", Some(serde_json::json!(["never"]))),
        ("setInterval(callback, 100);", Some(serde_json::json!(["never"]))),
        ("setInterval(callback, 0, arg1);", Some(serde_json::json!(["never"]))),
        ("setTimeout(callback, 500, arg1);", Some(serde_json::json!(["never"]))),
        ("setTimeout(callback, 0, arg1, arg2);", Some(serde_json::json!(["never"]))),
    ];

    let fail = vec![
        (r#"setTimeout(() => console.log("Hello"));"#, None),
        ("setInterval(callback);", None),
        (r#"window.setTimeout(() => console.log("Hello"));"#, None),
        ("globalThis.setInterval(callback);", None),
        ("global.setTimeout(fn);", None),
        ("self.setTimeout(fn);", None),
        (
            r#"setTimeout(
                () => console.log("Hello")
            );"#,
            None,
        ),
        (
            "setInterval(
                callback
            );",
            None,
        ),
        ("setTimeout((callback));", None),
        (r#"setTimeout(() => console.log("Hello"), 0);"#, Some(serde_json::json!(["never"]))),
        ("setInterval(callback, 0);", Some(serde_json::json!(["never"]))),
        ("setTimeout((callback), 0);", Some(serde_json::json!(["never"]))),
        ("self.setTimeout(fn, 0);", Some(serde_json::json!(["never"]))),
        ("self.setInterval(fn, 0);", Some(serde_json::json!(["never"]))),
        (
            r#"window.setTimeout(() => console.log("Hello"), 0);"#,
            Some(serde_json::json!(["never"])),
        ),
        ("globalThis.setInterval(callback, 0);", Some(serde_json::json!(["never"]))),
        ("global.setTimeout(fn, 0);", Some(serde_json::json!(["never"]))),
        (r#"setTimeout(() => console.log("Hello"), -0);"#, Some(serde_json::json!(["never"]))),
        ("setTimeout(callback, +0);", Some(serde_json::json!(["never"]))),
        ("setInterval(callback, +(0));", Some(serde_json::json!(["never"]))),
        ("setTimeout(callback, (-0));", Some(serde_json::json!(["never"]))),
        (
            r#"setTimeout(
                () => console.log("Hello"),
                0
            );"#,
            Some(serde_json::json!(["never"])),
        ),
        (
            "setInterval(
                callback,
                0
            );",
            Some(serde_json::json!(["never"])),
        ),
        ("setTimeout(callback, (0));", Some(serde_json::json!(["never"]))),
    ];

    let fix = vec![
        (
            r#"setTimeout(() => console.log("Hello"));"#,
            r#"setTimeout(() => console.log("Hello"), 0);"#,
            None,
        ),
        ("setInterval(callback);", "setInterval(callback, 0);", None),
        (
            r#"window.setTimeout(() => console.log("Hello"));"#,
            r#"window.setTimeout(() => console.log("Hello"), 0);"#,
            None,
        ),
        ("globalThis.setInterval(callback);", "globalThis.setInterval(callback, 0);", None),
        ("global.setTimeout(fn);", "global.setTimeout(fn, 0);", None),
        ("self.setTimeout(fn);", "self.setTimeout(fn, 0);", None),
        (
            r#"setTimeout(
                () => console.log("Hello")
            );"#,
            r#"setTimeout(
                () => console.log("Hello"), 0
            );"#,
            None,
        ),
        (
            "setInterval(
                callback
            );",
            "setInterval(
                callback, 0
            );",
            None,
        ),
        ("setTimeout((callback));", "setTimeout((callback), 0);", None),
        (
            r#"setTimeout(() => console.log("Hello"), 0);"#,
            r#"setTimeout(() => console.log("Hello"));"#,
            Some(serde_json::json!(["never"])),
        ),
        ("setInterval(callback, 0);", "setInterval(callback);", Some(serde_json::json!(["never"]))),
        (
            "setTimeout((callback), 0);",
            "setTimeout((callback));",
            Some(serde_json::json!(["never"])),
        ),
        ("self.setTimeout(fn, 0);", "self.setTimeout(fn);", Some(serde_json::json!(["never"]))),
        ("self.setInterval(fn, 0);", "self.setInterval(fn);", Some(serde_json::json!(["never"]))),
        (
            r#"window.setTimeout(() => console.log("Hello"), 0);"#,
            r#"window.setTimeout(() => console.log("Hello"));"#,
            Some(serde_json::json!(["never"])),
        ),
        (
            "globalThis.setInterval(callback, 0);",
            "globalThis.setInterval(callback);",
            Some(serde_json::json!(["never"])),
        ),
        ("global.setTimeout(fn, 0);", "global.setTimeout(fn);", Some(serde_json::json!(["never"]))),
        (
            r#"setTimeout(() => console.log("Hello"), -0);"#,
            r#"setTimeout(() => console.log("Hello"));"#,
            Some(serde_json::json!(["never"])),
        ),
        ("setTimeout(callback, +0);", "setTimeout(callback);", Some(serde_json::json!(["never"]))),
        (
            "setInterval(callback, +(0));",
            "setInterval(callback);",
            Some(serde_json::json!(["never"])),
        ),
        (
            "setTimeout(callback, (-0));",
            "setTimeout(callback);",
            Some(serde_json::json!(["never"])),
        ),
        (
            r#"setTimeout(
                () => console.log("Hello"),
                0
            );"#,
            r#"setTimeout(
                () => console.log("Hello")
            );"#,
            Some(serde_json::json!(["never"])),
        ),
        (
            "setInterval(
                callback,
                0
            );",
            "setInterval(
                callback
            );",
            Some(serde_json::json!(["never"])),
        ),
        ("setTimeout(callback, (0));", "setTimeout(callback);", Some(serde_json::json!(["never"]))),
    ];

    Tester::new(ExplicitTimerDelay::NAME, ExplicitTimerDelay::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
