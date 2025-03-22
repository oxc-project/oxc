// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/take_in.rs`

#![allow(unused_imports, unused_variables)]

use std::cell::Cell;

use oxc_allocator::{Allocator, Box, TakeIn, Vec};

use crate::number::*;
use crate::operator::*;

impl<'a> TakeIn<'a> for NumberBase {
    /// Create a dummy [`NumberBase`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Float
    }
}

impl<'a> TakeIn<'a> for BigintBase {
    /// Create a dummy [`BigintBase`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Decimal
    }
}

impl<'a> TakeIn<'a> for AssignmentOperator {
    /// Create a dummy [`AssignmentOperator`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Assign
    }
}

impl<'a> TakeIn<'a> for BinaryOperator {
    /// Create a dummy [`BinaryOperator`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Equality
    }
}

impl<'a> TakeIn<'a> for LogicalOperator {
    /// Create a dummy [`LogicalOperator`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Or
    }
}

impl<'a> TakeIn<'a> for UnaryOperator {
    /// Create a dummy [`UnaryOperator`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::UnaryPlus
    }
}

impl<'a> TakeIn<'a> for UpdateOperator {
    /// Create a dummy [`UpdateOperator`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Increment
    }
}
