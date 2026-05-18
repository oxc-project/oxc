use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::{ToBoolean, WithoutGlobalReferenceInformation};
use oxc_span::Span;

use crate::{
    context::LintContext,
    utils::{
        KnownMemberExpressionParentKind, PossibleJestNode, is_equality_matcher,
        parse_expect_jest_fn_call,
    },
};

fn use_to_contain(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using `toContain()`.").with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

In order to have a better failure message, `toContain()` should be used upon
asserting expectations on an array containing an object.

### Why is this bad?

This rule triggers a warning if `toBe()`, `toEqual()` or `toStrictEqual()` is
used to assert object inclusion in an array

### Examples

Examples of **incorrect** code for this rule:
```javascript
expect(a.includes(b)).toBe(true);
expect(a.includes(b)).not.toBe(true);
expect(a.includes(b)).toBe(false);
expect(a.includes(b)).toEqual(true);
expect(a.includes(b)).toStrictEqual(true);
```

Examples of **correct** code for this rule:
```javascript
expect(a).toContain(b);
expect(a).not.toContain(b);
```
";

pub fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };
    let Some(jest_expect_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
    else {
        return;
    };
    let Some(parent) = jest_expect_fn_call.head.parent else {
        return;
    };
    let Some(matcher) = jest_expect_fn_call.matcher() else {
        return;
    };

    if !matches!(
        jest_expect_fn_call.head.parent_kind.unwrap(),
        KnownMemberExpressionParentKind::Call
    ) || jest_expect_fn_call.args.is_empty()
    {
        return;
    }

    let Some(jest_expect_first_arg) =
        jest_expect_fn_call.args.first().and_then(Argument::as_expression)
    else {
        return;
    };
    let Expression::CallExpression(expect_call_expr) = parent else {
        return;
    };

    // handle "expect()"
    if expect_call_expr.arguments.is_empty()
        || !matches!(jest_expect_first_arg.get_inner_expression(), Expression::BooleanLiteral(_))
    {
        return;
    }

    let Some(first_argument) = expect_call_expr.arguments.first() else {
        return;
    };
    let Argument::CallExpression(includes_call_expr) = first_argument else {
        return;
    };

    if !is_equality_matcher(matcher) || !is_fixable_includes_call_expression(includes_call_expr) {
        return;
    }

    ctx.diagnostic_with_fix(use_to_contain(matcher.span), |fixer| {
        let Some(mem_expr) = includes_call_expr.callee.as_member_expression() else {
            return fixer.noop();
        };

        let Some(argument) = includes_call_expr.arguments.first() else {
            return fixer.noop();
        };

        let negation = {
            let boolean_value = jest_expect_first_arg
                .get_inner_expression()
                .to_boolean(&WithoutGlobalReferenceInformation {})
                .unwrap_or(false);
            let has_not = jest_expect_fn_call
                .modifiers()
                .iter()
                .any(|modifier| modifier.is_name_equal("not"));

            match (boolean_value, has_not) {
                (false, true) | (true, false) => "",
                (true, true) | (false, false) => ".not",
            }
        };

        let mut formatter = fixer.codegen();

        formatter.print_expression(&expect_call_expr.callee);
        formatter.print_str("(");
        formatter.print_expression(mem_expr.object());
        formatter.print_str(format!("){negation}.toContain(").as_str());
        formatter.print_expression(argument.to_expression());
        formatter.print_str(")");

        fixer.replace(call_expr.span, formatter.into_source_text())
    });
}

fn is_fixable_includes_call_expression(call_expr: &CallExpression) -> bool {
    let Some(mem_expr) = call_expr.callee.as_member_expression() else {
        return false;
    };

    mem_expr.static_property_name() == Some("includes")
        // handle "expect(a.includes())"
        && !call_expr.arguments.is_empty()
        // handle "expect(a.includes(b,c))"
        && call_expr.arguments.len() == 1
        // handle "expect(a.includes(...[]))"
        && call_expr.arguments[0].is_expression()
}
