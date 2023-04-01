//! [JSX](https://facebook.github.io/jsx)

use oxc_allocator::{Box, Vec};
#[cfg(feature = "serde")]
use serde::Serialize;

#[allow(clippy::wildcard_imports)]
use crate::{ast::*, Atom, Span};

// 1.2 JSX Elements

/// `JSXElement` :
///   `JSXSelfClosingElement`
///   `JSXOpeningElement` `JSXChildren_opt` `JSXClosingElement`
#[derive(Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct JSXElement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub opening_element: Box<'a, JSXOpeningElement<'a>>,
    pub closing_element: Option<Box<'a, JSXClosingElement<'a>>>,
    pub children: Vec<'a, JSXChild<'a>>,
}

/// `JSXOpeningElement` :
///   < `JSXElementName` `JSXAttributes_opt` >
#[derive(Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct JSXOpeningElement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub self_closing: bool,
    pub name: JSXElementName<'a>,
    pub attributes: Vec<'a, JSXAttributeItem<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// `JSXClosingElement` :
///     < / `JSXElementName` >
#[derive(Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXClosingElement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: JSXElementName<'a>,
}

/// `JSXFragment` :
///   < > `JSXChildren_opt` < / >
#[derive(Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct JSXFragment<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub opening_fragment: JSXOpeningFragment,
    pub closing_fragment: JSXClosingFragment,
    pub children: Vec<'a, JSXChild<'a>>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXOpeningFragment {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXClosingFragment {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

/// `JSXElementName` :
///   `JSXIdentifier`
///   `JSXNamespacedName`
///   `JSXMemberExpression`
#[derive(Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum JSXElementName<'a> {
    Identifier(JSXIdentifier),
    NamespacedName(Box<'a, JSXNamespacedName>),
    MemberExpression(Box<'a, JSXMemberExpression<'a>>),
}

/// `JSXNamespacedName` :
///   `JSXIdentifier` : `JSXIdentifier`
#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXNamespacedName {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub namespace: JSXIdentifier,
    pub property: JSXIdentifier,
}

/// `JSXMemberExpression` :
/// `JSXIdentifier` . `JSXIdentifier`
/// `JSXMemberExpression` . `JSXIdentifier`
#[derive(Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXMemberExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub object: JSXMemberExpressionObject<'a>,
    pub property: JSXIdentifier,
}

impl<'a> JSXMemberExpression<'a> {
    #[must_use]
    pub fn get_object_identifier(&self) -> &JSXIdentifier {
        match &self.object {
            JSXMemberExpressionObject::Identifier(ident) => ident,
            JSXMemberExpressionObject::MemberExpression(expr) => expr.get_object_identifier(),
        }
    }
}

#[derive(Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum JSXMemberExpressionObject<'a> {
    Identifier(JSXIdentifier),
    MemberExpression(Box<'a, JSXMemberExpression<'a>>),
}

#[derive(Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXExpressionContainer<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: JSXExpression<'a>,
}

#[derive(Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum JSXExpression<'a> {
    Expression(Expression<'a>),
    EmptyExpression(JSXEmptyExpression),
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXEmptyExpression {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

// 1.3 JSX Attributes

/// `JSXAttributes` :
///   `JSXSpreadAttribute` `JSXAttributes_opt`
///   `JSXAttribute` `JSXAttributes_opt`
#[derive(Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum JSXAttributeItem<'a> {
    Attribute(Box<'a, JSXAttribute<'a>>),
    SpreadAttribute(Box<'a, JSXSpreadAttribute<'a>>),
}

/// `JSXAttribute` :
///   `JSXAttributeName` `JSXAttributeInitializer_opt`
#[derive(Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXAttribute<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: JSXAttributeName<'a>,
    pub value: Option<JSXAttributeValue<'a>>,
}

/// `JSXSpreadAttribute` :
///   { ... `AssignmentExpression` }
#[derive(Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXSpreadAttribute<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub argument: Expression<'a>,
}

/// `JSXAttributeName` :
///   `JSXIdentifier`
///   `JSXNamespacedName`
#[derive(Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum JSXAttributeName<'a> {
    Identifier(JSXIdentifier),
    NamespacedName(Box<'a, JSXNamespacedName>),
}

/// `JSXAttributeValue` :
///   " `JSXDoubleStringCharacters_opt` "
///   ' `JSXSingleStringCharacters_opt` '
///   { `AssignmentExpression` }
///   `JSXElement`
///   `JSXFragment`
#[derive(Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum JSXAttributeValue<'a> {
    StringLiteral(StringLiteral),
    ExpressionContainer(JSXExpressionContainer<'a>),
    Element(Box<'a, JSXElement<'a>>),
    Fragment(Box<'a, JSXFragment<'a>>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXIdentifier {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: Atom,
}

// 1.4 JSX Children

/// `JSXChild` :
///   `JSXText`
///   `JSXElement`
///   `JSXFragment`
///   { `JSXChildExpression_opt` }
#[derive(Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum JSXChild<'a> {
    Text(JSXText),
    Element(Box<'a, JSXElement<'a>>),
    Fragment(Box<'a, JSXFragment<'a>>),
    ExpressionContainer(JSXExpressionContainer<'a>),
    Spread(JSXSpreadChild<'a>),
}

#[derive(Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXSpreadChild<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

/// `JSXText` ::
///   `JSXTextCharacter` `JSXTextopt`
/// `JSXTextCharacter` ::
///   `JSXStringCharacter` but not one of { or < or > or }
#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXText {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub value: Atom,
}
