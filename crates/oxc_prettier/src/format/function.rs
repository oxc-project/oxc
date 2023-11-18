#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{doc::Doc, group, if_break, indent, softline, ss, Format, Prettier};

pub(super) fn print_function<'a>(
    p: &mut Prettier<'a>,
    func: &Function<'a>,
    property_name: Option<&str>,
) -> Doc<'a> {
    let mut parts = p.vec();
    if let Some(comments) = p.print_leading_comments(func.span) {
        parts.push(comments);
    }
    if func.r#async {
        parts.push(ss!("async "));
    }

    if let Some(name) = property_name {
        parts.push(p.str(name));
    } else {
        parts.push(ss!("function"));
        if func.generator {
            parts.push(ss!("*"));
        }

        parts.push(p.str(" "));
    }

    if let Some(type_params) = &func.type_parameters {
        parts.push(type_params.format(p));
    }
    if let Some(id) = &func.id {
        parts.push(p.str(id.name.as_str()));
    }
    if should_group_function_parameters(func) {
        parts.push(group!(p, func.params.format(p)));
    } else {
        parts.push(func.params.format(p));
    }
    if let Some(body) = &func.body {
        parts.push(ss!(" "));
        parts.push(body.format(p));
    }
    if p.options.semi && (func.is_ts_declare_function() || func.body.is_none()) {
        parts.push(p.str(";"));
    }

    Doc::Array(parts)
}

pub(super) fn print_return_or_throw_argument<'a>(
    p: &mut Prettier<'a>,
    argument: Option<&Expression<'a>>,
    is_return: bool,
) -> Doc<'a> {
    let mut parts = p.vec();

    parts.push(ss!(if is_return { "return" } else { "throw" }));

    if let Some(argument) = argument {
        parts.push(ss!(" "));
        parts.push(group![
            p,
            if_break!(p, "("),
            indent!(p, softline!(), argument.format(p)),
            softline!(),
            if_break!(p, ")")
        ]);
    }

    parts.push(p.str(";"));
    Doc::Array(parts)
}

fn should_group_function_parameters(func: &Function) -> bool {
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
