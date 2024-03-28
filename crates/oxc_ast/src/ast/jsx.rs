//! [JSX](https://facebook.github.io/jsx)

// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use oxc_allocator::{Box, Vec};
use oxc_macros::ast_node;
use oxc_span::{Atom, Span};
#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

use super::{js::*, literal::*, ts::*};

// 1.2 JSX Elements

/// JSX Element
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct JSXElement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub opening_element: Box<'a, JSXOpeningElement<'a>>,
    pub closing_element: Option<Box<'a, JSXClosingElement<'a>>>,
    pub children: Vec<'a, JSXChild<'a>>,
}

/// JSX Opening Element
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct JSXOpeningElement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub self_closing: bool,
    pub name: JSXElementName<'a>,
    pub attributes: Vec<'a, JSXAttributeItem<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// JSX Closing Element
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXClosingElement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: JSXElementName<'a>,
}

/// JSX Fragment
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct JSXFragment<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub opening_fragment: JSXOpeningFragment,
    pub closing_fragment: JSXClosingFragment,
    pub children: Vec<'a, JSXChild<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXOpeningFragment {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXClosingFragment {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

/// JSX Element Name
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXElementName<'a> {
    /// `<Apple />`
    Identifier(JSXIdentifier<'a>),
    /// `<Apple:Orange />`
    NamespacedName(Box<'a, JSXNamespacedName<'a>>),
    /// `<Apple.Orange />`
    MemberExpression(Box<'a, JSXMemberExpression<'a>>),
}

/// JSX Namespaced Name
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXNamespacedName<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub namespace: JSXIdentifier<'a>,
    pub property: JSXIdentifier<'a>,
}

impl<'a> std::fmt::Display for JSXNamespacedName<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace.name, self.property.name)
    }
}

/// JSX Member Expression
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXMemberExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub object: JSXMemberExpressionObject<'a>,
    pub property: JSXIdentifier<'a>,
}

impl<'a> JSXMemberExpression<'a> {
    pub fn get_object_identifier(&self) -> &JSXIdentifier {
        match &self.object {
            JSXMemberExpressionObject::Identifier(ident) => ident,
            JSXMemberExpressionObject::MemberExpression(expr) => expr.get_object_identifier(),
        }
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXMemberExpressionObject<'a> {
    Identifier(JSXIdentifier<'a>),
    MemberExpression(Box<'a, JSXMemberExpression<'a>>),
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXExpressionContainer<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: JSXExpression<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXExpression<'a> {
    Expression(Expression<'a>),
    EmptyExpression(JSXEmptyExpression),
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXEmptyExpression {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

// 1.3 JSX Attributes

/// JSX Attributes
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXAttributeItem<'a> {
    Attribute(Box<'a, JSXAttribute<'a>>),
    SpreadAttribute(Box<'a, JSXSpreadAttribute<'a>>),
}

/// JSX Attribute
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXAttribute<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: JSXAttributeName<'a>,
    pub value: Option<JSXAttributeValue<'a>>,
}

impl<'a> JSXAttribute<'a> {
    pub fn is_key(&self) -> bool {
        matches!(&self.name, JSXAttributeName::Identifier(ident) if ident.name == "key")
    }
}

/// JSX Spread Attribute
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXSpreadAttribute<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub argument: Expression<'a>,
}

/// JSX Attribute Name
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXAttributeName<'a> {
    Identifier(JSXIdentifier<'a>),
    NamespacedName(Box<'a, JSXNamespacedName<'a>>),
}

/// JSX Attribute Value
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXAttributeValue<'a> {
    StringLiteral(StringLiteral<'a>),
    ExpressionContainer(JSXExpressionContainer<'a>),
    Element(Box<'a, JSXElement<'a>>),
    Fragment(Box<'a, JSXFragment<'a>>),
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXIdentifier<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: Atom<'a>,
}

// 1.4 JSX Children

/// JSX Child
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXChild<'a> {
    Text(JSXText<'a>),
    Element(Box<'a, JSXElement<'a>>),
    Fragment(Box<'a, JSXFragment<'a>>),
    ExpressionContainer(JSXExpressionContainer<'a>),
    Spread(JSXSpreadChild<'a>),
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXSpreadChild<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXText<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub value: Atom<'a>,
}
