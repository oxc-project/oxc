use oxc_ast::{
    AstKind,
    ast::{
        Argument, CallExpression, Class, Expression, Function, MemberExpression, NewTarget,
        StaticMemberExpression, Super, ThisExpression,
    },
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    ast_util::is_method_call,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn prefer_queue_microtask_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `queueMicrotask()` over `{name}`."))
        .with_help(format!("Replace `{name}` with `queueMicrotask()`."))
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PreferQueueMicrotask(Box<PreferQueueMicrotaskConfig>);

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferQueueMicrotaskConfig {
    /// Whether to also check `setImmediate(callback)`.
    check_set_immediate: bool,
    /// Whether to also check `setTimeout(callback, 0)`.
    check_set_timeout: bool,
}

impl std::ops::Deref for PreferQueueMicrotask {
    type Target = PreferQueueMicrotaskConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers [`queueMicrotask()`](https://developer.mozilla.org/en-US/docs/Web/API/Window/queueMicrotask)
    /// over `process.nextTick()`. With the matching options enabled, it also covers
    /// `setImmediate()` and `setTimeout(callback, 0)`.
    ///
    /// ### Why is this bad?
    ///
    /// `queueMicrotask()` is the standard, portable way to queue a microtask in browsers
    /// and Node.js, whereas `process.nextTick()` and `setImmediate()` are Node.js-specific.
    ///
    /// Note that these APIs are not strictly equivalent: `process.nextTick()` callbacks run
    /// before promise microtasks, and `setImmediate()` / `setTimeout()` callbacks run in a
    /// later turn of the event loop. Review the autofix if your code depends on that ordering.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// process.nextTick(callback);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// queueMicrotask(callback);
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with `{ "checkSetImmediate": true }`:
    /// ```js
    /// setImmediate(callback);
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with `{ "checkSetTimeout": true }`:
    /// ```js
    /// setTimeout(callback, 0);
    /// ```
    ///
    /// Only calls whose timer handle is unused and whose callback is not obviously non-callable
    /// are checked with `checkSetImmediate` and `checkSetTimeout`. Calls with extra arguments, or
    /// with a `function` callback that uses `this`, are reported without a fix, because
    /// `queueMicrotask()` cannot forward arguments and calls the callback with `this` set to `undefined`.
    PreferQueueMicrotask,
    unicorn,
    style,
    fix,
    config = PreferQueueMicrotaskConfig,
    version = "next",
    short_description = "Prefer `queueMicrotask()` over `process.nextTick()`, `setImmediate()`, and `setTimeout(…, 0)`.",
);

impl Rule for PreferQueueMicrotask {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::StaticMemberExpression(member) => check_process_next_tick(member, node, ctx),
            AstKind::CallExpression(call) => self.check_timer_call(call, node, ctx),
            _ => {}
        }
    }
}

fn check_process_next_tick<'a>(
    member: &StaticMemberExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) {
    if member.optional || member.property.name != "nextTick" {
        return;
    }
    let Expression::Identifier(object) = &member.object else {
        return;
    };
    if object.name != "process" || !ctx.is_reference_to_global_variable(object) {
        return;
    }

    let parent_call = match ctx.nodes().parent_kind(node.id()) {
        AstKind::CallExpression(call) if call.callee.span() == member.span => Some(call),
        _ => None,
    };

    // `process.nextTick?.(callback)`
    if parent_call.is_some_and(|call| call.optional) {
        return;
    }

    let diagnostic = prefer_queue_microtask_diagnostic(member.property.span, "process.nextTick()");

    match parent_call {
        Some(call)
            if call.arguments.len() == 1
                && !call.arguments[0].is_spread()
                && !ctx.has_comments_between(member.span) =>
        {
            ctx.diagnostic_with_fix(diagnostic, |fixer| {
                fixer.replace(member.span, "queueMicrotask")
            });
        }
        _ => ctx.diagnostic(diagnostic),
    }
}

impl PreferQueueMicrotask {
    fn check_timer_call<'a>(
        &self,
        call: &CallExpression<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let Expression::Identifier(callee) = &call.callee else {
            return;
        };

        let (name, is_set_timeout) = match callee.name.as_str() {
            "setImmediate" if self.check_set_immediate && !call.arguments.is_empty() => {
                ("setImmediate()", false)
            }
            "setTimeout"
                if self.check_set_timeout
                    && call.arguments.len() >= 2
                    && is_zero(&call.arguments[1]) =>
            {
                ("setTimeout()", true)
            }
            _ => return,
        };

        let Some(callback) = call.arguments[0].as_expression() else {
            return;
        };

        if call.optional
            || is_node_value_not_function(callback)
            || !is_value_not_usable(node, ctx)
            || !ctx.is_reference_to_global_variable(callee)
        {
            return;
        }

        let diagnostic = prefer_queue_microtask_diagnostic(callee.span, name);

        if has_this_bound_callback(callback) {
            ctx.diagnostic(diagnostic);
            return;
        }

        if is_set_timeout {
            // Only `setTimeout(callback, 0)` can be fixed, by dropping the delay.
            if call.arguments.len() == 2
                && let Some(removal) = delay_argument_removal_span(call, ctx)
            {
                ctx.diagnostic_with_fix(diagnostic, |fixer| {
                    let fixer = fixer.for_multifix();
                    let mut fix = fixer.new_fix_with_capacity(2);
                    fix.push(fixer.replace(callee.span, "queueMicrotask"));
                    fix.push(fixer.delete_range(removal));
                    fix.with_message(
                        "Replace `setTimeout(callback, 0)` with `queueMicrotask(callback)`",
                    )
                });
            } else {
                ctx.diagnostic(diagnostic);
            }
        } else if call.arguments.len() == 1 {
            ctx.diagnostic_with_fix(diagnostic, |fixer| {
                fixer.replace(callee.span, "queueMicrotask")
            });
        } else {
            ctx.diagnostic(diagnostic);
        }
    }
}

fn is_zero(argument: &Argument<'_>) -> bool {
    matches!(
        argument.as_expression().map(Expression::without_parentheses),
        Some(Expression::NumericLiteral(literal)) if literal.value == 0.0
    )
}

/// The span from the comma before the delay argument to the end of the delay argument,
/// including a trailing comma. `None` if the span contains a comment, which would be lost.
fn delay_argument_removal_span(call: &CallExpression<'_>, ctx: &LintContext<'_>) -> Option<Span> {
    let callback_end = call.arguments[0].span().end;
    let delay = call.arguments[1].span();

    let comma = callback_end + ctx.find_next_token_within(callback_end, delay.start, ",")?;
    let end = match ctx.find_next_token_within(delay.end, call.span.end, ",") {
        Some(offset) => delay.end + offset + 1,
        None => delay.end,
    };

    // `call.span.end - 1` is the closing parenthesis.
    if ctx.has_comments_between(Span::new(comma, call.span.end - 1)) {
        return None;
    }

    Some(Span::new(comma, end))
}

/// Whether the value of `node` is discarded: it is an expression statement, or the
/// non-final (or itself discarded) element of a sequence expression.
fn is_value_not_usable(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let parent = ctx.nodes().parent_node(node.id());
    match parent.kind() {
        AstKind::ExpressionStatement(_) => true,
        AstKind::ParenthesizedExpression(_) => is_value_not_usable(parent, ctx),
        AstKind::SequenceExpression(sequence) => {
            sequence.expressions.last().is_some_and(|last| last.span() != node.kind().span())
                || is_value_not_usable(parent, ctx)
        }
        _ => false,
    }
}

/// `setTimeout()` / `setImmediate()` call the callback with the timer as `this`, while
/// `queueMicrotask()` calls it with `this === undefined`, so a `function` callback
/// that reads `this` is not interchangeable.
fn has_this_bound_callback(callback: &Expression<'_>) -> bool {
    let Expression::FunctionExpression(function) = callback else {
        return false;
    };
    let mut finder = LexicalThisFinder::default();
    finder.visit_formal_parameters(&function.params);
    if let Some(body) = &function.body {
        finder.visit_function_body(body);
    }
    finder.found
}

/// Finds `this`, `super` and `new.target`, the bindings an arrow function inherits from
/// its enclosing function.
#[derive(Default)]
struct LexicalThisFinder {
    found: bool,
}

impl<'a> Visit<'a> for LexicalThisFinder {
    fn visit_this_expression(&mut self, _: &ThisExpression) {
        self.found = true;
    }

    fn visit_super(&mut self, _: &Super) {
        self.found = true;
    }

    fn visit_new_target(&mut self, _: &NewTarget) {
        self.found = true;
    }

    // A nested non-arrow function rebinds `this`.
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}

    // A class body creates its own `this`, but the `extends` clause and computed keys are
    // evaluated in the enclosing scope.
    fn visit_class(&mut self, class: &Class<'a>) {
        if let Some(heritage) = &class.heritage {
            self.visit_expression(&heritage.expression);
        }
        for element in &class.body.body {
            if element.computed()
                && let Some(key) = element.property_key()
            {
                self.visit_property_key(key);
            }
        }
    }
}

/// Port of upstream's `isNodeValueNotFunction`: expressions that can't be, or most likely
/// aren't, a callable value.
fn is_node_value_not_function(expr: &Expression<'_>) -> bool {
    match expr.without_parentheses() {
        Expression::ArrayExpression(_)
        | Expression::BinaryExpression(_)
        | Expression::ClassExpression(_)
        | Expression::ObjectExpression(_)
        | Expression::TemplateLiteral(_)
        | Expression::UnaryExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::AssignmentExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::NewExpression(_)
        | Expression::TaggedTemplateExpression(_)
        | Expression::ThisExpression(_) => true,
        expr if expr.is_literal() || expr.is_undefined() => true,
        // A call returns a function only in the `fn.bind(…)` case.
        Expression::CallExpression(call) => {
            !(is_method_call(call, None, Some(&["bind"]), None, None)
                && !call.optional
                && !call.callee.get_member_expr().is_some_and(MemberExpression::optional))
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("queueMicrotask(callback);", None),
        ("process.nextTick?.(callback);", None),
        ("process?.nextTick(callback);", None),
        (r#"process["nextTick"](callback);"#, None),
        ("const process = {nextTick: callback => callback()}; process.nextTick(callback);", None),
        (r#"const process = await import("node:process"); process.nextTick(callback);"#, None),
        (
            "const object = {process: {nextTick: callback => callback()}}; object.process.nextTick(callback);",
            None,
        ),
        ("const {nextTick} = process; nextTick(callback);", None),
        (r#"import {nextTick} from "node:process"; nextTick(callback);"#, None),
        ("setImmediate(callback);", None),
        ("setTimeout(callback, 0);", None),
        ("setImmediate?.(callback);", Some(serde_json::json!([{"checkSetImmediate": true}]))),
        (r#"setImmediate("callback()");"#, Some(serde_json::json!([{"checkSetImmediate": true}]))),
        (
            "setImmediate(...argumentsArray);",
            Some(serde_json::json!([{"checkSetImmediate": true}])),
        ),
        (
            "const setImmediate = callback => callback(); setImmediate(callback);",
            Some(serde_json::json!([{"checkSetImmediate": true}])),
        ),
        (
            "globalThis.setImmediate(callback);",
            Some(serde_json::json!([{"checkSetImmediate": true}])),
        ),
        ("window.setTimeout(callback, 0);", Some(serde_json::json!([{"checkSetTimeout": true}]))),
        ("setTimeout(callback);", Some(serde_json::json!([{"checkSetTimeout": true}]))),
        ("setTimeout(callback, 1);", Some(serde_json::json!([{"checkSetTimeout": true}]))),
        (r#"setTimeout("callback()", 0);"#, Some(serde_json::json!([{"checkSetTimeout": true}]))),
        ("setTimeout(...argumentsArray);", Some(serde_json::json!([{"checkSetTimeout": true}]))),
        ("setTimeout(callback, delay);", Some(serde_json::json!([{"checkSetTimeout": true}]))),
        (
            "const timeout = setTimeout(callback, 0);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "const immediate = setImmediate(callback);",
            Some(serde_json::json!([{"checkSetImmediate": true}])),
        ),
        (
            "const setTimeout = (callback, delay) => callback(delay); setTimeout(callback, 0);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        // The timer handle or return value is used
        (
            "const f = () => setImmediate(callback);",
            Some(serde_json::json!([{"checkSetImmediate": true}])),
        ),
        (
            "x = (1, setImmediate(callback));",
            Some(serde_json::json!([{"checkSetImmediate": true}])),
        ),
        // callbacks that aren't functions
        ("setImmediate(callback());", Some(serde_json::json!([{"checkSetImmediate": true}]))),
        ("setTimeout(`callback()`, 0);", Some(serde_json::json!([{"checkSetTimeout": true}]))),
        // `0` must be the number zero
        ("setTimeout(callback, 0n);", Some(serde_json::json!([{"checkSetTimeout": true}]))),
        ("setTimeout(callback, -0);", Some(serde_json::json!([{"checkSetTimeout": true}]))),
    ];

    let fail = vec![
        ("process.nextTick(callback);", None),
        ("const result = process.nextTick(callback);", None),
        ("process /* keep */ .nextTick(callback);", None),
        ("process.nextTick(...argumentsArray);", None),
        ("process.nextTick(callback, value);", None),
        ("const tick = process.nextTick;", None),
        ("foo(process.nextTick);", None),
        ("process.nextTick;", None),
        ("setImmediate(callback);", Some(serde_json::json!([{"checkSetImmediate": true}]))),
        ("setImmediate(callback, value);", Some(serde_json::json!([{"checkSetImmediate": true}]))),
        ("setTimeout(callback, 0);", Some(serde_json::json!([{"checkSetTimeout": true}]))),
        ("setTimeout(callback, 0, value);", Some(serde_json::json!([{"checkSetTimeout": true}]))),
        (
            "setTimeout(callback, /* delay */ 0);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setTimeout(callback, 0 /* delay */);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setTimeout(callback, 0 // delay
            );",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        ("setTimeout(callback, 0,);", Some(serde_json::json!([{"checkSetTimeout": true}]))),
        (
            "setTimeout(callback, 0, /* trailing */);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setTimeout(callback /* keep */, 0);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        // `this` binds to the timer, `queueMicrotask` does not: reported, not fixed
        (
            "setTimeout(function () { this.x; }, 0);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setImmediate(function () { this.x; });",
            Some(serde_json::json!([{"checkSetImmediate": true}])),
        ),
        (
            "setTimeout(function () { return () => this.x; }, 0);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        // arrow functions and nested functions keep their own `this`
        ("setTimeout(() => { this.x; }, 0);", Some(serde_json::json!([{"checkSetTimeout": true}]))),
        (
            "setTimeout(function () { x; }, 0);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setTimeout(function () { function inner() { return this; } }, 0);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
    ];

    let fix = vec![
        ("process.nextTick(callback);", "queueMicrotask(callback);", None),
        (
            "const result = process.nextTick(callback);",
            "const result = queueMicrotask(callback);",
            None,
        ),
        (
            "process /* keep */ .nextTick(callback);",
            "process /* keep */ .nextTick(callback);",
            None,
        ),
        ("process.nextTick(...argumentsArray);", "process.nextTick(...argumentsArray);", None),
        ("process.nextTick(callback, value);", "process.nextTick(callback, value);", None),
        ("const tick = process.nextTick;", "const tick = process.nextTick;", None),
        (
            "setImmediate(callback);",
            "queueMicrotask(callback);",
            Some(serde_json::json!([{"checkSetImmediate": true}])),
        ),
        (
            "setImmediate(callback, value);",
            "setImmediate(callback, value);",
            Some(serde_json::json!([{"checkSetImmediate": true}])),
        ),
        (
            "(setImmediate(callback));",
            "(queueMicrotask(callback));",
            Some(serde_json::json!([{"checkSetImmediate": true}])),
        ),
        (
            "(setImmediate(callback), 1);",
            "(queueMicrotask(callback), 1);",
            Some(serde_json::json!([{"checkSetImmediate": true}])),
        ),
        (
            "setTimeout(callback, 0);",
            "queueMicrotask(callback);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setTimeout(callback, 0,);",
            "queueMicrotask(callback);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setTimeout(callback, (0));",
            "queueMicrotask(callback);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setTimeout(callback /* keep */, 0);",
            "queueMicrotask(callback /* keep */);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setTimeout(callback, 0, value);",
            "setTimeout(callback, 0, value);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setTimeout(callback, /* delay */ 0);",
            "setTimeout(callback, /* delay */ 0);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setTimeout(callback, 0 /* delay */);",
            "setTimeout(callback, 0 /* delay */);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setTimeout(callback, 0, /* trailing */);",
            "setTimeout(callback, 0, /* trailing */);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setTimeout(function () { this.x; }, 0);",
            "setTimeout(function () { this.x; }, 0);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setTimeout(function () { return () => this.x; }, 0);",
            "setTimeout(function () { return () => this.x; }, 0);",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setImmediate(function () { this.x; });",
            "setImmediate(function () { this.x; });",
            Some(serde_json::json!([{"checkSetImmediate": true}])),
        ),
        (
            "setTimeout(() => { this.x; }, 0);",
            "queueMicrotask(() => { this.x; });",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setTimeout(function () { x; }, 0);",
            "queueMicrotask(function () { x; });",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
        (
            "setTimeout(function () { function inner() { return this; } }, 0);",
            "queueMicrotask(function () { function inner() { return this; } });",
            Some(serde_json::json!([{"checkSetTimeout": true}])),
        ),
    ];

    Tester::new(PreferQueueMicrotask::NAME, PreferQueueMicrotask::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
