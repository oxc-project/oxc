use oxc_ast::ast::{ConditionalExpression, Expression, IfStatement, Statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{
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
    let Some(if_stmt_alternate) = &if_stmt.alternate else {
        return;
    };

    if matches!(if_stmt_alternate, Statement::IfStatement(_)) {
        return;
    }

    let test = if_stmt.test.without_parentheses();
    if !is_negated_expression(test) {
        return;
    }

    ctx.diagnostic_with_fix(no_negated_condition_diagnostic(test.span()), |fixer| {
        fix_if_statement(fixer, if_stmt, ctx)
    });
}

pub fn run_on_conditional_expression<'a>(
    conditional_expr: &ConditionalExpression<'a>,
    ctx: &LintContext<'a>,
) {
    let test = conditional_expr.test.without_parentheses();
    if !is_negated_expression(test) {
        return;
    }

    ctx.diagnostic_with_fix(no_negated_condition_diagnostic(test.span()), |fixer| {
        fix_conditional_expression(fixer, conditional_expr, ctx)
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

fn fix_if_statement<'a>(
    fixer: RuleFixer<'_, 'a>,
    if_stmt: &IfStatement<'a>,
    ctx: &LintContext<'a>,
) -> RuleFix {
    let Some(alternate) = &if_stmt.alternate else {
        return fixer.noop();
    };

    let fixer = fixer.for_multifix();
    let mut fixes = fixer.new_fix_with_capacity(4);

    convert_negated_condition(
        &mut fixes,
        &fixer,
        &if_stmt.test,
        /* is_if_statement */ true,
        ctx,
    );
    swap_if_branches(&mut fixes, &fixer, if_stmt.consequent.span(), alternate.span(), ctx);

    fixes.with_message(
        "Remove the negation operator and switch the consequent and alternate branches.",
    )
}

fn fix_conditional_expression<'a>(
    fixer: RuleFixer<'_, 'a>,
    conditional_expr: &ConditionalExpression<'a>,
    ctx: &LintContext<'a>,
) -> RuleFix {
    let fixer = fixer.for_multifix();
    let mut fixes = fixer.new_fix_with_capacity(6);

    let test_inner = conditional_expr.test.without_parentheses();
    let is_unary = matches!(
        test_inner,
        Expression::UnaryExpression(u) if u.operator == UnaryOperator::LogicalNot
    );

    convert_negated_condition(
        &mut fixes,
        &fixer,
        &conditional_expr.test,
        /* is_if_statement */ false,
        ctx,
    );
    swap_expression_branches(
        &mut fixes,
        &fixer,
        conditional_expr.consequent.span(),
        conditional_expr.alternate.span(),
        ctx,
    );

    if is_unary {
        // `return!a` / `throw!a` — insert space so keyword and operand don't glue.
        fix_space_around_keyword(&mut fixes, &fixer, conditional_expr, ctx);

        // Multi-line `return !` / `throw !` without parens around the whole conditional.
        if needs_return_or_throw_parens(conditional_expr, test_inner, ctx) {
            fixes.push(fixer.insert_text_before_range(conditional_expr.span, "("));
            fixes.push(fixer.insert_text_after_range(conditional_expr.span, ")"));
        } else if needs_asi_semicolon(conditional_expr, test_inner, ctx) {
            // ASI when removing leading `!` would merge with the previous line.
            fixes.push(fixer.insert_text_before_range(conditional_expr.span, ";"));
        }
    }

    fixes.with_message(
        "Remove the negation operator and switch the consequent and alternate branches.",
    )
}

fn convert_negated_condition<'a>(
    fixes: &mut RuleFix,
    fixer: &RuleFixer<'_, 'a>,
    test: &Expression<'a>,
    is_if_statement: bool,
    ctx: &LintContext<'a>,
) {
    let test_inner = test.without_parentheses();
    match test_inner {
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            // Delete the `!` operator token.
            let bang_span = Span::sized(unary.span.start, 1);
            fixes.push(fixer.delete_range(bang_span));

            // For `if` tests only, strip parentheses from the unary argument (unicorn behavior).
            if is_if_statement {
                remove_parentheses(fixes, fixer, &unary.argument);
            }
        }
        Expression::BinaryExpression(binary)
            if matches!(
                binary.operator,
                BinaryOperator::Inequality | BinaryOperator::StrictInequality
            ) =>
        {
            let op_span = operator_span(binary.left.span().end, binary.right.span().start, ctx);
            let replacement = match binary.operator {
                BinaryOperator::Inequality => "==",
                BinaryOperator::StrictInequality => "===",
                _ => unreachable!(),
            };
            fixes.push(fixer.replace(op_span, replacement));
        }
        _ => {}
    }
}

/// Span of the binary operator between left and right operands.
fn operator_span(left_end: u32, right_start: u32, ctx: &LintContext<'_>) -> Span {
    let between = ctx.source_range(Span::new(left_end, right_start));
    let Some(rel) = between.find("!=") else {
        return Span::new(left_end, right_start);
    };
    #[expect(clippy::cast_possible_truncation)]
    let start = left_end + rel as u32;
    let len = if between[rel..].starts_with("!==") { 3 } else { 2 };
    Span::sized(start, len)
}

fn remove_parentheses(fixes: &mut RuleFix, fixer: &RuleFixer<'_, '_>, expr: &Expression<'_>) {
    let mut current = expr;
    while let Expression::ParenthesizedExpression(paren) = current {
        fixes.push(fixer.delete_range(Span::sized(paren.span.start, 1)));
        fixes.push(fixer.delete_range(Span::sized(paren.span.end - 1, 1)));
        current = &paren.expression;
    }
}

fn swap_if_branches<'a>(
    fixes: &mut RuleFix,
    fixer: &RuleFixer<'_, 'a>,
    consequent_span: Span,
    alternate_span: Span,
    ctx: &LintContext<'a>,
) {
    let consequent_text = branch_text_for_if(ctx, consequent_span);
    let alternate_text = branch_text_for_if(ctx, alternate_span);

    if consequent_text == alternate_text {
        return;
    }

    fixes.push(fixer.replace(consequent_span, alternate_text));
    fixes.push(fixer.replace(alternate_span, consequent_text));
}

fn branch_text_for_if(ctx: &LintContext<'_>, span: Span) -> String {
    let text = ctx.source_range(span);
    // Non-block statement arms need braces after swap (ASI / structure safety).
    let trimmed = text.trim_start();
    if trimmed.starts_with('{') { text.to_owned() } else { format!("{{{text}}}") }
}

fn swap_expression_branches<'a>(
    fixes: &mut RuleFix,
    fixer: &RuleFixer<'_, 'a>,
    consequent_span: Span,
    alternate_span: Span,
    ctx: &LintContext<'a>,
) {
    let consequent_text = ctx.source_range(consequent_span);
    let alternate_text = ctx.source_range(alternate_span);

    if consequent_text == alternate_text {
        return;
    }

    fixes.push(fixer.replace(consequent_span, alternate_text.to_owned()));
    fixes.push(fixer.replace(alternate_span, consequent_text.to_owned()));
}

fn fix_space_around_keyword<'a>(
    fixes: &mut RuleFix,
    fixer: &RuleFixer<'_, 'a>,
    conditional_expr: &ConditionalExpression<'a>,
    ctx: &LintContext<'a>,
) {
    let start = conditional_expr.span.start;
    if start == 0 {
        return;
    }
    let before = ctx.source_range(Span::new(0, start));
    let Some(prev) = before.chars().next_back() else {
        return;
    };
    if !is_identifier_part(prev) {
        return;
    }
    let test_inner = conditional_expr.test.without_parentheses();
    if let Expression::UnaryExpression(unary) = test_inner {
        // Insert space before `!` so after deletion we get `return a` not `returna`.
        fixes.push(fixer.insert_text_before_range(Span::sized(unary.span.start, 1), " "));
    }
}

fn is_identifier_part(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '$'
}

fn needs_return_or_throw_parens<'a>(
    conditional_expr: &ConditionalExpression<'a>,
    test_inner: &Expression<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let Expression::UnaryExpression(unary) = test_inner else {
        return false;
    };

    // Bang and operand must be on different lines.
    let bang_end = unary.span.start + 1;
    let arg_start = unary.argument.span().start;
    if same_line(ctx, bang_end, arg_start) {
        return false;
    }

    // Whole conditional already parenthesized as return/throw argument
    // (`return (! … ? … : …)` — conditional.span starts at `(`).
    let source = ctx.source_text();
    let cond_start = conditional_expr.span.start as usize;
    if cond_start > 0 && source.as_bytes().get(cond_start) == Some(&b'(') {
        return false;
    }

    // Preceded by `return` or `throw` (keyword immediately before, ignoring whitespace/comments
    // is hard; match unicorn's common cases: keyword then optional spaces then our expression).
    let before = ctx.source_range(Span::new(0, conditional_expr.span.start));
    let trimmed = before.trim_end();
    let is_return_or_throw = trimmed.ends_with("return") || trimmed.ends_with("throw");
    if !is_return_or_throw {
        return false;
    }

    // `return ( \n ! // … \n a) ? b : c` — test itself is parenthesized around the unary only;
    // conditional.span still starts at `(`. Already excluded by leading `(` check if AST puts
    // paren on test. If test is ParenthesizedExpression, conditional.span starts at `(`.
    if matches!(conditional_expr.test, Expression::ParenthesizedExpression(_)) {
        // Parenthesized test is not the same as parenthesized conditional; span of conditional
        // still starts at `!` or `(`. For invalid(15), source is:
        //   return (
        //   ! // …
        //   a) ? b : c
        // conditional.span starts at `(` of `( \n ! … a)` which is the test paren — so leading
        // `(` means we must NOT add return parens. Good — returns false above when first char is `(`.
        // Wait: for invalid(15), first char of conditional IS `(` because test is ParenthesizedExpression.
        // So we return false. Correct for 15.
        return false;
    }

    true
}

fn needs_asi_semicolon<'a>(
    conditional_expr: &ConditionalExpression<'a>,
    test_inner: &Expression<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let Expression::UnaryExpression(unary) = test_inner else {
        return false;
    };

    // First character of what remains after deleting `!` (start of unary argument).
    let next_start = unary.argument.span().start;
    let next_byte = ctx.source_text().as_bytes().get(next_start as usize).copied();
    let needs_semi_for_next =
        matches!(next_byte, Some(b'[' | b'(' | b'`' | b'+' | b'-' | b'/' | b'*' | b'.'));
    if !needs_semi_for_next {
        return false;
    }

    let start = conditional_expr.span.start;
    if start == 0 {
        return false;
    }
    let before = ctx.source_range(Span::new(0, start));
    let Some(prev_non_ws_idx) = before.rfind(|c: char| !c.is_whitespace()) else {
        return false;
    };
    // Previous token must be on a prior line.
    if !before[prev_non_ws_idx..].contains('\n') {
        return false;
    }

    let prev_char = before.as_bytes()[prev_non_ws_idx] as char;
    // Statement-ending or continuation-safe tokens don't need ASI.
    if matches!(prev_char, ';' | '{' | '}' | '(' | '[' | ',' | ':' | '?') {
        return false;
    }
    // Identifier / literal / closing paren on previous line + dangerous next token → ASI.
    true
}

fn same_line(ctx: &LintContext<'_>, a: u32, b: u32) -> bool {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    !ctx.source_range(Span::new(lo, hi)).contains('\n')
}
