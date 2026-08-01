//! Static ETS AST node definitions.
//!
//! These nodes model syntax that belongs to the `ets-static` frontend and has no
//! JavaScript or TypeScript ESTree equivalent. They are deliberately separate
//! from ArkUI nodes: a plain `.ets` source continues to use the ArkUI grammar,
//! while these nodes can only be produced by an explicit static ETS source type.

use std::cell::Cell;

use oxc_allocator::{Box, CloneIn, Dummy, ReplaceWith, TakeIn, UnstableAddress, Vec};
use oxc_ast_macros::ast;
use oxc_estree::ESTree;
use oxc_span::{ContentEq, GetSpan, GetSpanMut, Span};
use oxc_syntax::node::NodeId;

use super::{js::*, ts::*};

/// Static ETS package header.
///
/// ```ets
/// package com.example.application;
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, ReplaceWith, TakeIn)]
#[generate_derive(ContentEq, ESTree, GetSpan, GetSpanMut, UnstableAddress)]
pub struct ETSPackageDeclaration<'a> {
    pub node_id: Cell<NodeId>,
    pub span: Span,
    pub name: Vec<'a, IdentifierName<'a>>,
}

/// Static ETS `instanceof` expression whose right operand is a type.
///
/// Unlike JavaScript, ETS parses the right-hand side through its type grammar,
/// including forms such as `keyof A` and parenthesized types.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, ReplaceWith, TakeIn)]
#[generate_derive(ContentEq, ESTree, GetSpan, GetSpanMut, UnstableAddress)]
pub struct ETSInstanceOfExpression<'a> {
    pub node_id: Cell<NodeId>,
    pub span: Span,
    pub left: Expression<'a>,
    pub right: TSType<'a>,
}

/// Static ETS class construction. The callee is parsed as a type, not as a
/// JavaScript expression.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, ReplaceWith, TakeIn)]
#[generate_derive(ContentEq, ESTree, GetSpan, GetSpanMut, UnstableAddress)]
pub struct ETSNewClassInstanceExpression<'a> {
    pub node_id: Cell<NodeId>,
    pub span: Span,
    pub type_annotation: TSType<'a>,
    pub arguments: Vec<'a, Argument<'a>>,
    pub has_arguments: bool,
}

/// Static ETS single-dimensional array construction.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, ReplaceWith, TakeIn)]
#[generate_derive(ContentEq, ESTree, GetSpan, GetSpanMut, UnstableAddress)]
pub struct ETSNewArrayInstanceExpression<'a> {
    pub node_id: Cell<NodeId>,
    pub span: Span,
    pub type_annotation: TSType<'a>,
    pub dimension: Expression<'a>,
    pub initializer: Option<Expression<'a>>,
}

/// Static ETS multi-dimensional array construction. es2panda keeps this node
/// even though the current language rules diagnose it as unsupported.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, ReplaceWith, TakeIn)]
#[generate_derive(ContentEq, ESTree, GetSpan, GetSpanMut, UnstableAddress)]
pub struct ETSNewMultiDimArrayInstanceExpression<'a> {
    pub node_id: Cell<NodeId>,
    pub span: Span,
    pub type_annotation: TSType<'a>,
    pub dimensions: Vec<'a, Expression<'a>>,
}

/// A static ETS call followed by a trailing lambda block.
///
/// es2panda stores this data directly on `CallExpression`. Oxc keeps it in a
/// zero-cost expression variant so the common JavaScript call node does not
/// grow for a feature that only exists in explicit `ets-static` mode.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, ReplaceWith, TakeIn)]
#[generate_derive(ContentEq, ESTree, GetSpan, GetSpanMut, UnstableAddress)]
pub struct ETSTrailingBlockExpression<'a> {
    pub node_id: Cell<NodeId>,
    pub span: Span,
    pub call: Box<'a, CallExpression<'a>>,
    pub block: Box<'a, BlockStatement<'a>>,
    pub is_trailing_call: bool,
    pub is_block_on_new_line: bool,
    pub has_trailing_comma: bool,
}

/// The context in which an ETS overload declaration occurs.
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, Dummy, ContentEq, ESTree)]
pub enum ETSOverloadDeclarationKind {
    Function = 0,
    ClassMethod = 1,
    InterfaceMethod = 2,
    StructMethod = 3,
}

/// Static ETS managed overload declaration.
///
/// ```ets
/// overload parse { parseInt, parseString }
/// overload constructor { fromInt, fromString }
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, ReplaceWith, TakeIn)]
#[generate_derive(ContentEq, ESTree, GetSpan, GetSpanMut, UnstableAddress)]
pub struct ETSOverloadDeclaration<'a> {
    pub node_id: Cell<NodeId>,
    pub span: Span,
    /// Annotations preceding the declaration.
    pub decorators: Vec<'a, Decorator<'a>>,
    /// Managed overload name, or the `constructor` identifier.
    pub key: PropertyKey<'a>,
    /// Functions or named constructors participating in the overload.
    pub overloads: Vec<'a, Expression<'a>>,
    pub kind: ETSOverloadDeclarationKind,
    pub accessibility: Option<TSAccessibility>,
    pub r#static: bool,
    pub r#abstract: bool,
    pub r#final: bool,
    pub native: bool,
    pub declare: bool,
}
