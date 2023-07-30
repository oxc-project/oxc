use trustfall::provider::{
    ContextIterator, ContextOutcomeIterator, EdgeParameters, ResolveEdgeInfo, VertexIterator,
};

use super::vertex::Vertex;
use crate::Adapter;

pub(super) fn resolve_astnode_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
    match edge_name {
        "ancestor" => astnode::ancestor(contexts, resolve_info, adapter),
        "parent" => astnode::parent(contexts, resolve_info, adapter),
        "span" => astnode::span(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'ASTNode'")
        }
    }
}

fn ancestors<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
    trustfall::provider::resolve_neighbors_with(contexts, |v| {
        Box::new(
            Box::new(
                adapter
                    .semantic
                    .nodes()
                    .ancestors(v.ast_node_id().expect("for vertex to have an ast_node")),
            )
            .map(|ancestor| *adapter.semantic.nodes().get_node(ancestor))
            .map(std::convert::Into::into),
        )
    })
}

fn parents<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
    trustfall::provider::resolve_neighbors_with(contexts, |v| {
        Box::new(
            adapter
                .semantic
                .nodes()
                .parent_node(v.ast_node_id().expect("for vertex to have an ast_node"))
                .as_ref()
                .map(|ast_node| Vertex::from(**ast_node))
                .into_iter(),
        )
    })
}

fn get_span<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
    trustfall::provider::resolve_neighbors_with(contexts, |v| {
        Box::new(std::iter::once(Vertex::Span(v.span())))
    })
}

mod astnode {
    use trustfall::provider::{
        ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo, VertexIterator,
    };

    use super::super::vertex::Vertex;
    use crate::Adapter;

    pub(super) fn ancestor<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
        adapter: &'a Adapter<'b>,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::ancestors(contexts, adapter)
    }

    pub(super) fn parent<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
        adapter: &'a Adapter<'b>,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::parents(contexts, adapter)
    }

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::get_span(contexts)
    }
}

pub(super) fn resolve_assignment_type_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::get_span(contexts)
    }
}

pub(super) fn resolve_class_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
    match edge_name {
        "constructor" => class::constructor(contexts, resolve_info),
        "entire_class_span" => class::entire_class_span(contexts, resolve_info),
        "getter" => class::getter(contexts, resolve_info),
        "method" => class::method(contexts, resolve_info),
        "name_span" => class::name_span(contexts, resolve_info),
        "property" => class::property(contexts, resolve_info),
        "setter" => class::setter(contexts, resolve_info),
        "ancestor" => ancestors(contexts, adapter),
        "span" => get_span(contexts),
        "parent" => parents(contexts, adapter),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'Class'")
        }
    }
}

mod class {
    use std::rc::Rc;

    use oxc_ast::ast::{ClassElement, MethodDefinitionKind};
    use trustfall::provider::{
        resolve_neighbors_with, ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo,
        VertexIterator,
    };

    use super::super::vertex::Vertex;
    use crate::vertex::{ClassMethodVertex, ClassPropertyVertex};

    macro_rules! class_fn_edge_implem {
        ($contexts:ident, $x:ident) => {
            resolve_neighbors_with($contexts, |v| {
                Box::new(
                    v.as_class()
                        .unwrap_or_else(|| {
                            panic!("expected to have a class vertex, instead have: {v:#?}")
                        })
                        .class
                        .body
                        .body
                        .iter()
                        .filter_map(|class_el| match class_el {
                            ClassElement::MethodDefinition(method)
                                if matches!(method.kind, MethodDefinitionKind::$x) =>
                            {
                                Some(Vertex::ClassMethod(
                                    ClassMethodVertex { method, is_abstract: false }.into(),
                                ))
                            }
                            ClassElement::TSAbstractMethodDefinition(def)
                                if matches!(
                                    def.method_definition.kind,
                                    MethodDefinitionKind::$x
                                ) =>
                            {
                                Some(Vertex::ClassMethod(Rc::new(ClassMethodVertex {
                                    method: &def.method_definition,
                                    is_abstract: true,
                                })))
                            }
                            _ => None,
                        }),
                )
            })
        };
    }

    pub(super) fn constructor<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        class_fn_edge_implem!(contexts, Constructor)
    }

    pub(super) fn entire_class_span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(std::iter::once(Vertex::Span(
                v.as_class()
                    .unwrap_or_else(|| {
                        panic!("expected to have a class vertex, instead have: {v:#?}")
                    })
                    .class
                    .span,
            )))
        })
    }

    pub(super) fn getter<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        class_fn_edge_implem!(contexts, Get)
    }

    pub(super) fn method<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        class_fn_edge_implem!(contexts, Method)
    }

    pub(super) fn name_span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(
                v.as_class()
                    .unwrap_or_else(|| {
                        panic!("expected to have a class vertex, instead have: {v:#?}")
                    })
                    .class
                    .id
                    .as_ref()
                    .map(|id| id.span)
                    .map(Vertex::Span)
                    .into_iter(),
            )
        })
    }

    pub(super) fn property<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(
                v.as_class()
                    .unwrap_or_else(|| {
                        panic!("expected to have a class vertex, instead have: {v:#?}")
                    })
                    .class
                    .body
                    .body
                    .iter()
                    .filter_map(|class_el| match class_el {
                        ClassElement::PropertyDefinition(property) => {
                            Some(Vertex::ClassProperty(Rc::new(ClassPropertyVertex {
                                property,
                                is_abstract: false,
                            })))
                        }
                        ClassElement::TSAbstractPropertyDefinition(def) => {
                            Some(Vertex::ClassProperty(Rc::new(ClassPropertyVertex {
                                property: &def.property_definition,
                                is_abstract: true,
                            })))
                        }
                        _ => None,
                    }),
            )
        })
    }

    pub(super) fn setter<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        class_fn_edge_implem!(contexts, Set)
    }
}

pub(super) fn resolve_class_method_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::get_span(contexts)
    }
}

pub(super) fn resolve_class_property_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::get_span(contexts)
    }
}

pub(super) fn resolve_default_import_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::get_span(contexts)
    }
}

pub(super) fn resolve_expression_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::get_span(contexts)
    }
}

pub(super) fn resolve_file_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
    match edge_name {
        "ast_node" => file::ast_node(contexts, resolve_info, adapter),
        "class" => file::class(contexts, resolve_info, adapter),
        "import" => file::import(contexts, resolve_info, adapter),
        "interface" => file::interface(contexts, resolve_info, adapter),
        "jsx_element" => file::jsx_element(contexts, resolve_info, adapter),
        "last_path_part" => file::last_path_part(contexts, resolve_info, adapter),
        "path_part" => file::path_part(contexts, resolve_info, adapter),
        "type_annotation" => file::type_annotation(contexts, resolve_info, adapter),
        "variable_declaration" => file::variable_declaration(contexts, resolve_info, adapter),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'File'")
        }
    }
}

mod file {
    use oxc_ast::{ast::ModuleDeclaration, AstKind};
    use trustfall::provider::{
        resolve_neighbors_with, ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo,
        VertexIterator,
    };

    use super::super::vertex::Vertex;
    use crate::Adapter;

    pub(super) fn ast_node<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
        adapter: &'a Adapter<'b>,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |_| {
            Box::new(adapter.semantic.nodes().iter().map(|node| (*node).into()))
        })
    }

    pub(super) fn class<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
        adapter: &'a Adapter<'b>,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |_| {
            Box::new(adapter.semantic.nodes().iter().filter_map(|x| {
                let AstKind::Class(_) = x.kind() else { return None };
                Some((*x).into())
            }))
        })
    }

    pub(super) fn import<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
        adapter: &'a Adapter<'b>,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |_| {
            Box::new(adapter.semantic.nodes().iter().filter_map(|x| {
                let AstKind::ModuleDeclaration(element) = x.kind() else { return None };
                let ModuleDeclaration::ImportDeclaration(_) = element else { return None };
                Some((*x).into())
            }))
        })
    }

    pub(super) fn interface<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
        adapter: &'a Adapter<'b>,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |_| {
            Box::new(adapter.semantic.nodes().iter().filter_map(|x| {
                let AstKind::TSInterfaceDeclaration(_) = x.kind() else { return None };
                Some((*x).into())
            }))
        })
    }

    pub(super) fn jsx_element<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
        adapter: &'a Adapter<'b>,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |_| {
            Box::new(adapter.semantic.nodes().iter().filter_map(|x| {
                let AstKind::JSXElement(_) = x.kind() else { return None };
                Some((*x).into())
            }))
        })
    }

    pub(super) fn last_path_part<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
        adapter: &'a Adapter<'b>,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        let len = adapter.path_components.len();
        resolve_neighbors_with(contexts, move |_| {
            Box::new(std::iter::once(Vertex::PathPart(len - 1)))
        })
    }

    pub(super) fn path_part<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
        adapter: &'a Adapter<'b>,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        let len = adapter.path_components.len();
        resolve_neighbors_with(contexts, move |_| Box::new((0..len).map(Vertex::PathPart)))
    }

    pub(super) fn type_annotation<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
        adapter: &'a Adapter<'b>,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |_| {
            Box::new(adapter.semantic.nodes().iter().filter_map(|x| {
                let AstKind::TSTypeAnnotation(_) = x.kind() else { return None };
                Some((*x).into())
            }))
        })
    }

    pub(super) fn variable_declaration<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
        adapter: &'a Adapter<'b>,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |_| {
            Box::new(adapter.semantic.nodes().iter().filter_map(|x| {
                let AstKind::VariableDeclarator(_) = x.kind() else { return None };
                Some((*x).into())
            }))
        })
    }
}

pub(super) fn resolve_has_span_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::get_span(contexts)
    }
}

pub(super) fn resolve_import_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
    match edge_name {
        "default_import" => import::default_import(contexts, resolve_info),
        "entire_span" => import::entire_span(contexts, resolve_info),
        "specific_import" => import::specific_import(contexts, resolve_info),
        "ancestor" => ancestors(contexts, adapter),
        "span" => get_span(contexts),
        "parent" => parents(contexts, adapter),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'Import'")
        }
    }
}

mod import {
    use oxc_ast::ast::ImportDeclarationSpecifier;
    use trustfall::provider::{
        resolve_neighbors_with, ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo,
        VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn default_import<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(
                v.as_import()
                    .unwrap_or_else(|| {
                        panic!("expected to have an import vertex, instead have: {v:#?}")
                    })
                    .import
                    .specifiers
                    .iter()
                    .filter_map(|the_specifier| {
                        if let ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) =
                            the_specifier
                        {
                            Some(Vertex::DefaultImport(specifier))
                        } else {
                            None
                        }
                    }),
            )
        })
    }

    pub(super) fn entire_span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(std::iter::once(Vertex::Span(
                v.as_import()
                    .unwrap_or_else(|| {
                        panic!("expected to have a import vertex, instead have: {v:#?}")
                    })
                    .import
                    .span,
            )))
        })
    }

    pub(super) fn specific_import<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(
                v.as_import()
                    .unwrap_or_else(|| {
                        panic!("expected to have an import vertex, instead have: {v:#?}")
                    })
                    .import
                    .specifiers
                    .iter()
                    .filter_map(|the_specifier| {
                        if let ImportDeclarationSpecifier::ImportSpecifier(specifier) =
                            the_specifier
                        {
                            Some(Vertex::SpecificImport(specifier))
                        } else {
                            None
                        }
                    }),
            )
        })
    }
}

pub(super) fn resolve_interface_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
    match edge_name {
        "entire_span" => interface::entire_span(contexts, resolve_info),
        "extend" => interface::extend(contexts, resolve_info),
        "name_span" => interface::name_span(contexts, resolve_info),
        "ancestor" => ancestors(contexts, adapter),
        "span" => get_span(contexts),
        "parent" => parents(contexts, adapter),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'Interface'")
        }
    }
}

mod interface {
    use std::rc::Rc;

    use trustfall::provider::{
        resolve_neighbors_with, ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo,
        VertexIterator,
    };

    use super::super::vertex::Vertex;
    use crate::vertex::InterfaceExtendVertex;

    pub(super) fn entire_span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(std::iter::once(Vertex::Span(
                v.as_interface()
                    .unwrap_or_else(|| {
                        panic!("expected to have an interface vertex, instead have: {v:#?}")
                    })
                    .interface
                    .span,
            )))
        })
    }

    pub(super) fn extend<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(
                v.as_interface()
                    .and_then(|data| data.interface.extends.as_ref())
                    .map(|extends| {
                        extends
                            .iter()
                            .map(|extend| InterfaceExtendVertex::from(&extend.expression))
                            .map(Rc::new)
                            .map(Vertex::InterfaceExtend)
                    })
                    .into_iter()
                    .flatten(),
            )
        })
    }

    pub(super) fn name_span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(std::iter::once(Vertex::Span(
                v.as_interface()
                    .unwrap_or_else(|| {
                        panic!("expected to have an interface vertex, instead have: {v:#?}")
                    })
                    .interface
                    .id
                    .span,
            )))
        })
    }
}

pub(super) fn resolve_interface_extend_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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

    use super::{super::vertex::Vertex, get_span};

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        get_span(contexts)
    }
}

pub(super) fn resolve_jsxattribute_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
    match edge_name {
        "span" => jsxattribute::span(contexts, resolve_info),
        "value_as_expression" => jsxattribute::value_as_expression(contexts, resolve_info),
        "value_as_url" => jsxattribute::value_as_url(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'JSXAttribute'"
            )
        }
    }
}

mod jsxattribute {
    use oxc_ast::ast::JSXAttributeValue;
    use trustfall::provider::{
        resolve_neighbors_with, ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo,
        VertexIterator,
    };

    use super::{super::vertex::Vertex, get_span};

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        get_span(contexts)
    }

    pub(super) fn value_as_expression<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            let attr = v.as_jsx_attribute().unwrap_or_else(|| {
                panic!("expected to have a jsxattribute vertex, instead have: {v:#?}")
            });
            Box::new(
                attr.value
                    .as_ref()
                    .and_then(|attr_value| match attr_value {
                        JSXAttributeValue::ExpressionContainer(expr) => match &expr.expression {
                            oxc_ast::ast::JSXExpression::Expression(expr) => {
                                Some(Vertex::from(expr))
                            }
                            oxc_ast::ast::JSXExpression::EmptyExpression(_) => None,
                        },
                        JSXAttributeValue::Fragment(_)
                        | JSXAttributeValue::StringLiteral(_)
                        | JSXAttributeValue::Element(_) => None,
                    })
                    .into_iter(),
            )
        })
    }

    pub(super) fn value_as_url<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(v.as_jsx_attribute().as_ref().and_then(|v| Vertex::make_url(v)).into_iter())
        })
    }
}

pub(super) fn resolve_jsxelement_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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
        "ancestor" => ancestors(contexts, adapter),
        "parent" => parents(contexts, adapter),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'JSXElement'")
        }
    }
}

mod jsxelement {
    use oxc_ast::ast::JSXChild;
    use trustfall::provider::{
        resolve_neighbors_with, ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo,
        VertexIterator,
    };

    use super::{super::vertex::Vertex, get_span};
    use crate::vertex::JSXElementVertex;

    macro_rules! child_edge_iter {
        ($contexts: ident, $vertex_type: ident, $jsx_child_type: ident) => {
            resolve_neighbors_with($contexts, |v| {
                Box::new(
                    v.as_jsx_element()
                        .unwrap_or_else(|| {
                            panic!("expected to have a JSXElement vertex, instead have: {v:#?}")
                        })
                        .element
                        .children
                        .iter()
                        .filter_map(|child| {
                            let JSXChild::$jsx_child_type(element) = &child else { return None };
                            Some(Vertex::$vertex_type(element))
                        }),
                )
            })
        };
    }

    pub(super) fn child_element<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(
                v.as_jsx_element()
                    .unwrap_or_else(|| {
                        panic!("expected to have a JSXElement vertex, instead have: {v:#?}")
                    })
                    .element
                    .children
                    .iter()
                    .filter_map(|child| {
                        let JSXChild::Element(element) = &child else { return None };
                        Some(Vertex::JSXElement(
                            JSXElementVertex { element, ast_node: None }.into(),
                        ))
                    }),
            )
        })
    }

    pub(super) fn child_expression_container<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        child_edge_iter!(contexts, JSXExpressionContainer, ExpressionContainer)
    }

    pub(super) fn child_fragment<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        child_edge_iter!(contexts, JSXFragment, Fragment)
    }

    pub(super) fn child_spread<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        child_edge_iter!(contexts, JSXSpreadChild, Spread)
    }

    pub(super) fn child_text<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        child_edge_iter!(contexts, JSXText, Text)
    }

    pub(super) fn opening_element<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(std::iter::once(Vertex::JSXOpeningElement(
                &v.as_jsx_element()
                    .unwrap_or_else(|| {
                        panic!("expected to have a JSXElement vertex, instead have: {v:#?}")
                    })
                    .element
                    .opening_element,
            )))
        })
    }

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        get_span(contexts)
    }
}

pub(super) fn resolve_jsxexpression_container_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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

    use super::{super::vertex::Vertex, get_span};

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        get_span(contexts)
    }
}

pub(super) fn resolve_jsxfragment_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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

    use super::{super::vertex::Vertex, get_span};

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        get_span(contexts)
    }
}

pub(super) fn resolve_jsxopening_element_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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
    use oxc_ast::ast::JSXAttributeItem;
    use trustfall::provider::{
        resolve_neighbors_with, ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo,
        VertexIterator,
    };

    use super::{super::vertex::Vertex, get_span};

    pub(super) fn attribute<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(
                v.as_jsx_opening_element()
                    .unwrap_or_else(|| {
                        panic!("expected to have a jsxopeningelement vertex, instead have: {v:#?}")
                    })
                    .attributes
                    .iter()
                    .filter_map(|attr| match attr {
                        JSXAttributeItem::Attribute(attr) => Some(Vertex::JSXAttribute(attr)),
                        JSXAttributeItem::SpreadAttribute(_) => None,
                    }),
            )
        })
    }

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        get_span(contexts)
    }

    pub(super) fn spread_attribute<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(
                v.as_jsx_opening_element()
                    .unwrap_or_else(|| {
                        panic!("expected to have a jsxopeningelement vertex, instead have: {v:#?}")
                    })
                    .attributes
                    .iter()
                    .filter_map(|attr| match attr {
                        JSXAttributeItem::Attribute(_) => None,
                        JSXAttributeItem::SpreadAttribute(spread_attr) => {
                            Some(Vertex::JSXSpreadAttribute(spread_attr))
                        }
                    }),
            )
        })
    }
}

pub(super) fn resolve_jsxspread_attribute_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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

    use super::{super::vertex::Vertex, get_span};

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        get_span(contexts)
    }
}

pub(super) fn resolve_jsxspread_child_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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

    use super::{super::vertex::Vertex, get_span};

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        get_span(contexts)
    }
}

pub(super) fn resolve_jsxtext_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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

    use super::{super::vertex::Vertex, get_span};

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        get_span(contexts)
    }
}

pub(super) fn resolve_member_extend_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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

    use super::{super::vertex::Vertex, get_span};

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        get_span(contexts)
    }
}

pub(super) fn resolve_object_literal_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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
    use std::rc::Rc;

    use oxc_ast::ast::ObjectPropertyKind;
    use trustfall::provider::{
        resolve_neighbors_with, ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo,
        VertexIterator,
    };

    use super::{super::vertex::Vertex, get_span};
    use crate::util::expr_to_maybe_const_string;

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        get_span(contexts)
    }

    pub(super) fn value<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        key: &str,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        let k = Rc::new(key.to_owned());
        resolve_neighbors_with(contexts, move |v| {
            let key = Rc::clone(&k);
            let obj = v.as_object_literal().unwrap_or_else(|| {
                panic!("expected to have an objectliteral vertex, instead have: {v:#?}")
            });

            Box::new(obj.properties.iter().filter_map(move |property| {
                let ObjectPropertyKind::ObjectProperty(prop) = property else { return None };

                let has_right_key_name = match &prop.key {
                    oxc_ast::ast::PropertyKey::Identifier(ident) => ident.name == key.as_str(),
                    oxc_ast::ast::PropertyKey::PrivateIdentifier(_) => {
                        unreachable!("private identifiers don't exist in objects")
                    }
                    oxc_ast::ast::PropertyKey::Expression(expr) => expr_to_maybe_const_string(expr)
                        .map_or(false, |key_from_iter| key_from_iter == key.as_str()),
                };

                if has_right_key_name {
                    Some(Vertex::from(&prop.value))
                } else {
                    None
                }
            }))
        })
    }
}

pub(super) fn resolve_path_part_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
    match edge_name {
        "next" => path_part::next(contexts, resolve_info, adapter),
        "prev" => path_part::prev(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'PathPart'")
        }
    }
}

mod path_part {
    use trustfall::provider::{
        resolve_neighbors_with, ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo,
        VertexIterator,
    };

    use super::super::vertex::Vertex;
    use crate::Adapter;

    pub(super) fn next<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
        adapter: &'a Adapter<'b>,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            let i = *v.as_path_part().unwrap_or_else(|| {
                panic!("expected to have a pathpart vertex, instead have: {v:#?}")
            });
            if i + 1 < adapter.path_components.len() {
                Box::new(std::iter::once(Vertex::PathPart(i + 1)))
            } else {
                Box::new(std::iter::empty())
            }
        })
    }

    pub(super) fn prev<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            let i = *v.as_path_part().unwrap_or_else(|| {
                panic!("expected to have a pathpart vertex, instead have: {v:#?}")
            });
            if i > 0 {
                Box::new(std::iter::once(Vertex::PathPart(i - 1)))
            } else {
                Box::new(std::iter::empty())
            }
        })
    }
}

pub(super) fn resolve_return_statement_ast_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
    match edge_name {
        "ancestor" => return_statement_ast::ancestor(contexts, resolve_info, adapter),
        "expression" => return_statement_ast::expression(contexts, resolve_info),
        "parent" => return_statement_ast::parent(contexts, resolve_info, adapter),
        "span" => return_statement_ast::span(contexts, resolve_info),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'ReturnStatementAST'"
            )
        }
    }
}

mod return_statement_ast {
    use std::convert::Into;

    use trustfall::provider::{
        resolve_neighbors_with, ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo,
        VertexIterator,
    };

    use super::super::vertex::Vertex;
    use crate::Adapter;

    pub(super) fn ancestor<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
        adapter: &'a Adapter<'b>,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::ancestors(contexts, adapter)
    }

    pub(super) fn expression<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            let neighbors = v
                .as_return_statement_ast()
                .unwrap()
                .return_statement
                .argument
                .as_ref()
                .map(Into::into)
                .into_iter();
            Box::new(neighbors)
        })
    }

    pub(super) fn parent<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
        adapter: &'a Adapter<'b>,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::parents(contexts, adapter)
    }

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::get_span(contexts)
    }
}

pub(super) fn resolve_simple_extend_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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

    use super::{super::vertex::Vertex, get_span};

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        get_span(contexts)
    }
}

pub(super) fn resolve_specific_import_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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

    use super::{super::vertex::Vertex, get_span};

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        get_span(contexts)
    }
}

pub(super) fn resolve_type_annotation_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
    match edge_name {
        "span" => type_annotation::span(contexts, resolve_info),
        "type" => type_annotation::type_(contexts, resolve_info),
        "ancestor" => ancestors(contexts, adapter),
        "parent" => parents(contexts, adapter),
        _ => {
            unreachable!(
                "attempted to resolve unexpected edge '{edge_name}' on type 'TypeAnnotation'"
            )
        }
    }
}

mod type_annotation {
    use trustfall::provider::{
        resolve_neighbors_with, ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo,
        VertexIterator,
    };

    use super::{super::vertex::Vertex, get_span};

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        get_span(contexts)
    }

    pub(super) fn type_<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(std::iter::once(Vertex::Type(
                &v.as_type_annotation()
                    .unwrap_or_else(|| {
                        panic!("expected to have a typeannotation vertex, instead have: {v:#?}")
                    })
                    .type_annotation
                    .type_annotation,
            )))
        })
    }
}

pub(super) fn resolve_type_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
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

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::get_span(contexts)
    }
}

pub(super) fn resolve_url_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
    match edge_name {
        "search_parameter" => url::search_parameter(contexts, resolve_info),
        _ => {
            unreachable!("attempted to resolve unexpected edge '{edge_name}' on type 'URL'")
        }
    }
}

mod url {
    use trustfall::provider::{
        resolve_neighbors_with, ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo,
        VertexIterator,
    };

    use super::super::vertex::Vertex;
    use crate::vertex::SearchParameterVertex;

    pub(super) fn search_parameter<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(
                v.as_url()
                    .unwrap_or_else(|| {
                        panic!("expected to have a url vertex, instead have: {v:#?}")
                    })
                    .query_pairs()
                    .map(|(key, value)| {
                        Vertex::SearchParameter(
                            SearchParameterVertex {
                                key: key.to_string(),
                                value: value.to_string(),
                            }
                            .into(),
                        )
                    })
                    .collect::<Vec<_>>()
                    .into_iter(),
            )
        })
    }
}

pub(super) fn resolve_variable_declaration_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
    match edge_name {
        "span" => variable_declaration::span(contexts, resolve_info),
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
        resolve_neighbors_with, ContextIterator, ContextOutcomeIterator, ResolveEdgeInfo,
        VertexIterator,
    };

    use super::super::vertex::Vertex;

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            Box::new(std::iter::once(Vertex::Span(
                v.as_variable_declaration()
                    .unwrap_or_else(|| {
                        panic!("expected to have a typeannotation vertex, instead have: {v:#?}")
                    })
                    .variable_declaration
                    .span,
            )))
        })
    }

    pub(super) fn left<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        resolve_neighbors_with(contexts, |v| {
            return Box::new(std::iter::once(Vertex::AssignmentType(
                &v.as_variable_declaration()
                    .unwrap_or_else(|| {
                        panic!("expected to have a typeannotation vertex, instead have: {v:#?}")
                    })
                    .variable_declaration
                    .id
                    .kind,
            )));
        })
    }
}

pub(super) fn resolve_variable_declaration_ast_edge<'a, 'b: 'a>(
    contexts: ContextIterator<'a, Vertex<'b>>,
    edge_name: &str,
    _parameters: &EdgeParameters,
    resolve_info: &ResolveEdgeInfo,
    adapter: &'a Adapter<'b>,
) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
    match edge_name {
        "ancestor" => variable_declaration_ast::ancestor(contexts, resolve_info, adapter),
        "span" => variable_declaration_ast::span(contexts, resolve_info),
        "left" => variable_declaration_ast::left(contexts, resolve_info),
        "parent" => variable_declaration_ast::parent(contexts, resolve_info, adapter),
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
    use crate::Adapter;

    pub(super) fn ancestor<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
        adapter: &'a Adapter<'b>,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::ancestors(contexts, adapter)
    }

    pub(super) fn span<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::variable_declaration::span(contexts, resolve_info)
    }

    pub(super) fn left<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::variable_declaration::left(contexts, resolve_info)
    }

    pub(super) fn parent<'a, 'b: 'a>(
        contexts: ContextIterator<'a, Vertex<'b>>,
        _resolve_info: &ResolveEdgeInfo,
        adapter: &'a Adapter<'b>,
    ) -> ContextOutcomeIterator<'a, Vertex<'b>, VertexIterator<'a, Vertex<'b>>> {
        super::parents(contexts, adapter)
    }
}
