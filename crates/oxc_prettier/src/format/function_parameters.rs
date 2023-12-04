use oxc_ast::{ast::*, AstKind};

use crate::{
    comments::CommentFlags,
    doc::{Doc, DocBuilder},
    group, hardline, line, ss, Format, Prettier,
};

pub(super) fn should_hug_the_only_function_parameter(
    p: &mut Prettier<'_>,
    params: &FormalParameters<'_>,
) -> bool {
    if params.parameters_count() != 1 {
        return false;
    }
    let Some(parameter) = params.items.first() else { return false };

    let all_comment_flags = CommentFlags::Trailing
        | CommentFlags::Leading
        | CommentFlags::Dangling
        | CommentFlags::Block
        | CommentFlags::Line
        | CommentFlags::PrettierIgnore
        | CommentFlags::First
        | CommentFlags::Last;
    if p.has_comment(parameter.span, all_comment_flags) {
        return false;
    }

    match &parameter.pattern.kind {
        BindingPatternKind::ObjectPattern(_) | BindingPatternKind::ArrayPattern(_) => true,
        BindingPatternKind::BindingIdentifier(_) => {
            let Some(ts_type_annotation) = &parameter.pattern.type_annotation else { return false };
            matches!(
                ts_type_annotation.type_annotation,
                TSType::TSTypeLiteral(_) | TSType::TSMappedType(_)
            )
        }
        BindingPatternKind::AssignmentPattern(assignment_pattern) => {
            let left = &assignment_pattern.left.kind;
            if matches!(left, BindingPatternKind::ObjectPattern(_)) {
                return true;
            }

            if !matches!(left, BindingPatternKind::ArrayPattern(_)) {
                return false;
            }

            let right = &assignment_pattern.right;
            match right {
                Expression::Identifier(_) => true,
                Expression::ObjectExpression(obj_expr) => obj_expr.properties.len() == 0,
                Expression::ArrayExpression(arr_expr) => arr_expr.elements.len() == 0,
                _ => false,
            }
        }
    }
}

pub(super) fn print_function_parameters<'a>(
    p: &mut Prettier<'a>,
    params: &FormalParameters<'a>,
) -> Doc<'a> {
    let mut parts = p.vec();
    let is_arrow_function = matches!(p.parent_kind(), AstKind::ArrowExpression(_));
    let need_parens =
        !is_arrow_function || p.options.arrow_parens.is_always() || params.items.len() != 1;
    if need_parens {
        parts.push(ss!("("));
    }

    for (i, param) in params.items.iter().enumerate() {
        parts.push(param.format(p));
        if i == params.items.len() - 1 {
            break;
        }
        parts.push(ss!(","));
        if should_hug_the_only_function_parameter(p, params) {
            parts.push(ss!(" "));
        } else if p.is_next_line_empty(param.span) {
            parts.extend(hardline!());
            parts.extend(hardline!());
        } else {
            parts.push(line!());
        }
    }

    if let Some(rest) = &params.rest {
        if !params.items.is_empty() {
            parts.push(ss!(", "));
        }
        parts.push(rest.format(p));
    }

    if need_parens {
        parts.push(ss!(")"));
    }

    group!(p, Doc::Array(parts))
}
