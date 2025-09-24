// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`.

#![allow(unused_imports)]

use std::mem::{align_of, offset_of, size_of};

use crate::*;

#[cfg(target_pointer_width = "64")]
const _: () = {
    // Padding: 3 bytes
    assert!(size_of::<RawTransferMetadata2>() == 16);
    assert!(align_of::<RawTransferMetadata2>() == 8);
    assert!(offset_of!(RawTransferMetadata2, data_offset) == 8);
    assert!(offset_of!(RawTransferMetadata2, is_ts) == 12);
    assert!(offset_of!(RawTransferMetadata2, _padding) == 0);
};

#[cfg(target_pointer_width = "32")]
const _: () = if cfg!(target_family = "wasm") || align_of::<u64>() == 8 {
    // Padding: 3 bytes
    assert!(size_of::<RawTransferMetadata2>() == 16);
    assert!(align_of::<RawTransferMetadata2>() == 8);
    assert!(offset_of!(RawTransferMetadata2, data_offset) == 8);
    assert!(offset_of!(RawTransferMetadata2, is_ts) == 12);
    assert!(offset_of!(RawTransferMetadata2, _padding) == 0);
};

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
