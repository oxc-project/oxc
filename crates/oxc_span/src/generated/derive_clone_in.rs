// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/clone_in.rs`.

#![allow(unused_imports, unused_variables, clippy::default_trait_access, clippy::inline_always)]

use std::cell::Cell;

use oxc_allocator::{Allocator, CloneIn, CloneInSemanticIds};

use crate::source_type::*;

impl<'new_alloc> CloneIn<'new_alloc> for SourceType {
    type Cloned = SourceType;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        SourceType {
            language: CloneIn::clone_in_impl(&self.language, with_semantic_ids, allocator),
            module_kind: CloneIn::clone_in_impl(&self.module_kind, with_semantic_ids, allocator),
            variant: CloneIn::clone_in_impl(&self.variant, with_semantic_ids, allocator),
            extension: CloneIn::clone_in_impl(&self.extension, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Language {
    type Cloned = Language;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ModuleKind {
    type Cloned = ModuleKind;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for LanguageVariant {
    type Cloned = LanguageVariant;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for FileExtension {
    type Cloned = FileExtension;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}
