//! AST builder.
//!
//! This is undergoing change at present.
//!
//! Explanation of the motivation for this change here: <https://github.com/oxc-project/oxc/issues/23043>.
//!
//! ## Old builder
//!
//! [`AstBuilder`] used to be a [`Copy`] type, which had own methods for:
//!
//! * Creating AST nodes e.g. `builder.statement_expression(span, expr)`
//! * Creating primitives e.g. `builder.vec()`, `builder.ident(str)`
//!
//! These methods are defined in `../generated/ast_builder.rs` and `methods.rs`.
//!
//! ## New builder
//!
//! We have now added methods to AST types themselves which perform the same role,
//! and are passed an `&B where B: GetAstBuilder` or `&A where A: GetAllocator`:
//!
//! * `Statement::new_expression_statement(span, expr, &builder)`
//! * `Vec::new_in(&builder)`, `Ident::from_str_in(str, &builder)`
//!
//! `AstBuilder` is no longer `Copy` or `Clone`, and is passed by reference.
//! Its `allocator` field is no longer public. Use `allocator` method provided by `GetAllocator` trait.
//!
//! Implementing `GetAstBuilder` on types which hold an `AstBuilder` allows for a shorter syntax:
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
//! ## Migration
//!
//! To minimize immediate breaking changes for downstream consumers, and allow them to migrate incrementally,
//! at present `AstBuilder` still has its own methods, but can also be passed to the AST type builder methods.
//!
//! Once a project has migrated, they should enable the `disable_old_builder` Cargo feature,
//! which will remove the old-style own methods on `AstBuilder`.
//!
//! After a few weeks, we will remove `AstBuilder`'s own methods entirely.
//!
//! ## Oxc
//!
//! All Oxc crates have been migrated to the new usage.
//!
//! `disable_old_builder` Cargo feature is enabled in all Oxc crates which utilize `AstBuilder`.
//! Where those crates expose the `AstBuilder` they use to user code, the feature is only enabled in tests.

use std::cell::Cell;

use oxc_allocator::{Allocator, ArenaBox, FromIn, GetAllocator};
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
    /// Record creation of an AST node.
    #[inline]
    fn record_node(&self) {}

    /// Get [`NodeId`] to assign to an AST node.
    fn node_id(&self) -> NodeId;

    /// Record creation of an AST node which may create a lexical scope.
    #[inline]
    fn record_scope(&self) {}

    /// Record creation of an AST node which may create a semantic symbol.
    #[inline]
    fn record_symbol(&self) {}

    /// Record creation of an AST node which may create an identifier reference.
    #[inline]
    fn record_reference(&self) {}
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

/// Approximate count of AST nodes and semantic data created by an [`AstBuilder`].
///
/// These counts are intended for pre-allocation and must be treated as upper bounds.
/// Some parser paths may overcount because speculative parsing and reparsing can create
/// abandoned AST nodes, and exact scope/symbol/reference counts require semantic context.
#[derive(Debug, Default, Clone, Copy)]
pub struct AstBuilderStats {
    /// Count of AST nodes created.
    pub nodes: u32,
    /// Approximate count of lexical scopes created.
    pub scopes: u32,
    /// Approximate count of semantic symbols created.
    pub symbols: u32,
    /// Approximate count of identifier references created.
    pub references: u32,
}

/// AST builder which assigns dummy [`NodeId`]s to AST nodes.
///
/// For use where no `NodeId`s are required on AST nodes as they are built
/// e.g. parser, because `NodeId`s are assigned later when building `Semantic`.
pub struct AstBuilder<'a> {
    /// The memory allocator used to allocate AST nodes in the arena.
    #[cfg(feature = "disable_old_builder")]
    allocator: &'a Allocator,

    /// The memory allocator used to allocate AST nodes in the arena.
    #[cfg(not(feature = "disable_old_builder"))]
    pub allocator: &'a Allocator,

    /// Count of AST nodes created.
    nodes: Cell<u32>,

    /// Approximate count of lexical scopes created.
    scopes: Cell<u32>,

    /// Approximate count of semantic symbols created.
    symbols: Cell<u32>,

    /// Approximate count of identifier references created.
    references: Cell<u32>,
}

impl<'a> AstBuilder<'a> {
    /// Create a new [`AstBuilder`] that will allocate AST types in the provided [`Allocator`].
    #[inline]
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            nodes: Cell::new(0),
            scopes: Cell::new(0),
            symbols: Cell::new(0),
            references: Cell::new(0),
        }
    }

    /// Get approximate statistics for AST and semantic data created by this builder.
    #[inline]
    pub fn stats(&self) -> AstBuilderStats {
        AstBuilderStats {
            nodes: self.nodes.get(),
            scopes: self.scopes.get(),
            symbols: self.symbols.get(),
            references: self.references.get(),
        }
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
    #[inline]
    fn record_node(&self) {
        self.nodes.set(self.nodes.get() + 1);
    }

    /// Get [`NodeId`] to assign to an AST node.
    ///
    /// [`AstBuilder`] does not assign real `NodeId`s - it always returns [`NodeId::DUMMY`].
    #[inline]
    fn node_id(&self) -> NodeId {
        self.record_node();
        NodeId::DUMMY
    }

    #[inline]
    fn record_scope(&self) {
        self.scopes.set(self.scopes.get() + 1);
    }

    #[inline]
    fn record_symbol(&self) {
        self.symbols.set(self.symbols.get() + 1);
    }

    #[inline]
    fn record_reference(&self) {
        self.references.set(self.references.get() + 1);
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

/// Type that can be used in any AST builder method call which requires an `IntoIn<'a, Option<Anything<'a>>>`.
/// Pass `NONE` instead of `None::<Anything<'a>>`.
#[expect(clippy::upper_case_acronyms)]
pub struct NONE;

impl<'a, T> FromIn<'a, NONE> for Option<ArenaBox<'a, T>> {
    fn from_in(_: NONE, _: &'a Allocator) -> Self {
        None
    }
}
