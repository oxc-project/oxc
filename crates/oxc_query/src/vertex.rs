use std::rc::Rc;

use enum_as_inner::EnumAsInner;
use oxc_ast::{
    ast::{
        BindingPatternKind, Class, Expression, IdentifierReference, ImportDeclaration,
        ImportDefaultSpecifier, ImportSpecifier, JSXAttribute, JSXElement, JSXExpressionContainer,
        JSXFragment, JSXOpeningElement, JSXSpreadAttribute, JSXSpreadChild, JSXText,
        MemberExpression, MethodDefinition, ModuleDeclaration, ObjectExpression,
        PropertyDefinition, ReturnStatement, TSInterfaceDeclaration, TSType, TSTypeAnnotation,
        VariableDeclarator,
    },
    AstKind,
};
use oxc_semantic::{AstNode, AstNodeId};
use oxc_span::{GetSpan, Span};
use trustfall::provider::Typename;
use url::Url;

use crate::util::{expr_to_maybe_const_string, jsx_attribute_to_constant_string};

#[non_exhaustive]
#[derive(Debug, Clone, EnumAsInner)]
pub enum Vertex<'a> {
    ASTNode(AstNode<'a>),
    AssignmentType(&'a BindingPatternKind<'a>),
    Class(Rc<ClassVertex<'a>>),
    ClassMethod(Rc<ClassMethodVertex<'a>>),
    ClassProperty(Rc<ClassPropertyVertex<'a>>),
    DefaultImport(&'a ImportDefaultSpecifier),
    Expression(&'a Expression<'a>),
    File,
    Import(Rc<ImportVertex<'a>>),
    Interface(Rc<InterfaceVertex<'a>>),
    InterfaceExtend(Rc<InterfaceExtendVertex<'a>>),
    JSXAttribute(&'a JSXAttribute<'a>),
    JSXElement(Rc<JSXElementVertex<'a>>),
    JSXExpressionContainer(&'a JSXExpressionContainer<'a>),
    JSXFragment(&'a JSXFragment<'a>),
    JSXOpeningElement(&'a JSXOpeningElement<'a>),
    JSXSpreadAttribute(&'a JSXSpreadAttribute<'a>),
    JSXSpreadChild(&'a JSXSpreadChild<'a>),
    JSXText(&'a JSXText),
    ObjectLiteral(&'a ObjectExpression<'a>),
    PathPart(usize),
    SearchParameter(Rc<SearchParameterVertex>),
    Span(Span),
    SpecificImport(&'a ImportSpecifier),
    TypeAnnotation(Rc<TypeAnnotationVertex<'a>>),
    Type(&'a TSType<'a>),
    Url(Rc<Url>),
    VariableDeclaration(Rc<VariableDeclarationVertex<'a>>),
    ReturnStatementAST(Rc<ReturnStatementVertex<'a>>),
}

impl<'a> Vertex<'a> {
    pub fn span(&self) -> Span {
        match &self {
            Self::AssignmentType(data) => data.span(),
            Self::ASTNode(data) => data.kind().span(),
            Self::Class(data) => data.class.span,
            Self::ClassMethod(data) => data.method.span,
            Self::ClassProperty(data) => data.property.span,
            Self::DefaultImport(data) => data.span,
            Self::Expression(data) => data.span(),
            Self::Import(data) => data.import.span,
            Self::Interface(data) => data.interface.span,
            Self::InterfaceExtend(data) => match **data {
                InterfaceExtendVertex::Identifier(ident) => ident.span,
                InterfaceExtendVertex::MemberExpression(membexpr) => (*membexpr).span(),
            },
            Self::JSXAttribute(data) => data.span,
            Self::JSXElement(data) => data.element.span,
            Self::JSXExpressionContainer(data) => data.span,
            Self::JSXFragment(data) => data.span,
            Self::JSXOpeningElement(data) => data.span,
            Self::JSXSpreadAttribute(data) => data.span,
            Self::JSXSpreadChild(data) => data.span,
            Self::JSXText(data) => data.span,
            Self::ObjectLiteral(data) => data.span,
            Self::SpecificImport(data) => data.span,
            Self::TypeAnnotation(data) => data.type_annotation.span,
            Self::Type(data) => data.span(),
            Self::VariableDeclaration(data) => data.variable_declaration.span,
            Self::ReturnStatementAST(data) => data.return_statement.span,
            Self::File
            | Self::Url(_)
            | Self::PathPart(_)
            | Self::SearchParameter(_)
            | Self::Span(_) => {
                unreachable!("Tried to get the span from a {self:#?}")
            }
        }
    }

    pub fn ast_node_id(&self) -> Option<AstNodeId> {
        match &self {
            Vertex::ASTNode(data) => Some(data.id()),
            Vertex::Class(data) => data.ast_node.map(|x| x.id()),
            Vertex::Import(data) => data.ast_node.map(|x| x.id()),
            Vertex::Interface(data) => data.ast_node.map(|x| x.id()),
            Vertex::JSXElement(data) => data.ast_node.map(|x| x.id()),
            Vertex::TypeAnnotation(data) => data.ast_node.map(|x| x.id()),
            Vertex::VariableDeclaration(data) => data.ast_node.map(|x| x.id()),
            Vertex::ReturnStatementAST(data) => data.ast_node.map(|x| x.id()),
            Vertex::DefaultImport(_)
            | Vertex::AssignmentType(_)
            | Vertex::ClassMethod(_)
            | Vertex::Expression(_)
            | Vertex::File
            | Vertex::InterfaceExtend(_)
            | Vertex::JSXAttribute(_)
            | Vertex::JSXExpressionContainer(_)
            | Vertex::JSXFragment(_)
            | Vertex::ObjectLiteral(_)
            | Vertex::JSXText(_)
            | Vertex::JSXSpreadChild(_)
            | Vertex::JSXSpreadAttribute(_)
            | Vertex::JSXOpeningElement(_)
            | Vertex::PathPart(_)
            | Vertex::Url(_)
            | Vertex::Type(_)
            | Vertex::SpecificImport(_)
            | Vertex::Span(_)
            | Vertex::SearchParameter(_)
            | Vertex::ClassProperty(_) => None,
        }
    }

    pub fn make_url(attr: &'a JSXAttribute<'a>) -> Option<Self> {
        let Some(maybe_url) = jsx_attribute_to_constant_string(attr) else { return None };
        let Ok(parsed_url) = Url::parse(&maybe_url) else { return None };
        return Some(Vertex::Url(Rc::new(parsed_url)));
    }

    pub fn as_constant_string(&self) -> Option<String> {
        match &self {
            Vertex::Expression(expr) => expr_to_maybe_const_string(expr),
            _ => None,
        }
    }
}

impl Typename for Vertex<'_> {
    fn typename(&self) -> &'static str {
        match self {
            Vertex::ASTNode(_) => "ASTNode",
            Vertex::AssignmentType(_) => "AssignmentType",
            Vertex::Class(class) => class.typename(),
            Vertex::ClassMethod(_) => "ClassMethod",
            Vertex::ClassProperty(_) => "ClassProperty",
            Vertex::DefaultImport(_) => "DefaultImport",
            Vertex::Expression(_) => "Expression",
            Vertex::File => "File",
            Vertex::Import(import) => import.typename(),
            Vertex::Interface(iface) => iface.typename(),
            Vertex::InterfaceExtend(iex) => match **iex {
                InterfaceExtendVertex::Identifier(_) => "SimpleExtend",
                InterfaceExtendVertex::MemberExpression(_) => "MemberExtend",
            },
            Vertex::JSXAttribute(_) => "JSXAttribute",
            Vertex::JSXElement(jsx) => jsx.typename(),
            Vertex::JSXExpressionContainer(_) => "JSXExpressionContainer",
            Vertex::JSXFragment(_) => "JSXFragment",
            Vertex::JSXOpeningElement(_) => "JSXOpeningElement",
            Vertex::JSXSpreadAttribute(_) => "JSXSpreadAttribute",
            Vertex::JSXSpreadChild(_) => "JSXSpreadChild",
            Vertex::JSXText(_) => "JSXText",
            Vertex::ObjectLiteral(_) => "ObjectLiteral",
            Vertex::PathPart(_) => "PathPart",
            Vertex::SearchParameter(_) => "SearchParameter",
            Vertex::Span(_) => "Span",
            Vertex::SpecificImport(_) => "SpecificImport",
            Vertex::TypeAnnotation(tn) => tn.typename(),
            Vertex::Type(_) => "Type",
            Vertex::Url(_) => "URL",
            Vertex::VariableDeclaration(_) => "VariableDeclaration",
            Vertex::ReturnStatementAST(_) => "ReturnStatementAST",
        }
    }
}

impl<'a> From<AstNode<'a>> for Vertex<'a> {
    fn from(ast_node: AstNode<'a>) -> Self {
        match ast_node.kind() {
            AstKind::ReturnStatement(return_statement) => Self::ReturnStatementAST(
                ReturnStatementVertex { ast_node: Some(ast_node), return_statement }.into(),
            ),
            AstKind::JSXElement(element) => {
                Self::JSXElement(JSXElementVertex { ast_node: Some(ast_node), element }.into())
            }
            AstKind::TSInterfaceDeclaration(interface) => {
                Self::Interface(InterfaceVertex { ast_node: Some(ast_node), interface }.into())
            }
            AstKind::TSTypeAnnotation(type_annotation) => Self::TypeAnnotation(
                TypeAnnotationVertex { ast_node: Some(ast_node), type_annotation }.into(),
            ),
            AstKind::VariableDeclarator(variable_declaration) => Self::VariableDeclaration(
                VariableDeclarationVertex { ast_node: Some(ast_node), variable_declaration }.into(),
            ),
            AstKind::ModuleDeclaration(ModuleDeclaration::ImportDeclaration(import)) => {
                Self::Import(ImportVertex { ast_node: Some(ast_node), import }.into())
            }
            AstKind::Class(class) => {
                Self::Class(ClassVertex { ast_node: Some(ast_node), class }.into())
            }
            _ => Vertex::ASTNode(ast_node),
        }
    }
}

impl<'a> From<&'a Expression<'a>> for Vertex<'a> {
    fn from(expr: &'a Expression<'a>) -> Self {
        // FIXME: We just get rid of all parentheses here, but we shouldn't do that...

        // NOTE: When string literal / template literal is added, add to as_constant_string
        match &expr.get_inner_expression() {
            Expression::ObjectExpression(objexpr) => Vertex::ObjectLiteral(objexpr),
            Expression::JSXElement(element) => {
                Vertex::JSXElement(JSXElementVertex { ast_node: None, element }.into())
            }
            _ => Vertex::Expression(expr),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ClassVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub class: &'a Class<'a>,
}

impl<'a> Typename for ClassVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() { "ClassAST" } else { "Class" }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ClassMethodVertex<'a> {
    pub method: &'a MethodDefinition<'a>,
    pub is_abstract: bool,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ClassPropertyVertex<'a> {
    pub property: &'a PropertyDefinition<'a>,
    pub is_abstract: bool,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ImportVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub import: &'a ImportDeclaration<'a>,
}

impl<'a> Typename for ImportVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() { "ImportAST" } else { "Import" }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct InterfaceVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub interface: &'a TSInterfaceDeclaration<'a>,
}

impl<'a> Typename for InterfaceVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() { "InterfaceAST" } else { "Interface" }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum InterfaceExtendVertex<'a> {
    Identifier(&'a IdentifierReference),
    MemberExpression(&'a MemberExpression<'a>),
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct JSXElementVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub element: &'a JSXElement<'a>,
}

impl<'a> Typename for JSXElementVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() { "JSXElementAST" } else { "JSXElement" }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ReturnStatementVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub return_statement: &'a ReturnStatement<'a>,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct TypeAnnotationVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub type_annotation: &'a TSTypeAnnotation<'a>,
}

impl<'a> Typename for TypeAnnotationVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() { "TypeAnnotationAST" } else { "TypeAnnotation" }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SearchParameterVertex {
    pub key: String,
    pub value: String,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct VariableDeclarationVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub variable_declaration: &'a VariableDeclarator<'a>,
}
