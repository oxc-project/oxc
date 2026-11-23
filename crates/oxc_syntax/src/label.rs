//! Label IDs.

use oxc_allocator::{Allocator, CloneIn, CloneInSemanticIds};
use oxc_index::define_nonmax_u32_index_type;

use oxc_ast_macros::ast;

use crate::semantic_id::SemanticId;

define_nonmax_u32_index_type! {
    #[ast]
    #[builder(default)]
    #[clone_in(semantic_id)]
    #[content_eq(skip)]
    #[estree(skip)]
    pub struct LabelId;
}

impl<'alloc> CloneIn<'alloc> for LabelId {
    type Cloned = Self;

    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this method only delegates
    fn clone_in_impl(&self, with_semantic_ids: CloneInSemanticIds, _: &'alloc Allocator) -> Self {
        self.clone_id(with_semantic_ids)
    }
}

impl SemanticId for LabelId {}
