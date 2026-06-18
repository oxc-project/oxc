use oxc_allocator::{Allocator, GetAllocator};
use oxc_syntax::node::NodeId;

// Re-export as part of `builder` module
pub use crate::ast_builder_common::NONE;

mod custom;

/// Trait for types which can create AST nodes.
///
/// Implemented by [`AstBuilder`], and provides the memory arena and [`NodeId`]s used by the AST node
/// builder methods defined on AST types (e.g. `Statement::expression_statement`).
///
/// AST node builder methods are generic over `A: GetAstBuilder<'a>`, so they can be called with a
/// builder directly, or with a type which exposes one by implementing [`GetAstBuilder`]
/// (e.g. parser or traverse context).
///
/// Further [`AstBuild`] implementations will be added later:
///
/// * Version for parser which counts AST nodes (to accurately pre-allocate `Vec`s in `SemanticBuilder`).
/// * Version for transformer/minifier which assigns unique [`NodeId`]s all AST nodes.
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
    pub allocator: &'a Allocator,
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
