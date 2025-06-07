//! Comment Node ID.

use oxc_index::Idx;
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer};

use oxc_allocator::{Allocator, CloneIn, Dummy};
use oxc_ast_macros::ast;

/// Comment Node ID.
#[ast]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[generate_derive(CloneIn)]
#[clone_in(default)]
#[content_eq(skip)]
#[estree(skip)]
pub struct CommentNodeId(u32);

impl CommentNodeId {
    /// Mock comment node ID.
    ///
    /// This is used for synthetically-created AST nodes, among other things.
    pub const DUMMY: Self = CommentNodeId::new(0);

    /// Create `CommentNodeId` from `u32`.
    pub const fn new(idx: u32) -> Self {
        Self(idx)
    }
}

impl Idx for CommentNodeId {
    fn from_usize(idx: usize) -> Self {
        Self(u32::try_from(idx).expect("`idx` is greater than `u32::MAX`"))
    }

    fn index(self) -> usize {
        self.0 as usize
    }
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

#[cfg(feature = "serialize")]
impl Serialize for CommentNodeId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.0)
    }
}
