//! [`Arena::new_fixed_size`]
//!
//! Construct an [`Arena`] of fixed size, aligned on a high boundary, backed by an allocation made via
//! [`System`] allocator. Used by raw transfer.
//!
//! Allocating via [`System`] bypasses any registered alternative global allocator (e.g. Mimalloc in linter).
//! Mimalloc complains that it cannot serve allocations with high alignment, and presumably it's pointless
//! to try to obtain such large allocations from a thread-local heap, so better to go direct to the system
//! allocator anyway.
//!
//! The implementation differs between platforms, due to differences in what platforms' system allocators support:
//!
//! * Mac OS: System allocator refuses allocations with 4 GiB alignment.
//!   See <https://github.com/rust-lang/rust/issues/30170>.
//!   We over-allocate `BLOCK_SIZE + TWO_GIB` (4 GiB - 16) bytes with 2 GiB alignment,
//!   then use whichever half of the allocation is aligned on `BLOCK_ALIGN`.
//!
//! * Linux: Linux MUSL accepts allocation requests with 4 GiB alignment, but then segfaults when the allocation
//!   is freed. So we use the same trick as on Mac OS - over-allocate with 2 GiB alignment.
//!
//! * Windows: System allocator also doesn't support high alignment allocations, but Rust's `std` contains
//!   a workaround for servicing high-alignment requests.
//!   We side-step that by over-allocating `BLOCK_SIZE + BLOCK_ALIGN` (6 GiB - 16) bytes with alignment 16,
//!   then aligning the returned pointer to `BLOCK_ALIGN` (4 GiB) ourselves.
//!   This avoids `std`'s workaround committing a whole extra page just to store the real allocation pointer.
//!
//! * Other: Same as Mac and Linux.
//!   Note: Fixed size arenas are only supported on 64-bit platforms, so WASM32 is not relevant here.
//!
//! [`Arena`]: super::Arena
//! [`Arena::new_fixed_size`]: super::Arena::new_fixed_size
//! [`System`]: std::alloc::System

use crate::generated::fixed_size_constants::BLOCK_SIZE;

use super::{CHUNK_ALIGN, CHUNK_FOOTER_SIZE};

// `ChunkFooter` lives in the last `CHUNK_FOOTER_SIZE` bytes of the block and must be aligned on `CHUNK_ALIGN` (16).
// `BLOCK_SIZE` and `CHUNK_FOOTER_SIZE` are both multiples of `CHUNK_ALIGN`.
const _: () = {
    assert!(BLOCK_SIZE > 0);
    assert!(BLOCK_SIZE.is_multiple_of(CHUNK_ALIGN));
    assert!(BLOCK_SIZE >= CHUNK_FOOTER_SIZE);
    assert!(CHUNK_FOOTER_SIZE.is_multiple_of(CHUNK_ALIGN));
};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::dealloc_fixed_size_arena_chunk;

#[cfg(not(target_os = "windows"))]
mod unix;
#[cfg(not(target_os = "windows"))]
pub use unix::dealloc_fixed_size_arena_chunk;
