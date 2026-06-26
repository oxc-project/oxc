use oxc_ast::{
    AstKind,
    ast::{Argument, BinaryExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::{
    context::LintContext,
    fixer::{FixKind, RuleFixer},
    utils::{
        KnownMemberExpressionProperty, PossibleJestNode, is_equality_matcher,
        parse_expect_jest_fn_call,
    },
};

fn use_equality_matcher_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using the built-in equality matchers.")
        .with_help("Prefer using one of the equality matchers instead")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

Jest has built-in matchers for expecting equality, which allow for more readable
tests and error messages if an expectation fails.

### Why is this bad?

Testing equality expressions with generic matchers like `toBe(true)`
makes tests harder to read and understand. When tests fail, the error
messages are less helpful because they don't show what the actual values
were. Using specific equality matchers provides clearer test intent and
better debugging information.

### Examples

Examples of **incorrect** code for this rule:
```javascript
expect(x === 5).toBe(true);
expect(name === 'Carl').not.toEqual(true);
expect(myObj !== thatObj).toStrictEqual(true);
```

Examples of **correct** code for this rule:
```javascript
expect(x).toBe(5);
expect(name).not.toEqual('Carl');
expect(myObj).toStrictEqual(thatObj);
```
";

pub fn run_on_jest_node<'a, 'c>(
    possible_jest_node: &PossibleJestNode<'a, 'c>,
    ctx: &'c LintContext<'a>,
) {
    let node = possible_jest_node.node;
    let AstKind::CallExpression(matcher_call_expr) = node.kind() else {
        return;
    };
    let Some(jest_fn_call) = parse_expect_jest_fn_call(matcher_call_expr, possible_jest_node, ctx)
    else {
        return;
    };

    let Some(expect_parent) = jest_fn_call.head.parent else {
        return;
    };
    let expr = expect_parent.get_inner_expression();
    let Expression::CallExpression(expect_call_expr) = expr else {
        return;
    };
    let Some(argument) = expect_call_expr.arguments.first() else {
        return;
    };

    let Argument::BinaryExpression(binary_expr) = argument else {
        return;
    };

    if binary_expr.operator != BinaryOperator::StrictEquality
        && binary_expr.operator != BinaryOperator::StrictInequality
    {
        return;
    }

    let Some(matcher) = jest_fn_call.matcher() else {
        return;
    };

    if !is_equality_matcher(matcher) {
        return;
    }
    let Some(first_matcher_arg) = jest_fn_call.args.first().and_then(Argument::as_expression)
    else {
        return;
    };
    let Expression::BooleanLiteral(matcher_arg_value) = first_matcher_arg.get_inner_expression()
    else {
        return;
    };

    let modifiers = jest_fn_call.modifiers();
    let has_not_modifier = modifiers.iter().any(|modifier| modifier.is_name_equal("not"));
    let add_not_modifier = (if binary_expr.operator == BinaryOperator::StrictInequality {
        !matcher_arg_value.value
    } else {
        matcher_arg_value.value
    }) == has_not_modifier;

    let fixer = RuleFixer::new(FixKind::Suggestion, ctx);
    let suggestions = ["toBe", "toEqual", "toStrictEqual"].into_iter().map(|eq_matcher| {
        // Preserve trailing commas: expect(a === b,).toBe(true,) -> expect(a,).toBe(b,)
        let call_span_end =
            fixer.source_range(Span::new(binary_expr.span.end, expect_call_expr.span.end));
        let arg_span_end =
            fixer.source_range(Span::new(matcher_arg_value.span.end, matcher_call_expr.span.end));
        let content = build_code(
            binary_expr,
            call_span_end,
            arg_span_end,
            &jest_fn_call.local,
            &modifiers,
            eq_matcher,
            add_not_modifier,
            fixer,
        );
        fixer.replace(matcher_call_expr.span, content).with_message(format!("Use `{eq_matcher}`"))
    });

    ctx.diagnostic_with_suggestions(use_equality_matcher_diagnostic(matcher.span), suggestions);
}

fn build_code<'a>(
    binary_expr: &BinaryExpression<'a>,
    call_span_end: &str,
    arg_span_end: &str,
    local_name: &str,
    modifiers: &[&KnownMemberExpressionProperty<'a>],
    equality_matcher: &str,
    add_not_modifier: bool,
    fixer: RuleFixer<'_, 'a>,
) -> String {
    let mut content = fixer.codegen();
    content.print_str(local_name);
    content.print_ascii_byte(b'(');
    content.print_expression(&binary_expr.left);
    content.print_str(call_span_end);
    content.print_ascii_byte(b'.');
    for modifier in modifiers {
        let Some(modifier_name) = modifier.name() else {
            continue;
        };
        if modifier_name != "not" {
            content.print_str(&modifier_name);
            content.print_ascii_byte(b'.');
        }
    }
    if add_not_modifier {
        content.print_str("not.");
    }
    content.print_str(equality_matcher);
    content.print_ascii_byte(b'(');
    content.print_expression(&binary_expr.right);
    content.print_str(arg_span_end);
    content.into_source_text()
}
