use oxc_ast::ast::*;

use crate::{
    format::function_parameters::should_group_function_parameters,
    group,
    ir::{Doc, DocBuilder},
    p_str, p_vec, Format, Prettier,
};

pub(super) fn print_function<'a>(
    p: &mut Prettier<'a>,
    func: &Function<'a>,
    property_name: Option<&str>,
) -> Doc<'a> {
    let mut parts = p.vec();

    if func.declare {
        parts.push(p._p_text("declare "));
    }

    if func.r#async {
        parts.push(p._p_text("async "));
    }

    if let Some(name) = property_name {
        parts.push(p._p_text(p_str!(p, name)));
    } else {
        parts.push(p._p_text("function"));
        if func.generator {
            parts.push(p._p_text("*"));
        }

        parts.push(p._p_text(" "));
    }

    if let Some(id) = &func.id {
        parts.push(p._p_text(p_str!(p, id.name.as_str())));
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
        parts.push(p._p_text(": "));
        parts.push(return_type.type_annotation.format(p));
    }

    if let Some(body) = &func.body {
        parts.push(p._p_space());
        parts.push(body.format(p));
    }
    if func.is_ts_declare_function() || func.body.is_none() {
        if let Some(semi) = p.semi() {
            parts.push(semi);
        }
    }

    p._p_array(parts)
}

pub(super) fn print_method<'a>(p: &mut Prettier<'a>, method: &MethodDefinition<'a>) -> Doc<'a> {
    let mut parts = p.vec();

    if let Some(accessibility) = &method.accessibility {
        parts.push(p._p_text(accessibility.as_str()));
        parts.push(p._p_space());
    }

    if method.r#static {
        parts.push(p._p_text("static "));
    }

    if matches!(method.r#type, MethodDefinitionType::TSAbstractMethodDefinition) {
        parts.push(p._p_text("abstract "));
    }

    if method.r#override {
        parts.push(p._p_text("override "));
    }

    match method.kind {
        MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => {}
        MethodDefinitionKind::Get => {
            parts.push(p._p_text("get "));
        }
        MethodDefinitionKind::Set => {
            parts.push(p._p_text("set "));
        }
    }

    if method.value.r#async {
        parts.push(p._p_text("async "));
    }

    if method.value.generator {
        parts.push(p._p_text("*"));
    }

    parts.push(method.key.format(p));

    if method.optional {
        parts.push(p._p_text("?"));
    }

    parts.push(print_method_value(p, &method.value));

    p._p_array(parts)
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
        parts.push(p._p_text(": "));
        parts.push(ret_typ.type_annotation.format(p));
    }

    if let Some(body) = &function.body {
        parts.push(p._p_space());
        parts.push(body.format(p));
    } else if p.options.semi {
        parts.push(p._p_text(";"));
    }

    p._p_array(parts)
}

pub(super) fn print_return_or_throw_argument<'a>(
    p: &mut Prettier<'a>,
    argument: Option<&Expression<'a>>,
    is_return: bool,
) -> Doc<'a> {
    let mut parts = p.vec();

    parts.push(p._p_text(if is_return { "return" } else { "throw" }));

    if let Some(argument) = argument {
        parts.push(p._p_space());
        parts.push(
            if argument.is_binaryish() || matches!(argument, Expression::SequenceExpression(_)) {
                let argument_doc = argument.format(p);
                group![
                    p,
                    p._p_if_break(p.boxed(p._p_text("(")), p.boxed(p._p_text("")), None),
                    p._p_indent(p_vec!(p, p._p_softline(), argument_doc)),
                    p._p_softline(),
                    p._p_if_break(p.boxed(p._p_text(")")), p.boxed(p._p_text("")), None),
                ]
            } else {
                argument.format(p)
            },
        );
    }

    if let Some(semi) = p.semi() {
        parts.push(semi);
    }
    p._p_array(parts)
}
