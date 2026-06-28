use oxc_ast::{
    AstKind,
    ast::{BinaryExpression, ConditionalExpression, Expression, IfStatement, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};
use oxc_syntax::identifier::is_identifier_start;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

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

    push_invert_test_fixes(&mut fixes, &fixer, negated_test, /* for_if_statement */ true, ctx);
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
    let mut fixes = fixer.new_fix_with_capacity(6);

    let is_unary = matches!(negated_test, Expression::UnaryExpression(_));
    let source = ctx.source_text();
    let expr_span = conditional_expr.span;

    // Detect ASI / keyword spacing from what the expression will look like after removing `!`.
    let (needs_space_before, needs_asi_semi, needs_return_throw_parens) = if is_unary {
        let Expression::UnaryExpression(unary) = negated_test else { unreachable!() };
        // Text after deleting the leading `!` (preserves comments between `!` and operand).
        let after_bang = ctx.source_range(Span::new(unary.span.start + 1, unary.span.end));
        let first_significant = after_bang.trim_start().chars().next();
        let has_newline_in_remainder = after_bang.contains('\n');

        let needs_return_throw_parens = has_newline_in_remainder
            && !matches!(conditional_expr.test, Expression::ParenthesizedExpression(_))
            && matches!(
                ctx.nodes().parent_kind(node.id()),
                AstKind::ReturnStatement(_) | AstKind::ThrowStatement(_)
            );

        let needs_space_before = !needs_return_throw_parens && {
            let start = expr_span.start as usize;
            start > 0
                && source[..start].chars().next_back().is_some_and(is_identifier_start)
                && first_significant.is_some_and(|c| {
                    is_identifier_start(c)
                        || c == '('
                        || c == '['
                        || c == '`'
                        || c == '/'
                        || c == '+'
                        || c == '-'
                })
        };

        let needs_asi_semi = !needs_return_throw_parens
            && !needs_space_before
            && matches!(first_significant, Some('(' | '[' | '`' | '+' | '-' | '/'))
            && could_be_asi_hazard(node, ctx);

        (needs_space_before, needs_asi_semi, needs_return_throw_parens)
    } else {
        (false, false, false)
    };

    if needs_asi_semi {
        fixes.push(fixer.insert_text_before_range(expr_span, ";"));
    } else if needs_space_before {
        fixes.push(fixer.insert_text_before_range(expr_span, " "));
    }

    if needs_return_throw_parens {
        fixes.push(fixer.insert_text_before_range(expr_span, "("));
        fixes.push(fixer.insert_text_after_range(expr_span, ")"));
    }

    push_invert_test_fixes(
        &mut fixes,
        &fixer,
        negated_test,
        /* for_if_statement */ false,
        ctx,
    );
    push_swap_expression_branches(
        &mut fixes,
        &fixer,
        &conditional_expr.consequent,
        &conditional_expr.alternate,
        ctx,
    );

    fixes.with_message(
        "Remove the negation operator and switch the consequent and alternate branches.",
    )
}

fn push_invert_test_fixes<'a>(
    fixes: &mut RuleFix,
    fixer: &RuleFixer<'_, 'a>,
    negated_test: &Expression<'a>,
    for_if_statement: bool,
    ctx: &LintContext<'a>,
) {
    match negated_test {
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            // Delete only the `!` punctuator (1 byte) so comments/whitespace after it stay.
            fixes.push(fixer.delete_range(Span::new(unary.span.start, unary.span.start + 1)));

            // For `if`, also drop parentheses around the argument (`if (!((a)))` → `if (a)`).
            if for_if_statement {
                let inner = unary.argument.get_inner_expression();
                let arg_span = unary.argument.span();
                if inner.span() != arg_span {
                    // Replace the parenthesized argument with the inner expression text only.
                    let inner_text = ctx.source_range(inner.span());
                    fixes.push(fixer.replace(arg_span, inner_text.to_string()));
                }
            }
        }
        Expression::BinaryExpression(binary) => {
            // Replace only the operator token so surrounding spacing is preserved (`a!=b` → `a==b`).
            if let Some(op_span) = binary_operator_span(binary, ctx) {
                let replacement = match binary.operator {
                    BinaryOperator::Inequality => "==",
                    BinaryOperator::StrictInequality => "===",
                    _ => unreachable!(),
                };
                fixes.push(fixer.replace(op_span, replacement));
            }
        }
        _ => {}
    }
}

/// Locate `!=` / `!==` between left and right without copying the operands.
fn binary_operator_span(binary: &BinaryExpression<'_>, ctx: &LintContext<'_>) -> Option<Span> {
    let binary_operator_str = binary.operator.as_str();
    ctx.find_next_token_within(
        binary.left.span().end,
        binary.right.span().start,
        binary_operator_str,
    )
    .map(|s| Span::sized(binary.left.span().end + s, binary_operator_str.len() as u32))
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
    fixes.push(fixer.replace(alt_span, cons_src.to_string()));
}
