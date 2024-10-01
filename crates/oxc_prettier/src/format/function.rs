use oxc_ast::ast::*;

use crate::{
    doc::{Doc, DocBuilder},
    format::function_parameters::should_group_function_parameters,
    group, if_break, indent, softline, space, ss, Format, Prettier,
};

pub(super) fn print_function<'a>(
    p: &mut Prettier<'a>,
    func: &Function<'a>,
    property_name: Option<&str>,
) -> Doc<'a> {
    let mut parts = p.vec();

    if func.declare {
        parts.push(ss!("declare "));
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

    if let Some(id) = &func.id {
        parts.push(p.str(id.name.as_str()));
    }

    if let Some(type_params) = &func.type_parameters {
        parts.push(type_params.format(p));
    }
    // Prettier has `returnTypeDoc` to group together, write this for keep same with prettier.
    parts.push(group!(p, {
        if should_group_function_parameters(func) {
            group!(p, func.params.format(p))
        } else {
            func.params.format(p)
        }
    }));

    if let Some(return_type) = &func.return_type {
        parts.push(ss!(": "));
        parts.push(return_type.type_annotation.format(p));
    }

    if let Some(body) = &func.body {
        parts.push(space!());
        parts.push(body.format(p));
    }
    if func.is_ts_declare_function() || func.body.is_none() {
        if let Some(semi) = p.semi() {
            parts.push(semi);
        }
    }

    Doc::Array(parts)
}

pub(super) fn print_method<'a>(p: &mut Prettier<'a>, method: &MethodDefinition<'a>) -> Doc<'a> {
    let mut parts = p.vec();

    if let Some(accessibility) = &method.accessibility {
        parts.push(ss!(accessibility.as_str()));
        parts.push(space!());
    }

    if method.r#static {
        parts.push(ss!("static "));
    }

    if matches!(method.r#type, MethodDefinitionType::TSAbstractMethodDefinition) {
        parts.push(ss!("abstract "));
    }

    if method.r#override {
        parts.push(ss!("override "));
    }

    match method.kind {
        MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => {}
        MethodDefinitionKind::Get => {
            parts.push(ss!("get "));
        }
        MethodDefinitionKind::Set => {
            parts.push(ss!("set "));
        }
    }

    if method.value.r#async {
        parts.push(ss!("async "));
    }

    if method.value.generator {
        parts.push(ss!("*"));
    }

    parts.push(method.key.format(p));

    if method.optional {
        parts.push(ss!("?"));
    }

    parts.push(print_method_value(p, &method.value));

    Doc::Array(parts)
}

fn print_method_value<'a>(p: &mut Prettier<'a>, function: &Function<'a>) -> Doc<'a> {
    let mut parts = p.vec();
    let parameters_doc = function.params.format(p);
    let should_group_parameters = should_group_function_parameters(function);
    let parameters_doc =
        if should_group_parameters { group!(p, parameters_doc) } else { parameters_doc };

    if let Some(type_parameters) = &function.type_parameters {
        parts.push(type_parameters.format(p));
    }

    parts.push(group!(p, parameters_doc));

    if let Some(ret_typ) = &function.return_type {
        parts.push(ss!(": "));
        parts.push(ret_typ.type_annotation.format(p));
    }

    if let Some(body) = &function.body {
        parts.push(space!());
        parts.push(body.format(p));
    } else if p.options.semi {
        parts.push(ss!(";"));
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
        parts.push(space!());
        parts.push(
            if argument.is_binaryish() || matches!(argument, Expression::SequenceExpression(_)) {
                group![
                    p,
                    if_break!(p, "("),
                    indent!(p, softline!(), argument.format(p)),
                    softline!(),
                    if_break!(p, ")"),
                ]
            } else {
                argument.format(p)
            },
        );
    }

    if let Some(semi) = p.semi() {
        parts.push(semi);
    }
    Doc::Array(parts)
}
