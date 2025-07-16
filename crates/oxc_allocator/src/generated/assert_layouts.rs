// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`.

#![allow(unused_imports)]

use std::mem::{align_of, offset_of, size_of};

use crate::*;

#[cfg(target_pointer_width = "64")]
const _: () = {
    // Padding: 6 bytes
    assert!(size_of::<RawTransferMetadata2>() == 32);
    assert!(align_of::<RawTransferMetadata2>() == 8);
    assert!(offset_of!(RawTransferMetadata2, data_offset) == 16);
    assert!(offset_of!(RawTransferMetadata2, is_ts) == 24);
    assert!(offset_of!(RawTransferMetadata2, id) == 20);
    assert!(offset_of!(RawTransferMetadata2, can_be_freed) == 25);
    assert!(offset_of!(RawTransferMetadata2, alloc_ptr) == 8);
    assert!(offset_of!(RawTransferMetadata2, _padding) == 0);
};

#[cfg(target_pointer_width = "32")]
const _: () = {
    // Padding: 2 bytes
    assert!(size_of::<RawTransferMetadata2>() == 24);
    assert!(align_of::<RawTransferMetadata2>() == 8);
    assert!(offset_of!(RawTransferMetadata2, data_offset) == 12);
    assert!(offset_of!(RawTransferMetadata2, is_ts) == 20);
    assert!(offset_of!(RawTransferMetadata2, id) == 16);
    assert!(offset_of!(RawTransferMetadata2, can_be_freed) == 21);
    assert!(offset_of!(RawTransferMetadata2, alloc_ptr) == 8);
    assert!(offset_of!(RawTransferMetadata2, _padding) == 0);
};

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
