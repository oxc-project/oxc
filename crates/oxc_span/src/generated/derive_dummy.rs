// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/dummy.rs`.

#![allow(unused_variables, clippy::inline_always)]

use oxc_allocator::{Allocator, Dummy};

use crate::source_type::*;

impl<'a> Dummy<'a> for SourceType {
    /// Create a dummy [`SourceType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            language: Dummy::dummy(allocator),
            module_kind: Dummy::dummy(allocator),
            variant: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for Language {
    /// Create a dummy [`Language`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::JavaScript
    }
}

impl<'a> Dummy<'a> for ModuleKind {
    /// Create a dummy [`ModuleKind`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Script
    }
}

impl<'a> Dummy<'a> for LanguageVariant {
    /// Create a dummy [`LanguageVariant`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Standard
    }
}
