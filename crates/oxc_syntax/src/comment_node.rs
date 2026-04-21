//! Comment Node ID.

use oxc_index::define_index_type;

use oxc_allocator::{Allocator, CloneIn, Dummy};
use oxc_ast_macros::ast;

define_index_type! {
    /// Comment Node ID.
    #[ast]
    #[generate_derive(CloneIn)]
    #[clone_in(default)]
    #[content_eq(skip)]
    #[estree(skip)]
    pub struct CommentNodeId = u32;
}

impl CommentNodeId {
    /// Mock comment node ID.
    ///
    /// This is used for synthetically-created AST nodes, among other things.
    pub const DUMMY: Self = CommentNodeId::new(0);
}

impl Default for CommentNodeId {
    #[inline]
    fn default() -> Self {
        Self::DUMMY
    }
}

impl<'a> Dummy<'a> for CommentNodeId {
    #[inline]
    fn dummy(_allocator: &'a Allocator) -> Self {
        Self::DUMMY
    }
}
