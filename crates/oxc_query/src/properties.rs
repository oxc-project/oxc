use std::convert::Into;

use oxc_ast::ast::{BindingPatternKind, Expression, MemberExpression};
use oxc_span::GetSpan;
use trustfall::{
    provider::{resolve_property_with, ContextIterator, ContextOutcomeIterator, ResolveInfo},
    FieldValue,
};

use super::vertex::Vertex;
use crate::{
    util::{
        accessibility_to_string, jsx_attribute_name_to_string, jsx_attribute_to_constant_string,
        jsx_element_name_to_string,
    },
    vertex::InterfaceExtendVertex,
    Adapter,
};

fn interface_extend_implem<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    resolve_property_with(contexts, |v| {
        match v.as_interface_extend().unwrap().as_ref() {
            InterfaceExtendVertex::Identifier(ident) => ident.name.to_string(),
            InterfaceExtendVertex::MemberExpression(first_membexpr) => {
                let MemberExpression::StaticMemberExpression(static_membexpr) = first_membexpr
                else {
                    unreachable!("TS:2499")
                };
                let mut parts = vec![static_membexpr.property.name.to_string()];
                let mut membexpr = first_membexpr.object();
                while let Expression::MemberExpression(expr) = membexpr {
                    let MemberExpression::StaticMemberExpression(static_membexpr) = &expr.0 else {
                        unreachable!("TS:2499")
                    };
                    parts.push(static_membexpr.property.name.to_string());
                    membexpr = expr.object();
                }

                let Expression::Identifier(ident) = membexpr else { unreachable!("TS:2499") };
                parts.push(ident.name.to_string());

                parts.reverse();

                parts.join(".")
            }
        }
        .into()
    })
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
            let Some(Expression::Identifier(ref ident)) = v.as_class().unwrap().class.super_class
            else {
                return FieldValue::Null;
            };
            ident.name.to_string().into()
        }),
        "is_abstract" => resolve_property_with(contexts, |v| {
            v.as_class()
                .unwrap()
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
                .unwrap()
                .method
                .accessibility
                .map(accessibility_to_string)
                .map_or(FieldValue::Null, Into::into)
        }),
        "is_abstract" => {
            resolve_property_with(contexts, |v| v.as_class_method().unwrap().is_abstract.into())
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
                .unwrap()
                .property
                .accessibility
                .map(accessibility_to_string)
                .map_or(FieldValue::Null, Into::into)
        }),
        "is_abstract" => {
            resolve_property_with(contexts, |v| v.as_class_property().unwrap().is_abstract.into())
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
            v.as_default_import().unwrap().local.name.to_string().into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'DefaultImport'"
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

pub(super) fn resolve_import_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "from_path" => resolve_property_with(contexts, |v| {
            let Vertex::Import(data) = &v else { unreachable!() };
            data.import.source.value.to_string().into()
        }),
        _ => {
            unreachable!("attempted to read unexpected property '{property_name}' on type 'Import'")
        }
    }
}

pub(super) fn resolve_import_ast_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "from_path" => resolve_import_property(contexts, property_name, resolve_info),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'ImportAST'"
            )
        }
    }
}

pub(super) fn resolve_interface_extend_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "str" => interface_extend_implem(contexts),
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
            jsx_attribute_name_to_string(&v.as_jsx_attribute().unwrap().name).into()
        }),
        "value_as_constant_string" => resolve_property_with(contexts, |v| {
            let attr = v.as_jsx_attribute().unwrap();
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
        "child_count" => resolve_property_with(contexts, |v| {
            let Vertex::JSXElement(data) = &v else { unreachable!() };
            (data.element.children.len() as u64).into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'JSXElement'"
            )
        }
    }
}

pub(super) fn resolve_jsxelement_ast_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "child_count" | "as_constant_string" => {
            resolve_jsxelement_property(contexts, property_name, resolve_info)
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'JSXElementAST'"
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
            let jsx = v.as_jsx_opening_element().unwrap();
            (jsx.attributes.len() as u64).into()
        }),
        "name" => resolve_property_with(contexts, |v| {
            let jsx = v.as_jsx_opening_element().unwrap();
            jsx_element_name_to_string(&jsx.name).into()
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
        "text" => {
            resolve_property_with(contexts, |v| v.as_jsx_text().unwrap().value.to_string().into())
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'JSXText'"
            )
        }
    }
}

pub(super) fn resolve_member_extend_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "str" => interface_extend_implem(contexts),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'MemberExtend'"
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

pub(super) fn resolve_path_part_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "is_first" => resolve_property_with(contexts, |v| (*v.as_path_part().unwrap() == 0).into()),
        "is_last" => {
            let len = adapter.path_components.len();
            resolve_property_with(contexts, move |v| (*v.as_path_part().unwrap() == len - 1).into())
        }
        "name" => resolve_property_with(contexts, |v| {
            adapter.path_components[*v.as_path_part().unwrap()]
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

pub(super) fn resolve_search_parameter_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "key" => {
            resolve_property_with(contexts, |v| v.as_search_parameter().unwrap().key.clone().into())
        }
        "value" => resolve_property_with(contexts, |v| {
            v.as_search_parameter().unwrap().value.clone().into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'SearchParameter'"
            )
        }
    }
}

pub(super) fn resolve_simple_extend_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "str" => interface_extend_implem(contexts),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'SimpleExtend'"
            )
        }
    }
}

pub(super) fn resolve_span_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "end" => resolve_property_with(contexts, |v| v.as_span().unwrap().end.into()),
        "start" => resolve_property_with(contexts, |v| v.as_span().unwrap().start.into()),
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
            v.as_specific_import().unwrap().local.name.to_string().into()
        }),
        "original_name" => resolve_property_with(contexts, |v| {
            v.as_specific_import().unwrap().imported.name().to_string().into()
        }),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'SpecificImport'"
            )
        }
    }
}

pub(super) fn resolve_type_property<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, FieldValue> {
    match property_name {
        "str" => resolve_property_with(contexts, |v| {
            let span = v.as_type().unwrap().span();
            adapter.semantic.source_text()[span.start as usize..span.end as usize].into()
        }),
        _ => {
            unreachable!("attempted to read unexpected property '{property_name}' on type 'Type_'")
        }
    }
}
