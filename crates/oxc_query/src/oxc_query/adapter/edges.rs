use trustfall::provider::{
    ContextIterator, ContextOutcomeIterator, EdgeParameters, ResolveEdgeInfo, VertexIterator,
};

use super::vertex::Vertex;

pub(super) fn resolve_astnode_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "ancestor" => astnode::ancestor(contexts, resolve_info),
        "parent" => astnode::parent(contexts, resolve_info),
        "span" => astnode::span(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'ASTNode'")
        }
    }
}

mod astnode {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn ancestor<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'ancestor' for type 'ASTNode'")
    }

    pub(super) fn parent<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'parent' for type 'ASTNode'")
    }

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'ASTNode'")
    }
}

pub(super) fn resolve_assignment_type_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => assignment_type::span(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'AssignmentType'"
            )
        }
    }
}

mod assignment_type {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'AssignmentType'")
    }
}

pub(super) fn resolve_class_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "constructor" => class::constructor(contexts, resolve_info),
        "entire_class_span" => class::entire_class_span(contexts, resolve_info),
        "getter" => class::getter(contexts, resolve_info),
        "method" => class::method(contexts, resolve_info),
        "name_span" => class::name_span(contexts, resolve_info),
        "property" => class::property(contexts, resolve_info),
        "setter" => class::setter(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'Class'")
        }
    }
}

mod class {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn constructor<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'constructor' for type 'Class'")
    }

    pub(super) fn entire_class_span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'entire_class_span' for type 'Class'")
    }

    pub(super) fn getter<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'getter' for type 'Class'")
    }

    pub(super) fn method<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'method' for type 'Class'")
    }

    pub(super) fn name_span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'name_span' for type 'Class'")
    }

    pub(super) fn property<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'property' for type 'Class'")
    }

    pub(super) fn setter<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'setter' for type 'Class'")
    }
}

pub(super) fn resolve_class_ast_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "ancestor" => class_ast::ancestor(contexts, resolve_info),
        "constructor" => class_ast::constructor(contexts, resolve_info),
        "entire_class_span" => class_ast::entire_class_span(contexts, resolve_info),
        "getter" => class_ast::getter(contexts, resolve_info),
        "method" => class_ast::method(contexts, resolve_info),
        "name_span" => class_ast::name_span(contexts, resolve_info),
        "parent" => class_ast::parent(contexts, resolve_info),
        "property" => class_ast::property(contexts, resolve_info),
        "setter" => class_ast::setter(contexts, resolve_info),
        "span" => class_ast::span(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'ClassAST'")
        }
    }
}

mod class_ast {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn ancestor<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'ancestor' for type 'ClassAST'")
    }

    pub(super) fn constructor<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'constructor' for type 'ClassAST'")
    }

    pub(super) fn entire_class_span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'entire_class_span' for type 'ClassAST'")
    }

    pub(super) fn getter<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'getter' for type 'ClassAST'")
    }

    pub(super) fn method<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'method' for type 'ClassAST'")
    }

    pub(super) fn name_span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'name_span' for type 'ClassAST'")
    }

    pub(super) fn parent<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'parent' for type 'ClassAST'")
    }

    pub(super) fn property<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'property' for type 'ClassAST'")
    }

    pub(super) fn setter<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'setter' for type 'ClassAST'")
    }

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'ClassAST'")
    }
}

pub(super) fn resolve_class_method_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => class_method::span(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'ClassMethod'")
        }
    }
}

mod class_method {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'ClassMethod'")
    }
}

pub(super) fn resolve_class_property_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => class_property::span(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'ClassProperty'"
            )
        }
    }
}

mod class_property {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'ClassProperty'")
    }
}

pub(super) fn resolve_default_import_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => default_import::span(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'DefaultImport'"
            )
        }
    }
}

mod default_import {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'DefaultImport'")
    }
}

pub(super) fn resolve_expression_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => expression::span(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'Expression'")
        }
    }
}

mod expression {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'Expression'")
    }
}

pub(super) fn resolve_file_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "ast_node" => file::ast_node(contexts, resolve_info),
        "class" => file::class(contexts, resolve_info),
        "import" => file::import(contexts, resolve_info),
        "interface" => file::interface(contexts, resolve_info),
        "jsx_element" => file::jsx_element(contexts, resolve_info),
        "last_path_part" => file::last_path_part(contexts, resolve_info),
        "path_part" => file::path_part(contexts, resolve_info),
        "type_annotation" => file::type_annotation(contexts, resolve_info),
        "variable_declaration" => file::variable_declaration(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'File'")
        }
    }
}

mod file {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn ast_node<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'ast_node' for type 'File'")
    }

    pub(super) fn class<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'class' for type 'File'")
    }

    pub(super) fn import<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'import' for type 'File'")
    }

    pub(super) fn interface<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'interface' for type 'File'")
    }

    pub(super) fn jsx_element<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'jsx_element' for type 'File'")
    }

    pub(super) fn last_path_part<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'last_path_part' for type 'File'")
    }

    pub(super) fn path_part<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'path_part' for type 'File'")
    }

    pub(super) fn type_annotation<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'type_annotation' for type 'File'")
    }

    pub(super) fn variable_declaration<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'variable_declaration' for type 'File'")
    }
}

pub(super) fn resolve_has_span_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => has_span::span(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'HasSpan'")
        }
    }
}

mod has_span {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'HasSpan'")
    }
}

pub(super) fn resolve_import_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "default_import" => import::default_import(contexts, resolve_info),
        "entire_span" => import::entire_span(contexts, resolve_info),
        "specific_import" => import::specific_import(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'Import'")
        }
    }
}

mod import {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn default_import<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'default_import' for type 'Import'")
    }

    pub(super) fn entire_span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'entire_span' for type 'Import'")
    }

    pub(super) fn specific_import<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'specific_import' for type 'Import'")
    }
}

pub(super) fn resolve_import_ast_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "ancestor" => import_ast::ancestor(contexts, resolve_info),
        "default_import" => import_ast::default_import(contexts, resolve_info),
        "entire_span" => import_ast::entire_span(contexts, resolve_info),
        "parent" => import_ast::parent(contexts, resolve_info),
        "span" => import_ast::span(contexts, resolve_info),
        "specific_import" => import_ast::specific_import(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'ImportAST'")
        }
    }
}

mod import_ast {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn ancestor<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'ancestor' for type 'ImportAST'")
    }

    pub(super) fn default_import<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'default_import' for type 'ImportAST'")
    }

    pub(super) fn entire_span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'entire_span' for type 'ImportAST'")
    }

    pub(super) fn parent<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'parent' for type 'ImportAST'")
    }

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'ImportAST'")
    }

    pub(super) fn specific_import<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'specific_import' for type 'ImportAST'")
    }
}

pub(super) fn resolve_interface_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "entire_span" => interface::entire_span(contexts, resolve_info),
        "extend" => interface::extend(contexts, resolve_info),
        "name_span" => interface::name_span(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'Interface'")
        }
    }
}

mod interface {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn entire_span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'entire_span' for type 'Interface'")
    }

    pub(super) fn extend<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'extend' for type 'Interface'")
    }

    pub(super) fn name_span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'name_span' for type 'Interface'")
    }
}

pub(super) fn resolve_interface_ast_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "ancestor" => interface_ast::ancestor(contexts, resolve_info),
        "entire_span" => interface_ast::entire_span(contexts, resolve_info),
        "extend" => interface_ast::extend(contexts, resolve_info),
        "name_span" => interface_ast::name_span(contexts, resolve_info),
        "parent" => interface_ast::parent(contexts, resolve_info),
        "span" => interface_ast::span(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'InterfaceAST'"
            )
        }
    }
}

mod interface_ast {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn ancestor<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'ancestor' for type 'InterfaceAST'")
    }

    pub(super) fn entire_span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'entire_span' for type 'InterfaceAST'")
    }

    pub(super) fn extend<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'extend' for type 'InterfaceAST'")
    }

    pub(super) fn name_span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'name_span' for type 'InterfaceAST'")
    }

    pub(super) fn parent<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'parent' for type 'InterfaceAST'")
    }

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'InterfaceAST'")
    }
}

pub(super) fn resolve_interface_extend_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => interface_extend::span(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'InterfaceExtend'"
            )
        }
    }
}

mod interface_extend {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'InterfaceExtend'")
    }
}

pub(super) fn resolve_jsxattribute_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => jsxattribute::span(contexts, resolve_info),
        "value_as_expression" => jsxattribute::value_as_expression(contexts, resolve_info),
        "value_as_jsx_element" => jsxattribute::value_as_jsx_element(contexts, resolve_info),
        "value_as_url" => jsxattribute::value_as_url(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'JSXAttribute'"
            )
        }
    }
}

mod jsxattribute {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'JSXAttribute'")
    }

    pub(super) fn value_as_expression<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'value_as_expression' for type 'JSXAttribute'")
    }

    pub(super) fn value_as_jsx_element<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'value_as_jsx_element' for type 'JSXAttribute'")
    }

    pub(super) fn value_as_url<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'value_as_url' for type 'JSXAttribute'")
    }
}

pub(super) fn resolve_jsxelement_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "child_element" => jsxelement::child_element(contexts, resolve_info),
        "child_expression_container" => {
            jsxelement::child_expression_container(contexts, resolve_info)
        }
        "child_fragment" => jsxelement::child_fragment(contexts, resolve_info),
        "child_spread" => jsxelement::child_spread(contexts, resolve_info),
        "child_text" => jsxelement::child_text(contexts, resolve_info),
        "opening_element" => jsxelement::opening_element(contexts, resolve_info),
        "span" => jsxelement::span(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'JSXElement'")
        }
    }
}

mod jsxelement {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn child_element<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'child_element' for type 'JSXElement'")
    }

    pub(super) fn child_expression_container<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'child_expression_container' for type 'JSXElement'")
    }

    pub(super) fn child_fragment<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'child_fragment' for type 'JSXElement'")
    }

    pub(super) fn child_spread<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'child_spread' for type 'JSXElement'")
    }

    pub(super) fn child_text<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'child_text' for type 'JSXElement'")
    }

    pub(super) fn opening_element<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'opening_element' for type 'JSXElement'")
    }

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'JSXElement'")
    }
}

pub(super) fn resolve_jsxelement_ast_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "ancestor" => jsxelement_ast::ancestor(contexts, resolve_info),
        "child_element" => jsxelement_ast::child_element(contexts, resolve_info),
        "child_expression_container" => {
            jsxelement_ast::child_expression_container(contexts, resolve_info)
        }
        "child_fragment" => jsxelement_ast::child_fragment(contexts, resolve_info),
        "child_spread" => jsxelement_ast::child_spread(contexts, resolve_info),
        "child_text" => jsxelement_ast::child_text(contexts, resolve_info),
        "opening_element" => jsxelement_ast::opening_element(contexts, resolve_info),
        "parent" => jsxelement_ast::parent(contexts, resolve_info),
        "span" => jsxelement_ast::span(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'JSXElementAST'"
            )
        }
    }
}

mod jsxelement_ast {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn ancestor<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'ancestor' for type 'JSXElementAST'")
    }

    pub(super) fn child_element<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'child_element' for type 'JSXElementAST'")
    }

    pub(super) fn child_expression_container<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'child_expression_container' for type 'JSXElementAST'")
    }

    pub(super) fn child_fragment<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'child_fragment' for type 'JSXElementAST'")
    }

    pub(super) fn child_spread<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'child_spread' for type 'JSXElementAST'")
    }

    pub(super) fn child_text<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'child_text' for type 'JSXElementAST'")
    }

    pub(super) fn opening_element<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'opening_element' for type 'JSXElementAST'")
    }

    pub(super) fn parent<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'parent' for type 'JSXElementAST'")
    }

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'JSXElementAST'")
    }
}

pub(super) fn resolve_jsxexpression_container_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => jsxexpression_container::span(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'JSXExpressionContainer'"
            )
        }
    }
}

mod jsxexpression_container {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'JSXExpressionContainer'")
    }
}

pub(super) fn resolve_jsxfragment_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => jsxfragment::span(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'JSXFragment'")
        }
    }
}

mod jsxfragment {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'JSXFragment'")
    }
}

pub(super) fn resolve_jsxopening_element_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "attribute" => jsxopening_element::attribute(contexts, resolve_info),
        "span" => jsxopening_element::span(contexts, resolve_info),
        "spread_attribute" => jsxopening_element::spread_attribute(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'JSXOpeningElement'"
            )
        }
    }
}

mod jsxopening_element {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn attribute<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'attribute' for type 'JSXOpeningElement'")
    }

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'JSXOpeningElement'")
    }

    pub(super) fn spread_attribute<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'spread_attribute' for type 'JSXOpeningElement'")
    }
}

pub(super) fn resolve_jsxspread_attribute_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => jsxspread_attribute::span(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'JSXSpreadAttribute'"
            )
        }
    }
}

mod jsxspread_attribute {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'JSXSpreadAttribute'")
    }
}

pub(super) fn resolve_jsxspread_child_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => jsxspread_child::span(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'JSXSpreadChild'"
            )
        }
    }
}

mod jsxspread_child {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'JSXSpreadChild'")
    }
}

pub(super) fn resolve_jsxtext_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => jsxtext::span(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'JSXText'")
        }
    }
}

mod jsxtext {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'JSXText'")
    }
}

pub(super) fn resolve_member_extend_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => member_extend::span(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'MemberExtend'"
            )
        }
    }
}

mod member_extend {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'MemberExtend'")
    }
}

pub(super) fn resolve_object_literal_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => object_literal::span(contexts, resolve_info),
        "value" => {
            let key: &str = parameters
                .get("key")
                .expect("failed to find parameter 'key' for edge 'value' on type 'ObjectLiteral'")
                .as_str()
                .expect("unexpected null or other incorrect datatype for Trustfall type 'String!'");
            object_literal::value(contexts, key, resolve_info)
        }
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'ObjectLiteral'"
            )
        }
    }
}

mod object_literal {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'ObjectLiteral'")
    }

    pub(super) fn value<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        key: &str,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'value' for type 'ObjectLiteral'")
    }
}

pub(super) fn resolve_path_part_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "next" => path_part::next(contexts, resolve_info),
        "prev" => path_part::prev(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'PathPart'")
        }
    }
}

mod path_part {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn next<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'next' for type 'PathPart'")
    }

    pub(super) fn prev<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'prev' for type 'PathPart'")
    }
}

pub(super) fn resolve_return_statement_ast_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "ancestor" => return_statement_ast::ancestor(contexts, resolve_info),
        "expression" => return_statement_ast::expression(contexts, resolve_info),
        "parent" => return_statement_ast::parent(contexts, resolve_info),
        "span" => return_statement_ast::span(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'ReturnStatementAST'"
            )
        }
    }
}

mod return_statement_ast {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn ancestor<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'ancestor' for type 'ReturnStatementAST'")
    }

    pub(super) fn expression<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'expression' for type 'ReturnStatementAST'")
    }

    pub(super) fn parent<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'parent' for type 'ReturnStatementAST'")
    }

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'ReturnStatementAST'")
    }
}

pub(super) fn resolve_simple_extend_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => simple_extend::span(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'SimpleExtend'"
            )
        }
    }
}

mod simple_extend {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'SimpleExtend'")
    }
}

pub(super) fn resolve_specific_import_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => specific_import::span(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'SpecificImport'"
            )
        }
    }
}

mod specific_import {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'SpecificImport'")
    }
}

pub(super) fn resolve_type_annotation_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => type_annotation::span(contexts, resolve_info),
        "type_" => type_annotation::type_(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'TypeAnnotation'"
            )
        }
    }
}

mod type_annotation {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'TypeAnnotation'")
    }

    pub(super) fn type_<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'type_' for type 'TypeAnnotation'")
    }
}

pub(super) fn resolve_type_annotation_ast_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "ancestor" => type_annotation_ast::ancestor(contexts, resolve_info),
        "parent" => type_annotation_ast::parent(contexts, resolve_info),
        "span" => type_annotation_ast::span(contexts, resolve_info),
        "type_" => type_annotation_ast::type_(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'TypeAnnotationAST'"
            )
        }
    }
}

mod type_annotation_ast {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn ancestor<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'ancestor' for type 'TypeAnnotationAST'")
    }

    pub(super) fn parent<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'parent' for type 'TypeAnnotationAST'")
    }

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'TypeAnnotationAST'")
    }

    pub(super) fn type_<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'type_' for type 'TypeAnnotationAST'")
    }
}

pub(super) fn resolve_type_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "span" => type_::span(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'Type_'")
        }
    }
}

mod type_ {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'span' for type 'Type_'")
    }
}

pub(super) fn resolve_url_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "search_parameter" => url::search_parameter(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'URL'")
        }
    }
}

mod url {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn search_parameter<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'search_parameter' for type 'URL'")
    }
}

pub(super) fn resolve_variable_declaration_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "assignment_span" => variable_declaration::assignment_span(contexts, resolve_info),
        "left" => variable_declaration::left(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'VariableDeclaration'"
            )
        }
    }
}

mod variable_declaration {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn assignment_span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'assignment_span' for type 'VariableDeclaration'")
    }

    pub(super) fn left<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'left' for type 'VariableDeclaration'")
    }
}

pub(super) fn resolve_variable_declaration_ast_edge<'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
    match edge_name {
        "ancestor" => variable_declaration_ast::ancestor(contexts, resolve_info),
        "assignment_span" => variable_declaration_ast::assignment_span(contexts, resolve_info),
        "left" => variable_declaration_ast::left(contexts, resolve_info),
        "parent" => variable_declaration_ast::parent(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'VariableDeclarationAST'"
            )
        }
    }
}

mod variable_declaration_ast {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn ancestor<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'ancestor' for type 'VariableDeclarationAST'")
    }

    pub(super) fn assignment_span<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'assignment_span' for type 'VariableDeclarationAST'")
    }

    pub(super) fn left<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'left' for type 'VariableDeclarationAST'")
    }

    pub(super) fn parent<'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex, VertexIterator<'a, Vertex>> {
        todo!("implement edge 'parent' for type 'VariableDeclarationAST'")
    }
}
