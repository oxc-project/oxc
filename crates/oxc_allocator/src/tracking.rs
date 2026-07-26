//! Allocation tracking.
//!
//! Tracking is only used in `tasks/track_memory_allocations`.
//!
//! We don't want this enabled outside of `tasks/track_memory_allocations`,
//! as it imposes a performance cost on making allocations.
//!
//! This module is only loaded when `track_allocations` feature is enabled, and `disable_track_allocations`
//! feature is *not* enabled. The reason for the 2nd feature is to ensure that compiling with `--all-features`
//! will not load this module.

use std::{
    cell::Cell,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{Allocator, arena::Arena};

/// Whether the next call to the system allocator is an [`Arena`] chunk being allocated
/// or deallocated.
///
/// Chunk operations are marked so that `tasks/track_memory_allocations`'s global allocator
/// wrapper can exclude them from its heap metrics: chunk sizes are quantized and
/// platform-dependent (whether an arena needs one more chunk depends on byte totals, which
/// vary across architectures with type layout — see #22621), so counting them would make
/// the snapshots platform-dependent.
///
/// This is a marker, not a counter: [`start_chunk_operation`] is called immediately before
/// the chunk's `alloc`/`dealloc` call, and the global allocator wrapper consumes the marker
/// while that call is on the stack. There are no allocations in between, but another
/// *thread* allocating at that exact moment could steal the marker; the tracking task's
/// measured workload is single-threaded, so this cannot happen in practice.
static CHUNK_OPERATION_PENDING: AtomicBool = AtomicBool::new(false);

/// Mark that the next system allocator call is an [`Arena`] chunk operation.
pub fn start_chunk_operation() {
    CHUNK_OPERATION_PENDING.store(true, Ordering::Relaxed);
}

/// Counters of allocations and reallocations made in an [`Allocator`].
#[derive(Default, Debug)]
pub struct AllocationStats {
    /// Number of allocations
    num_alloc: Cell<usize>,
    /// Number of reallocations
    num_realloc: Cell<usize>,
}

impl AllocationStats {
    /// Record that an allocation was made.
    pub(crate) fn record_allocation(&self) {
        // Counter maxes out at `usize::MAX`, but if there's that many allocations,
        // the exact number is not important
        self.num_alloc.set(self.num_alloc.get().saturating_add(1));
    }

    /// Record that a reallocation was made.
    pub(crate) fn record_reallocation(&self) {
        // Counter maxes out at `usize::MAX`, but if there's that many allocations,
        // the exact number is not important
        self.num_realloc.set(self.num_realloc.get().saturating_add(1));
    }

    /// Record that a reallocation was made, after it was initially recorded as an allocation.
    pub(crate) fn record_reallocation_after_allocation(&self) {
        // Reduce counter by 1 to "uncount" the allocation which was recorded.
        // If counter is `usize::MAX`, don't reduce it because it might have saturated.
        let num_alloc = self.num_alloc.get();
        if num_alloc != usize::MAX {
            self.num_alloc.set(num_alloc - 1);
        }

        // Counter maxes out at `usize::MAX`, but if there's that many allocations,
        // the exact number is not important
        self.num_realloc.set(self.num_realloc.get().saturating_add(1));
    }

    /// Reset allocation counters.
    pub(crate) fn reset(&self) {
        self.num_alloc.set(0);
        self.num_realloc.set(0);
    }
}

impl Arena {
    /// Get number of allocations and reallocations made in this [`Arena`].
    fn get_allocation_stats(&self) -> (usize, usize) {
        let num_alloc = self.stats.num_alloc.get();
        let num_realloc = self.stats.num_realloc.get();
        (num_alloc, num_realloc)
    }
}

impl Allocator {
    /// Get number of allocations and reallocations made in this [`Allocator`].
    #[doc(hidden)]
    pub fn get_allocation_stats(&self) -> (usize, usize) {
        self.arena().get_allocation_stats()
    }

    /// Consume the pending chunk-operation marker (see [`CHUNK_OPERATION_PENDING`]).
    ///
    /// Returns `true` if the system allocator call currently on the stack is an [`Arena`]
    /// chunk being allocated or deallocated.
    #[doc(hidden)]
    pub fn take_pending_chunk_operation() -> bool {
        CHUNK_OPERATION_PENDING.swap(false, Ordering::Relaxed)
    }
}
