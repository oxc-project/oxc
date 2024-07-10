//! [JSX](https://facebook.github.io/jsx)

// NB: `#[span]`, `#[scope(...)]`, `#[visit(...)]`, `#[visit_as(...)]` and `#[visit_args(...)]` do
// not do anything to the code, They are purely markers for codegen used in
// `tasts/ast_codegen` and `crates/oxc_traverse/scripts`. See docs in that crate.

// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use oxc_allocator::{Box, Vec};
use oxc_ast_macros::visited_node;
use oxc_span::{Atom, Span};
#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

use super::{inherit_variants, js::*, literal::*, ts::*};

// 1.2 JSX Elements

/// JSX Element
#[visited_node]
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
#[visited_node]
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
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXClosingElement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: JSXElementName<'a>,
}

/// JSX Fragment
#[visited_node]
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

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXOpeningFragment {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXClosingFragment {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

/// JSX Element Name
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXElementName<'a> {
    /// `<Apple />`
    Identifier(Box<'a, JSXIdentifier<'a>>),
    /// `<Apple:Orange />`
    NamespacedName(Box<'a, JSXNamespacedName<'a>>),
    /// `<Apple.Orange />`
    MemberExpression(Box<'a, JSXMemberExpression<'a>>),
}

/// JSX Namespaced Name
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXNamespacedName<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub namespace: JSXIdentifier<'a>,
    pub property: JSXIdentifier<'a>,
}

/// JSX Member Expression
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXMemberExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub object: JSXMemberExpressionObject<'a>,
    pub property: JSXIdentifier<'a>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXMemberExpressionObject<'a> {
    Identifier(Box<'a, JSXIdentifier<'a>>),
    MemberExpression(Box<'a, JSXMemberExpression<'a>>),
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXExpressionContainer<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: JSXExpression<'a>,
}

inherit_variants! {
/// JSX Expression
///
/// Inherits variants from [`Expression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[visited_node]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXExpression<'a> {
    EmptyExpression(JSXEmptyExpression) = 64,
    // `Expression` variants added here by `inherit_variants!` macro
    @inherit Expression
}
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXEmptyExpression {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

// 1.3 JSX Attributes

/// JSX Attributes
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXAttributeItem<'a> {
    Attribute(Box<'a, JSXAttribute<'a>>),
    SpreadAttribute(Box<'a, JSXSpreadAttribute<'a>>),
}

/// JSX Attribute
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXAttribute<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: JSXAttributeName<'a>,
    pub value: Option<JSXAttributeValue<'a>>,
}

/// JSX Spread Attribute
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXSpreadAttribute<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub argument: Expression<'a>,
}

/// JSX Attribute Name
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXAttributeName<'a> {
    Identifier(Box<'a, JSXIdentifier<'a>>),
    NamespacedName(Box<'a, JSXNamespacedName<'a>>),
}

/// JSX Attribute Value
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXAttributeValue<'a> {
    StringLiteral(Box<'a, StringLiteral<'a>>),
    ExpressionContainer(Box<'a, JSXExpressionContainer<'a>>),
    Element(Box<'a, JSXElement<'a>>),
    Fragment(Box<'a, JSXFragment<'a>>),
}

#[visited_node]
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
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXChild<'a> {
    Text(Box<'a, JSXText<'a>>),
    Element(Box<'a, JSXElement<'a>>),
    Fragment(Box<'a, JSXFragment<'a>>),
    ExpressionContainer(Box<'a, JSXExpressionContainer<'a>>),
    Spread(Box<'a, JSXSpreadChild<'a>>),
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXSpreadChild<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXText<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub value: Atom<'a>,
}
