use oxc_ast::ast::*;

use crate::{
    binaryish::{BinaryishLeft, BinaryishOperator},
    doc::{Doc, DocBuilder, Group},
    group, line, ss, Format, Prettier,
};

pub(super) fn print_binaryish_expression<'a>(
    p: &mut Prettier<'a>,
    left: BinaryishLeft<'a, '_>,
    operator: BinaryishOperator,
    right: &Expression<'a>,
) -> Doc<'a> {
    let mut parts = p.vec();

    if left.operator().is_some_and(|left_operator| operator.should_flatten(left_operator)) {
        parts.push(match left {
            BinaryishLeft::Expression(Expression::BinaryExpression(e)) => {
                print_binaryish_expression(p, (&e.left).into(), e.operator.into(), &e.right)
            }
            BinaryishLeft::Expression(Expression::LogicalExpression(e)) => {
                print_binaryish_expression(p, (&e.left).into(), e.operator.into(), &e.right)
            }
            _ => unreachable!(),
        });
    } else {
        parts.push(group!(p, left.format(p)));
    }
    parts.push(ss!(" "));

    if operator.is_binary() {
        parts.push(group!(p, ss!(operator.as_str()), line!(), right.format(p)));
        Doc::Group(Group::new(parts, false))
    } else {
        parts.push(ss!(operator.as_str()));
        parts.push(line!());
        parts.push(right.format(p));
        Doc::Array(parts)
    }
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
