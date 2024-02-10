use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    binaryish::BinaryishOperator,
    comments::CommentFlags,
    doc::{Doc, DocBuilder, Group},
    group, line, space, ss, Format, Prettier,
};

pub(super) fn print_binaryish_expression<'a>(
    p: &mut Prettier<'a>,
    left: &Expression<'a>,
    operator: BinaryishOperator,
    right: &Expression<'a>,
) -> Doc<'a> {
    print_binaryish_expressions(p, left, operator, right)
}

fn print_binaryish_expressions<'a>(
    p: &mut Prettier<'a>,
    left: &Expression<'a>,
    operator: BinaryishOperator,
    right: &Expression<'a>,
) -> Doc<'a> {
    let mut parts = p.vec();

    let left_operator = match left {
        Expression::LogicalExpression(e) => Some(BinaryishOperator::LogicalOperator(e.operator)),
        Expression::BinaryExpression(e) => Some(BinaryishOperator::BinaryOperator(e.operator)),
        _ => None,
    };

    if left_operator.is_some_and(|left_operator| operator.should_flatten(left_operator)) {
        parts.push(match left {
            Expression::BinaryExpression(e) => {
                print_binaryish_expressions(p, &e.left, e.operator.into(), &e.right)
            }
            Expression::LogicalExpression(e) => {
                print_binaryish_expressions(p, &e.left, e.operator.into(), &e.right)
            }
            _ => unreachable!(),
        });
    } else {
        parts.push(group!(p, left.format(p)));
    }
    let should_inline = should_inline_logical_expression(right);
    let line_before_operator = false;

    let right = if should_inline {
        p.vec()
    } else {
        let mut parts = p.vec();
        if line_before_operator {
            parts.push(line!());
        }
        parts.push(ss!(operator.as_str()));
        // FIXME:
        // parts.push(if line_before_operator { space!() } else { line!() });
        parts.push(space!());
        parts.push(right.format(p));
        parts
    };

    let should_break = p.has_comment(left.span(), CommentFlags::Trailing | CommentFlags::Line);
    let should_group = false;

    if !line_before_operator {
        parts.push(space!());
    }

    if should_group {
        let group = Doc::Group(Group::new(right, should_break));
        parts.push(group);
    } else {
        parts.push(Doc::Array(right));
    }

    Doc::Array(parts)
}

pub(super) fn should_inline_logical_expression(expr: &Expression) -> bool {
    let Expression::LogicalExpression(logical_expr) = expr else { return false };

    if let Expression::ObjectExpression(obj_expr) = &logical_expr.right {
        if obj_expr.properties.len() > 0 {
            return true;
        }
    }

    if let Expression::ArrayExpression(array_expr) = &logical_expr.right {
        if array_expr.elements.len() > 0 {
            return true;
        }
    }

    if matches!(logical_expr.right, Expression::JSXElement(_) | Expression::JSXFragment(_)) {
        return true;
    }

    false
}
