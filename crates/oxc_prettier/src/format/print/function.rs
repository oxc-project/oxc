use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{
    Format, Prettier, array, dynamic_text,
    format::print::{function, function_parameters, property},
    group, if_break, indent,
    ir::Doc,
    softline, text,
};

pub fn print_function<'a>(
    p: &mut Prettier<'a>,
    func: &Function<'a>,
    property_name: Option<&'a str>,
) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);

    if func.declare {
        parts.push(text!("declare "));
    }

    if func.r#async {
        parts.push(text!("async "));
    }

    if let Some(name) = property_name {
        parts.push(dynamic_text!(p, name));
    } else {
        parts.push(text!("function"));
        if func.generator {
            parts.push(text!("*"));
        }

        parts.push(text!(" "));
    }

    if let Some(id) = &func.id {
        parts.push(dynamic_text!(p, id.name.as_str()));
    }

    if let Some(type_params) = &func.type_parameters {
        parts.push(type_params.format(p));
    }
    // Prettier has `returnTypeDoc` to group together, write this for keep same with prettier.
    let params_doc = func.params.format(p);
    parts.push(group!(
        p,
        [{
            if function_parameters::should_group_function_parameters(func) {
                group!(p, [params_doc])
            } else {
                params_doc
            }
        }]
    ));

    if let Some(return_type) = &func.return_type {
        parts.push(text!(": "));
        parts.push(return_type.type_annotation.format(p));
    }

    if let Some(body) = &func.body {
        parts.push(text!(" "));
        parts.push(body.format(p));
    }
    if (func.is_ts_declare_function() || func.body.is_none()) && p.options.semi {
        parts.push(text!(";"));
    }

    array!(p, parts)
}

pub fn print_object_method<'a>(p: &mut Prettier<'a>, method: &ObjectProperty<'a>) -> Doc<'a> {
    let Expression::FunctionExpression(func_expr) = &method.value else {
        unreachable!();
    };

    let mut parts = Vec::new_in(p.allocator);

    match method.kind {
        PropertyKind::Init => {
            if func_expr.r#async {
                parts.push(text!("async "));
            }
            if func_expr.generator {
                parts.push(text!("*"));
            }
        }
        PropertyKind::Get => {
            parts.push(text!("get "));
        }
        PropertyKind::Set => {
            parts.push(text!("set "));
        }
    }

    parts.push(property::print_property_key(
        p,
        &property::PropertyKeyLike::PropertyKey(&method.key),
        method.computed,
    ));

    parts.push(function::print_method_value(p, func_expr));

    array!(p, parts)
}

pub fn print_class_method<'a>(p: &mut Prettier<'a>, method: &MethodDefinition<'a>) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);

    match method.kind {
        MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => {
            if method.value.r#async {
                parts.push(text!("async "));
            }
            if method.value.generator {
                parts.push(text!("*"));
            }
        }
        MethodDefinitionKind::Get => {
            parts.push(text!("get "));
        }
        MethodDefinitionKind::Set => {
            parts.push(text!("set "));
        }
    }

    parts.push(property::print_property_key(
        p,
        &property::PropertyKeyLike::PropertyKey(&method.key),
        method.computed,
    ));
    if method.optional {
        parts.push(text!("?"));
    }

    parts.push(print_method_value(p, &method.value));

    array!(p, parts)
}

pub fn print_method_value<'a>(p: &mut Prettier<'a>, function: &Function<'a>) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);
    let parameters_doc = function.params.format(p);
    let should_group_parameters = function_parameters::should_group_function_parameters(function);
    let parameters_doc =
        if should_group_parameters { group!(p, [parameters_doc]) } else { parameters_doc };

    if let Some(type_parameters) = &function.type_parameters {
        parts.push(type_parameters.format(p));
    }

    parts.push(group!(p, [parameters_doc]));

    if let Some(ret_type) = &function.return_type {
        parts.push(text!(": "));
        parts.push(ret_type.type_annotation.format(p));
    }

    if let Some(body) = &function.body {
        parts.push(text!(" "));
        parts.push(body.format(p));
    } else if p.options.semi {
        parts.push(text!(";"));
    }

    array!(p, parts)
}

pub fn print_return_or_throw_argument<'a>(
    p: &mut Prettier<'a>,
    argument: Option<&Expression<'a>>,
) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);

    if let Some(argument) = argument {
        parts.push(text!(" "));
        parts.push(
            if argument.is_binaryish() || matches!(argument, Expression::SequenceExpression(_)) {
                let argument_doc = argument.format(p);
                group!(
                    p,
                    [
                        if_break!(p, text!("(")),
                        indent!(p, [softline!(), argument_doc]),
                        softline!(),
                        if_break!(p, text!(")")),
                    ]
                )
            } else {
                argument.format(p)
            },
        );
    }

    if p.options.semi {
        parts.push(text!(";"));
    }

    array!(p, parts)
}
