use oxc_ast::{
    AstKind,
    ast::{ConditionalExpression, Expression, IfStatement, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    identifier::is_identifier_part,
    line_terminator::is_line_terminator,
    operator::{BinaryOperator, UnaryOperator},
};

use crate::{
    AstNode,
    ast_util::could_be_asi_hazard,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
};

fn no_negated_condition_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected negated condition.")
        .with_help("Remove the negation operator and switch the consequent and alternate branches.")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

Disallow negated conditions.

### Why is this bad?

Negated conditions are more difficult to understand. Code can be made more readable by inverting the condition.

### Examples

Examples of **incorrect** code for this rule:
```javascript
if (!a) {
	doSomethingC();
} else {
	doSomethingB();
}

!a ? doSomethingC() : doSomethingB()
```

Examples of **correct** code for this rule:
```javascript
if (a) {
	doSomethingB();
} else {
	doSomethingC();
}

a ? doSomethingB() : doSomethingC()
```
";

pub fn run_on_if_statement<'a>(if_stmt: &IfStatement<'a>, ctx: &LintContext<'a>) {
    let Some(alternate) = &if_stmt.alternate else {
        return;
    };

    if matches!(alternate, Statement::IfStatement(_)) {
        return;
    }

    let test = if_stmt.test.without_parentheses();
    if !is_negated_expression(test) {
        return;
    }

    ctx.diagnostic_with_fix(no_negated_condition_diagnostic(test.span()), |fixer| {
        fix_if_statement(fixer, if_stmt, alternate, test, ctx)
    });
}

pub fn run_on_conditional_expression<'a>(
    node: &AstNode<'a>,
    conditional_expr: &ConditionalExpression<'a>,
    ctx: &LintContext<'a>,
) {
    let test = conditional_expr.test.without_parentheses();
    if !is_negated_expression(test) {
        return;
    }

    ctx.diagnostic_with_fix(no_negated_condition_diagnostic(test.span()), |fixer| {
        fix_conditional_expression(fixer, node, conditional_expr, test, ctx)
    });
}

fn is_negated_expression(expr: &Expression) -> bool {
    match expr {
        Expression::UnaryExpression(unary_expr) => unary_expr.operator == UnaryOperator::LogicalNot,
        Expression::BinaryExpression(binary_expr) => matches!(
            binary_expr.operator,
            BinaryOperator::Inequality | BinaryOperator::StrictInequality
        ),
        _ => false,
    }
}

/// Span-local edits only: invert the negated test in place, then swap branches.
/// Avoids rebuilding the whole `if` / ternary (important for large branch bodies).
fn fix_if_statement<'a>(
    fixer: RuleFixer<'_, 'a>,
    if_stmt: &IfStatement<'a>,
    alternate: &Statement<'a>,
    negated_test: &Expression<'a>,
    ctx: &LintContext<'a>,
) -> RuleFix {
    let fixer = fixer.for_multifix();
    // invert test (1–2) + swap branches (2) — capacity 4 is enough
    let mut fixes = fixer.new_fix_with_capacity(4);

    push_invert_test_fixes(&mut fixes, &fixer, negated_test, ctx);
    push_swap_statement_branches(&mut fixes, &fixer, &if_stmt.consequent, alternate, ctx);

    fixes.with_message(
        "Remove the negation operator and switch the consequent and alternate branches.",
    )
}

fn fix_conditional_expression<'a>(
    fixer: RuleFixer<'_, 'a>,
    node: &AstNode<'a>,
    conditional_expr: &ConditionalExpression<'a>,
    negated_test: &Expression<'a>,
    ctx: &LintContext<'a>,
) -> RuleFix {
    let fixer = fixer.for_multifix();
    let mut fixes = fixer.new_fix_with_capacity(10);

    let is_unary = matches!(negated_test, Expression::UnaryExpression(_));
    let source = ctx.source_text();
    let expr_span = conditional_expr.span;

    // Detect ASI / keyword spacing from what the expression will look like after removing `!`.
    let (needs_space_before, needs_asi_semi, needs_restricted_parens, needs_exposed_expr_parens) =
        if is_unary {
            let Expression::UnaryExpression(unary) = negated_test else { unreachable!() };
            // Text after deleting the leading `!` (preserves comments between `!` and operand).
            let after_bang = ctx.source_range(Span::new(unary.span.start + 1, unary.span.end));
            let first_significant = after_bang.trim_start().chars().next();
            let has_line_terminator_in_remainder = after_bang.chars().any(is_line_terminator);

            let needs_restricted_parens = has_line_terminator_in_remainder
                && !matches!(conditional_expr.test, Expression::ParenthesizedExpression(_))
                && is_restricted_statement_or_yield_argument(node, ctx);

            let needs_space_before = !needs_restricted_parens && {
                let start = expr_span.start as usize;
                start > 0
                    && source[..start].chars().next_back().is_some_and(can_continue_token)
                    && first_significant.is_some_and(needs_separator_after_keyword)
            };

            let needs_asi_semi = !needs_restricted_parens
                && !needs_space_before
                && matches!(first_significant, Some('(' | '[' | '`' | '+' | '-' | '/'))
                && could_be_asi_hazard(node, ctx);

            let needs_exposed_expr_parens = is_expression_statement_start(node, ctx)
                && matches!(
                    unary.argument.without_parentheses(),
                    Expression::ObjectExpression(_)
                        | Expression::FunctionExpression(_)
                        | Expression::ClassExpression(_)
                );

            (needs_space_before, needs_asi_semi, needs_restricted_parens, needs_exposed_expr_parens)
        } else {
            (false, false, false, false)
        };

    if needs_asi_semi {
        fixes.push(fixer.insert_text_before_range(expr_span, ";"));
    } else if needs_space_before {
        fixes.push(fixer.insert_text_before_range(expr_span, " "));
    }

    if needs_restricted_parens {
        fixes.push(fixer.insert_text_before_range(expr_span, "("));
        fixes.push(fixer.insert_text_after_range(expr_span, ")"));
    }

    if needs_exposed_expr_parens && let Expression::UnaryExpression(unary) = negated_test {
        let argument_span = unary.argument.span();
        fixes.push(fixer.insert_text_before_range(argument_span, "("));
        fixes.push(fixer.insert_text_after_range(argument_span, ")"));
    }

    push_invert_test_fixes(&mut fixes, &fixer, negated_test, ctx);
    push_swap_expression_branches(
        &mut fixes,
        &fixer,
        node,
        conditional_expr,
        &conditional_expr.consequent,
        &conditional_expr.alternate,
        ctx,
    );

    fixes.with_message(
        "Remove the negation operator and switch the consequent and alternate branches.",
    )
}

#[expect(clippy::cast_possible_truncation)]
fn push_invert_test_fixes<'a>(
    fixes: &mut RuleFix,
    fixer: &RuleFixer<'_, 'a>,
    negated_test: &Expression<'a>,
    ctx: &LintContext<'a>,
) {
    match negated_test {
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            // Delete only the `!` punctuator (1 byte) so comments/whitespace after it stay.
            fixes.push(fixer.delete_range(Span::new(unary.span.start, unary.span.start + 1)));
        }
        Expression::BinaryExpression(binary) => {
            let binary_operator_str = binary.operator.as_str();
            if let Some(op_span) = ctx
                .find_next_token_within(
                    binary.left.span().end,
                    binary.right.span().start,
                    binary_operator_str,
                )
                .map(|s| Span::sized(binary.left.span().end + s, binary_operator_str.len() as u32))
                && let Some(inverse_op) = binary.operator.equality_inverse_operator()
            {
                fixes.push(fixer.replace(op_span, inverse_op.as_str()));
            }
        }
        _ => {}
    }
}

fn push_swap_statement_branches<'a>(
    fixes: &mut RuleFix,
    fixer: &RuleFixer<'_, 'a>,
    consequent: &Statement<'a>,
    alternate: &Statement<'a>,
    ctx: &LintContext<'a>,
) {
    let cons_span = consequent.span();
    let alt_span = alternate.span();
    let cons_src = ctx.source_range(cons_span);
    let alt_src = ctx.source_range(alt_span);

    // Identical branches: only the test needs changing.
    if cons_src == alt_src
        && matches!(consequent, Statement::BlockStatement(_))
            == matches!(alternate, Statement::BlockStatement(_))
    {
        return;
    }

    let cons_is_block = matches!(consequent, Statement::BlockStatement(_));
    let alt_is_block = matches!(alternate, Statement::BlockStatement(_));

    // Non-block branches must be wrapped after the swap so ASI / dangling `else` stay valid.
    let new_cons = if alt_is_block { alt_src.to_string() } else { format!("{{{alt_src}}}") };
    let new_alt = if cons_is_block { cons_src.to_string() } else { format!("{{{cons_src}}}") };

    fixes.push(fixer.replace(cons_span, new_cons));
    fixes.push(fixer.replace(alt_span, new_alt));
}

fn push_swap_expression_branches<'a>(
    fixes: &mut RuleFix,
    fixer: &RuleFixer<'_, 'a>,
    node: &AstNode<'a>,
    conditional_expr: &ConditionalExpression<'a>,
    consequent: &Expression<'a>,
    alternate: &Expression<'a>,
    ctx: &LintContext<'a>,
) {
    let cons_span = consequent.span();
    let alt_span = alternate.span();
    let cons_src = ctx.source_range(cons_span);
    let alt_src = ctx.source_range(alt_span);

    if cons_src == alt_src {
        return;
    }

    // Must own the replacement strings because both edits reference the other branch's text.
    fixes.push(fixer.replace(cons_span, alt_src.to_string()));
    let new_alt = if is_for_statement_init(node, conditional_expr, ctx)
        && contains_unparenthesized_in_expression(consequent)
    {
        format!("({cons_src})")
    } else {
        cons_src.to_string()
    };
    fixes.push(fixer.replace(alt_span, new_alt));
}

fn is_restricted_statement_or_yield_argument(node: &AstNode, ctx: &LintContext) -> bool {
    matches!(
        ctx.nodes().parent_kind(node.id()),
        AstKind::ReturnStatement(_) | AstKind::ThrowStatement(_) | AstKind::YieldExpression(_)
    )
}

fn can_continue_token(c: char) -> bool {
    is_identifier_part(c) || c.is_ascii_digit()
}

fn needs_separator_after_keyword(c: char) -> bool {
    is_identifier_part(c) || c == '\\' || matches!(c, '(' | '[' | '`' | '/' | '+' | '-' | '.')
}

fn is_expression_statement_start(node: &AstNode, ctx: &LintContext) -> bool {
    let node_span = node.span();
    for ancestor in ctx.nodes().ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::ExpressionStatement(expr_stmt) => {
                return node_span.start == expr_stmt.span.start;
            }
            AstKind::ParenthesizedExpression(_)
            | AstKind::ChainExpression(_)
            | AstKind::SequenceExpression(_)
            | AstKind::AssignmentExpression(_)
            | AstKind::LogicalExpression(_)
            | AstKind::BinaryExpression(_)
            | AstKind::ConditionalExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSTypeAssertion(_)
            | AstKind::TSInstantiationExpression(_) => {}
            _ => return false,
        }
    }
    false
}

fn is_for_statement_init(
    node: &AstNode,
    conditional_expr: &ConditionalExpression,
    ctx: &LintContext,
) -> bool {
    matches!(
        ctx.nodes().parent_kind(node.id()),
        AstKind::ForStatement(for_stmt)
            if for_stmt.init.as_ref().is_some_and(|init| init.span() == conditional_expr.span)
    )
}

fn contains_unparenthesized_in_expression(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::In
    )
}
