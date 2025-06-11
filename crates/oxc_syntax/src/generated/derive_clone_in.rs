// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/clone_in.rs`.

#![allow(unused_variables, clippy::default_trait_access, clippy::inline_always)]

use oxc_allocator::{Allocator, CloneIn};

use crate::comment_node::*;
use crate::number::*;
use crate::operator::*;

impl<'new_alloc> CloneIn<'new_alloc> for CommentNodeId {
    type Cloned = CommentNodeId;

    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Default::default()
    }

    fn clone_in_with_semantic_ids(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Default::default()
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for NumberBase {
    type Cloned = NumberBase;

    #[inline(always)]
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        *self
    }

    #[inline(always)]
    fn clone_in_with_semantic_ids(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BigintBase {
    type Cloned = BigintBase;

    #[inline(always)]
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        *self
    }

    #[inline(always)]
    fn clone_in_with_semantic_ids(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentOperator {
    type Cloned = AssignmentOperator;

    #[inline(always)]
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        *self
    }

    #[inline(always)]
    fn clone_in_with_semantic_ids(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BinaryOperator {
    type Cloned = BinaryOperator;

    #[inline(always)]
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        *self
    }

    #[inline(always)]
    fn clone_in_with_semantic_ids(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for LogicalOperator {
    type Cloned = LogicalOperator;

    #[inline(always)]
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        *self
    }

    #[inline(always)]
    fn clone_in_with_semantic_ids(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for UnaryOperator {
    type Cloned = UnaryOperator;

    #[inline(always)]
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        *self
    }

    #[inline(always)]
    fn clone_in_with_semantic_ids(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for UpdateOperator {
    type Cloned = UpdateOperator;

    #[inline(always)]
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        *self
    }

    #[inline(always)]
    fn clone_in_with_semantic_ids(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        *self
    }
}
