use trustfall::{
    provider::{ContextIterator, ContextOutcomeIterator, ResolveInfo},
    FieldValue,
};

use super::vertex::Vertex;

pub(super) fn resolve_assignment_type_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "assignment_to_variable_name" => {
            todo!(
                "implement property 'assignment_to_variable_name' in fn `resolve_assignment_type_property()`"
            )
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'AssignmentType'"
            )
        }
    }
}

pub(super) fn resolve_class_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "is_abstract" => {
            todo!("implement property 'is_abstract' in fn `resolve_class_property()`")
        }
        _ => {
            unreachable!("attempted to read unexpected property '{property_name}' on type 'Class'")
        }
    }
}

pub(super) fn resolve_class_ast_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "extended_class_name" => {
            todo!("implement property 'extended_class_name' in fn `resolve_class_ast_property()`")
        }
        "is_abstract" => {
            todo!("implement property 'is_abstract' in fn `resolve_class_ast_property()`")
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'ClassAST'"
            )
        }
    }
}

pub(super) fn resolve_class_method_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "accessibility" => {
            todo!("implement property 'accessibility' in fn `resolve_class_method_property()`")
        }
        "is_abstract" => {
            todo!("implement property 'is_abstract' in fn `resolve_class_method_property()`")
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'ClassMethod'"
            )
        }
    }
}

pub(super) fn resolve_class_property_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "accessibility" => {
            todo!("implement property 'accessibility' in fn `resolve_class_property_property()`")
        }
        "is_abstract" => {
            todo!("implement property 'is_abstract' in fn `resolve_class_property_property()`")
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'ClassProperty'"
            )
        }
    }
}

pub(super) fn resolve_default_import_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "local_name" => {
            todo!("implement property 'local_name' in fn `resolve_default_import_property()`")
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'DefaultImport'"
            )
        }
    }
}

pub(super) fn resolve_expression_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "as_constant_string" => {
            todo!("implement property 'as_constant_string' in fn `resolve_expression_property()`")
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'Expression'"
            )
        }
    }
}

pub(super) fn resolve_import_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "from_path" => {
            todo!("implement property 'from_path' in fn `resolve_import_property()`")
        }
        _ => {
            unreachable!("attempted to read unexpected property '{property_name}' on type 'Import'")
        }
    }
}

pub(super) fn resolve_import_ast_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "from_path" => {
            todo!("implement property 'from_path' in fn `resolve_import_ast_property()`")
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'ImportAST'"
            )
        }
    }
}

pub(super) fn resolve_interface_extend_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "str" => {
            todo!("implement property 'str' in fn `resolve_interface_extend_property()`")
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'InterfaceExtend'"
            )
        }
    }
}

pub(super) fn resolve_jsxattribute_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "name" => {
            todo!("implement property 'name' in fn `resolve_jsxattribute_property()`")
        }
        "value_as_constant_string" => {
            todo!(
                "implement property 'value_as_constant_string' in fn `resolve_jsxattribute_property()`"
            )
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'JSXAttribute'"
            )
        }
    }
}

pub(super) fn resolve_jsxelement_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "as_constant_string" => {
            todo!("implement property 'as_constant_string' in fn `resolve_jsxelement_property()`")
        }
        "child_count" => {
            todo!("implement property 'child_count' in fn `resolve_jsxelement_property()`")
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'JSXElement'"
            )
        }
    }
}

pub(super) fn resolve_jsxelement_ast_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "as_constant_string" => {
            todo!(
                "implement property 'as_constant_string' in fn `resolve_jsxelement_ast_property()`"
            )
        }
        "child_count" => {
            todo!("implement property 'child_count' in fn `resolve_jsxelement_ast_property()`")
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'JSXElementAST'"
            )
        }
    }
}

pub(super) fn resolve_jsxopening_element_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "attribute_count" => {
            todo!(
                "implement property 'attribute_count' in fn `resolve_jsxopening_element_property()`"
            )
        }
        "name" => {
            todo!("implement property 'name' in fn `resolve_jsxopening_element_property()`")
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'JSXOpeningElement'"
            )
        }
    }
}

pub(super) fn resolve_jsxtext_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "text" => todo!("implement property 'text' in fn `resolve_jsxtext_property()`"),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'JSXText'"
            )
        }
    }
}

pub(super) fn resolve_member_extend_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "str" => {
            todo!("implement property 'str' in fn `resolve_member_extend_property()`")
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'MemberExtend'"
            )
        }
    }
}

pub(super) fn resolve_object_literal_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "as_constant_string" => {
            todo!(
                "implement property 'as_constant_string' in fn `resolve_object_literal_property()`"
            )
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'ObjectLiteral'"
            )
        }
    }
}

pub(super) fn resolve_path_part_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "is_first" => {
            todo!("implement property 'is_first' in fn `resolve_path_part_property()`")
        }
        "is_last" => {
            todo!("implement property 'is_last' in fn `resolve_path_part_property()`")
        }
        "name" => todo!("implement property 'name' in fn `resolve_path_part_property()`"),
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'PathPart'"
            )
        }
    }
}

pub(super) fn resolve_search_parameter_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "key" => {
            todo!("implement property 'key' in fn `resolve_search_parameter_property()`")
        }
        "value" => {
            todo!("implement property 'value' in fn `resolve_search_parameter_property()`")
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'SearchParameter'"
            )
        }
    }
}

pub(super) fn resolve_simple_extend_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "str" => {
            todo!("implement property 'str' in fn `resolve_simple_extend_property()`")
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'SimpleExtend'"
            )
        }
    }
}

pub(super) fn resolve_span_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "end" => todo!("implement property 'end' in fn `resolve_span_property()`"),
        "start" => todo!("implement property 'start' in fn `resolve_span_property()`"),
        _ => {
            unreachable!("attempted to read unexpected property '{property_name}' on type 'Span'")
        }
    }
}

pub(super) fn resolve_specific_import_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "local_name" => {
            todo!("implement property 'local_name' in fn `resolve_specific_import_property()`")
        }
        "original_name" => {
            todo!("implement property 'original_name' in fn `resolve_specific_import_property()`")
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'SpecificImport'"
            )
        }
    }
}

pub(super) fn resolve_type_property<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, Vertex, FieldValue> {
    match property_name {
        "str" => todo!("implement property 'str' in fn `resolve_type_property()`"),
        _ => {
            unreachable!("attempted to read unexpected property '{property_name}' on type 'Type_'")
        }
    }
}
