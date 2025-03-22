// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/take_in.rs`

#![allow(unused_imports, unused_variables)]

use std::cell::Cell;

use oxc_allocator::{Allocator, Box, TakeIn, Vec};

use crate::source_type::*;

impl<'a> TakeIn<'a> for SourceType {
    /// Create a dummy [`SourceType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            language: TakeIn::dummy_in(allocator),
            module_kind: TakeIn::dummy_in(allocator),
            variant: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for Language {
    /// Create a dummy [`Language`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::JavaScript
    }
}

impl<'a> TakeIn<'a> for ModuleKind {
    /// Create a dummy [`ModuleKind`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Script
    }
}

impl<'a> TakeIn<'a> for LanguageVariant {
    /// Create a dummy [`LanguageVariant`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Standard
    }
}
