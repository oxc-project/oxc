//! Allocation tracking.
//!
//! This module is only loaded when `track_allocations` feature is enabled.
//! This feature is only used in `tasks/track_memory_allocations`.
//!
//! Current implementation is unsound - see comment on [`get_stats_ref`] below.
//! It's OK to use it in our internal `tasks/track_memory_allocations` tool,
//! but we must take great care that this is NEVER enabled in any other circumstances.
//!
//! Even without the unsoundness, we don't want this enabled outside of `tasks/track_memory_allocations`,
//! as it imposes a performance cost on making allocations.
//!
//! The 2nd cargo feature `disable_track_allocations` is to ensure that compiling with `--all-features`
//! will not load this module.
//!
//! As soon as we replace `bumpalo` with our own arena allocator, we'll remove the hack from `get_stats_ref`,
//! and make this sound.

use std::{cell::Cell, ptr};

use bumpalo::Bump;

use crate::{Allocator, allocator::STATS_FIELD_OFFSET};

/// Counters of allocations and reallocations made in an [`Allocator`].
#[derive(Default)]
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

    /// Reset allocation counters.
    pub(crate) fn reset(&self) {
        self.num_alloc.set(0);
        self.num_realloc.set(0);
    }
}

impl Allocator {
    /// Get number of allocations and reallocations made in this [`Allocator`].
    #[doc(hidden)]
    pub fn get_allocation_stats(&self) -> (usize, usize) {
        let num_alloc = self.stats.num_alloc.get();
        let num_realloc = self.stats.num_realloc.get();
        (num_alloc, num_realloc)
    }
}

/// Get reference to [`AllocationStats`] for a [`Bump`].
///
/// # SAFETY
///
/// Caller must guarantee that the `Bump` provided to this function is wrapped in an [`Allocator`].
///
/// In Oxc, we never use `Bump` alone, without it being wrapped in an `Allocator`.
/// However, we have no static guarantee of this relationship between `Bump` and `Allocator`,
/// so it's usually impossible for callers to proveably satisfy the safety requirements of this method.
///
/// Even if the `Bump` *is* wrapped in an `Allocator`, this may still be UB, as we project beyond
/// the bounds of the `&Bump`. Certainly stacked borrows memory model says this is UB, though it's unclear
/// to me (@overlookmotel) whether stacked borrows is unnecessarily strict on this point.
/// <https://github.com/rust-lang/unsafe-code-guidelines/issues/134>
///
/// This function (and the `track_allocations` feature in general) must only be used for internal tools,
/// and must NEVER be compiled in production code.
pub unsafe fn get_stats_ref(bump: &Bump) -> &AllocationStats {
    // We assume the `Bump` is wrapped in an `Allocator`. We can therefore get a pointer to the `stats`
    // field of `Allocator` from the memory location of the `Bump`.
    // SAFETY: This is UNSOUND. See above.
    unsafe {
        let stats_ptr =
            ptr::from_ref(bump).byte_offset(STATS_FIELD_OFFSET).cast::<AllocationStats>();
        stats_ptr.as_ref().unwrap_unchecked()
    }
}
