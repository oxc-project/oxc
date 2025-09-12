// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`.

#![allow(unused_imports)]

use std::mem::{align_of, offset_of, size_of};

use crate::*;

#[cfg(target_pointer_width = "64")]
const _: () = {
    // Padding: 3 bytes
    assert!(size_of::<FixedSizeAllocatorMetadata>() == 16);
    assert!(align_of::<FixedSizeAllocatorMetadata>() == 8);
    assert!(offset_of!(FixedSizeAllocatorMetadata, id) == 8);
    assert!(offset_of!(FixedSizeAllocatorMetadata, alloc_ptr) == 0);
    assert!(offset_of!(FixedSizeAllocatorMetadata, is_double_owned) == 12);
};

#[cfg(target_pointer_width = "32")]
const _: () = if cfg!(target_family = "wasm") || align_of::<u64>() == 8 {
    // Padding: 3 bytes
    assert!(size_of::<FixedSizeAllocatorMetadata>() == 12);
    assert!(align_of::<FixedSizeAllocatorMetadata>() == 4);
    assert!(offset_of!(FixedSizeAllocatorMetadata, id) == 4);
    assert!(offset_of!(FixedSizeAllocatorMetadata, alloc_ptr) == 0);
    assert!(offset_of!(FixedSizeAllocatorMetadata, is_double_owned) == 8);
};

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
