use std::rc::Rc;

use oxc_ast::{ast::*, AstKind};
use oxc_semantic::AstNode;
use oxc_span::Span;
use trustfall::provider::TrustfallEnumVertex;

#[derive(Debug, Clone, TrustfallEnumVertex)]
pub enum Vertex<'a> {
    ASTNode(AstNode<'a>),
    AssignmentType(&'a BindingPatternKind<'a>),
    Class(Rc<ClassVertex<'a>>),
    ClassMethod(Rc<ClassMethodVertex<'a>>),
    ClassProperty(Rc<ClassPropertyVertex<'a>>),
    DefaultImport(()),
    Expression(()),
    File(()),
    HasSpan(()),
    Import(()),
    ImportAST(()),
    Interface(()),
    InterfaceAST(()),
    InterfaceExtend(()),
    JSXAttribute(()),
    JSXElement(()),
    JSXElementAST(()),
    JSXExpressionContainer(()),
    JSXFragment(()),
    JSXOpeningElement(()),
    JSXSpreadAttribute(()),
    JSXSpreadChild(()),
    JSXText(()),
    MemberExtend(()),
    ObjectLiteral(()),
    PathPart(()),
    ReturnStatementAST(()),
    SearchParameter(()),
    SimpleExtend(()),
    Span(Span),
    SpecificImport(()),
    TypeAnnotation(()),
    TypeAnnotationAST(()),
    Type_(()),
    URL(()),
    VariableDeclaration(()),
    VariableDeclarationAST(()),
}

impl<'a> Vertex<'a> {
    fn make_type_name(&self) -> String {
        match self {
            Vertex::ASTNode(_) => "ASTNode".to_owned(),
            Vertex::AssignmentType(_) => "AssignmentType".to_owned(),
            Vertex::Class(class) => class.type_name(),
            Vertex::ClassMethod(_) => todo!(),
            Vertex::ClassProperty(_) => todo!(),
            Vertex::DefaultImport(_) => todo!(),
            Vertex::Expression(_) => todo!(),
            Vertex::File(_) => todo!(),
            Vertex::HasSpan(_) => todo!(),
            Vertex::Import(_) => todo!(),
            Vertex::ImportAST(_) => todo!(),
            Vertex::Interface(_) => todo!(),
            Vertex::InterfaceAST(_) => todo!(),
            Vertex::InterfaceExtend(_) => todo!(),
            Vertex::JSXAttribute(_) => todo!(),
            Vertex::JSXElement(_) => todo!(),
            Vertex::JSXElementAST(_) => todo!(),
            Vertex::JSXExpressionContainer(_) => todo!(),
            Vertex::JSXFragment(_) => todo!(),
            Vertex::JSXOpeningElement(_) => todo!(),
            Vertex::JSXSpreadAttribute(_) => todo!(),
            Vertex::JSXSpreadChild(_) => todo!(),
            Vertex::JSXText(_) => todo!(),
            Vertex::MemberExtend(_) => todo!(),
            Vertex::ObjectLiteral(_) => todo!(),
            Vertex::PathPart(_) => todo!(),
            Vertex::ReturnStatementAST(_) => todo!(),
            Vertex::SearchParameter(_) => todo!(),
            Vertex::SimpleExtend(_) => todo!(),
            Vertex::Span(_) => todo!(),
            Vertex::SpecificImport(_) => todo!(),
            Vertex::TypeAnnotation(_) => todo!(),
            Vertex::TypeAnnotationAST(_) => todo!(),
            Vertex::Type_(_) => todo!(),
            Vertex::URL(_) => todo!(),
            Vertex::VariableDeclaration(_) => todo!(),
            Vertex::VariableDeclarationAST(_) => todo!(),
        }
    }
}

impl<'a> From<AstNode<'a>> for Vertex<'a> {
    fn from(ast_node: AstNode<'a>) -> Self {
        match ast_node.kind() {
            AstKind::Class(class) => {
                Self::Class(ClassVertex { ast_node: Some(ast_node), class }.into())
            }
            _ => Vertex::ASTNode(ast_node),
        }
    }
}

pub trait TypeName {
    fn type_name(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct ClassVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub class: &'a Class<'a>,
}

impl<'a> TypeName for ClassVertex<'a> {
    fn type_name(&self) -> String {
        if self.ast_node.is_some() { "ClassAST" } else { "Class" }.to_owned()
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
