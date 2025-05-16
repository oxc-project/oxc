/// Size of [`ChunkFooter`](super::footer::ChunkFooter).
pub use super::footer::FOOTER_SIZE;

/// The typical page size these days.
///
/// Note that we don't need to exactly match page size for correctness, and it's OK if this is smaller
/// than the real page size in practice. It isn't worth the portability concerns and lack of const
/// propagation that dynamically looking up the actual page size implies.
pub const TYPICAL_PAGE_SIZE: usize = 0x1000; // 4 KiB

/// Maximum typical overhead per allocation imposed by allocators.
pub const MALLOC_OVERHEAD: usize = 16;

/// Alignment of chunks. This is also maximum limit on `ArenaConfig::MIN_ALIGN`.
pub const CHUNK_ALIGN: usize = 16;

/// Total overhead from malloc, footer and alignment.
///
/// For instance, if we want to request a chunk of memory that has at least X bytes usable for
/// allocations (where `X` is aligned to [`CHUNK_ALIGN`]), then we expect that after adding a footer,
/// malloc overhead and alignment, the chunk of memory the allocator actually sets aside for us is
/// `X + OVERHEAD` rounded up to the nearest suitable size boundary.
pub const OVERHEAD: usize = (MALLOC_OVERHEAD + FOOTER_SIZE).next_multiple_of(CHUNK_ALIGN);

/// Maximum initial capacity for an [`Arena`](super::Arena).
pub const MAX_INITIAL_CAPACITY: usize = (isize::MAX as usize + 1) - FOOTER_SIZE - CHUNK_ALIGN;

/// Default size of first chunk, including overhead
// TODO: Make this bigger?
const FIRST_CHUNK_DEFAULT_SIZE: usize = 512;

/// Default data capacity of first chunk
pub const FIRST_CHUNK_DEFAULT_CAPACITY: usize = FIRST_CHUNK_DEFAULT_SIZE - OVERHEAD;
