//! AST builder.
//!
//! AST nodes are created by builder methods defined on the AST types themselves,
//! which are passed a `&B where B: GetAstBuilder` or `&A where A: GetAllocator`:
//!
//! * `Statement::new_expression_statement(span, expr, &builder)`
//! * `Vec::new_in(&builder)`, `Ident::from_str_in(str, &builder)`
//!
//! [`AstBuilder`] provides the memory arena and [`NodeId`]s that these methods use.
//! It is not [`Copy`] or [`Clone`], and is passed by reference.
//! Its `allocator` field is private - use the `allocator` method provided by the [`GetAllocator`] trait.
//!
//! Implementing [`GetAstBuilder`] on types which hold an [`AstBuilder`] allows for a shorter syntax:
//!
//! * Long: `Vec::new_in(&self.ast)`
//! * Short: `Vec::new_in(self)`
//!
//! e.g.:
//!
//! ```
//! use oxc_allocator::{Allocator, ArenaVec, GetAllocator};
//! use oxc_ast::{ast::*, builder::{AstBuilder, GetAstBuilder}};
//! use oxc_span::SPAN;
//!
//! struct MyAstProcessor<'a> {
//!     builder: AstBuilder<'a>,
//! }
//!
//! impl<'a> GetAstBuilder<'a> for MyAstProcessor<'a> {
//!     type Builder = AstBuilder<'a>;
//!
//!     fn builder(&self) -> &AstBuilder<'a> {
//!         &self.builder
//!     }
//! }
//!
//! impl<'a> GetAllocator<'a> for MyAstProcessor<'a> {
//!     fn allocator(&self) -> &'a Allocator {
//!         self.builder.allocator()
//!     }
//! }
//!
//! impl<'a> MyAstProcessor<'a> {
//!     pub fn new(allocator: &'a Allocator) -> Self {
//!         let builder = AstBuilder::new(allocator);
//!         Self { builder }
//!     }
//!
//!     /// Create a `Vec` of 3 x `null` expressions.
//!     pub fn null_literals_array(&self) -> ArenaVec<'a, Expression<'a>> {
//!         // Can just pass `self` to all these methods, because `GetAstBuilder`
//!         // and `GetAllocator` are implemented on `MyAstProcessor`
//!         ArenaVec::from_array_in(
//!             [
//!                 Expression::new_null_literal(SPAN, self),
//!                 Expression::new_null_literal(SPAN, self),
//!                 Expression::new_null_literal(SPAN, self),
//!             ],
//!             self
//!         )
//!     }
//! }
//! ```
//!
//! These AST type builder methods are defined in `../generated/builder_methods.rs` and `custom.rs`.
//!
//! ## Migration from the old builder
//!
//! [`AstBuilder`] used to be a [`Copy`] type with its own methods for creating AST nodes
//! (e.g. `builder.statement_expression(span, expr)`) and primitives (e.g. `builder.vec()`,`builder.ident(str)`).
//! Those methods have now been removed. Use the AST type builder methods described above instead.
//!
//! [`AstBuilder`] and [`NONE`] are no longer re-exported from the crate root either -
//! import them from this module instead.
//!
//! Explanation of the motivation for this change here: <https://github.com/oxc-project/oxc/issues/23043>.

use oxc_allocator::{Allocator, ArenaBox, ArenaVec, FromIn, GetAllocator};
use oxc_syntax::node::NodeId;

mod custom;

/// Trait for types which can create AST nodes.
///
/// Implemented by [`AstBuilder`], and provides the memory arena and [`NodeId`]s used by the AST node
/// builder methods defined on AST types (e.g. `Statement::new_expression_statement`).
///
/// AST node builder methods are generic over `A: GetAstBuilder<'a>`, so they can be called with a
/// builder directly, or with a type which exposes one by implementing [`GetAstBuilder`]
/// (e.g. parser or traverse context).
///
/// Further [`AstBuild`] implementations will be added later:
///
/// * Version for parser which counts AST nodes (to accurately pre-allocate `Vec`s in `SemanticBuilder`).
/// * Version for transformer/minifier which assigns unique [`NodeId`]s to all AST nodes.
pub trait AstBuild<'a>: GetAllocator<'a> {
    /// Get [`NodeId`] to assign to an AST node.
    fn node_id(&self) -> NodeId;
}

/// Trait for types which provide access to an [`AstBuild`]er.
///
/// Implemented by the [`AstBuild`]ers themselves (returning `self`), and by types which hold one
/// (e.g. parser or traverse context). AST node builder methods are generic over `A: GetAstBuilder<'a>`,
/// so they can be called with a builder directly, or with a type which holds one.
pub trait GetAstBuilder<'a> {
    /// The [`AstBuild`]er type that this provides access to.
    type Builder: AstBuild<'a>;

    /// Get the [`AstBuild`]er.
    fn builder(&self) -> &Self::Builder;
}

/// AST builder which assigns dummy [`NodeId`]s to AST nodes.
///
/// For use where no `NodeId`s are required on AST nodes as they are built
/// e.g. parser, because `NodeId`s are assigned later when building `Semantic`.
pub struct AstBuilder<'a> {
    /// The memory allocator used to allocate AST nodes in the arena.
    allocator: &'a Allocator,
}

impl<'a> AstBuilder<'a> {
    /// Create a new [`AstBuilder`] that will allocate AST types in the provided [`Allocator`].
    #[inline]
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { allocator }
    }
}

impl<'a> GetAllocator<'a> for AstBuilder<'a> {
    /// Get the memory [`Allocator`] to allocate AST types in.
    #[inline]
    fn allocator(&self) -> &'a Allocator {
        self.allocator
    }
}

impl<'a> AstBuild<'a> for AstBuilder<'a> {
    /// Get [`NodeId`] to assign to an AST node.
    ///
    /// [`AstBuilder`] does not assign real `NodeId`s - it always returns [`NodeId::DUMMY`].
    #[inline]
    fn node_id(&self) -> NodeId {
        NodeId::DUMMY
    }
}

/// [`AstBuilder`] implements [`GetAstBuilder`] so it can be passed directly to AST build methods.
impl<'a> GetAstBuilder<'a> for AstBuilder<'a> {
    type Builder = Self;

    #[inline]
    fn builder(&self) -> &Self {
        self
    }
}

/// Type that can be used in any AST builder method call which requires either:
///
/// * `IntoIn<'a, Option<Box<'a, T>>`.
/// * `IntoIn<'a, Option<Vec<'a, T>>`.
///
/// Pass `NONE` instead of `None::<Box<'a, T>>`.
#[expect(clippy::upper_case_acronyms)]
pub struct NONE;

impl<'a, T> FromIn<'a, NONE> for Option<ArenaBox<'a, T>> {
    fn from_in(_: NONE, _: &'a Allocator) -> Self {
        None
    }
}

impl<'a, T> FromIn<'a, NONE> for Option<ArenaVec<'a, T>> {
    fn from_in(_: NONE, _: &'a Allocator) -> Self {
        None
    }
}
