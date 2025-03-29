use oxc_allocator::Vec;
use oxc_ast::{AstKind, ast::*};

use crate::{
    Format, Prettier, array, format::print::arrow_function, ir::Doc, needs_parens, text, utils,
};

pub fn print_expression_statement<'a>(
    p: &mut Prettier<'a>,
    expression_statement: &ExpressionStatement<'a>,
) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);

    if should_print_leading_semicolon(p, expression_statement) {
        parts.push(text!(";"));
    }

    parts.push(expression_statement.expression.format(p));

    if p.options.semi {
        parts.push(text!(";"));
    }

    array!(p, parts)
}

fn should_print_leading_semicolon<'a>(
    p: &mut Prettier<'a>,
    expr_statement: &ExpressionStatement<'a>,
) -> bool {
    if p.options.semi {
        return false;
    }

    if !matches!(
        p.parent_kind(),
        AstKind::Program(_)
            | AstKind::BlockStatement(_)
            | AstKind::StaticBlock(_)
            | AstKind::SwitchCase(_)
            | AstKind::TSModuleBlock(_)
    ) {
        return false;
    }

    expr_needs_asi_protection(p, &expr_statement.expression)
}

fn expr_needs_asi_protection<'a>(p: &mut Prettier<'a>, expr: &Expression<'a>) -> bool {
    if match expr {
        Expression::ParenthesizedExpression(_)
        | Expression::ArrayExpression(_)
        // TODO: ArrayPattern?
        | Expression::TemplateLiteral(_)
        | Expression::RegExpLiteral(_)
         | Expression::JSXElement(_) | Expression::JSXFragment(_) => true,
        Expression::ArrowFunctionExpression(arrow_expr)
            if !arrow_function::should_print_params_without_parens(p, arrow_expr) => true,
        Expression::UnaryExpression(unary_expr)
            if unary_expr.operator.is_arithmetic() => true,
        _ => false,
    } {
        return true;
    }

    // PERF: I'm not sure if this is the best way to handle this
    let expr = p.alloc(expr);

    let expr_kind = AstKind::from_expression(expr);

    // TODO: Consider this is a temporary hack or the right way to handle
    // The current implementation is:
    // - parent-child relationship such as `stack`, `current_node()` result are updated only by the `wrap!` macro
    // - and the `wrap!` macro is only used with a `Format` trait for each node
    // Therefore, at the time this code is executed, the outermost node in `stack` == the current node is `ExpressionStatement`.
    // However, `expr_needs_asi_protection()` should be called for `.expression` of `ExpressionStatement`.
    // That is, the code inside `need_parens()` should have that `.expression` as the current node, but it is not!
    // To resolve this gap, manually update the `stack` then call `need_parens()`.
    p.stack.push(expr_kind);
    let need_parens = p.need_parens(expr_kind);
    p.stack.pop();
    if need_parens {
        return true;
    }

    if !utils::has_naked_left_side(expr_kind) {
        return false;
    }

    // Check left side
    utils::get_left_side_expression(expr).is_some_and(|e| expr_needs_asi_protection(p, e))
}
