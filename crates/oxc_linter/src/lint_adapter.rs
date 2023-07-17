#![allow(clippy::too_many_lines)]
#![allow(clippy::unimplemented)]
use std::{collections::BTreeMap, convert::Into, path::PathBuf, rc::Rc, sync::Arc};

use oxc_ast::{
    ast::{TSAccessibility, *},
    AstKind,
};
use oxc_semantic::{AstNode, AstNodeId, Semantic};
use oxc_span::{GetSpan, Span};
use regex::Regex;
use serde::Deserialize;
use trustfall::{
    provider::{
        resolve_coercion_with, resolve_neighbors_with, resolve_property_with, Adapter,
        ContextIterator, ContextOutcomeIterator, EdgeParameters, ResolveEdgeInfo, ResolveInfo,
        TrustfallEnumVertex, VertexIterator,
    },
    FieldValue, TransparentValue,
};
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct InputQuery {
    pub name: String,
    pub query: String,
    pub args: BTreeMap<Arc<str>, TransparentValue>,
    pub reason: String,
    #[serde(default)]
    pub tests: QueryTests,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct QueryTests {
    pub pass: Vec<SingleTest>,
    pub fail: Vec<SingleTest>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SingleTest {
    pub file_path: String,
    pub code: String,
}

// https://typescript-eslint.io/rules/prefer-function-type
#[derive(Debug, Clone, TrustfallEnumVertex)]
pub enum Vertex<'a> {
    TypeAnnotation(&'a TSTypeAnnotation<'a>, Option<AstNodeId>),
    Type(&'a TSType<'a>),
    Span(Span),
    Class(&'a Class<'a>, Option<AstNodeId>), // ASTNode
    ClassMethod(Rc<ClassMethod<'a>>),
    ClassProperty(Rc<ClassProperty<'a>>),
    Interface(&'a TSInterfaceDeclaration<'a>, Option<AstNodeId>),
    InterfaceExtends(Rc<InterfaceExtends<'a>>),
    JsxOpeningElement(&'a JSXOpeningElement<'a>),
    JsxAttribute(&'a JSXAttribute<'a>),
    JsxSpreadAttribute(&'a JSXSpreadAttribute<'a>),
    JsxText(&'a JSXText),
    JsxFragment(&'a JSXFragment<'a>),
    JsxExpressionContainer(&'a JSXExpressionContainer<'a>),
    JsxSpreadChild(&'a JSXSpreadChild<'a>),
    Import(&'a ImportDeclaration<'a>, Option<AstNodeId>),
    SpecificImport(&'a ImportSpecifier),
    DefaultImport(&'a ImportDefaultSpecifier),
    VariableDeclaration(&'a VariableDeclarator<'a>, Option<AstNodeId>),
    AssignmentType(&'a BindingPatternKind<'a>),
    Url(Rc<Url>),
    File,
    PathCompareResult(bool),
    SearchParameter(SearchParameter),
    // ASTNode
    AstNode(AstNode<'a>),
    ReturnStatementAst(&'a ReturnStatement<'a>, AstNodeId),
    // Expression
    Expression(&'a Expression<'a>),
    JsxElement(&'a JSXElement<'a>, Option<AstNodeId>),
    ObjectLiteral(&'a ObjectExpression<'a>),
}

#[derive(Debug, Clone)]
pub enum InterfaceExtends<'a> {
    Identifier(&'a IdentifierReference),
    MemberExpression(&'a MemberExpression<'a>),
}

#[derive(Debug, Clone)]
pub struct ClassMethod<'a> {
    method: &'a MethodDefinition<'a>,
    is_abstract: bool,
}

#[derive(Debug, Clone)]
pub struct SearchParameter {
    key: String,
    value: String,
}

#[derive(Debug, Clone)]
pub struct ClassProperty<'a> {
    property: &'a PropertyDefinition<'a>,
    is_abstract: bool,
}

pub struct LintAdapter<'a> {
    pub semantic: Rc<Semantic<'a>>,
    pub path: PathBuf,
}

impl<'b, 'a: 'b> Adapter<'b> for &'b LintAdapter<'a> {
    type Vertex = Vertex<'a>;

    fn resolve_starting_vertices(
        &self,
        edge_name: &Arc<str>,
        _parameters: &EdgeParameters,
        _resolve_info: &ResolveInfo,
    ) -> VertexIterator<'b, Self::Vertex> {
        match edge_name.as_ref() {
            "File" => Box::new(std::iter::once(Vertex::File)),
            _ => unimplemented!("unexpected starting edge: {edge_name}"),
        }
    }

    fn resolve_property(
        &self,
        contexts: ContextIterator<'b, Self::Vertex>,
        type_name: &Arc<str>,
        property_name: &Arc<str>,
        _resolve_info: &ResolveInfo,
    ) -> ContextOutcomeIterator<'b, Self::Vertex, FieldValue> {
        match (type_name.as_ref(), property_name.as_ref()) {
            ("Span", "start") => {
                resolve_property_with(contexts, |v| v.as_span().unwrap().start.into())
            }
            ("Span", "end") => resolve_property_with(contexts, |v| v.as_span().unwrap().end.into()),
            ("Type", "str") => resolve_property_with(contexts, |v| {
                let span = v.as_type().unwrap().span();
                self.semantic.source_text()[span.start as usize..span.end as usize].into()
            }),
            ("JSXOpeningElement", "name") => resolve_property_with(contexts, |v| {
                let jsx = v.as_jsx_opening_element().unwrap();
                jsx.name.to_string_jsx().into()
            }),
            ("JSXOpeningElement", "attribute_count") => resolve_property_with(contexts, |v| {
                let jsx = v.as_jsx_opening_element().unwrap();
                (jsx.attributes.len() as u64).into()
            }),
            ("JSXAttribute", "name") => resolve_property_with(contexts, |v| {
                let attr = v.as_jsx_attribute().unwrap();
                attr.name.to_string_jsx().into()
            }),
            ("JSXAttribute", "value_as_constant_string") => resolve_property_with(contexts, |v| {
                let attr = v.as_jsx_attribute().unwrap();
                jsx_attribute_to_constant_string(attr).map_or_else(|| FieldValue::Null, Into::into)
            }),
            ("ImportAST" | "Import", "from_path") => resolve_property_with(contexts, |v| {
                let Vertex::Import(import, ..) = &v else {unreachable!()};
                import.source.value.to_string().into()
            }),
            ("SpecificImport", "original_name") => resolve_property_with(contexts, |v| {
                v.as_specific_import().unwrap().imported.name().to_string().into()
            }),
            ("SpecificImport", "local_name") => resolve_property_with(contexts, |v| {
                v.as_specific_import().unwrap().local.name.to_string().into()
            }),
            ("DefaultImport", "local_name") => resolve_property_with(contexts, |v| {
                v.as_default_import().unwrap().local.name.to_string().into()
            }),
            ("JSXText", "text") => resolve_property_with(contexts, |v| {
                v.as_jsx_text().unwrap().value.to_string().into()
            }),
            ("JSXElementAST" | "JSXElement", "child_count") => {
                resolve_property_with(contexts, |v| {
                    let Vertex::JsxElement(el, ..) = &v else {unreachable!()};
                    (el.children.len() as u64).into()
                })
            }
            ("ClassAST" | "Class", "is_abstract") => resolve_property_with(contexts, |v| {
                v.as_class()
                    .unwrap()
                    .0
                    .modifiers
                    .contains(oxc_ast::ast::ModifierKind::Abstract)
                    .into()
            }),
            ("ClassAST" | "Class", "extended_class_name") => resolve_property_with(contexts, |v| {
                let Some(Expression::Identifier(ref ident)) = v.as_class().unwrap().0.super_class else {return FieldValue::Null};
                ident.name.to_string().into()
            }),
            ("ClassMethod", "is_abstract") => {
                resolve_property_with(contexts, |v| v.as_class_method().unwrap().is_abstract.into())
            }
            ("ClassProperty", "is_abstract") => resolve_property_with(contexts, |v| {
                v.as_class_property().unwrap().is_abstract.into()
            }),
            ("ClassMethod", "accessibility") => resolve_property_with(contexts, |v| {
                v.as_class_method().unwrap().method.accessibility.map_or(
                    FieldValue::Null,
                    |access| {
                        match access {
                            TSAccessibility::Private => "private",
                            TSAccessibility::Protected => "protected",
                            TSAccessibility::Public => "public",
                        }
                        .into()
                    },
                )
            }),
            ("ClassProperty", "accessibility") => resolve_property_with(contexts, |v| {
                v.as_class_property().unwrap().property.accessibility.map_or(
                    FieldValue::Null,
                    |access| {
                        match access {
                            TSAccessibility::Private => "private",
                            TSAccessibility::Protected => "protected",
                            TSAccessibility::Public => "public",
                        }
                        .into()
                    },
                )
            }),
            ("InterfaceExtend" | "SimpleExtend" | "MemberExtend", "str") => {
                resolve_property_with(contexts, |v| {
                    match v.as_interface_extends().unwrap().as_ref() {
                        InterfaceExtends::Identifier(ident) => ident.name.to_string(),
                        InterfaceExtends::MemberExpression(first_membexpr) => {
                            let MemberExpression::StaticMemberExpression(static_membexpr) = first_membexpr else {unreachable!("TS:2499")};
                            let mut parts = vec![static_membexpr.property.name.to_string()];
                            let mut membexpr = first_membexpr.object();
                            while let Expression::MemberExpression(expr) = membexpr {
                                let MemberExpression::StaticMemberExpression(static_membexpr) = &expr.0 else {unreachable!("TS:2499")};
                                parts.push(static_membexpr.property.name.to_string());
                                membexpr = expr.object();
                            }

                            let Expression::Identifier(ident) = membexpr else {unreachable!("TS:2499")};
                            parts.push(ident.name.to_string());

                            parts.reverse();

                            parts.join(".")
                        }
                    }
                    .into()
                })
            }
            ("AssignmentType", "assignment_to_variable_name") => {
                resolve_property_with(contexts, |v| {
                    let Vertex::AssignmentType(BindingPatternKind::BindingIdentifier(ident)) = v else {return FieldValue::Null};
                    ident.name.to_string().into()
                })
            }
            // expression case
            (_, "as_constant_string") => resolve_property_with(contexts, |v| match v {
                Vertex::Expression(expr) => {
                    expr_to_maybe_const_string(expr).map_or_else(|| FieldValue::Null, Into::into)
                }
                _ => FieldValue::Null,
            }),
            ("PathCompareResult", "result") => {
                resolve_property_with(contexts, |v| (*v.as_path_compare_result().unwrap()).into())
            }
            ("SearchParameter", "key") => resolve_property_with(contexts, |v| {
                v.as_search_parameter().unwrap().key.clone().into()
            }),
            ("SearchParameter", "value") => resolve_property_with(contexts, |v| {
                v.as_search_parameter().unwrap().value.clone().into()
            }),
            _ => unimplemented!("unexpected property of: {type_name}.{property_name}"),
        }
    }

    fn resolve_neighbors(
        &self,
        contexts: ContextIterator<'b, Self::Vertex>,
        type_name: &Arc<str>,
        edge_name: &Arc<str>,
        parameters: &EdgeParameters,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'b, Self::Vertex, VertexIterator<'b, Self::Vertex>> {
        match (type_name.as_ref(), edge_name.as_ref()) {
            ("File", "ast_node") => resolve_neighbors_with(contexts, |_| {
                Box::new(self.semantic.nodes().iter().map(|node| materialize_ast_node(*node)))
            }),
            ("File", "type_annotation") => resolve_neighbors_with(contexts, |_| {
                Box::new(self.semantic.nodes().iter().filter_map(|x| {
                    let AstKind::TSTypeAnnotation(annot) = x.kind() else {return None};
                    Some(materialize_ast_node(*x))
                }))
            }),
            ("File", "class") => resolve_neighbors_with(contexts, |_| {
                Box::new(self.semantic.nodes().iter().filter_map(|x| {
                    let AstKind::Class(class) = x.kind() else {return None};
                    Some(materialize_ast_node(*x))
                }))
            }),
            ("File", "interface") => resolve_neighbors_with(contexts, |_| {
                Box::new(self.semantic.nodes().iter().filter_map(|x| {
                    let AstKind::TSInterfaceDeclaration(class) = x.kind() else {return None};
                    Some(materialize_ast_node(*x))
                }))
            }),
            ("File", "jsx_element") => resolve_neighbors_with(contexts, |_| {
                Box::new(self.semantic.nodes().iter().filter_map(|x| {
                    let AstKind::JSXElement(element) = x.kind() else {return None};
                    Some(materialize_ast_node(*x))
                }))
            }),
            ("File", "variable_declaration") => resolve_neighbors_with(contexts, |_| {
                Box::new(self.semantic.nodes().iter().filter_map(|x| {
                    let AstKind::VariableDeclarator(element) = x.kind() else {return None};
                    Some(materialize_ast_node(*x))
                }))
            }),
            ("File", "import") => resolve_neighbors_with(contexts, |_| {
                Box::new(self.semantic.nodes().iter().filter_map(|x| {
                    let AstKind::ModuleDeclaration(element) = x.kind() else {return None};
                    let ModuleDeclaration::ImportDeclaration(import) = element else {return None};
                    Some(materialize_ast_node(*x))
                }))
            }),
            ("File", "path_part") => {
                let params = parameters.clone();
                let path = self.path.clone();
                resolve_neighbors_with(contexts, move |v| {
                    let matcher =
                        Regex::new(params["regex"].as_str().expect(
                            "for File.path_includes to have a regex parameter of type string",
                        ))
                        .expect("for File.path_includes to have a valid regex passed to it");
                    Box::new(std::iter::once(Vertex::PathCompareResult(
                        path.components()
                            .skip_while(|comp| !matches!(comp, std::path::Component::Normal(_)))
                            .any(|x| match x {
                                std::path::Component::ParentDir
                                | std::path::Component::CurDir
                                | std::path::Component::RootDir
                                | std::path::Component::Prefix(_) => unreachable!(),
                                std::path::Component::Normal(file_or_directory) => {
                                    file_or_directory
                                        .to_str()
                                        .map_or_else(|| false, |str| matcher.is_match(str))
                                }
                            }),
                    )))
                })
            }
            ("File", "path_starts_with") => {
                let params = parameters.clone();
                let path = self.path.clone();
                resolve_neighbors_with(contexts, move |v| {
                    Box::new(std::iter::once(Vertex::PathCompareResult(
                        path.components().skip_while(|comp| !matches!(comp, std::path::Component::Normal(_)))
                            .zip(
                                params["regex"]
                                    .as_slice()
                                    .expect("to have slice of values")
                                    .iter()
                                    .map(|x| x.as_str().unwrap()),
                            )
                            .all(|x| match x.0 {
                                std::path::Component::ParentDir | std::path::Component::CurDir | std::path::Component::RootDir | std::path::Component::Prefix(_) => unreachable!(),
                                std::path::Component::Normal(file_or_directory) => {
                                    file_or_directory.to_str().map_or_else(
                                        || false,
                                        |str| {
                                            Regex::new(x.1).expect(&format!(
                                                "The following is invalid regex put into File.path_starts_with: {}",
                                                x.1
                                            )).is_match(str)
                                        },
                                    )
                                },
                            }),
                    )))
                })
            }
            ("File", "path_ends_with") => {
                let params = parameters.clone();
                let path = self.path.clone();
                resolve_neighbors_with(contexts, move |v| {
                    Box::new(std::iter::once(Vertex::PathCompareResult(
                        path.components().filter(|comp| matches!(comp, std::path::Component::Normal(_)))
                            .rev()
                            .zip(
                                params["regex"]
                                    .as_slice()
                                    .expect("to have slice of values")
                                    .iter()
                                    .rev()
                                    .map(|x| x.as_str().unwrap()),
                            )
                            .all(|x| match x.0 {
                                std::path::Component::ParentDir | std::path::Component::CurDir | std::path::Component::RootDir | std::path::Component::Prefix(_) => unreachable!(),
                                std::path::Component::Normal(file_or_directory) => {
                                    file_or_directory.to_str().map_or_else(
                                        || false,
                                        |str| {
                                            Regex::new(x.1).expect(&format!(
                                                "The following is invalid regex put into File.path_ends_with: {}",
                                                x.1
                                            )).is_match(str)
                                        },
                                    )
                                }
                            }),
                    )))
                })
            }
            ("TypeAnnotationAST" | "TypeAnnotation", "type") => {
                resolve_neighbors_with(contexts, |v| {
                    let Vertex::TypeAnnotation(ta, ..) = &v else {unreachable!()};
                    Box::new(std::iter::once(Vertex::Type(&ta.type_annotation)))
                })
            }
            ("ClassAST" | "Class", "method") => resolve_neighbors_with(contexts, |v| {
                Box::new(v.as_class().unwrap().0.body.body.iter().filter_map(|class_el| {
                    if let ClassElement::MethodDefinition(method) = class_el && matches!(method.kind, MethodDefinitionKind::Method) {
                        Some(Vertex::ClassMethod(Rc::new(ClassMethod{ method, is_abstract: false })))
                    } else if let ClassElement::TSAbstractMethodDefinition(def) = class_el && matches!(def.method_definition.kind, MethodDefinitionKind::Method) {
                        Some(Vertex::ClassMethod(Rc::new(ClassMethod{ method: &def.method_definition, is_abstract: true })))
                    } else {
                        None
                    }
                }))
            }),
            ("ClassAST" | "Class", "getter") => resolve_neighbors_with(contexts, |v| {
                Box::new(v.as_class().unwrap().0.body.body.iter().filter_map(|class_el| {
                    if let ClassElement::MethodDefinition(method) = class_el && matches!(method.kind, MethodDefinitionKind::Get) {
                        Some(Vertex::ClassMethod(Rc::new(ClassMethod{ method, is_abstract: false })))
                    } else if let ClassElement::TSAbstractMethodDefinition(def) = class_el && matches!(def.method_definition.kind, MethodDefinitionKind::Get) {
                        Some(Vertex::ClassMethod(Rc::new(ClassMethod{ method: &def.method_definition, is_abstract: true })))
                    } else {
                        None
                    }
                }))
            }),
            ("ClassAST" | "Class", "setter") => resolve_neighbors_with(contexts, |v| {
                Box::new(v.as_class().unwrap().0.body.body.iter().filter_map(|class_el| {
                    if let ClassElement::MethodDefinition(method) = class_el && matches!(method.kind, MethodDefinitionKind::Set) {
                        Some(Vertex::ClassMethod(Rc::new(ClassMethod{ method, is_abstract: false })))
                    } else if let ClassElement::TSAbstractMethodDefinition(def) = class_el && matches!(def.method_definition.kind, MethodDefinitionKind::Set) {
                        Some(Vertex::ClassMethod(Rc::new(ClassMethod{ method: &def.method_definition, is_abstract: true })))
                    } else {
                        None
                    }
                }))
            }),
            ("ClassAST" | "Class", "constructor") => resolve_neighbors_with(contexts, |v| {
                Box::new(v.as_class().unwrap().0.body.body.iter().filter_map(|class_el| {
                    if let ClassElement::MethodDefinition(method) = class_el && matches!(method.kind, MethodDefinitionKind::Constructor) {
                        Some(Vertex::ClassMethod(Rc::new(ClassMethod{ method, is_abstract: false })))
                    } else if let ClassElement::TSAbstractMethodDefinition(def) = class_el && matches!(def.method_definition.kind, MethodDefinitionKind::Constructor) {
                        Some(Vertex::ClassMethod(Rc::new(ClassMethod{ method: &def.method_definition, is_abstract: true })))
                    } else {
                        None
                    }
                }))
            }),
            ("ClassAST" | "Class", "property") => resolve_neighbors_with(contexts, |v| {
                Box::new(v.as_class().unwrap().0.body.body.iter().filter_map(|class_el| {
                    if let ClassElement::PropertyDefinition(property) = class_el {
                        Some(Vertex::ClassProperty(Rc::new(ClassProperty {
                            property,
                            is_abstract: false,
                        })))
                    } else if let ClassElement::TSAbstractPropertyDefinition(def) = class_el {
                        Some(Vertex::ClassProperty(Rc::new(ClassProperty {
                            property: &def.property_definition,
                            is_abstract: true,
                        })))
                    } else {
                        None
                    }
                }))
            }),
            ("VariableDeclarationAST" | "VariableDeclaration", "entire_span") => {
                resolve_neighbors_with(contexts, |v| {
                    let Vertex::VariableDeclaration(vdecl, ..) = &v else {unreachable!()};
                    Box::new(std::iter::once(Vertex::Span(vdecl.span)))
                })
            }
            ("ImportAST" | "Import", "entire_span") => resolve_neighbors_with(contexts, |v| {
                let Vertex::Import(import, ..) = &v else {unreachable!()};
                Box::new(std::iter::once(Vertex::Span(import.span)))
            }),
            ("ImportAST" | "Import", "specific_import") => resolve_neighbors_with(contexts, |v| {
                let Vertex::Import(import, ..) = &v else {unreachable!()};
                Box::new(import.specifiers.iter().filter_map(|the_specifier| {
                    if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = the_specifier {
                        Some(Vertex::SpecificImport(specifier))
                    } else {
                        None
                    }
                }))
            }),
            ("ImportAST" | "Import", "default_import") => resolve_neighbors_with(contexts, |v| {
                let Vertex::Import(import, ..) = &v else {unreachable!()};
                Box::new(import.specifiers.iter().filter_map(|the_specifier| {
                    if let ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) =
                        the_specifier
                    {
                        Some(Vertex::DefaultImport(specifier))
                    } else {
                        None
                    }
                }))
            }),
            ("InterfaceAST" | "Interface", "name_span") => resolve_neighbors_with(contexts, |v| {
                let Vertex::Interface(iface, ..) = &v else {unreachable!()};
                Box::new(std::iter::once(Vertex::Span(iface.id.span)))
            }),
            ("InterfaceAST" | "Interface", "entire_span") => {
                resolve_neighbors_with(contexts, |v| {
                    let Vertex::Interface(iface, ..) = &v else {unreachable!()};
                    Box::new(std::iter::once(Vertex::Span(iface.span)))
                })
            }
            ("InterfaceAST" | "Interface", "extend") => resolve_neighbors_with(contexts, |v| {
                let Vertex::Interface(iface, ..) = &v else {unreachable!()};
                if let Some(extends) = &iface.extends {
                    Box::new(extends.iter().map(|extend| match &extend.expression {
                        Expression::Identifier(ident) => {
                            Vertex::InterfaceExtends(Rc::new(InterfaceExtends::Identifier(ident)))
                        }
                        Expression::MemberExpression(membexpr) => {
                            Vertex::InterfaceExtends(Rc::new(InterfaceExtends::MemberExpression(membexpr)))
                        }
                        _ => unreachable!("Only ever possible to have an interface extend an identifier or memberexpr. see TS:2499"),
                    }))
                } else {
                    Box::new(std::iter::empty())
                }
            }),
            ("ClassAST" | "Class", "name_span") => resolve_neighbors_with(contexts, |v| {
                if let Some(id) = &v.as_class().unwrap().0.id {
                    Box::new(std::iter::once(Vertex::Span(id.span)))
                } else {
                    Box::new(std::iter::empty())
                }
            }),
            ("ClassAST" | "Class", "entire_class_span") => resolve_neighbors_with(contexts, |v| {
                Box::new(std::iter::once(Vertex::Span(v.as_class().unwrap().0.span)))
            }),
            ("JSXOpeningElement", "attribute") => resolve_neighbors_with(contexts, |v| {
                Box::new(
                    v.as_jsx_opening_element()
                        .expect("to have a jsx opening element")
                        .attributes
                        .iter()
                        .filter_map(|attr| match attr {
                            JSXAttributeItem::Attribute(attr) => Some(Vertex::JsxAttribute(attr)),
                            JSXAttributeItem::SpreadAttribute(_) => None,
                        }),
                )
            }),
            ("JSXElementAST" | "JSXElement", "opening_element") => {
                resolve_neighbors_with(contexts, |v| {
                    let Vertex::JsxElement(el, ..) = &v else {unreachable!()};

                    Box::new(std::iter::once(Vertex::JsxOpeningElement(&el.opening_element)))
                })
            }
            ("JSXElementAST" | "JSXElement", "child_element") => {
                resolve_neighbors_with(contexts, |v| {
                    let Vertex::JsxElement(el, ..) = &v else {unreachable!()};
                    Box::new(el.children.iter().filter_map(|child| {
                        let JSXChild::Element(element) = &child else {return None};
                        Some(Vertex::JsxElement(element, None))
                    }))
                })
            }
            ("JSXElementAST" | "JSXElement", "child_text") => {
                resolve_neighbors_with(contexts, |v| {
                    let Vertex::JsxElement(el, ..) = &v else {unreachable!()};
                    Box::new(el.children.iter().filter_map(|child| {
                        let JSXChild::Text(t) = &child else {return None};
                        Some(Vertex::JsxText(t))
                    }))
                })
            }
            ("JSXElementAST" | "JSXElement", "child_fragment") => {
                resolve_neighbors_with(contexts, |v| {
                    let Vertex::JsxElement(el, ..) = &v else {unreachable!()};
                    Box::new(el.children.iter().filter_map(|child| {
                        let JSXChild::Fragment(f) = &child else {return None};
                        Some(Vertex::JsxFragment(f))
                    }))
                })
            }
            ("JSXElementAST" | "JSXElement", "child_expression_container") => {
                resolve_neighbors_with(contexts, |v| {
                    let Vertex::JsxElement(el, ..) = &v else {unreachable!()};
                    Box::new(el.children.iter().filter_map(|child| {
                        let JSXChild::ExpressionContainer(expr_cont) = &child else {return None};
                        Some(Vertex::JsxExpressionContainer(expr_cont))
                    }))
                })
            }
            ("JSXElementAST" | "JSXElement", "child_spread") => {
                resolve_neighbors_with(contexts, |v| {
                    let Vertex::JsxElement(el, ..) = &v else {unreachable!()};
                    Box::new(el.children.iter().filter_map(|child| {
                        let JSXChild::Spread(s) = &child else {return None};
                        Some(Vertex::JsxSpreadChild(s))
                    }))
                })
            }
            ("JSXOpeningElement", "spread_attribute") => resolve_neighbors_with(contexts, |v| {
                Box::new(
                    v.as_jsx_opening_element()
                        .expect("to have a jsx opening element")
                        .attributes
                        .iter()
                        .filter_map(|attr| match attr {
                            JSXAttributeItem::Attribute(_) => None,
                            JSXAttributeItem::SpreadAttribute(spread_attr) => {
                                Some(Vertex::JsxSpreadAttribute(spread_attr))
                            }
                        }),
                )
            }),
            ("JSXAttribute", "value_as_expression") => resolve_neighbors_with(contexts, |v| {
                let attr = v.as_jsx_attribute().unwrap();
                let Some(attr_value) = &attr.value else {return Box::new(std::iter::empty())};
                Box::new(
                    std::iter::once(match attr_value {
                        JSXAttributeValue::ExpressionContainer(expr) => match &expr.expression {
                            oxc_ast::ast::JSXExpression::Expression(expr) => {
                                Some(Vertex::Expression(expr))
                            }
                            oxc_ast::ast::JSXExpression::EmptyExpression(_) => None,
                        },
                        JSXAttributeValue::Fragment(_)
                        | JSXAttributeValue::StringLiteral(_)
                        | JSXAttributeValue::Element(_) => None,
                    })
                    .flatten(),
                )
            }),
            ("JSXAttribute", "value_as_url") => resolve_neighbors_with(contexts, |v| {
                let attr = v.as_jsx_attribute().unwrap();
                let Some(maybe_url) = jsx_attribute_to_constant_string(attr) else {return Box::new(std::iter::empty())};
                let Ok(parsed_url) = Url::parse(&maybe_url) else {return Box::new(std::iter::empty())};
                return Box::new(std::iter::once(Vertex::Url(Rc::new(parsed_url))));
            }),
            ("VariableDeclarationAST" | "VariableDeclaration", "left") => {
                resolve_neighbors_with(contexts, |v| {
                    let Vertex::VariableDeclaration(var, ..) = &v else {unreachable!()};
                    return Box::new(std::iter::once(Vertex::AssignmentType(&var.id.kind)));
                })
            }
            ("URL", "search_parameter") => resolve_neighbors_with(contexts, |v| {
                Box::new(
                    v.as_url()
                        .unwrap()
                        .query_pairs()
                        .map(|(key, value)| {
                            Vertex::SearchParameter(SearchParameter {
                                key: key.to_string(),
                                value: value.to_string(),
                            })
                        })
                        .collect::<Vec<_>>()
                        .into_iter(),
                )
            }),
            ("ObjectLiteral", "value") => {
                let key_wanted = Rc::new(
                    parameters
                        .get("key")
                        .expect("key to always be non-null")
                        .as_str()
                        .expect("key to be a string")
                        .to_string(),
                );
                resolve_neighbors_with(contexts, move |v| {
                    let key_wanted = Rc::clone(&key_wanted);
                    let obj = v.as_object_literal().expect("to have an objectliteral");

                    Box::new(obj.properties.iter().filter(move |property| {
                        let ObjectPropertyKind::ObjectProperty(prop) = property else {return false};
                        match &prop.key {
                            oxc_ast::ast::PropertyKey::Identifier(ident) => {
                                ident.name == key_wanted.as_ref()
                            }
                            oxc_ast::ast::PropertyKey::PrivateIdentifier(priv_ident) => unimplemented!("getting a private property on an object is unimplemented as of yet"),
                            oxc_ast::ast::PropertyKey::Expression(expr) => expr_to_maybe_const_string(expr).map_or_else(|| false, |key| &key == key_wanted.as_ref())
                        }
                    }).map(|x| {
                        let ObjectPropertyKind::ObjectProperty(prop) = &x else {unreachable!()};
                        materialize_expression(&prop.value)
                    }))
                })
            }
            (
                "ASTNode"
                | "JSXElementAST"
                | "InterfaceAST"
                | "ImportAST"
                | "ClassAST"
                | "ReturnStatementAST"
                | "TypeAnnotationAST"
                | "VariableDeclarationAST",
                "ancestor",
            ) => resolve_neighbors_with(contexts, |v| {
                let id = match v {
                    Vertex::AstNode(node) => node.id(),
                    Vertex::JsxElement(_, Some(id))
                    | Vertex::Interface(_, Some(id))
                    | Vertex::Import(_, Some(id))
                    | Vertex::Class(_, Some(id))
                    | Vertex::TypeAnnotation(_, Some(id))
                    | Vertex::VariableDeclaration(_, Some(id))
                    | Vertex::ReturnStatementAst(_, id) => *id,
                    _ => unreachable!(),
                };
                Box::new(
                    Box::new(self.semantic.nodes().ancestors(id))
                        .map(|ancestor| self.semantic.nodes().get_node(ancestor))
                        .map(|ancestor| materialize_ast_node(*ancestor)),
                )
            }),
            (
                "ASTNode"
                | "JSXElementAST"
                | "InterfaceAST"
                | "ImportAST"
                | "ClassAST"
                | "ReturnStatementAST"
                | "TypeAnnotationAST"
                | "VariableDeclarationAST",
                "parent",
            ) => resolve_neighbors_with(contexts, |v| {
                let id = match v {
                    Vertex::AstNode(node) => node.id(),
                    Vertex::JsxElement(_, Some(id))
                    | Vertex::Interface(_, Some(id))
                    | Vertex::Import(_, Some(id))
                    | Vertex::Class(_, Some(id))
                    | Vertex::TypeAnnotation(_, Some(id))
                    | Vertex::VariableDeclaration(_, Some(id))
                    | Vertex::ReturnStatementAst(_, id) => *id,
                    _ => unreachable!(),
                };
                if let Some(parent) = self.semantic.nodes().parent_node(id) {
                    Box::new(std::iter::once(materialize_ast_node(*parent)))
                } else {
                    Box::new(std::iter::empty())
                }
            }),
            ("ReturnStatementAST", "expression") => resolve_neighbors_with(contexts, |v| {
                if let Some(expr) = &v.as_return_statement_ast().unwrap().0.argument {
                    Box::new(std::iter::once(materialize_expression(expr)))
                } else {
                    Box::new(std::iter::empty())
                }
            }),
            (_, "span") => resolve_neighbors_with(contexts, |v| {
                Box::new(std::iter::once(Vertex::Span(match v {
                    Vertex::TypeAnnotation(ta, ..) => ta.span,
                    Vertex::Type(typed) => typed.span(),
                    Vertex::Url(_)
                    | Vertex::SearchParameter(..)
                    | Vertex::PathCompareResult(_)
                    | Vertex::Class(..)
                    | Vertex::Span(_)
                    | Vertex::File => unreachable!(),
                    Vertex::ClassMethod(it) => it.method.span,
                    Vertex::ClassProperty(it) => it.property.span,
                    Vertex::Interface(it, ..) => it.span,
                    Vertex::InterfaceExtends(iextends) => match iextends.as_ref() {
                        InterfaceExtends::Identifier(ident) => ident.span,
                        InterfaceExtends::MemberExpression(membexpr) => membexpr.span(),
                    },
                    Vertex::JsxOpeningElement(el) => el.span,
                    Vertex::JsxAttribute(attr) => attr.span,
                    Vertex::JsxSpreadAttribute(spr_attr) => spr_attr.span,
                    Vertex::Expression(expr) => expr.span(),
                    Vertex::Import(import, ..) => import.span,
                    Vertex::SpecificImport(import) => import.span,
                    Vertex::DefaultImport(import) => import.span,
                    Vertex::AstNode(node) => node.kind().span(),
                    Vertex::JsxElement(expr, ..) => expr.span,
                    Vertex::ObjectLiteral(objlit) => objlit.span,
                    Vertex::ReturnStatementAst(node, ..) => node.span,
                    Vertex::JsxText(t) => t.span,
                    Vertex::JsxFragment(f) => f.span,
                    Vertex::JsxExpressionContainer(c) => c.span,
                    Vertex::JsxSpreadChild(sc) => sc.span,
                    Vertex::VariableDeclaration(vd, ..) => vd.span,
                    Vertex::AssignmentType(assmt, ..) => assmt.span(),
                })))
            }),
            _ => {
                unimplemented!("unexpected neighbor of: {type_name}.{edge_name}")
            }
        }
    }

    fn resolve_coercion(
        &self,
        contexts: ContextIterator<'b, Self::Vertex>,
        _type_name: &Arc<str>,
        coerce_to_type: &Arc<str>,
        _resolve_info: &ResolveInfo,
    ) -> ContextOutcomeIterator<'b, Self::Vertex, bool> {
        match coerce_to_type.as_ref() {
            "SimpleExtend" => resolve_coercion_with(
                contexts,
                |v| matches!(v, Vertex::InterfaceExtends(iex) if matches!(iex.as_ref(), InterfaceExtends::Identifier(..))),
            ),
            "MemberExtend" => resolve_coercion_with(
                contexts,
                |v| matches!(v, Vertex::InterfaceExtends(iex) if matches!(iex.as_ref(), InterfaceExtends::MemberExpression(..))),
            ),
            // Expression
            "ObjectLiteral" => {
                resolve_coercion_with(contexts, |v| matches!(v, Vertex::ObjectLiteral(_)))
            }
            "JSXElementAST" => {
                resolve_coercion_with(contexts, |v| matches!(v, Vertex::JsxElement(_, Some(_))))
            }
            "JSXElement" => {
                resolve_coercion_with(contexts, |v| matches!(v, Vertex::JsxElement(..)))
            }
            // ASTNode
            "ReturnStatementAST" => {
                resolve_coercion_with(contexts, |v| matches!(v, Vertex::ReturnStatementAst(..)))
            }
            "ClassAST" => {
                resolve_coercion_with(contexts, |v| matches!(v, Vertex::Class(_, Some(_))))
            }
            "InterfaceAST" => {
                resolve_coercion_with(contexts, |v| matches!(v, Vertex::Class(_, Some(_))))
            }
            _ => unimplemented!("unexpected coercion to: {coerce_to_type}"),
        }
    }
}

trait ToStringJSX {
    fn to_string_jsx(&self) -> String;
}

impl ToStringJSX for JSXIdentifier {
    fn to_string_jsx(&self) -> String {
        self.name.to_string()
    }
}

impl ToStringJSX for JSXNamespacedName {
    fn to_string_jsx(&self) -> String {
        format!("{}:{}", &self.namespace.name, &self.property.name)
    }
}

impl<'a> ToStringJSX for JSXMemberExpression<'a> {
    fn to_string_jsx(&self) -> String {
        let mut parts = vec![self.property.name.to_string()];
        let mut obj = &self.object;
        loop {
            match obj {
                oxc_ast::ast::JSXMemberExpressionObject::Identifier(ident) => {
                    parts.push(ident.name.to_string());
                    break;
                }
                oxc_ast::ast::JSXMemberExpressionObject::MemberExpression(inner_memexpr) => {
                    parts.push(inner_memexpr.property.name.to_string());
                    obj = &inner_memexpr.object;
                }
            }
        }
        parts.reverse();
        parts.join(".")
    }
}

impl<'a> ToStringJSX for JSXElementName<'a> {
    fn to_string_jsx(&self) -> String {
        match self {
            JSXElementName::Identifier(ident) => ident.to_string_jsx(),
            JSXElementName::NamespacedName(nsn) => nsn.to_string_jsx(),
            JSXElementName::MemberExpression(memexpr) => memexpr.to_string_jsx(),
        }
    }
}

impl<'a> ToStringJSX for JSXAttributeName<'a> {
    fn to_string_jsx(&self) -> String {
        match self {
            JSXAttributeName::Identifier(ident) => ident.to_string_jsx(),
            JSXAttributeName::NamespacedName(nsn) => nsn.to_string_jsx(),
        }
    }
}

fn try_get_constant_string_field_value_from_template_lit(tlit: &TemplateLiteral) -> Option<String> {
    if tlit.expressions.len() == 0 && tlit.quasis.len() == 1 {
        let quasi = &tlit.quasis[0].value;
        Some(quasi.cooked.as_ref().unwrap_or(&quasi.raw).to_string())
    } else {
        None
    }
}

fn expr_to_maybe_const_string<'a>(expr: &'a Expression<'a>) -> Option<String> {
    match expr {
        Expression::StringLiteral(slit) => Some(slit.value.to_string()),
        Expression::TemplateLiteral(tlit) => {
            try_get_constant_string_field_value_from_template_lit(tlit.0)
        }
        _ => None,
    }
}

fn jsx_attribute_to_constant_string<'a>(attr: &'a JSXAttribute<'a>) -> Option<String> {
    let Some(attr_value) = &attr.value else {return None};
    match attr_value {
        JSXAttributeValue::StringLiteral(slit) => slit.value.to_string().into(),
        JSXAttributeValue::ExpressionContainer(expr) => match &expr.expression {
            oxc_ast::ast::JSXExpression::Expression(expr) => expr_to_maybe_const_string(expr),
            oxc_ast::ast::JSXExpression::EmptyExpression(_) => None,
        },
        JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_) => None,
    }
}

fn materialize_expression<'a>(expr: &'a Expression<'a>) -> Vertex {
    match &expr.get_inner_expression() {
        Expression::ObjectExpression(objexpr) => Vertex::ObjectLiteral(objexpr),
        Expression::JSXElement(element) => Vertex::JsxElement(element, None),
        _ => Vertex::Expression(expr),
    }
}

fn materialize_ast_node(ast_node: AstNode<'_>) -> Vertex {
    match ast_node.kind() {
        AstKind::ReturnStatement(ret_stmt) => Vertex::ReturnStatementAst(ret_stmt, ast_node.id()),
        AstKind::Class(class) => Vertex::Class(class, Some(ast_node.id())),
        AstKind::JSXElement(el) => Vertex::JsxElement(el, Some(ast_node.id())),
        AstKind::TSInterfaceDeclaration(iface) => Vertex::Interface(iface, Some(ast_node.id())),
        AstKind::TSTypeAnnotation(anno) => Vertex::TypeAnnotation(anno, Some(ast_node.id())),
        AstKind::VariableDeclarator(var) => Vertex::VariableDeclaration(var, Some(ast_node.id())),
        AstKind::ModuleDeclaration(md) if matches!(md, ModuleDeclaration::ImportDeclaration(_)) => {
            let ModuleDeclaration::ImportDeclaration(id) = md else {unreachable!()};
            Vertex::Import(id, Some(ast_node.id()))
        }
        _ => Vertex::AstNode(ast_node),
    }
}
