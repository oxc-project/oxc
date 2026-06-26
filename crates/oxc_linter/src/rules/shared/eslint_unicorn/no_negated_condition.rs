use oxc_ast::ast::{ConditionalExpression, Expression, IfStatement, Statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
};

const FIX_MESSAGE: &str =
    "Remove the negation operator and switch the consequent and alternate branches.";

fn diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected negated condition.").with_help(FIX_MESSAGE).with_label(span)
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
    // Only `if`/`else` — not `else if` (alternate is another `IfStatement`).
    let Some(alternate) = &if_stmt.alternate else {
        return;
    };
    if matches!(alternate, Statement::IfStatement(_)) {
        return;
    }

    let test = if_stmt.test.without_parentheses();
    if !is_negated(test) {
        return;
    }

    ctx.diagnostic_with_fix(diagnostic(test.span()), |fixer| {
        let fixer = fixer.for_multifix();
        let mut fixes = fixer.new_fix_with_capacity(4);

        invert_test(&mut fixes, &fixer, &if_stmt.test, /* strip_arg_parens */ true, ctx);
        swap_spans(
            &mut fixes,
            &fixer,
            if_stmt.consequent.span(),
            alternate.span(),
            /* brace_non_blocks */ true,
            ctx,
        );

        fixes.with_message(FIX_MESSAGE)
    });
}

pub fn run_on_conditional_expression<'a>(expr: &ConditionalExpression<'a>, ctx: &LintContext<'a>) {
    let test = expr.test.without_parentheses();
    if !is_negated(test) {
        return;
    }

    ctx.diagnostic_with_fix(diagnostic(test.span()), |fixer| {
        let fixer = fixer.for_multifix();
        let mut fixes = fixer.new_fix_with_capacity(6);

        invert_test(&mut fixes, &fixer, &expr.test, /* strip_arg_parens */ false, ctx);
        swap_spans(
            &mut fixes,
            &fixer,
            expr.consequent.span(),
            expr.alternate.span(),
            /* brace_non_blocks */ false,
            ctx,
        );

        // Extra care only for `!… ? … : …` (not `!=` / `!==`).
        if let Expression::UnaryExpression(unary) = test
            && unary.operator == UnaryOperator::LogicalNot
        {
            fix_unary_ternary_edges(
                &mut fixes,
                &fixer,
                expr,
                unary.span.start,
                &unary.argument,
                ctx,
            );
        }

        fixes.with_message(FIX_MESSAGE)
    });
}

fn is_negated(expr: &Expression) -> bool {
    match expr {
        Expression::UnaryExpression(u) => u.operator == UnaryOperator::LogicalNot,
        Expression::BinaryExpression(b) => {
            matches!(b.operator, BinaryOperator::Inequality | BinaryOperator::StrictInequality)
        }
        _ => false,
    }
}

/// Turn `!x` into `x`, or `a != b` / `a !== b` into `a == b` / `a === b`.
///
/// When `strip_arg_parens` is set (if-tests only), also remove parens on the `!` operand
/// so `if (!((a)))` becomes `if (a)` rather than `if ((a))`.
fn invert_test<'a>(
    fixes: &mut RuleFix,
    fixer: &RuleFixer<'_, 'a>,
    test: &Expression<'a>,
    strip_arg_parens: bool,
    ctx: &LintContext<'a>,
) {
    match test.without_parentheses() {
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            fixes.push(fixer.delete_range(Span::sized(unary.span.start, 1)));
            if strip_arg_parens {
                let mut inner = &unary.argument;
                while let Expression::ParenthesizedExpression(paren) = inner {
                    fixes.push(fixer.delete_range(Span::sized(paren.span.start, 1)));
                    fixes.push(fixer.delete_range(Span::sized(paren.span.end - 1, 1)));
                    inner = &paren.expression;
                }
            }
        }
        Expression::BinaryExpression(binary)
            if matches!(
                binary.operator,
                BinaryOperator::Inequality | BinaryOperator::StrictInequality
            ) =>
        {
            let op = find_inequality_op(binary.left.span().end, binary.right.span().start, ctx);
            let replacement =
                if binary.operator == BinaryOperator::StrictInequality { "===" } else { "==" };
            fixes.push(fixer.replace(op, replacement));
        }
        _ => {}
    }
}

fn find_inequality_op(left_end: u32, right_start: u32, ctx: &LintContext<'_>) -> Span {
    let between = ctx.source_range(Span::new(left_end, right_start));
    let Some(offset) = between.find("!=") else {
        return Span::new(left_end, right_start);
    };
    #[expect(clippy::cast_possible_truncation)]
    let start = left_end + offset as u32;
    let len = if between[offset..].starts_with("!==") { 3 } else { 2 };
    Span::sized(start, len)
}

/// Swap two spans' source text. Optionally wrap non-`{…}` if-arms in braces.
fn swap_spans<'a>(
    fixes: &mut RuleFix,
    fixer: &RuleFixer<'_, 'a>,
    left: Span,
    right: Span,
    brace_non_blocks: bool,
    ctx: &LintContext<'a>,
) {
    let left_text = arm_source(ctx, left, brace_non_blocks);
    let right_text = arm_source(ctx, right, brace_non_blocks);
    if left_text == right_text {
        return;
    }
    fixes.push(fixer.replace(left, right_text));
    fixes.push(fixer.replace(right, left_text));
}

fn arm_source(ctx: &LintContext<'_>, span: Span, brace_non_blocks: bool) -> String {
    let text = ctx.source_range(span);
    if brace_non_blocks && !text.trim_start().starts_with('{') {
        format!("{{{text}}}")
    } else {
        text.to_owned()
    }
}

/// After removing `!` from a ternary, keep the program parseable:
/// - space after `return`/`throw` (`return!a` → `return a`)
/// - parens for multi-line `return !` / `throw !`
/// - leading `;` when ASI would otherwise glue to the previous line
fn fix_unary_ternary_edges<'a>(
    fixes: &mut RuleFix,
    fixer: &RuleFixer<'_, 'a>,
    expr: &ConditionalExpression<'a>,
    bang_start: u32,
    argument: &Expression<'a>,
    ctx: &LintContext<'a>,
) {
    let before = ctx.source_range(Span::new(0, expr.span.start));

    // `return!a` — keyword abuts `!`; insert a space before the bang (deleted by invert_test).
    if before.chars().next_back().is_some_and(is_ident_char) {
        fixes.push(fixer.insert_text_before_range(Span::sized(bang_start, 1), " "));
    }

    let bang_and_arg_on_different_lines =
        ctx.source_range(Span::new(bang_start + 1, argument.span().start)).contains('\n');

    // `return !\n a ? b : c` — without parens, `return` only applies to `!`.
    // Skip if the test is already `(…)` (e.g. `return (!\n a) ? b : c`).
    if bang_and_arg_on_different_lines
        && !matches!(expr.test, Expression::ParenthesizedExpression(_))
        && {
            let t = before.trim_end();
            t.ends_with("return") || t.ends_with("throw")
        }
    {
        fixes.push(fixer.insert_text_before_range(expr.span, "("));
        fixes.push(fixer.insert_text_after_range(expr.span, ")"));
        return;
    }

    // `a\n![] ? b : c` → after deleting `!`, `[]` continues the previous line; insert `;`.
    if needs_asi_semicolon(before, argument, ctx) {
        fixes.push(fixer.insert_text_before_range(expr.span, ";"));
    }
}

fn needs_asi_semicolon(before: &str, argument: &Expression<'_>, ctx: &LintContext<'_>) -> bool {
    // Tokens that can continue an expression on the previous line once `!` is gone.
    let next = ctx.source_text().as_bytes().get(argument.span().start as usize).copied();
    if !matches!(next, Some(b'[' | b'(' | b'`' | b'+' | b'-' | b'/' | b'*' | b'.')) {
        return false;
    }

    let Some(prev_idx) = before.rfind(|c: char| !c.is_whitespace()) else {
        return false;
    };
    // Previous token must be on an earlier line.
    if !before[prev_idx..].contains('\n') {
        return false;
    }
    // Already terminated / continuation-safe.
    !matches!(before.as_bytes()[prev_idx] as char, ';' | '{' | '}' | '(' | '[' | ',' | ':' | '?')
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '$'
}
