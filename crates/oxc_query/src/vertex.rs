use std::rc::Rc;

use enum_as_inner::EnumAsInner;
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::{AstNode, AstNodeId};
use oxc_span::{GetSpan, Span};
use url::Url;

use crate::util::{expr_to_maybe_const_string, jsx_attribute_to_constant_string};

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
    URL(Rc<Url>),
    VariableDeclaration(Rc<VariableDeclarationVertex<'a>>),
    ReturnStatementAST(Rc<ReturnStatementVertex<'a>>),
}

impl<'a> Vertex<'a> {
    pub fn make_type_name(&self) -> &'static str {
        match self {
            Vertex::ASTNode(_) => "ASTNode",
            Vertex::AssignmentType(_) => "AssignmentType",
            Vertex::Class(class) => class.type_name(),
            Vertex::ClassMethod(_) => "ClassMethod",
            Vertex::ClassProperty(_) => "ClassProperty",
            Vertex::DefaultImport(_) => "DefaultImport",
            Vertex::Expression(_) => "Expression",
            Vertex::File => "File",
            Vertex::Import(import) => import.type_name(),
            Vertex::Interface(iface) => iface.type_name(),
            Vertex::InterfaceExtend(iex) => match **iex {
                InterfaceExtendVertex::Identifier(_) => "SimpleExtend",
                InterfaceExtendVertex::MemberExpression(_) => "MemberExtend",
            },
            Vertex::JSXAttribute(_) => "JSXAttribute",
            Vertex::JSXElement(jsx) => jsx.type_name(),
            Vertex::JSXExpressionContainer(_) => "JSXExpressionContainer",
            Vertex::JSXFragment(_) => "JSXFragment",
            Vertex::JSXOpeningElement(_) => "JSXOpeningElement",
            Vertex::JSXSpreadAttribute(_) => "JSXSpreadAttribute",
            Vertex::JSXSpreadChild(_) => "JSXSpreadChild",
            Vertex::JSXText(_) => "JSXText",
            Vertex::ObjectLiteral(_) => "ObjectLiteral",
            Vertex::PathPart(_) => "PathPart".into(),
            Vertex::SearchParameter(_) => "SearchParameter",
            Vertex::Span(_) => "Span",
            Vertex::SpecificImport(_) => "SpecificImport",
            Vertex::TypeAnnotation(_) => "TypeAnnotation",
            Vertex::Type(_) => "Type",
            Vertex::URL(_) => "URL",
            Vertex::VariableDeclaration(_) => "VariableDeclaration",
            Vertex::ReturnStatementAST(_) => "ReturnStatementAST",
        }
    }

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
            Self::URL(_) => unreachable!(),
            Self::VariableDeclaration(data) => data.variable_declaration.span,
            Self::ReturnStatementAST(data) => data.return_statement.span,
            Self::File | Self::PathPart(_) | Self::SearchParameter(_) | Self::Span(_) => {
                unreachable!()
            }
        }
    }

    pub fn ast_node_id(&self) -> AstNodeId {
        match &self {
            Vertex::ASTNode(data) => data.id(),
            Vertex::Class(data) => data.ast_node.unwrap().id(),
            Vertex::Import(data) => data.ast_node.unwrap().id(),
            Vertex::Interface(data) => data.ast_node.unwrap().id(),
            Vertex::JSXElement(data) => data.ast_node.unwrap().id(),
            Vertex::TypeAnnotation(data) => data.ast_node.unwrap().id(),
            Vertex::VariableDeclaration(data) => data.ast_node.unwrap().id(),
            Vertex::ReturnStatementAST(data) => data.ast_node.unwrap().id(),
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
            | Vertex::URL(_)
            | Vertex::Type(_)
            | Vertex::SpecificImport(_)
            | Vertex::Span(_)
            | Vertex::SearchParameter(_)
            | Vertex::ClassProperty(_) => {
                unreachable!()
            }
        }
    }

    pub fn make_url(attr: &'a JSXAttribute<'a>) -> Option<Self> {
        let Some(maybe_url) = jsx_attribute_to_constant_string(attr) else {return None};
        let Ok(parsed_url) = Url::parse(&maybe_url) else {return None};
        return Some(Vertex::URL(Rc::new(parsed_url)));
    }

    pub fn as_constant_string(&self) -> Option<String> {
        match &self {
            Vertex::Expression(expr) => expr_to_maybe_const_string(expr),
            _ => None,
        }
    }
}

impl trustfall::provider::Typename for Vertex<'_> {
    fn typename(&self) -> &'static str {
        self.make_type_name()
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
            AstKind::ModuleDeclaration(md)
                if matches!(md, ModuleDeclaration::ImportDeclaration(_)) =>
            {
                let ModuleDeclaration::ImportDeclaration(import) = md else {unreachable!()};
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

        // NOTE: When string literal / template lietal is added, add to as_constant_string
        match &expr.get_inner_expression() {
            Expression::ObjectExpression(objexpr) => Vertex::ObjectLiteral(objexpr),
            Expression::JSXElement(element) => {
                Vertex::JSXElement(JSXElementVertex { ast_node: None, element }.into())
            }
            _ => Vertex::Expression(expr),
        }
    }
}

pub trait TypeName {
    fn type_name(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub struct ClassVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub class: &'a Class<'a>,
}

impl<'a> TypeName for ClassVertex<'a> {
    fn type_name(&self) -> &'static str {
        if self.ast_node.is_some() { "ClassAST" } else { "Class" }
    }
}

#[derive(Debug, Clone)]
pub struct ClassMethodVertex<'a> {
    pub method: &'a MethodDefinition<'a>,
    pub is_abstract: bool,
}

#[derive(Debug, Clone)]
pub struct ClassPropertyVertex<'a> {
    pub property: &'a PropertyDefinition<'a>,
    pub is_abstract: bool,
}

#[derive(Debug, Clone)]
pub struct ImportVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub import: &'a ImportDeclaration<'a>,
}

impl<'a> TypeName for ImportVertex<'a> {
    fn type_name(&self) -> &'static str {
        if self.ast_node.is_some() { "ImportAST" } else { "Import" }
    }
}

#[derive(Debug, Clone)]
pub struct InterfaceVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub interface: &'a TSInterfaceDeclaration<'a>,
}

impl<'a> TypeName for InterfaceVertex<'a> {
    fn type_name(&self) -> &'static str {
        if self.ast_node.is_some() { "InteraceAST" } else { "Interface" }
    }
}

#[derive(Debug, Clone)]
pub enum InterfaceExtendVertex<'a> {
    Identifier(&'a IdentifierReference),
    MemberExpression(&'a MemberExpression<'a>),
}

#[derive(Debug, Clone)]
pub struct JSXElementVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub element: &'a JSXElement<'a>,
}

impl<'a> TypeName for JSXElementVertex<'a> {
    fn type_name(&self) -> &'static str {
        if self.ast_node.is_some() { "JSXElementAST" } else { "JSXElement" }
    }
}

#[derive(Debug, Clone)]
pub struct ReturnStatementVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub return_statement: &'a ReturnStatement<'a>,
}

#[derive(Debug, Clone)]
pub struct TypeAnnotationVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub type_annotation: &'a TSTypeAnnotation<'a>,
}

impl<'a> TypeName for TypeAnnotationVertex<'a> {
    fn type_name(&self) -> &'static str {
        if self.ast_node.is_some() { "TypeAnnotationAST" } else { "TypeAnnotation" }
    }
}

#[derive(Debug, Clone)]
pub struct SearchParameterVertex {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct VariableDeclarationVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub variable_declaration: &'a VariableDeclarator<'a>,
}
