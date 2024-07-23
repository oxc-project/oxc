//! [JSX](https://facebook.github.io/jsx)

// NB: `#[span]`, `#[scope(...)]` and `#[visit(...)]` do NOT do anything to the code.
// They are purely markers for codegen used in
// `tasks/ast_codegen` and `crates/oxc_traverse/scripts`. See docs in those crates.

// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use oxc_allocator::{Box, Vec};
use oxc_ast_macros::ast;
use oxc_span::{Atom, Span};
#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

use super::{inherit_variants, js::*, literal::*, ts::*};

// 1.2 JSX Elements

/// JSX Element
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct JSXElement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub opening_element: Box<'a, JSXOpeningElement<'a>>,
    pub closing_element: Option<Box<'a, JSXClosingElement<'a>>>,
    pub children: Vec<'a, JSXChild<'a>>,
}

/// JSX Opening Element
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
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
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct JSXClosingElement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub name: JSXElementName<'a>,
}

/// JSX Fragment
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct JSXFragment<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub opening_fragment: JSXOpeningFragment,
    pub closing_fragment: JSXClosingFragment,
    pub children: Vec<'a, JSXChild<'a>>,
}

#[ast]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct JSXOpeningFragment {
    #[serde(flatten)]
    pub span: Span,
}

#[ast]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct JSXClosingFragment {
    #[serde(flatten)]
    pub span: Span,
}

/// JSX Element Name
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum JSXElementName<'a> {
    /// `<Apple />`
    Identifier(Box<'a, JSXIdentifier<'a>>),
    /// `<Apple:Orange />`
    NamespacedName(Box<'a, JSXNamespacedName<'a>>),
    /// `<Apple.Orange />`
    MemberExpression(Box<'a, JSXMemberExpression<'a>>),
}

/// JSX Namespaced Name
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct JSXNamespacedName<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub namespace: JSXIdentifier<'a>,
    pub property: JSXIdentifier<'a>,
}

/// JSX Member Expression
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct JSXMemberExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub object: JSXMemberExpressionObject<'a>,
    pub property: JSXIdentifier<'a>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum JSXMemberExpressionObject<'a> {
    Identifier(Box<'a, JSXIdentifier<'a>>),
    MemberExpression(Box<'a, JSXMemberExpression<'a>>),
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct JSXExpressionContainer<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: JSXExpression<'a>,
}

inherit_variants! {
/// JSX Expression
///
/// Inherits variants from [`Expression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum JSXExpression<'a> {
    EmptyExpression(JSXEmptyExpression) = 64,
    // `Expression` variants added here by `inherit_variants!` macro
    @inherit Expression
}
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct JSXEmptyExpression {
    #[serde(flatten)]
    pub span: Span,
}

// 1.3 JSX Attributes

/// JSX Attributes
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum JSXAttributeItem<'a> {
    Attribute(Box<'a, JSXAttribute<'a>>),
    SpreadAttribute(Box<'a, JSXSpreadAttribute<'a>>),
}

/// JSX Attribute
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct JSXAttribute<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub name: JSXAttributeName<'a>,
    pub value: Option<JSXAttributeValue<'a>>,
}

/// JSX Spread Attribute
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct JSXSpreadAttribute<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub argument: Expression<'a>,
}

/// JSX Attribute Name
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum JSXAttributeName<'a> {
    Identifier(Box<'a, JSXIdentifier<'a>>),
    NamespacedName(Box<'a, JSXNamespacedName<'a>>),
}

/// JSX Attribute Value
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum JSXAttributeValue<'a> {
    StringLiteral(Box<'a, StringLiteral<'a>>),
    ExpressionContainer(Box<'a, JSXExpressionContainer<'a>>),
    Element(Box<'a, JSXElement<'a>>),
    Fragment(Box<'a, JSXFragment<'a>>),
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct JSXIdentifier<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub name: Atom<'a>,
}

// 1.4 JSX Children

/// JSX Child
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum JSXChild<'a> {
    Text(Box<'a, JSXText<'a>>),
    Element(Box<'a, JSXElement<'a>>),
    Fragment(Box<'a, JSXFragment<'a>>),
    ExpressionContainer(Box<'a, JSXExpressionContainer<'a>>),
    Spread(Box<'a, JSXSpreadChild<'a>>),
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct JSXSpreadChild<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: Expression<'a>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct JSXText<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub value: Atom<'a>,
}
