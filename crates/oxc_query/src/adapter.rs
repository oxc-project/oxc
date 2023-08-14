use std::{
    rc::Rc,
    sync::{Arc, OnceLock},
};

use oxc_semantic::Semantic;
use trustfall::{
    provider::{
        resolve_coercion_using_schema, resolve_property_with, ContextIterator,
        ContextOutcomeIterator, EdgeParameters, ResolveEdgeInfo, ResolveInfo, Typename,
        VertexIterator,
    },
    FieldValue, Schema,
};

use super::vertex::Vertex;

static SCHEMA: OnceLock<Schema> = OnceLock::new();

#[non_exhaustive]
pub struct Adapter<'a> {
    pub semantic: Rc<Semantic<'a>>,
    pub path_components: Vec<Option<String>>,
}

pub const SCHEMA_TEXT: &str = include_str!("./schema.graphql");

/// Returns the schema from a OnceLock
///
/// # Panics
/// If the schema parse returns an error, which will not happen unless the schema get's corrupted.
pub fn schema() -> &'static Schema {
    // internal note: this might not parser correctly due to making an incorrect schema during development
    SCHEMA.get_or_init(|| Schema::parse(SCHEMA_TEXT).unwrap_or_else(|e| panic!("{}", e)))
}

impl<'a> Adapter<'a> {
    pub fn new(semantic: Rc<Semantic<'a>>, path_components: Vec<Option<String>>) -> Self {
        Self { semantic, path_components }
    }
}

impl<'a, 'b: 'a> trustfall::provider::Adapter<'a> for &'a Adapter<'b> {
    type Vertex = Vertex<'b>;

    fn resolve_starting_vertices(
        &self,
        edge_name: &Arc<str>,
        _parameters: &EdgeParameters,
        resolve_info: &ResolveInfo,
    ) -> VertexIterator<'a, Self::Vertex> {
        match edge_name.as_ref() {
            "File" => super::entrypoints::file(resolve_info),
            _ => {
                unreachable!(
                    "attempted to resolve starting vertices for unexpected edge name: {edge_name}"
                )
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn resolve_property(
        &self,
        contexts: ContextIterator<'a, Self::Vertex>,
        type_name: &Arc<str>,
        property_name: &Arc<str>,
        resolve_info: &ResolveInfo,
    ) -> ContextOutcomeIterator<'a, Self::Vertex, FieldValue> {
        if property_name.as_ref() == "__typename" {
            return resolve_property_with(contexts, |v| v.typename().into());
        }
        match type_name.as_ref() {
            "AssignmentType" => super::properties::resolve_assignment_type_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "ClassAST" | "Class" => super::properties::resolve_class_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "ClassMethod" => super::properties::resolve_class_method_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "ClassProperty" => super::properties::resolve_class_property_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "DefaultImport" => super::properties::resolve_default_import_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "DotPropertyAST" | "DotProperty" => super::properties::resolve_dot_property_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "Expression" => super::properties::resolve_expression_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "FnCallAST" | "FnCall" => super::properties::resolve_fn_call_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "ImportAST" | "Import" => super::properties::resolve_import_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "InterfaceAST" | "Interface" => super::properties::resolve_interface_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "JSXAttribute" => super::properties::resolve_jsxattribute_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "JSXElementAST" | "JSXElement" => super::properties::resolve_jsxelement_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "JSXOpeningElementAST" | "JSXOpeningElement" => {
                super::properties::resolve_jsxopening_element_property(
                    contexts,
                    property_name.as_ref(),
                    resolve_info,
                )
            }
            "JSXText" => super::properties::resolve_jsxtext_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "NameAST" | "Name" => super::properties::resolve_name_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "NumberLiteralAST" | "NumberLiteral" => {
                super::properties::resolve_number_literal_property(
                    contexts,
                    property_name.as_ref(),
                    resolve_info,
                )
            }
            "ObjectLiteralAST" | "ObjectLiteral" => {
                super::properties::resolve_object_literal_property(
                    contexts,
                    property_name.as_ref(),
                    resolve_info,
                )
            }
            "PathPart" => super::properties::resolve_path_part_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
                self,
            ),
            "ReassignmentAST" | "Reassignment" => super::properties::resolve_reassignment_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "SearchParameter" => super::properties::resolve_search_parameter_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "Span" => super::properties::resolve_span_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
                self,
            ),
            "SpecificImport" => super::properties::resolve_specific_import_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            _ => {
                unreachable!(
                    "attempted to read property '{property_name}' on unexpected type: {type_name}"
                )
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn resolve_neighbors(
        &self,
        contexts: ContextIterator<'a, Self::Vertex>,
        type_name: &Arc<str>,
        edge_name: &Arc<str>,
        parameters: &EdgeParameters,
        resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Self::Vertex, VertexIterator<'a, Self::Vertex>> {
        match type_name.as_ref() {
            "ASTNode" => super::edges::resolve_astnode_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "Argument" => super::edges::resolve_argument_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "AssignmentType" => super::edges::resolve_assignment_type_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "Class" | "ClassAST" => super::edges::resolve_class_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "ClassMethod" => super::edges::resolve_class_method_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "ClassProperty" => super::edges::resolve_class_property_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "DotPropertyAST" | "DotProperty" => super::edges::resolve_dot_property_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "DefaultImport" => super::edges::resolve_default_import_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "Expression" => super::edges::resolve_expression_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "File" => super::edges::resolve_file_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "FnCallAST" | "FnCall" => super::edges::resolve_fn_call_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "HasSpan" => super::edges::resolve_has_span_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "IfStatementAST" => super::edges::resolve_if_statement_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "Import" | "ImportAST" => super::edges::resolve_import_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "Interface" | "InterfaceAST" => super::edges::resolve_interface_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "InterfaceExtend" => super::edges::resolve_interface_extend_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "JSXAttribute" => super::edges::resolve_jsxattribute_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "JSXElement" | "JSXElementAST" => super::edges::resolve_jsxelement_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "JSXExpressionContainer" => super::edges::resolve_jsxexpression_container_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "JSXFragment" => super::edges::resolve_jsxfragment_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "JSXOpeningElementAST" | "JSXOpeningElement" => {
                super::edges::resolve_jsxopening_element_edge(
                    contexts,
                    edge_name.as_ref(),
                    parameters,
                    resolve_info,
                    self,
                )
            }
            "JSXSpreadAttribute" => super::edges::resolve_jsxspread_attribute_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "JSXSpreadChild" => super::edges::resolve_jsxspread_child_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "JSXText" => super::edges::resolve_jsxtext_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "MemberExtend" => super::edges::resolve_member_extend_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "NameAST" | "Name" => super::edges::resolve_name_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "NumberLiteralAST" | "NumberLiteral" => super::edges::resolve_number_literal_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "ObjectEntryAST" | "ObjectEntry" => super::edges::resolve_object_entry_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "ObjectLiteral" | "ObjectLiteralAST" => super::edges::resolve_object_literal_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "ObjectProperty" => super::edges::resolve_object_property_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "PathPart" => super::edges::resolve_path_part_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "ReassignmentAST" | "Reassignment" => super::edges::resolve_reassignment_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "ReturnStatementAST" => super::edges::resolve_return_statement_ast_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "SimpleExtend" => super::edges::resolve_simple_extend_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "SpecificImport" => super::edges::resolve_specific_import_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "SpreadIntoObjectAST" | "SpreadIntoObject" => {
                super::edges::resolve_spread_into_object_edge(
                    contexts,
                    edge_name.as_ref(),
                    parameters,
                    resolve_info,
                    self,
                )
            }
            "TypeAnnotation" | "TypeAnnotationAST" => super::edges::resolve_type_annotation_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
                self,
            ),
            "Type" => super::edges::resolve_type_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "URL" => super::edges::resolve_url_edge(
                contexts,
                edge_name.as_ref(),
                parameters,
                resolve_info,
            ),
            "VariableDeclaration" | "VariableDeclarationAST" => {
                super::edges::resolve_variable_declaration_edge(
                    contexts,
                    edge_name.as_ref(),
                    parameters,
                    resolve_info,
                    self,
                )
            }
            _ => {
                unreachable!(
                    "attempted to resolve edge '{edge_name}' on unexpected type: {type_name}"
                )
            }
        }
    }

    fn resolve_coercion(
        &self,
        contexts: ContextIterator<'a, Self::Vertex>,
        _type_name: &Arc<str>,
        coerce_to_type: &Arc<str>,
        _resolve_info: &ResolveInfo,
    ) -> ContextOutcomeIterator<'a, Self::Vertex, bool> {
        resolve_coercion_using_schema(contexts, schema(), coerce_to_type.as_ref())
    }
}
