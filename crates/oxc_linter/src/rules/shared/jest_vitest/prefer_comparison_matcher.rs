use oxc_ast::{
    AstKind,
    ast::{Argument, BinaryExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::{
    context::LintContext,
    fixer::RuleFixer,
    utils::{
        KnownMemberExpressionProperty, PossibleJestNode, is_equality_matcher,
        parse_expect_jest_fn_call,
    },
};

fn use_to_be_comparison(preferred_method: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using the built-in comparison matchers")
        .with_help(format!("Prefer using `{preferred_method:?}` instead"))
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

This rule checks for comparisons in tests that could be replaced with one of the
following built-in comparison matchers:
- `toBeGreaterThan`
- `toBeGreaterThanOrEqual`
- `toBeLessThan`
- `toBeLessThanOrEqual`

### Why is this bad?

Using generic matchers like `toBe(true)` with comparison expressions
makes tests less readable and provides less helpful error messages when
they fail. Jest's specific comparison matchers offer clearer intent and
better error output that shows the actual values being compared.

### Examples

Examples of **incorrect** code for this rule:
```js
expect(x > 5).toBe(true);
expect(x < 7).not.toEqual(true);
expect(x <= y).toStrictEqual(true);
```

Examples of **correct** code for this rule:
```js
expect(x).toBeGreaterThan(5);
expect(x).not.toBeLessThanOrEqual(7);
expect(x).toBeLessThanOrEqual(y);
// special case - see below
expect(x < 'Carl').toBe(true);
```
";

pub fn run_on_jest_node<'a, 'c>(
    possible_jest_node: &PossibleJestNode<'a, 'c>,
    ctx: &'c LintContext<'a>,
) {
    let node = possible_jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };
    let Some(parse_expect_jest_fn) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
    else {
        return;
    };
    let Some(matcher) = parse_expect_jest_fn.matcher() else {
        return;
    };
    let Some(parent_node) = parse_expect_jest_fn.head.parent else {
        return;
    };
    let Expression::CallExpression(parent_call_expr) = parent_node else {
        return;
    };
    let Some(Argument::BinaryExpression(binary_expr)) = parent_call_expr.arguments.first() else {
        return;
    };
    let Some(first_matcher_arg) =
        parse_expect_jest_fn.args.first().and_then(Argument::as_expression)
    else {
        return;
    };

    if is_comparing_to_string(binary_expr) || !is_equality_matcher(matcher) {
        return;
    }

    let has_not_modifier =
        parse_expect_jest_fn.modifiers().iter().any(|modifier| modifier.is_name_equal("not"));
    let Expression::BooleanLiteral(matcher_arg_value) = first_matcher_arg.get_inner_expression()
    else {
        return;
    };
    let negated = matcher_arg_value.value == has_not_modifier;
    let preferred_matcher = determine_matcher(binary_expr.operator, negated);
    let Some(prefer_matcher_name) = preferred_matcher else {
        return;
    };

    ctx.diagnostic_with_fix(use_to_be_comparison(prefer_matcher_name, matcher.span), |fixer| {
        // This is to handle the case can be transform into the following case:
        // expect(value > 1,).toEqual(true,) => expect(value,).toBeGreaterThan(1,)
        //                 ^              ^
        // Therefore the range starting after ',' and before '.' is called as call_span_end,
        // and the same as `arg_span_end`.
        let call_span_end =
            fixer.source_range(Span::new(binary_expr.span.end, parent_call_expr.span.end));
        let arg_span_end =
            fixer.source_range(Span::new(matcher_arg_value.span.end, call_expr.span.end));
        let content = building_code(
            binary_expr,
            call_span_end,
            arg_span_end,
            &parse_expect_jest_fn.local,
            &parse_expect_jest_fn.modifiers(),
            prefer_matcher_name,
            fixer,
        );
        fixer.replace(call_expr.span, content)
    });
}

fn is_comparing_to_string(expr: &BinaryExpression) -> bool {
    matches!(expr.left, Expression::StringLiteral(_) | Expression::TemplateLiteral(_))
        || matches!(expr.right, Expression::StringLiteral(_) | Expression::TemplateLiteral(_))
}

fn determine_matcher(operator: BinaryOperator, negated: bool) -> Option<&'static str> {
    let op = if negated { invert_operator(operator) } else { Some(operator) };

    match op {
        Some(BinaryOperator::GreaterThan) => Some("toBeGreaterThan"),
        Some(BinaryOperator::GreaterEqualThan) => Some("toBeGreaterThanOrEqual"),
        Some(BinaryOperator::LessThan) => Some("toBeLessThan"),
        Some(BinaryOperator::LessEqualThan) => Some("toBeLessThanOrEqual"),
        _ => None,
    }
}

fn invert_operator(operator: BinaryOperator) -> Option<BinaryOperator> {
    match operator {
        BinaryOperator::GreaterThan => Some(BinaryOperator::LessEqualThan),
        BinaryOperator::LessThan => Some(BinaryOperator::GreaterEqualThan),
        BinaryOperator::GreaterEqualThan => Some(BinaryOperator::LessThan),
        BinaryOperator::LessEqualThan => Some(BinaryOperator::GreaterThan),
        _ => None,
    }
}

fn building_code<'a>(
    binary_expr: &BinaryExpression<'a>,
    call_span_end: &str,
    arg_span_end: &str,
    local_name: &str,
    modifiers: &[&KnownMemberExpressionProperty<'a>],
    prefer_matcher_name: &str,
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

        if !modifier_name.eq("not") {
            content.print_str(&modifier_name);
            content.print_ascii_byte(b'.');
        }
    }
    content.print_str(prefer_matcher_name);
    content.print_ascii_byte(b'(');
    content.print_expression(&binary_expr.right);
    content.print_str(arg_span_end);
    content.into_source_text()
}
