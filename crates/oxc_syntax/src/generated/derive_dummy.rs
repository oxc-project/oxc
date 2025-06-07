// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/dummy.rs`.

#![allow(unused_variables, clippy::inline_always)]

use oxc_allocator::{Allocator, Dummy};

use crate::number::*;
use crate::operator::*;

impl<'a> Dummy<'a> for NumberBase {
    /// Create a dummy [`NumberBase`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Float
    }
}

impl<'a> Dummy<'a> for BigintBase {
    /// Create a dummy [`BigintBase`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Decimal
    }
}

impl<'a> Dummy<'a> for AssignmentOperator {
    /// Create a dummy [`AssignmentOperator`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Assign
    }
}

impl<'a> Dummy<'a> for BinaryOperator {
    /// Create a dummy [`BinaryOperator`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Equality
    }
}

impl<'a> Dummy<'a> for LogicalOperator {
    /// Create a dummy [`LogicalOperator`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Or
    }
}

impl<'a> Dummy<'a> for UnaryOperator {
    /// Create a dummy [`UnaryOperator`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::UnaryPlus
    }
}

impl<'a> Dummy<'a> for UpdateOperator {
    /// Create a dummy [`UpdateOperator`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Increment
    }
}
