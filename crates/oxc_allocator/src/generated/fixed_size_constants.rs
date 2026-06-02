// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer.rs`.

//! Constants for raw transfer / fixed-sized allocators.
//!
//! See `crates/oxc_allocator/src/pool/fixed_size.rs` for a diagram showing
//! how the constituent parts of the arena fit together.
//!
//! This file must be feature-gated so it is not loaded on 32-bit platforms.
//! It will cause a compilation error on 32-bit platforms, as some values are too large for 32-bit `usize`.

#![expect(clippy::unreadable_literal)]
#![allow(dead_code)]

/// Total size of the allocator block (including metadata and allocator `ChunkFooter`).
pub const BLOCK_SIZE: usize = 2147483632;

/// Required alignment of the allocator block (4 GiB).
pub const BLOCK_ALIGN: usize = 4294967296;

/// Total size of the transfer buffer used on JS side, in bytes
/// (`BLOCK_SIZE` minus `FixedSizeAllocatorMetadata` and `ChunkFooter`).
pub const BUFFER_SIZE: usize = 2147483576;

/// Size of the active data region in bytes - the region where source text and AST live
/// (`BUFFER_SIZE` minus `RawTransferMetadata`).
pub const ACTIVE_SIZE: usize = 2147483560;

/// Size of `RawTransferMetadata` in bytes.
pub const RAW_METADATA_SIZE: usize = 16;

/// Alignment of `RawTransferMetadata`.
pub const RAW_METADATA_ALIGN: usize = 4;

/// Size of `ChunkFooter` struct in bytes.
pub const CHUNK_FOOTER_SIZE: usize = 48;

/// Minimum alignment requirement for `Arena`'s cursor pointer (`ARENA::MIN_ALIGN`).
pub const CURSOR_MIN_ALIGN: usize = 1;
