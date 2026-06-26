use oxc_ast::ast::{ConditionalExpression, Expression, IfStatement, Statement};
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

fn invert_test_text(test: &Expression<'_>, ctx: &LintContext<'_>, for_if_statement: bool) -> String {
    match test {
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            // For `if` statements, drop parentheses around the argument so
            // `if (!((a)))` becomes `if (a)` rather than `if (((a)))`.
            let argument = if for_if_statement {
                unary.argument.get_inner_expression()
            } else {
                &unary.argument
            };
            ctx.source_range(argument.span()).to_string()
        }
        Expression::BinaryExpression(binary) => {
            let op = match binary.operator {
                BinaryOperator::Inequality => "==",
                BinaryOperator::StrictInequality => "===",
                _ => unreachable!(),
            };
            format!(
                "{} {op} {}",
                ctx.source_range(binary.left.span()),
                ctx.source_range(binary.right.span())
            )
        }
        _ => unreachable!(),
    }
}

/// Rebuild the test expression text while preserving any outer parentheses that
/// were around the negated expression (e.g. `(( !a ))` → `(( a ))`).
fn invert_test_preserving_outer_parens(
    full_test: &Expression<'_>,
    negated: &Expression<'_>,
    ctx: &LintContext<'_>,
    for_if_statement: bool,
) -> String {
    let inverted = invert_test_text(negated, ctx, for_if_statement);
    let full_span = full_test.span();
    let neg_span = negated.span();
    if full_span == neg_span {
        return inverted;
    }
    let source = ctx.source_text();
    let prefix = &source[full_span.start as usize..neg_span.start as usize];
    let suffix = &source[neg_span.end as usize..full_span.end as usize];
    format!("{prefix}{inverted}{suffix}")
}

fn statement_text_for_swap(stmt: &Statement<'_>, ctx: &LintContext<'_>) -> String {
    let text = ctx.source_range(stmt.span());
    if matches!(stmt, Statement::BlockStatement(_)) {
        text.to_string()
    } else {
        // Non-block branches must be wrapped so ASI / dangling `else` stay valid
        // after the swap, e.g. `if (!a) b(); else c()` → `if (a) {c()} else {b()}`.
        format!("{{{text}}}")
    }
}

fn fix_if_statement<'a>(
    fixer: RuleFixer<'_, 'a>,
    if_stmt: &IfStatement<'a>,
    alternate: &Statement<'a>,
    negated_test: &Expression<'a>,
    ctx: &LintContext<'a>,
) -> RuleFix {
    let inverted_test =
        invert_test_preserving_outer_parens(&if_stmt.test, negated_test, ctx, true);
    let consequent_text = statement_text_for_swap(&if_stmt.consequent, ctx);
    let alternate_text = statement_text_for_swap(alternate, ctx);

    let source = ctx.source_text();
    let if_start = if_stmt.span.start as usize;
    let test_start = if_stmt.test.span().start as usize;
    let test_end = if_stmt.test.span().end as usize;
    let cons_start = if_stmt.consequent.span().start as usize;
    let cons_end = if_stmt.consequent.span().end as usize;
    let alt_start = alternate.span().start as usize;

    let before_test = &source[if_start..test_start];
    let after_test = &source[test_end..cons_start];
    let between_consequent_and_else = &source[cons_end..alt_start];

    let replacement = format!(
        "{before_test}{inverted_test}{after_test}{alternate_text}{between_consequent_and_else}{consequent_text}"
    );

    fixer.replace(if_stmt.span, replacement)
}

fn fix_conditional_expression<'a>(
    fixer: RuleFixer<'_, 'a>,
    node: &AstNode<'a>,
    conditional_expr: &ConditionalExpression<'a>,
    negated_test: &Expression<'a>,
    ctx: &LintContext<'a>,
) -> RuleFix {
    let mut inverted_test =
        invert_test_preserving_outer_parens(&conditional_expr.test, negated_test, ctx, false);
    let consequent_text = ctx.source_range(conditional_expr.consequent.span());
    let alternate_text = ctx.source_range(conditional_expr.alternate.span());

    let source = ctx.source_text();
    let test_span = conditional_expr.test.span();
    let cons_span = conditional_expr.consequent.span();
    let alt_span = conditional_expr.alternate.span();
    let expr_span = conditional_expr.span;

    let before_test = &source[expr_span.start as usize..test_span.start as usize];
    let between_test_and_cons = &source[test_span.end as usize..cons_span.start as usize];
    let between_cons_and_alt = &source[cons_span.end as usize..alt_span.start as usize];
    let after_alt = &source[alt_span.end as usize..expr_span.end as usize];

    // `return!a` / `throw!a` → insert space after keyword when removing `!`
    let is_unary = matches!(negated_test, Expression::UnaryExpression(_));
    if is_unary {
        let start = expr_span.start as usize;
        if start > 0 {
            let char_before = source[..start].chars().next_back();
            let first_of_inverted = inverted_test.chars().next();
            if char_before.is_some_and(is_identifier_start)
                && first_of_inverted.is_some_and(|c| {
                    is_identifier_start(c)
                        || c == '('
                        || c == '['
                        || c == '`'
                        || c == '/'
                        || c == '+'
                        || c == '-'
                })
            {
                inverted_test.insert(0, ' ');
            }
        }
    }

    let mut replacement = format!(
        "{before_test}{inverted_test}{between_test_and_cons}{alternate_text}{between_cons_and_alt}{consequent_text}{after_alt}"
    );

    // ASI hazard when removing leading `!` leaves `(`, `[`, etc. at statement start.
    if is_unary {
        let trimmed = inverted_test.trim_start();
        let first_char = trimmed.chars().next();
        if matches!(first_char, Some('(' | '[' | '`' | '+' | '-' | '/'))
            && could_be_asi_hazard(node, ctx)
            && !replacement.starts_with([';', ' '])
        {
            replacement.insert(0, ';');
        }
    }

    fixer.replace(expr_span, replacement)
}
