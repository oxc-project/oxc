use oxc_ast::{ast::*, AstKind};

use crate::{
    comments::CommentFlags,
    doc::{Doc, DocBuilder, Group},
    hardline, if_break, indent, line, softline, space, ss, Format, Prettier,
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
    let is_arrow_function = matches!(p.parent_kind(), AstKind::ArrowFunctionExpression(_));
    let need_parens =
        !is_arrow_function || p.options.arrow_parens.is_always() || params.items.len() != 1;
    if need_parens {
        parts.push(ss!("("));
    }

    let should_hug_the_only_function_parameter = should_hug_the_only_function_parameter(p, params);

    let mut printed = p.vec();
    let len = params.items.len();
    let has_rest = params.rest.is_some();
    for (i, param) in params.items.iter().enumerate() {
        if let Some(accessibility) = &param.accessibility {
            printed.push(ss!(accessibility.as_str()));
            printed.push(space!());
        }

        if param.r#override {
            printed.push(ss!("override "));
        }

        if param.readonly {
            printed.push(ss!("readonly "));
        }

        printed.push(param.format(p));
        if i == len - 1 && !has_rest {
            break;
        }
        printed.push(ss!(","));
        if should_hug_the_only_function_parameter {
            printed.push(space!());
        } else if p.is_next_line_empty(param.span) {
            printed.extend(hardline!());
            printed.extend(hardline!());
        } else {
            printed.push(line!());
        }
    }
    if let Some(rest) = &params.rest {
        printed.push(rest.format(p));
    }

    if should_hug_the_only_function_parameter {
        let mut array = p.vec();
        array.push(ss!("("));
        array.extend(printed);
        array.push(ss!(")"));
        return Doc::Array(array);
    }

    let mut indented = p.vec();
    indented.push(softline!());
    indented.extend(printed);
    let indented = indent!(p, Doc::Array(indented));
    parts.push(indented);
    let has_rest_parameter = params.rest.is_some();
    parts.push(if_break!(p, if has_rest_parameter { "" } else { "," }));
    parts.push(softline!());
    if need_parens {
        parts.push(ss!(")"));
    }

    if p.args.expand_first_arg {
        Doc::Array(parts)
    } else {
        Doc::Group(Group::new(parts))
    }
}

pub(super) fn should_group_function_parameters(func: &Function) -> bool {
    let Some(return_type) = &func.return_type else {
        return false;
    };
    let type_parameters = func.type_parameters.as_ref().map(|x| &x.params);

    if let Some(type_parameter) = type_parameters {
        if type_parameter.len() > 1 {
            return false;
        }

        if let Some(type_parameter) = type_parameter.first() {
            if type_parameter.constraint.is_some() || type_parameter.default.is_some() {
                return false;
            }
        }
    }

    // TODO: need union `willBreak`
    func.params.parameters_count() == 1
        && (matches!(
            return_type.type_annotation,
            TSType::TSTypeLiteral(_) | TSType::TSMappedType(_)
        ))
}
