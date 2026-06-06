//! Types which are in common between old `AstBuilder` and new one.
//!
//! TODO: Merge this into main AST builder file once we delete the old AST builder.

use oxc_allocator::{Allocator, Box, FromIn};

/// Type that can be used in any AST builder method call which requires an `IntoIn<'a, Anything<'a>>`.
/// Pass `NONE` instead of `None::<Anything<'a>>`.
#[expect(clippy::upper_case_acronyms)]
pub struct NONE;

impl<'a, T> FromIn<'a, NONE> for Option<Box<'a, T>> {
    fn from_in(_: NONE, _: &'a Allocator) -> Self {
        None
    }
}
