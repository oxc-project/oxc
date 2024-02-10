use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstKind};
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
    let is_inside_parenthesis = matches!(
        p.parent_kind(),
        AstKind::IfStatement(_)
            | AstKind::WhileStatement(_)
            | AstKind::SwitchStatement(_)
            | AstKind::DoWhileStatement(_)
    );

    let parts = print_binaryish_expressions(p, left, operator, right);

    if is_inside_parenthesis {
        return Doc::Array(parts);
    }

    let first_group_index = parts.iter().position(|part| {
        matches!(part, Doc::Str(_))
            && !matches!(part, Doc::Array(_))
            && matches!(part, Doc::Group(_))
    });

    // Separate the leftmost expression, possibly with its leading comments.
    let first_group_index = first_group_index.map_or(1, |index| index + 1);

    let mut group = p.vec();
    let mut rest = p.vec();
    for (i, part) in parts.into_iter().enumerate() {
        if i < first_group_index {
            group.push(part);
        } else {
            rest.push(part);
        }
    }
    group.push(Doc::Indent(rest));
    Doc::Group(Group::new(group, false))
}

fn print_binaryish_expressions<'a>(
    p: &mut Prettier<'a>,
    left: &Expression<'a>,
    operator: BinaryishOperator,
    right: &Expression<'a>,
) -> Vec<'a, Doc<'a>> {
    let mut parts = p.vec();

    let left_operator = match left {
        Expression::LogicalExpression(e) => Some(BinaryishOperator::LogicalOperator(e.operator)),
        Expression::BinaryExpression(e) => Some(BinaryishOperator::BinaryOperator(e.operator)),
        _ => None,
    };

    if left_operator.is_some_and(|left_operator| operator.should_flatten(left_operator)) {
        parts.push(match left {
            Expression::BinaryExpression(e) => {
                Doc::Array(print_binaryish_expressions(p, &e.left, e.operator.into(), &e.right))
            }
            Expression::LogicalExpression(e) => {
                Doc::Array(print_binaryish_expressions(p, &e.left, e.operator.into(), &e.right))
            }
            _ => unreachable!(),
        });
    } else {
        parts.push(group!(p, left.format(p)));
    }

    let should_inline = should_inline_logical_expression(right);
    let line_before_operator = false;

    let right = if should_inline {
        let mut parts = p.vec();
        parts.push(ss!(operator.as_str()));
        parts.push(space!());
        parts.push(right.format(p));
        parts
    } else {
        let mut parts = p.vec();
        if line_before_operator {
            parts.push(line!());
        }
        parts.push(ss!(operator.as_str()));
        parts.push(if line_before_operator { space!() } else { line!() });
        parts.push(right.format(p));
        parts
    };

    let should_break = p.has_comment(left.span(), CommentFlags::Trailing | CommentFlags::Line);
    let should_group = should_break;

    if !line_before_operator {
        parts.push(space!());
    }

    parts.push(if should_group {
        Doc::Group(Group::new(right, should_break))
    } else {
        Doc::Array(right)
    });

    parts
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
