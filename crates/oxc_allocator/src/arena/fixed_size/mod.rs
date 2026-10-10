//! [`Arena::new_fixed_size`]
//!
//! Construct an [`Arena`] of fixed size, aligned on a 4 GiB boundary, for raw transfer.
//!
//! Unix reserves address space with `mmap(PROT_NONE)` and makes pages writable on demand.
//! This supports Linux systems with strict overcommit and avoids the high-alignment limitations
//! of `System` on macOS and musl. Windows uses `VirtualAlloc` reservation and commitment.
//! Other 64-bit targets retain the `System` allocator implementation.
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

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::dealloc_fixed_size_arena_chunk;

#[cfg(all(not(unix), not(target_os = "windows")))]
mod other;
#[cfg(all(not(unix), not(target_os = "windows")))]
pub use other::dealloc_fixed_size_arena_chunk;
