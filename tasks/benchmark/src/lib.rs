pub use criterion::*;
use talc::{ClaimOnOom, Span, Talc, Talck};

// Global allocator for use in benchmarks.
//
// `talc` crate provides a global allocator which uses a provided block of memory to allocate from.
// Talc is quite a simple allocator, so my assumption is that it behaves deterministically.
// (not as simple as Bumpalo - it keeps a free list, and it will free memory on drop)
//
// The memory block which all allocations are made from is allocated up front, so no calls to global
// allocator will be made during benchmark runs.
//
// Combine these two properties, and we should have completely deterministic allocation behavior
// in benchmarks, which hopefully will reduce variance to zero (at least for single-threaded code).
//
// NB: The allocator doesn't have to be `talc` specifically. It was just one allocator I found which
// exhibits the property of determinism that we require. There are probably other viable options.

/// 1 GiB memory limit
const ARENA_SIZE: usize = 0x4000_0000;

static mut ARENA: [u8; ARENA_SIZE] = [0; ARENA_SIZE];

#[global_allocator]
static GLOBAL: Talck<spin::Mutex<()>, ClaimOnOom> = Talc::new({
    // SAFETY: Copied from `talc`'s docs https://github.com/SFBdragon/talc#setup
    unsafe { ClaimOnOom::new(Span::from_const_array(core::ptr::addr_of!(ARENA))) }
})
.lock();
