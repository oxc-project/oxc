//! [JSX](https://facebook.github.io/jsx)

use oxc_allocator::{Box, Vec};
use oxc_macros::SerAttrs;
use oxc_span::{Atom, Span};
#[cfg(feature = "serde")]
use serde::Serialize;
#[cfg(feature = "wasm")]
use tsify::Tsify;

use super::{js::*, literal::*, ts::*};

// 1.2 JSX Elements

/// JSX Element
#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct JSXElement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub opening_element: Box<'a, JSXOpeningElement<'a>>,
    pub closing_element: Option<Box<'a, JSXClosingElement<'a>>>,
    pub children: Vec<'a, JSXChild<'a>>,
}

/// JSX Opening Element
#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct JSXOpeningElement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub self_closing: bool,
    pub name: JSXElementName<'a>,
    pub attributes: Vec<'a, JSXAttributeItem<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// JSX Closing Element
#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type")]
pub struct JSXClosingElement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub name: JSXElementName<'a>,
}

/// JSX Fragment
#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct JSXFragment<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub opening_fragment: JSXOpeningFragment,
    pub closing_fragment: JSXClosingFragment,
    pub children: Vec<'a, JSXChild<'a>>,
}

#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type")]
pub struct JSXOpeningFragment {
    #[serde(flatten)]
    pub span: Span,
}

#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type")]
pub struct JSXClosingFragment {
    #[serde(flatten)]
    pub span: Span,
}

/// JSX Element Name
#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(untagged)]
pub enum JSXElementName<'a> {
    /// `<Apple />`
    Identifier(JSXIdentifier<'a>),
    /// `<Apple:Orange />`
    NamespacedName(Box<'a, JSXNamespacedName<'a>>),
    /// `<Apple.Orange />`
    MemberExpression(Box<'a, JSXMemberExpression<'a>>),
}

/// JSX Namespaced Name
#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type")]
pub struct JSXNamespacedName<'a> {
    #[serde(flatten)]
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
#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type")]
pub struct JSXMemberExpression<'a> {
    #[serde(flatten)]
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

#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(untagged)]
pub enum JSXMemberExpressionObject<'a> {
    Identifier(JSXIdentifier<'a>),
    MemberExpression(Box<'a, JSXMemberExpression<'a>>),
}

#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type")]
pub struct JSXExpressionContainer<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: JSXExpression<'a>,
}

#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(untagged)]
pub enum JSXExpression<'a> {
    Expression(Expression<'a>),
    EmptyExpression(JSXEmptyExpression),
}

#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type")]
pub struct JSXEmptyExpression {
    #[serde(flatten)]
    pub span: Span,
}

// 1.3 JSX Attributes

/// JSX Attributes
#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(untagged)]
pub enum JSXAttributeItem<'a> {
    Attribute(Box<'a, JSXAttribute<'a>>),
    SpreadAttribute(Box<'a, JSXSpreadAttribute<'a>>),
}

/// JSX Attribute
#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type")]
pub struct JSXAttribute<'a> {
    #[serde(flatten)]
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
#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type")]
pub struct JSXSpreadAttribute<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub argument: Expression<'a>,
}

/// JSX Attribute Name
#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(untagged)]
pub enum JSXAttributeName<'a> {
    Identifier(JSXIdentifier<'a>),
    NamespacedName(Box<'a, JSXNamespacedName<'a>>),
}

/// JSX Attribute Value
#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(untagged)]
pub enum JSXAttributeValue<'a> {
    StringLiteral(StringLiteral<'a>),
    ExpressionContainer(JSXExpressionContainer<'a>),
    Element(Box<'a, JSXElement<'a>>),
    Fragment(Box<'a, JSXFragment<'a>>),
}

#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type")]
pub struct JSXIdentifier<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub name: Atom<'a>,
}

// 1.4 JSX Children

/// JSX Child
#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(untagged)]
pub enum JSXChild<'a> {
    Text(JSXText<'a>),
    Element(Box<'a, JSXElement<'a>>),
    Fragment(Box<'a, JSXFragment<'a>>),
    ExpressionContainer(JSXExpressionContainer<'a>),
    Spread(JSXSpreadChild<'a>),
}

#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type")]
pub struct JSXSpreadChild<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: Expression<'a>,
}

#[derive(Debug, Hash, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type")]
pub struct JSXText<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub value: Atom<'a>,
}
