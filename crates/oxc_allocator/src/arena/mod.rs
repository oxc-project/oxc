mod alloc;
mod allocator_api2;
#[expect(clippy::module_inception)]
mod arena;
mod config;
mod constants;
mod footer;

pub use arena::Arena;
pub use config::{
    ArenaConfig, ArenaConfigDefault, ArenaConfigPointerAligned, ArenaConfigSingleChunk,
};
pub use constants::{CHUNK_ALIGN, FOOTER_SIZE, MAX_INITIAL_CAPACITY};

/// Default [`Arena`], with `MIN_ALIGN = 1`.
///
/// i.e. all types will be stored with only their required alignment.
pub type ArenaDefault = Arena<ArenaConfigDefault>;

/// [`Arena`] with `MIN_ALIGN = align_of::<usize>()`.
///
/// i.e. all types will be stored with pointer alignment (or greater if the type requires it).
pub type ArenaPointerAligned = Arena<ArenaConfigPointerAligned>;

/// [`Arena`] with a single 4 GiB chunk for branchless allocation.
///
/// Uses virtual memory - only pages that are written to consume physical memory.
/// Ideal for parsing where total AST size is bounded well below 4 GiB.
pub type ArenaSingleChunk = Arena<ArenaConfigSingleChunk>;

/// The arena type used by [`Allocator`](crate::Allocator).
///
/// This is the arena type used throughout the crate for allocation.
/// Currently uses single-chunk mode for branchless allocation.
pub type AllocatorArena = ArenaSingleChunk;
