use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, LogicalOperator, UnaryOperator};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    ast_util::outermost_paren_parent,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::is_same_expression,
};

fn prefer_style_diagnostic(span: Span, expected_style: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer {expected_style} conditional object spreads."))
        .with_help(format!("Rewrite this conditional spread to use the {expected_style} style."))
        .with_label(span)
}

fn unnecessary_boolean_cast_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unnecessary boolean cast. A falsy value spreads nothing.")
        .with_help("Remove the unnecessary boolean cast.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ConsistentConditionalObjectSpread(Box<ConsistentConditionalObjectSpreadConfig>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ConsistentConditionalObjectSpreadConfig {
    /// Prefer `...(condition && object)`, without a boolean cast of `condition`.
    #[default]
    Logical,
    /// Prefer `...(condition ? object : {})`.
    Ternary,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a consistent style for conditionally spreading properties into an object literal,
    /// either using logical `&&` (`...(condition && object)`) or a ternary (`...(condition ? object : {})`).
    ///
    /// ### Why is this bad?
    ///
    /// Mixing conditional object spread styles or adding redundant boolean casts makes object literals
    /// harder to read. In `'logical'` mode (the default), ternary spreads with an empty branch (`{}`,
    /// `undefined`, or `null`) and boolean casts on `&&` conditions are unnecessary because falsy values
    /// spread no properties.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const object = {...(condition ? {property} : {})};
    /// const object = {...(condition ? {property} : undefined)};
    /// const object = {...(!!condition && {property})};
    /// const object = {...(Boolean(condition) && {property})};
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const object = {...(condition && {property})};
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the `"ternary"` option:
    /// ```js
    /// /* eslint unicorn/consistent-conditional-object-spread: ["error", "ternary"] */
    /// const object = {...(condition && {property})};
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `"ternary"` option:
    /// ```js
    /// /* eslint unicorn/consistent-conditional-object-spread: ["error", "ternary"] */
    /// const object = {...(condition ? {property} : {})};
    /// ```
    ConsistentConditionalObjectSpread,
    unicorn,
    style,
    conditional_fix,
    config = ConsistentConditionalObjectSpreadConfig,
    version = "next",
    short_description = "Enforce consistent conditional object spread style.",
);

impl Rule for ConsistentConditionalObjectSpread {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ConditionalExpression(conditional_expr)
                if *self.0 == ConsistentConditionalObjectSpreadConfig::Logical
                    && is_object_spread_argument(node, ctx) =>
            {
                check_conditional_expression(conditional_expr, ctx);
            }
            AstKind::LogicalExpression(logical_expr)
                if logical_expr.operator == LogicalOperator::And
                    && is_object_spread_argument(node, ctx) =>
            {
                if *self.0 == ConsistentConditionalObjectSpreadConfig::Ternary {
                    check_logical_expression_for_ternary(logical_expr, ctx);
                } else {
                    check_boolean_cast_problems(logical_expr, ctx);
                }
            }
            _ => {}
        }
    }
}

fn is_object_spread_argument<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let Some(parent) = outermost_paren_parent(node, ctx) else {
        return false;
    };
    if !matches!(parent.kind(), AstKind::SpreadElement(_)) {
        return false;
    }
    let Some(grandparent) = outermost_paren_parent(parent, ctx) else {
        return false;
    };
    matches!(grandparent.kind(), AstKind::ObjectExpression(_))
}

fn is_empty_spread_branch(expr: &Expression<'_>, ctx: &LintContext<'_>) -> bool {
    match expr.without_parentheses() {
        Expression::ObjectExpression(object_expr) => object_expr.properties.is_empty(),
        Expression::NullLiteral(_) => true,
        Expression::Identifier(ident) => {
            ident.name == "undefined" && ident.is_global_reference(ctx.scoping())
        }
        _ => false,
    }
}

fn check_conditional_expression<'a>(
    conditional_expr: &oxc_ast::ast::ConditionalExpression<'a>,
    ctx: &LintContext<'a>,
) {
    let is_alternate_empty = is_empty_spread_branch(&conditional_expr.alternate, ctx);
    let is_consequent_empty = is_empty_spread_branch(&conditional_expr.consequent, ctx);

    if is_alternate_empty == is_consequent_empty {
        return;
    }

    let kept_branch =
        if is_alternate_empty { &conditional_expr.consequent } else { &conditional_expr.alternate };
    let test = &conditional_expr.test;
    let nullish_test = get_nullish_test(test, ctx);

    let is_same_as_test = if is_alternate_empty {
        is_same_node(test, kept_branch, ctx)
    } else {
        matches!(
            test.without_parentheses(),
            Expression::UnaryExpression(unary)
                if unary.operator == UnaryOperator::LogicalNot
                    && is_same_node(&unary.argument, kept_branch, ctx)
        )
    };

    if is_same_as_test
        || nullish_test.is_some_and(|nt| {
            (if is_alternate_empty { !nt.is_true_when_nullish } else { nt.is_true_when_nullish })
                && is_same_node(nt.reference, kept_branch, ctx)
        })
    {
        return;
    }

    let test_text = if is_alternate_empty {
        get_logical_operand_text(test, ctx)
    } else {
        get_negated_test_text(test, ctx)
    };
    let kept_branch_text = get_logical_operand_text(kept_branch, ctx);

    let preserved_test_span = if !is_alternate_empty
        && let Expression::UnaryExpression(unary) = test.without_parentheses()
        && unary.operator == UnaryOperator::LogicalNot
    {
        unary.argument.span()
    } else {
        test.span()
    };

    let diagnostic = prefer_style_diagnostic(conditional_expr.span, "logical");
    if would_remove_comments(conditional_expr.span, &[preserved_test_span, kept_branch.span()], ctx)
    {
        ctx.diagnostic(diagnostic);
    } else {
        ctx.diagnostic_with_fix(diagnostic, |fixer| {
            fixer.replace(conditional_expr.span, format!("{test_text} && {kept_branch_text}"))
        });
    }
}

fn check_logical_expression_for_ternary<'a>(
    logical_expr: &oxc_ast::ast::LogicalExpression<'a>,
    ctx: &LintContext<'a>,
) {
    if is_empty_spread_branch(&logical_expr.right, ctx) {
        return;
    }

    if is_same_node(&logical_expr.left, &logical_expr.right, ctx) {
        return;
    }

    if let Some(nullish_test) = get_nullish_test(&logical_expr.left, ctx)
        && !nullish_test.is_true_when_nullish
        && is_same_node(nullish_test.reference, &logical_expr.right, ctx)
    {
        return;
    }

    let test_text = get_conditional_child_text(&logical_expr.left, ctx);
    let consequent_text = get_conditional_child_text(&logical_expr.right, ctx);

    let diagnostic = prefer_style_diagnostic(logical_expr.span, "ternary");
    if would_remove_comments(
        logical_expr.span,
        &[logical_expr.left.span(), logical_expr.right.span()],
        ctx,
    ) {
        ctx.diagnostic(diagnostic);
    } else {
        ctx.diagnostic_with_fix(diagnostic, |fixer| {
            fixer.replace(logical_expr.span, format!("{test_text} ? {consequent_text} : {{}}"))
        });
    }
}

fn check_boolean_cast_problems<'a>(
    logical_expr: &oxc_ast::ast::LogicalExpression<'a>,
    ctx: &LintContext<'a>,
) {
    let mut current = &logical_expr.left;
    loop {
        if let Expression::LogicalExpression(inner) = current.without_parentheses()
            && inner.operator == LogicalOperator::And
        {
            check_boolean_cast_node(&inner.right, ctx);
            current = &inner.left;
        } else {
            check_boolean_cast_node(current, ctx);
            break;
        }
    }
}

fn check_boolean_cast_node<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) {
    let unwrapped = expr.without_parentheses();
    let Some(cast_argument) = get_boolean_cast_argument(unwrapped, ctx) else {
        return;
    };

    let cast_span = unwrapped.span();
    let diagnostic = unnecessary_boolean_cast_diagnostic(cast_span);
    if would_remove_comments(cast_span, &[cast_argument.span()], ctx) {
        ctx.diagnostic(diagnostic);
    } else {
        let replacement = get_logical_operand_text(cast_argument, ctx);
        ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.replace(cast_span, replacement));
    }
}

fn get_boolean_cast_argument<'a, 'b>(
    expr: &'b Expression<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'b Expression<'a>> {
    if let Expression::UnaryExpression(outer) = expr
        && outer.operator == UnaryOperator::LogicalNot
        && let Expression::UnaryExpression(inner) = outer.argument.without_parentheses()
        && inner.operator == UnaryOperator::LogicalNot
    {
        return Some(&inner.argument);
    }

    if let Expression::CallExpression(call) = expr
        && !call.optional
        && call.arguments.len() == 1
        && let Expression::Identifier(callee) = &call.callee
        && callee.name == "Boolean"
        && callee.is_global_reference(ctx.scoping())
    {
        return call.arguments[0].as_expression();
    }

    None
}

fn would_remove_comments(outer_span: Span, kept_spans: &[Span], ctx: &LintContext<'_>) -> bool {
    ctx.comments_range(outer_span.start..outer_span.end)
        .any(|comment| !kept_spans.iter().any(|kept| kept.contains_inclusive(comment.span)))
}

fn get_logical_operand_text(expr: &Expression<'_>, ctx: &LintContext<'_>) -> String {
    let text = ctx.source_range(expr.span());
    if !matches!(expr, Expression::ParenthesizedExpression(_))
        && should_add_parens_for_logical_and_child(expr)
    {
        format!("({text})")
    } else {
        text.to_string()
    }
}

fn should_add_parens_for_logical_and_child(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::LogicalExpression(logical) => logical.operator != LogicalOperator::And,
        Expression::AwaitExpression(_)
        | Expression::BinaryExpression(_)
        | Expression::ConditionalExpression(_)
        | Expression::AssignmentExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::YieldExpression(_)
        | Expression::SequenceExpression(_) => true,
        _ => false,
    }
}

fn get_conditional_child_text(expr: &Expression<'_>, ctx: &LintContext<'_>) -> String {
    let text = ctx.source_range(expr.span());
    if !matches!(expr, Expression::ParenthesizedExpression(_))
        && should_add_parens_for_conditional_child(expr)
    {
        format!("({text})")
    } else {
        text.to_string()
    }
}

fn should_add_parens_for_conditional_child(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::AwaitExpression(_)
            | Expression::AssignmentExpression(_)
            | Expression::YieldExpression(_)
            | Expression::SequenceExpression(_)
            | Expression::TSAsExpression(_)
            | Expression::TSTypeAssertion(_)
    )
}

fn get_negated_test_text(test: &Expression<'_>, ctx: &LintContext<'_>) -> String {
    if let Expression::UnaryExpression(unary) = test.without_parentheses()
        && unary.operator == UnaryOperator::LogicalNot
    {
        return get_logical_operand_text(&unary.argument, ctx);
    }

    let inner = test.without_parentheses();
    if matches!(test, Expression::ParenthesizedExpression(_))
        && would_remove_comments(test.span(), &[inner.span()], ctx)
    {
        return format!("!{}", ctx.source_range(test.span()));
    }

    let inner_text = ctx.source_range(inner.span());
    if should_add_parens_for_logical_not(inner) {
        format!("!({inner_text})")
    } else {
        format!("!{inner_text}")
    }
}

fn should_add_parens_for_logical_not(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::SequenceExpression(_)
            | Expression::AssignmentExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::YieldExpression(_)
            | Expression::ConditionalExpression(_)
            | Expression::LogicalExpression(_)
            | Expression::BinaryExpression(_)
            | Expression::TSAsExpression(_)
            | Expression::TSSatisfiesExpression(_)
            | Expression::UpdateExpression(_)
            | Expression::TSNonNullExpression(_)
            | Expression::TSTypeAssertion(_)
    )
}

fn is_same_node(left: &Expression<'_>, right: &Expression<'_>, ctx: &LintContext<'_>) -> bool {
    let left = left.without_parentheses();
    let right = right.without_parentheses();

    if is_same_expression(left, right, ctx) {
        return true;
    }

    match (left, right) {
        (Expression::AwaitExpression(l), Expression::AwaitExpression(r)) => {
            is_same_node(&l.argument, &r.argument, ctx)
        }
        (Expression::LogicalExpression(l), Expression::LogicalExpression(r)) => {
            l.operator == r.operator
                && is_same_node(&l.left, &r.left, ctx)
                && is_same_node(&l.right, &r.right, ctx)
        }
        (Expression::UnaryExpression(l), Expression::UnaryExpression(r)) => {
            l.operator == r.operator && is_same_node(&l.argument, &r.argument, ctx)
        }
        (Expression::UpdateExpression(_), Expression::UpdateExpression(_)) => false,
        _ => {
            std::mem::discriminant(left) == std::mem::discriminant(right)
                && ctx.source_range(left.span()) == ctx.source_range(right.span())
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NullishKind {
    Null,
    Undefined,
    Nullish,
}

struct NullishTest<'a, 'b> {
    reference: &'b Expression<'a>,
    is_true_when_nullish: bool,
}

struct NullishBinaryCheck<'a, 'b> {
    reference: &'b Expression<'a>,
    kind: NullishKind,
    is_true_when_nullish: bool,
}

fn get_nullish_kind(expr: &Expression<'_>, ctx: &LintContext<'_>) -> Option<NullishKind> {
    match expr.without_parentheses() {
        Expression::NullLiteral(_) => Some(NullishKind::Null),
        Expression::Identifier(ident)
            if ident.name == "undefined" && ident.is_global_reference(ctx.scoping()) =>
        {
            Some(NullishKind::Undefined)
        }
        _ => None,
    }
}

fn get_nullish_binary_check<'a, 'b>(
    expr: &'b Expression<'a>,
    ctx: &LintContext<'a>,
) -> Option<NullishBinaryCheck<'a, 'b>> {
    let Expression::BinaryExpression(bin) = expr.without_parentheses() else {
        return None;
    };

    let (is_true_when_nullish, is_loose) = match bin.operator {
        BinaryOperator::Equality => (true, true),
        BinaryOperator::StrictEquality => (true, false),
        BinaryOperator::Inequality => (false, true),
        BinaryOperator::StrictInequality => (false, false),
        _ => return None,
    };

    let left_kind = get_nullish_kind(&bin.left, ctx);
    let right_kind = get_nullish_kind(&bin.right, ctx);

    if left_kind.is_some() == right_kind.is_some() {
        return None;
    }

    let reference = if left_kind.is_some() { &bin.right } else { &bin.left };
    let kind = if is_loose { NullishKind::Nullish } else { left_kind.or(right_kind)? };

    Some(NullishBinaryCheck { reference, kind, is_true_when_nullish })
}

fn get_nullish_test<'a, 'b>(
    expr: &'b Expression<'a>,
    ctx: &LintContext<'a>,
) -> Option<NullishTest<'a, 'b>> {
    if let Some(binary_check) = get_nullish_binary_check(expr, ctx)
        && binary_check.kind == NullishKind::Nullish
    {
        return Some(NullishTest {
            reference: binary_check.reference,
            is_true_when_nullish: binary_check.is_true_when_nullish,
        });
    }

    let Expression::LogicalExpression(logical) = expr.without_parentheses() else {
        return None;
    };

    let left = get_nullish_binary_check(&logical.left, ctx)?;
    let right = get_nullish_binary_check(&logical.right, ctx)?;

    if !is_same_node(left.reference, right.reference, ctx) {
        return None;
    }

    let checks_null_and_undefined = matches!(
        (left.kind, right.kind),
        (NullishKind::Null, NullishKind::Undefined) | (NullishKind::Undefined, NullishKind::Null)
    );
    if !checks_null_and_undefined {
        return None;
    }

    match logical.operator {
        LogicalOperator::Or if left.is_true_when_nullish && right.is_true_when_nullish => {
            Some(NullishTest { reference: left.reference, is_true_when_nullish: true })
        }
        LogicalOperator::And if !left.is_true_when_nullish && !right.is_true_when_nullish => {
            Some(NullishTest { reference: left.reference, is_true_when_nullish: false })
        }
        _ => None,
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        ("const object = {...(foo && {bar: true})}", None, None, None),
        ("const object = {...(!foo && {bar: true})}", None, None, None),
        ("const object = {...(foo && bar)}", None, None, None),
        ("const object = {...(foo || {})}", None, None, None),
        ("const object = {...(foo ?? {})}", None, None, None),
        ("const object = {...(foo && {})}", None, None, None),
        ("const object = {...(foo ? {bar: true} : {baz: true})}", None, None, None),
        ("const object = {...(foo ? {} : {})}", None, None, None),
        ("const object = {...(foo ? foo : {})}", None, None, None),
        ("const object = {...(!foo ? {} : foo)}", None, None, None),
        ("const object = {...(foo.bar ? foo.bar : {})}", None, None, None),
        ("const object = {...(foo() ? foo() : {})}", None, None, None),
        ("async () => ({...(await foo ? await foo : {})})", None, None, None),
        ("const object = {...(foo == null ? {} : foo)}", None, None, None),
        ("const object = {...(foo != null ? foo : {})}", None, None, None),
        ("const object = {...(foo === null || foo === undefined ? {} : foo)}", None, None, None),
        ("const object = {...(foo !== null && foo !== undefined ? foo : {})}", None, None, None),
        ("const object = {...(foo == null ? undefined : foo)}", None, None, None),
        ("const object = {...(foo == null ? null : foo)}", None, None, None),
        ("const object = {...(foo != null ? foo : null)}", None, None, None),
        ("const object = {...(foo ? undefined : undefined)}", None, None, None),
        ("const object = {...(foo ? null : null)}", None, None, None),
        ("const object = {...(foo ? {bar: true} : void 0)}", None, None, None),
        ("const array = [...(foo ? {bar: true} : {})]", None, None, None),
        ("const object = {...(foo ? {bar: true} : {}).bar}", None, None, None),
        ("const object = {...(Boolean() && {a: 1})}", None, None, None),
        ("const object = {...(Boolean(foo, bar) && {a: 1})}", None, None, None),
        ("const object = {...(Boolean(...foo) && {a: 1})}", None, None, None),
        ("const object = {...(Boolean?.(foo) && {a: 1})}", None, None, None),
        ("const object = {...(new Boolean(foo) && {a: 1})}", None, None, None),
        ("const object = {...(foo.Boolean(bar) && {a: 1})}", None, None, None),
        ("const object = {...(Number(foo) && {a: 1})}", None, None, None),
        (
            "const Boolean = value => value; const object = {...(Boolean(foo) && {a: 1})}",
            None,
            None,
            None,
        ),
        ("const object = {...(!Boolean(foo) && {a: 1})}", None, None, None),
        ("const object = {...(!!foo || {a: 1})}", None, None, None),
        ("const object = {...(!!foo ?? {a: 1})}", None, None, None),
        ("const object = {...(foo && !!bar)}", None, None, None),
        ("const object = {...!!foo}", None, None, None),
        ("const object = {...(!!foo && bar).baz}", None, None, None),
        ("const array = [...(!!foo && [1])]", None, None, None),
        ("const value = !!foo && {a: 1}", None, None, None),
        ("foo(!!bar && {a: 1})", None, None, None),
        ("const object = {...((!!foo ?? bar) && {a: 1})}", None, None, None),
        ("const object = {...((!!foo || bar) && {a: 1})}", None, None, None),
        ("const object = {...(foo && (!!bar && {a: 1}))}", None, None, None),
        ("const object = {...((!!foo as boolean) && {a: 1})}", None, None, None),
        ("const element = <div {...(!!foo && {a: 1})} />", None, None, None),
        (
            "const object = {...(foo && {bar: true})}",
            Some(serde_json::json!(["logical"])),
            None,
            None,
        ),
        (
            "const object = {...(foo ? {bar: true} : {})}",
            Some(serde_json::json!(["ternary"])),
            None,
            None,
        ),
        (
            "const object = {...(foo && undefined)}",
            Some(serde_json::json!(["ternary"])),
            None,
            None,
        ),
        ("const object = {...(foo && null)}", Some(serde_json::json!(["ternary"])), None, None),
        ("const object = {...(foo && foo)}", Some(serde_json::json!(["ternary"])), None, None),
        (
            "const object = {...(foo.bar && foo.bar)}",
            Some(serde_json::json!(["ternary"])),
            None,
            None,
        ),
        ("const object = {...(foo() && foo())}", Some(serde_json::json!(["ternary"])), None, None),
        (
            "const object = {...(foo != null && foo)}",
            Some(serde_json::json!(["ternary"])),
            None,
            None,
        ),
        (
            "const object = {...((foo !== null && foo !== undefined) && foo)}",
            Some(serde_json::json!(["ternary"])),
            None,
            None,
        ),
        (
            "const object = {...(foo !== null && foo !== undefined && foo)}",
            Some(serde_json::json!(["ternary"])),
            None,
            None,
        ),
        (
            "async () => ({...(await foo && await foo)})",
            Some(serde_json::json!(["ternary"])),
            None,
            None,
        ),
        ("const object = {...(foo || bar ? foo || bar : {})}", None, None, None),
        ("const object = {...(!foo ? !foo : {})}", None, None, None),
        ("const object = {...(null == foo ? {} : foo)}", None, None, None),
        ("const object = {...(undefined === foo || null === foo ? {} : foo)}", None, None, None),
        ("const object = {...(foo == null ? /* keep */ {} : foo)}", None, None, None),
        (
            "const undefined = 1; const object = {...(foo ? {bar: true} : undefined)}",
            None,
            None,
            None,
        ),
    ];

    let fail = vec![
        ("const object = {...(flag ? {/* keep */ value} : {})};", None, None, None),
        ("const object = {...(!(/* keep */ flag) ? {} : data)};", None, None, None),
        (
            "const object = {...(read(/* keep */) && data)};",
            Some(serde_json::json!(["ternary"])),
            None,
            None,
        ),
        ("const object = {...(Boolean(read(/* keep */)) && data)};", None, None, None),
        ("const object = {...(Boolean(value as /* keep */ string) && data)};", None, None, None),
        (
            "const object = {...((flag as /* keep */ boolean) && data)};",
            Some(serde_json::json!(["ternary"])),
            None,
            None,
        ),
        ("const object = {...(foo ? {bar: true} : {})}", None, None, None),
        ("const object = {...(foo ? {} : {bar: true})}", None, None, None),
        ("const object = {...(!foo ? {} : {bar: true})}", None, None, None),
        ("const object = {...(foo ? bar : {})}", None, None, None),
        ("const object = {...(foo ? {} : bar)}", None, None, None),
        ("const object = {...(a || b ? {x: 1} : {})}", None, None, None),
        ("const object = {...(a && b ? {} : {x: 1})}", None, None, None),
        ("const object = {...(a ?? b ? {x: 1} : {})}", None, None, None),
        ("async () => ({...(await foo ? {a: 1} : {})})", None, None, None),
        ("const object = {...((a, b) ? {x: 1} : {})}", None, None, None),
        ("const object = {...((a, b) ? {} : {x: 1})}", None, None, None),
        ("const object = {...(foo ? (() => x) : {})}", None, None, None),
        ("const object = {...(foo ? a || b : {})}", None, None, None),
        ("const object = {...(foo ? a ?? b : {})}", None, None, None),
        ("const object = {...(foo ? {} : a ?? b)}", None, None, None),
        ("const object = {...(foo?.bar ? {a: 1} : {})}", None, None, None),
        ("const object = {...((( foo )) ? (( {a: 1} )) : (( {} )))}", None, None, None),
        ("const object = ({...foo ? {a: 1} : {}})", None, None, None),
        ("const object = {...(!!foo ? {} : {bar: true})}", None, None, None),
        ("const object = {...(foo ? bar! : {})}", None, None, None),
        ("const object = {...(foo as string ? {} : bar)}", None, None, None),
        ("const object = {...(foo satisfies string ? {} : bar)}", None, None, None),
        (
            "const object = {...(<string>foo ? {} : bar)}",
            None,
            None,
            Some(PathBuf::from("test.ts")),
        ),
        ("const object = {...(foo! ? {} : bar)}", None, None, None),
        ("const object = {...(foo ? {bar: true} : undefined)}", None, None, None),
        ("const object = {...(foo ? {bar: true} : null)}", None, None, None),
        ("const object = {...(foo ? undefined : {bar: true})}", None, None, None),
        ("const object = {...(!foo ? undefined : {bar: true})}", None, None, None),
        ("const object = {...(foo ? null : {bar: true})}", None, None, None),
        ("const object = {...(foo ? bar : undefined)}", None, None, None),
        ("const object = {...(foo ? null : bar)}", None, None, None),
        ("const object = {...(foo == null ? undefined : bar)}", None, None, None),
        ("const object = {...(foo != null ? bar : undefined)}", None, None, None),
        ("const object = {...(foo ? {a: 1} : /* keep */ {})}", None, None, None),
        ("const object = {...(foo ? {a: 1} : /* keep */ undefined)}", None, None, None),
        ("const object = {...(foo ? {a: 1} : /* keep */ null)}", None, None, None),
        ("const object = {...(/* keep */ foo ? {a: 1} : {})}", None, None, None),
        (
            "const object = {...(foo && {bar: true})}",
            Some(serde_json::json!(["ternary"])),
            None,
            None,
        ),
        (
            "const object = {...(!foo && {bar: true})}",
            Some(serde_json::json!(["ternary"])),
            None,
            None,
        ),
        ("const object = {...(foo && bar)}", Some(serde_json::json!(["ternary"])), None, None),
        (
            "const object = {...((a || b) && {x: 1})}",
            Some(serde_json::json!(["ternary"])),
            None,
            None,
        ),
        (
            "const object = {...((a, b) && {x: 1})}",
            Some(serde_json::json!(["ternary"])),
            None,
            None,
        ),
        (
            "async () => ({...(await foo && {a: 1})})",
            Some(serde_json::json!(["ternary"])),
            None,
            None,
        ),
        (
            "const object = {...(foo && (() => x))}",
            Some(serde_json::json!(["ternary"])),
            None,
            None,
        ),
        ("const object = {...(foo && (a = b))}", Some(serde_json::json!(["ternary"])), None, None),
        ("const object = {...(foo && (a, b))}", Some(serde_json::json!(["ternary"])), None, None),
        ("const object = {...(foo && bar!)}", Some(serde_json::json!(["ternary"])), None, None),
        (
            "const object = {...(foo /* keep */ && {bar: true})}",
            Some(serde_json::json!(["ternary"])),
            None,
            None,
        ),
        ("const object = {...(!!foo && {bar: true})}", None, None, None),
        ("const object = {...!!foo && {bar: true}}", None, None, None),
        ("const object = {...(Boolean(foo) && {bar: true})}", None, None, None),
        ("const object = {...(!!foo && bar)}", None, None, None),
        ("const object = {...(!!foo.bar && {a: 1})}", None, None, None),
        ("const object = {...(!!foo() && {a: 1})}", None, None, None),
        ("const object = {...(!!(foo) && {a: 1})}", None, None, None),
        ("const object = {...((!!foo) && {a: 1})}", None, None, None),
        ("const object = {...(!(!foo) && {a: 1})}", None, None, None),
        ("const object = {...(!!!foo && {a: 1})}", None, None, None),
        ("const object = {...(Boolean(!!foo) && {a: 1})}", None, None, None),
        ("const object = {...(!!(a || b) && {x: 1})}", None, None, None),
        ("const object = {...(Boolean(a || b) && {x: 1})}", None, None, None),
        ("const object = {...(!!(a ?? b) && {x: 1})}", None, None, None),
        ("const object = {...(Boolean(a ?? b) && {x: 1})}", None, None, None),
        ("const object = {...(!!(a && b) && {x: 1})}", None, None, None),
        ("const object = {...(Boolean(a && b) && {x: 1})}", None, None, None),
        ("const object = {...(Boolean((a, b)) && {x: 1})}", None, None, None),
        ("const object = {...(Boolean(a = b) && {x: 1})}", None, None, None),
        ("const object = {...(Boolean(a ? b : c) && {x: 1})}", None, None, None),
        ("const object = {...((!!(a || b)) && {x: 1})}", None, None, None),
        ("async () => ({...(!!await foo && {a: 1})})", None, None, None),
        ("const object = {...(!!foo && {a: 1}), ...(Boolean(bar) && {b: 2})}", None, None, None),
        ("const object = {...(!! /* keep */ foo && {a: 1})}", None, None, None),
        ("const object = {...(Boolean(foo /* keep */) && {a: 1})}", None, None, None),
        ("const object = {...(!!(/* keep */ foo) && {a: 1})}", None, None, None),
        ("const object = {...(!!foo ? {a: 1} : {})}", None, None, None),
        ("const object = {...(!!foo! && {a: 1})}", None, None, None),
        ("const object = {...(Boolean(foo as string) && {a: 1})}", None, None, None),
        ("const object = {...(Boolean(foo satisfies string) && {a: 1})}", None, None, None),
        (
            "const object = {...(!!<string>foo && {a: 1})}",
            None,
            None,
            Some(PathBuf::from("test.ts")),
        ),
        ("const object = {...(!!foo && {a: 1})}", Some(serde_json::json!(["ternary"])), None, None),
        ("const object = {...(!!foo?.bar && {a: 1})}", None, None, None),
        ("const object = {...(!!foo && bar && {a: 1})}", None, None, None),
        ("const object = {...(foo && !!bar && {a: 1})}", None, None, None),
        ("const object = {...(!!a && Boolean(b) && !!c && {x: 1})}", None, None, None),
        ("const object = {...(a && !!(b || c) && {x: 1})}", None, None, None),
        ("const object = {...(a && Boolean(b ?? c) && {x: 1})}", None, None, None),
        ("const object = {...(a && Boolean(b && c) && {x: 1})}", None, None, None),
        ("const object = {...(!!a && !! /* keep */ b && {x: 1})}", None, None, None),
        ("const object = {...(!!foo /* keep */ && {a: 1})}", None, None, None),
        ("const object = {...(!!foo && {})}", None, None, None),
        ("const element = <div style={{...(!!foo && {color: 'red'})}} />", None, None, None),
        (
            "function * generator() {\n\treturn {...(Boolean(yield foo) && {a: 1})};\n}",
            None,
            None,
            None,
        ),
        ("const object = {...(Boolean(foo) ? {} : {a: 1})}", None, None, None),
        ("const object = {...(foo++ ? foo++ : {})}", None, None, None),
        ("const object = {...(foo || bar ? foo && bar : {})}", None, None, None),
        ("const object = {...(foo === bar ? {} : foo)}", None, None, None),
    ];

    let fix = vec![
        (
            "const object = {...(flag ? {/* keep */ value} : {})};",
            "const object = {...(flag && {/* keep */ value})};",
            None,
        ),
        (
            "const object = {...(!(/* keep */ flag) ? {} : data)};",
            "const object = {...((/* keep */ flag) && data)};",
            None,
        ),
        (
            "const object = {...(read(/* keep */) && data)};",
            "const object = {...(read(/* keep */) ? data : {})};",
            Some(serde_json::json!(["ternary"])),
        ),
        (
            "const object = {...(Boolean(read(/* keep */)) && data)};",
            "const object = {...(read(/* keep */) && data)};",
            None,
        ),
        (
            "const object = {...(Boolean(value as /* keep */ string) && data)};",
            "const object = {...(value as /* keep */ string && data)};",
            None,
        ),
        (
            "const object = {...((flag as /* keep */ boolean) && data)};",
            "const object = {...((flag as /* keep */ boolean) ? data : {})};",
            Some(serde_json::json!(["ternary"])),
        ),
        (
            "const object = {...(foo ? {bar: true} : {})}",
            "const object = {...(foo && {bar: true})}",
            None,
        ),
        (
            "const object = {...(foo ? {} : {bar: true})}",
            "const object = {...(!foo && {bar: true})}",
            None,
        ),
        (
            "const object = {...(!foo ? {} : {bar: true})}",
            "const object = {...(foo && {bar: true})}",
            None,
        ),
        (
            "const object = {...(a && b ? {} : {x: 1})}",
            "const object = {...(!(a && b) && {x: 1})}",
            None,
        ),
        (
            "async () => ({...(await foo ? {a: 1} : {})})",
            "async () => ({...((await foo) && {a: 1})})",
            None,
        ),
        (
            "const object = {...((a, b) ? {} : {x: 1})}",
            "const object = {...(!(a, b) && {x: 1})}",
            None,
        ),
        ("const object = {...(foo ? a ?? b : {})}", "const object = {...(foo && (a ?? b))}", None),
        (
            "const object = {...((( foo )) ? (( {a: 1} )) : (( {} )))}",
            "const object = {...((( foo )) && (( {a: 1} )))}",
            None,
        ),
        (
            "const object = {...(foo as string ? {} : bar)}",
            "const object = {...(!(foo as string) && bar)}",
            None,
        ),
        ("const object = {...(foo! ? {} : bar)}", "const object = {...(!(foo!) && bar)}", None),
        (
            "const object = {...(foo ? {bar: true} : undefined)}",
            "const object = {...(foo && {bar: true})}",
            None,
        ),
        (
            "const object = {...(foo ? null : {bar: true})}",
            "const object = {...(!foo && {bar: true})}",
            None,
        ),
        (
            "const object = {...(/* keep */ foo ? {a: 1} : {})}",
            "const object = {...(/* keep */ foo && {a: 1})}",
            None,
        ),
        (
            "const object = {...(foo ? {a: 1} : /* keep */ {})}",
            "const object = {...(foo ? {a: 1} : /* keep */ {})}",
            None,
        ),
        (
            "const object = {...(foo && {bar: true})}",
            "const object = {...(foo ? {bar: true} : {})}",
            Some(serde_json::json!(["ternary"])),
        ),
        (
            "async () => ({...(await foo && {a: 1})})",
            "async () => ({...((await foo) ? {a: 1} : {})})",
            Some(serde_json::json!(["ternary"])),
        ),
        (
            "const object = {...(foo /* keep */ && {bar: true})}",
            "const object = {...(foo /* keep */ && {bar: true})}",
            Some(serde_json::json!(["ternary"])),
        ),
        (
            "const object = {...(!!foo && {bar: true})}",
            "const object = {...(foo && {bar: true})}",
            None,
        ),
        (
            "const object = {...(Boolean(foo) && {bar: true})}",
            "const object = {...(foo && {bar: true})}",
            None,
        ),
        ("const object = {...(!(!foo) && {a: 1})}", "const object = {...(foo && {a: 1})}", None),
        ("const object = {...(!!!foo && {a: 1})}", "const object = {...(!foo && {a: 1})}", None),
        (
            "const object = {...(Boolean(!!foo) && {a: 1})}",
            "const object = {...(!!foo && {a: 1})}",
            None,
        ),
        (
            "const object = {...(Boolean(a && b) && {x: 1})}",
            "const object = {...(a && b && {x: 1})}",
            None,
        ),
        (
            "const object = {...(!! /* keep */ foo && {a: 1})}",
            "const object = {...(!! /* keep */ foo && {a: 1})}",
            None,
        ),
        (
            "const object = {...(!!(/* keep */ foo) && {a: 1})}",
            "const object = {...((/* keep */ foo) && {a: 1})}",
            None,
        ),
        (
            "const object = {...(!!a && Boolean(b) && !!c && {x: 1})}",
            "const object = {...(a && b && c && {x: 1})}",
            None,
        ),
        (
            "const object = {...(!!a && !! /* keep */ b && {x: 1})}",
            "const object = {...(a && !! /* keep */ b && {x: 1})}",
            None,
        ),
        (
            "const object = {...(!!foo /* keep */ && {a: 1})}",
            "const object = {...(foo /* keep */ && {a: 1})}",
            None,
        ),
        (
            "function * generator() {\n\treturn {...(Boolean(yield foo) && {a: 1})};\n}",
            "function * generator() {\n\treturn {...((yield foo) && {a: 1})};\n}",
            None,
        ),
        (
            "const object = {...(foo || bar ? foo && bar : {})}",
            "const object = {...((foo || bar) && foo && bar)}",
            None,
        ),
    ];

    Tester::new(
        ConsistentConditionalObjectSpread::NAME,
        ConsistentConditionalObjectSpread::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
