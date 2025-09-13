// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`.

#![allow(unused_imports)]

use std::mem::{align_of, offset_of, size_of};

use crate::*;

#[cfg(target_pointer_width = "64")]
const _: () = {
    // Padding: 0 bytes
    assert!(size_of::<Span>() == 8);
    assert!(align_of::<Span>() == 8);
    assert!(offset_of!(Span, start) == 0);
    assert!(offset_of!(Span, end) == 4);

    // Padding: 0 bytes
    assert!(size_of::<SourceType>() == 3);
    assert!(align_of::<SourceType>() == 1);
    assert!(offset_of!(SourceType, language) == 0);
    assert!(offset_of!(SourceType, module_kind) == 1);
    assert!(offset_of!(SourceType, variant) == 2);

    assert!(size_of::<Language>() == 1);
    assert!(align_of::<Language>() == 1);

    assert!(size_of::<ModuleKind>() == 1);
    assert!(align_of::<ModuleKind>() == 1);

    assert!(size_of::<LanguageVariant>() == 1);
    assert!(align_of::<LanguageVariant>() == 1);
};

#[cfg(target_pointer_width = "32")]
const _: () = if cfg!(target_family = "wasm") || align_of::<u64>() == 8 {
    // Padding: 0 bytes
    assert!(size_of::<Span>() == 8);
    assert!(align_of::<Span>() == 4);
    assert!(offset_of!(Span, start) == 0);
    assert!(offset_of!(Span, end) == 4);

    // Padding: 0 bytes
    assert!(size_of::<SourceType>() == 3);
    assert!(align_of::<SourceType>() == 1);
    assert!(offset_of!(SourceType, language) == 0);
    assert!(offset_of!(SourceType, module_kind) == 1);
    assert!(offset_of!(SourceType, variant) == 2);

    assert!(size_of::<Language>() == 1);
    assert!(align_of::<Language>() == 1);

    assert!(size_of::<ModuleKind>() == 1);
    assert!(align_of::<ModuleKind>() == 1);

    assert!(size_of::<LanguageVariant>() == 1);
    assert!(align_of::<LanguageVariant>() == 1);
};

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
