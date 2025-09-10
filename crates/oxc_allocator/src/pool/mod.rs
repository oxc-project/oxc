use std::{mem::ManuallyDrop, ops::Deref};

use crate::Allocator;

// Fixed size allocators are only supported on 64-bit little-endian platforms at present.
// They are only enabled if `fixed_size` feature enabled, and `disable_fixed_size` feature is not enabled.
//
// Note: Importing the `fixed_size` module would cause a compilation error on 32-bit systems.
#[cfg(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
))]
mod fixed_size;
#[cfg(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
))]
pub use fixed_size::*;

#[cfg(not(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
)))]
mod standard;
#[cfg(not(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
)))]
pub use standard::*;

/// A guard object representing exclusive access to an [`Allocator`] from the pool.
///
/// On drop, the `Allocator` is reset and returned to the pool.
pub struct AllocatorGuard<'alloc_pool> {
    allocator: ManuallyDrop<Allocator>,
    pool: &'alloc_pool AllocatorPool,
}

impl Deref for AllocatorGuard<'_> {
    type Target = Allocator;

    fn deref(&self) -> &Self::Target {
        &self.allocator
    }
}

impl Drop for AllocatorGuard<'_> {
    /// Return [`Allocator`] back to the pool.
    fn drop(&mut self) {
        // SAFETY: After taking ownership of the `Allocator`, we do not touch the `ManuallyDrop` again
        let allocator = unsafe { ManuallyDrop::take(&mut self.allocator) };
        self.pool.add(allocator);
    }
}
