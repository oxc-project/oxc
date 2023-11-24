use std::convert::Into;

use oxc_ast::ast::{BindingPatternKind, Expression};
use oxc_semantic::ScopeFlags;
use trustfall::{
    provider::{
        field_property, resolve_property_with, ContextIterator, ContextOutcomeIterator, ResolveInfo,
    },
    FieldValue,
};

use super::vertex::Vertex;
use crate::{
    util::{
        accessibility_to_string, jsx_attribute_name_to_string, jsx_attribute_to_constant_string,
        jsx_element_name_to_string,
    },
    Adapter,
};

pub(super) fn resolve_array_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!("attempted to read unexpected property '{property_name}' on type 'Array'")
        }
    }
}

pub(super) fn resolve_argument_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "is_spread" => resolve_property_with(contexts, |v| {
            matches!(
                v.as_argument()
                    .unwrap_or_else(|| {
                        panic!("expected to have an argument vertex, instead have: {v:#?}")
                    })
                    .argument,
                oxc_ast::ast::Argument::SpreadElement(_)
            )
            .into()
        }),
        "index" => resolve_property_with(contexts, |v| {
            let arg_index: u64 = v
                .as_argument()
                .unwrap_or_else(|| {
                    panic!("expected to have an argument vertex, instead have: {v:#?}")
                })
                .index(adapter)
                .try_into()
                .expect("to index of argument into u64");
            arg_index.into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'Argument'"
            )
        }
    }
}

pub(super) fn resolve_arrow_function_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "is_async" => resolve_property_with(contexts, |v| v.function_is_async().into()),
        "is_generator" => resolve_property_with(contexts, |v| v.function_is_generator().into()),
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        "is_getter" => resolve_property_with(contexts, |v| {
            v.function_scope_flag(adapter).contains(ScopeFlags::GetAccessor).into()
        }),
        "is_setter" => resolve_property_with(contexts, |v| {
            v.function_scope_flag(adapter).contains(ScopeFlags::SetAccessor).into()
        }),
        "is_constructor" => resolve_property_with(contexts, |v| {
            v.function_scope_flag(adapter).contains(ScopeFlags::Constructor).into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'ArrowFunction'"
            )
        }
    }
}

pub(super) fn resolve_assignment_type_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "assignment_to_variable_name" => resolve_property_with(contexts, |v| {
            let Vertex::AssignmentType(BindingPatternKind::BindingIdentifier(ident)) = v else {
                return FieldValue::Null;
            };
            ident.name.to_string().into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'AssignmentType'"
            )
        }
    }
}

pub(super) fn resolve_class_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "extended_class_name" => resolve_property_with(contexts, |v| {
            v.as_class()
                .unwrap_or_else(|| panic!("expected to have a class vertex, instead have: {v:#?}"))
                .class
                .super_class
                .as_ref()
                .and_then(|expr| match expr {
                    Expression::Identifier(ident) => Some(ident.name.to_string().into()),
                    _ => None,
                })
                .unwrap_or(FieldValue::Null)
        }),
        "name" => resolve_property_with(contexts, |v| {
            v.as_class()
                .unwrap_or_else(|| panic!("expected to have a class vertex, instead have: {v:#?}"))
                .class
                .id
                .as_ref()
                .map_or_else(|| FieldValue::Null, |x| x.name.to_string().into())
        }),
        "is_abstract" => resolve_property_with(contexts, |v| {
            v.as_class()
                .unwrap_or_else(|| panic!("expected to have a class vertex, instead have: {v:#?}"))
                .class
                .modifiers
                .contains(oxc_ast::ast::ModifierKind::Abstract)
                .into()
        }),
        _ => {
            unreachable!("attempted to read unexpected property '{property_name}' on type 'Class'")
        }
    }
}

pub(super) fn resolve_class_method_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "accessibility" => resolve_property_with(contexts, |v| {
            v.as_class_method()
                .unwrap_or_else(|| {
                    panic!("expected to have a classmethod vertex, instead have: {v:#?}")
                })
                .method
                .accessibility
                .map(accessibility_to_string)
                .map_or(FieldValue::Null, Into::into)
        }),
        "is_abstract" => {
            resolve_property_with(contexts, field_property!(as_class_method, is_abstract))
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'ClassMethod'"
            )
        }
    }
}

pub(super) fn resolve_class_property_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "accessibility" => resolve_property_with(contexts, |v| {
            v.as_class_property()
                .unwrap_or_else(|| {
                    panic!("expected to have a classproperty vertex, instead have: {v:#?}")
                })
                .property
                .accessibility
                .map(accessibility_to_string)
                .map_or(FieldValue::Null, Into::into)
        }),
        "is_abstract" => {
            resolve_property_with(contexts, field_property!(as_class_method, is_abstract))
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'ClassProperty'"
            )
        }
    }
}

pub(super) fn resolve_default_import_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "local_name" => resolve_property_with(contexts, |v| {
            v.as_default_import()
                .unwrap_or_else(|| {
                    panic!("expected to have a defaultimport vertex, instead have: {v:#?}")
                })
                .local
                .name
                .to_string()
                .into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'DefaultImport'"
            )
        }
    }
}

pub(super) fn resolve_dot_property_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'DotProperty'"
            )
        }
    }
}

pub(super) fn resolve_expression_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'Expression'"
            )
        }
    }
}

pub(super) fn resolve_fn_declaration_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "name" => resolve_property_with(contexts, |v| {
            v.as_fn_declaration()
                .unwrap_or_else(|| {
                    panic!("expected to have a fndeclaration vertex, instead have: {v:#?}")
                })
                .function
                .id
                .as_ref()
                .map_or_else(|| FieldValue::Null, |f| f.name.to_string().into())
        }),
        "is_async" => resolve_property_with(contexts, |v| v.function_is_async().into()),
        "is_generator" => resolve_property_with(contexts, |v| v.function_is_generator().into()),
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        "is_getter" => resolve_property_with(contexts, |v| {
            v.function_scope_flag(adapter).contains(ScopeFlags::GetAccessor).into()
        }),
        "is_setter" => resolve_property_with(contexts, |v| {
            v.function_scope_flag(adapter).contains(ScopeFlags::SetAccessor).into()
        }),
        "is_constructor" => resolve_property_with(contexts, |v| {
            v.function_scope_flag(adapter).contains(ScopeFlags::Constructor).into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'FnDeclaration'"
            )
        }
    }
}

pub(super) fn resolve_fn_call_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!("attempted to read unexpected property '{property_name}' on type 'FnCall'")
        }
    }
}

pub(super) fn resolve_function_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "is_async" => resolve_property_with(contexts, |v| v.function_is_async().into()),
        "is_generator" => resolve_property_with(contexts, |v| v.function_is_generator().into()),
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        "is_getter" => resolve_property_with(contexts, |v| {
            v.function_scope_flag(adapter).contains(ScopeFlags::GetAccessor).into()
        }),
        "is_setter" => resolve_property_with(contexts, |v| {
            v.function_scope_flag(adapter).contains(ScopeFlags::SetAccessor).into()
        }),
        "is_constructor" => resolve_property_with(contexts, |v| {
            v.function_scope_flag(adapter).contains(ScopeFlags::Constructor).into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'Function'"
            )
        }
    }
}

pub(super) fn resolve_import_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "from_path" => resolve_property_with(contexts, |v| {
            v.as_import()
                .unwrap_or_else(|| {
                    panic!("expected to have an import vertex, instead have: {v:#?}")
                })
                .import
                .source
                .value
                .to_string()
                .into()
        }),
        _ => {
            unreachable!("attempted to read unexpected property '{property_name}' on type 'Import'")
        }
    }
}

pub(super) fn resolve_interface_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "name" => resolve_property_with(contexts, |v| {
            v.as_interface().unwrap().interface.id.name.to_string().into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'InterfaceExtend'"
            )
        }
    }
}

pub(super) fn resolve_jsxattribute_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "name" => resolve_property_with(contexts, |v| {
            jsx_attribute_name_to_string(
                &v.as_jsx_attribute()
                    .unwrap_or_else(|| {
                        panic!("expected to have a jsxattribute vertex, instead have: {v:#?}")
                    })
                    .name,
            )
            .into()
        }),
        "value_as_constant_string" => resolve_property_with(contexts, |v| {
            let attr = v.as_jsx_attribute().unwrap_or_else(|| {
                panic!("expected to have a jsxattribute vertex, instead have: {v:#?}")
            });
            jsx_attribute_to_constant_string(attr).map_or_else(|| FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'JSXAttribute'"
            )
        }
    }
}

pub(super) fn resolve_jsxelement_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        "child_count" => resolve_property_with(
            contexts,
            field_property!(as_jsx_element, element, { (element.children.len() as u64).into() }),
        ),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'JSXElement'"
            )
        }
    }
}

pub(super) fn resolve_jsxopening_element_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "attribute_count" => resolve_property_with(contexts, |v| {
            let jsx = v.as_jsx_opening_element().unwrap_or_else(|| {
                panic!("expected to have a jsxopeningelement vertex, instead have: {v:#?}")
            });
            (jsx.opening_element.attributes.len() as u64).into()
        }),
        "name" => resolve_property_with(contexts, |v| {
            let data = v.as_jsx_opening_element().unwrap_or_else(|| {
                panic!("expected to have a jsxopeningelement vertex, instead have: {v:#?}")
            });

            jsx_element_name_to_string(&data.opening_element.name).into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'JSXOpeningElement'"
            )
        }
    }
}

pub(super) fn resolve_jsxtext_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "text" => resolve_property_with(contexts, |v| {
            v.as_jsx_text()
                .unwrap_or_else(|| {
                    panic!("expected to have a jsxtext vertex, instead have: {v:#?}")
                })
                .value
                .to_string()
                .into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'JSXText'"
            )
        }
    }
}

pub(super) fn resolve_logical_expression_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "operator" => resolve_property_with(contexts, |v| {
            v.as_logical_expression()
                .unwrap_or_else(|| {
                    panic!("expected to have a logicalexpression vertex, instead have: {v:#?}")
                })
                .logical_expression
                .operator
                .as_str()
                .into()
        }),
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'LogicalExpression'"
            )
        }
    }
}

pub(super) fn resolve_name_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "name" => resolve_property_with(contexts, |v| {
            v.as_name()
                .unwrap_or_else(|| panic!("expected to have a name vertex, instead have: {v:#?}"))
                .name
                .name
                .to_string()
                .into()
        }),
        _ => {
            unreachable!("attempted to read unexpected property '{property_name}' on type 'Name'")
        }
    }
}

pub(super) fn resolve_new_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!("attempted to read unexpected property '{property_name}' on type 'New'")
        }
    }
}

pub(super) fn resolve_number_literal_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        "number" => resolve_property_with(contexts, |v| {
            let number = v
                .as_number_literal()
                .unwrap_or_else(|| {
                    panic!("expected to have a numberliteral vertex, instead have: {v:#?}")
                })
                .number_literal
                .value;

            if number.is_finite() {
                FieldValue::Float64(number)
            } else {
                FieldValue::Null
            }
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'NumberLiteral'"
            )
        }
    }
}

pub(super) fn resolve_object_literal_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'ObjectLiteral'"
            )
        }
    }
}

pub(super) fn resolve_parameter_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "is_readonly" => resolve_property_with(contexts, |v| {
            v.as_parameter()
                .unwrap_or_else(|| {
                    panic!("expected to have a parameter vertex, instead have: {v:#?}")
                })
                .parameter
                .readonly
                .into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'Parameter'"
            )
        }
    }
}

pub(super) fn resolve_parenthesized_expression_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'ParenthesizedExpression'"
            )
        }
    }
}

pub(super) fn resolve_path_part_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "is_first" => resolve_property_with(contexts, |v| {
            (*v.as_path_part().unwrap_or_else(|| {
                panic!("expected to have a pathpart vertex, instead have: {v:#?}")
            }) == 0)
                .into()
        }),
        "is_last" => {
            let len = adapter.path_components.len();
            resolve_property_with(contexts, move |v| {
                (*v.as_path_part().unwrap_or_else(|| {
                    panic!("expected to have a pathpart vertex, instead have: {v:#?}")
                }) == len - 1)
                    .into()
            })
        }
        "name" => resolve_property_with(contexts, |v| {
            adapter.path_components[*v.as_path_part().unwrap_or_else(|| {
                panic!("expected to have a pathpart vertex, instead have: {v:#?}")
            })]
            .as_ref()
            .map_or(FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'PathPart'"
            )
        }
    }
}

pub(super) fn resolve_reassignment_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'Reassignment'"
            )
        }
    }
}

pub(super) fn resolve_regexp_literal_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        "pattern" => resolve_property_with(contexts, |v| {
            v.as_reg_exp_literal()
                .unwrap_or_else(|| {
                    panic!("expected to have a regexpliteral vertex, instead have: {v:#?}")
                })
                .regexp
                .regex
                .pattern
                .to_string()
                .into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'RegExpLiteral'"
            )
        }
    }
}

pub(super) fn resolve_search_parameter_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "key" => resolve_property_with(contexts, |v| {
            v.as_search_parameter()
                .unwrap_or_else(|| {
                    panic!("expected to have a searchparameter vertex, instead have: {v:#?}")
                })
                .key
                .clone()
                .into()
        }),
        "value" => resolve_property_with(contexts, |v| {
            v.as_search_parameter()
                .unwrap_or_else(|| {
                    panic!("expected to have a searchparameter vertex, instead have: {v:#?}")
                })
                .value
                .clone()
                .into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'SearchParameter'"
            )
        }
    }
}

pub(super) fn resolve_span_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "str" => resolve_property_with(contexts, |v| {
            let span = v
                .as_span()
                .unwrap_or_else(|| panic!("expected to have a span vertex, instead have: {v:#?}"));
            adapter.semantic.source_text()[span.start as usize..span.end as usize].into()
        }),
        "end" => resolve_property_with(contexts, |v| {
            v.as_span()
                .unwrap_or_else(|| panic!("expected to have a span vertex, instead have: {v:#?}"))
                .end
                .into()
        }),
        "start" => resolve_property_with(contexts, |v| {
            v.as_span()
                .unwrap_or_else(|| panic!("expected to have a span vertex, instead have: {v:#?}"))
                .start
                .into()
        }),
        _ => {
            unreachable!("attempted to read unexpected property '{property_name}' on type 'Span'")
        }
    }
}

pub(super) fn resolve_specific_import_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "local_name" => resolve_property_with(contexts, |v| {
            v.as_specific_import()
                .unwrap_or_else(|| {
                    panic!("expected to have a specificimport vertex, instead have: {v:#?}")
                })
                .local
                .name
                .to_string()
                .into()
        }),
        "original_name" => resolve_property_with(contexts, |v| {
            v.as_specific_import()
                .unwrap_or_else(|| {
                    panic!("expected to have a specificimport vertex, instead have: {v:#?}")
                })
                .imported
                .name()
                .to_string()
                .into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'SpecificImport'"
            )
        }
    }
}

pub(super) fn resolve_string_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!("attempted to read unexpected property '{property_name}' on type 'String'")
        }
    }
}

pub(super) fn resolve_template_literal_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'TemplateLiteral'"
            )
        }
    }
}

pub(super) fn resolve_throw_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!("attempted to read unexpected property '{property_name}' on type 'Throw'")
        }
    }
}

pub(super) fn resolve_ternary_expression_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'TernaryExpression'"
            )
        }
    }
}

pub(super) fn resolve_unary_expression_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "operator" => resolve_property_with(contexts, |v| {
            v.as_unary_expression()
                .unwrap_or_else(|| {
                    panic!("expected to have a unaryexpression vertex, instead have: {v:#?}")
                })
                .unary_expression
                .operator
                .as_str()
                .into()
        }),
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'UnaryExpression'"
            )
        }
    }
}

pub(super) fn resolve_variable_declaration_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "kind" => resolve_property_with(contexts, |v| {
            match v
                .as_variable_declaration()
                .unwrap_or_else(|| {
                    panic!("expected to have a variabledeclaration vertex, instead have: {v:#?}",)
                })
                .variable_declaration
                .kind
            {
                oxc_ast::ast::VariableDeclarationKind::Var => "var".into(),
                oxc_ast::ast::VariableDeclarationKind::Const => "const".into(),
                oxc_ast::ast::VariableDeclarationKind::Let => "let".into(),
            }
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'LogicalExpression'"
            )
        }
    }
}

pub(super) fn resolve_var_ref_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "name" => resolve_property_with(contexts, |v| {
            v.as_var_ref()
                .unwrap_or_else(|| panic!("expected to have a varref vertex, instead have: {v:#?}"))
                .identifier_reference
                .name
                .to_string()
                .into()
        }),
        "as_constant_string" => resolve_property_with(contexts, |v| {
            v.as_constant_string().map_or(FieldValue::Null, Into::into)
        }),
        _ => {
            unreachable!("attempted to read unexpected property '{property_name}' on type 'VarRef'")
        }
    }
}
