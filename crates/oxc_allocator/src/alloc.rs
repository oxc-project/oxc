// All methods just delegate to `Bump`'s methods
#![expect(clippy::inline_always)]

use std::{
    alloc::{Layout, handle_alloc_error},
    ptr::NonNull,
};

use allocator_api2::alloc::Allocator;
use bumpalo::Bump;

/// Trait describing an allocator.
///
/// It's a simpler version of `allocator_api2`'s [`Allocator`] trait.
///
/// The difference between these methods and [`Allocator`]'s versions of them are:
///
/// * `shrink` and `grow` return a pointer and panic/abort if allocation fails,
///   instead of returning `Result::Err`.
/// * All methods return a `NonNull<u8>`, instead of `NonNull<[u8]>`.
pub trait Alloc {
    /// Allocate space for an object with the given [`Layout`].
    ///
    /// The returned pointer points at uninitialized memory, and should be initialized
    /// with [`std::ptr::write`].
    ///
    /// # Panics
    ///
    /// Panics if reserving space for `layout` fails.
    fn alloc(&self, layout: Layout) -> NonNull<u8>;

    /// Deallocate the memory referenced by `ptr`.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must denote a block of memory currently allocated via this allocator.
    /// * `layout` must be the same [`Layout`] that block was originally allocated with.
    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout);

    /// Grow an existing allocation to new [`Layout`].
    ///
    /// If the allocation cannot be grown in place, the data from the whole of the old allocation
    /// is copied to the start of the new allocation.
    ///
    /// If the allocation is grown in place, no memory copying will occur.
    ///
    /// Either way, the pointer returned points to the new allocation.
    ///
    /// Any access to the old `ptr` is Undefined Behavior, even if the allocation was grown in-place.
    /// The newly returned pointer is the only valid pointer for accessing this memory now.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must denote a block of memory currently allocated via this allocator.
    /// * `old_layout` must be the same [`Layout`] that block was originally allocated with.
    /// * `new_layout.size()` must be greater than or equal to `old_layout.size()`.
    unsafe fn grow(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> NonNull<u8>;

    /// Shrink an existing allocation to new [`Layout`].
    ///
    /// If the allocation cannot be shrunk in place, `new_layout.size()` bytes of data
    /// from the old allocation are copied to the new allocation.
    ///
    /// If the allocation is shrunk in place, no memory copying will occur.
    ///
    /// Either way, the pointer returned points to the new allocation.
    ///
    /// Any access to the old `ptr` is Undefined Behavior, even if the allocation was shrunk in-place.
    /// The newly returned pointer is the only valid pointer for accessing this memory now.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must denote a block of memory currently allocated via this allocator.
    /// * `old_layout` must be the same [`Layout`] that block was originally allocated with.
    /// * `new_layout.size()` must be smaller than or equal to `old_layout.size()`.
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> NonNull<u8>;
}

/// Implement [`Alloc`] for [`bumpalo::Bump`].
///
/// All methods except `alloc` delegate to [`Bump`]'s impl of `allocator_api2`'s [`Allocator`] trait.
impl Alloc for Bump {
    /// Allocate space for an object with the given [`Layout`].
    ///
    /// The returned pointer points at uninitialized memory, and should be initialized
    /// with [`std::ptr::write`].
    ///
    /// # Panics
    ///
    /// Panics if reserving space for `layout` fails.
    #[inline(always)]
    fn alloc(&self, layout: Layout) -> NonNull<u8> {
        // SAFETY: This is UNSOUND (see comment on `get_stats_ref`). But usage is gated behind
        // `track_allocations` feature, so should never be compiled in production code.
        #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
        unsafe {
            crate::tracking::get_stats_ref(self).record_allocation();
        }

        self.alloc_layout(layout)
    }

    /// Deallocate the memory referenced by `ptr`.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must denote a block of memory currently allocated via this allocator.
    /// * `layout` must be the same [`Layout`] that block was originally allocated with.
    #[inline(always)]
    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        // SAFETY: Safety requirements of `Allocator::deallocate` are the same as for this method
        unsafe { self.deallocate(ptr, layout) }
    }

    /// Grow an existing allocation to new [`Layout`].
    ///
    /// If the allocation cannot be grown in place, the data from the whole of the old allocation
    /// is copied to the start of the new allocation.
    ///
    /// If the allocation is grown in place, no memory copying will occur.
    ///
    /// Either way, the pointer returned points to the new allocation.
    ///
    /// Any access to the old `ptr` is Undefined Behavior, even if the allocation was grown in-place.
    /// The newly returned pointer is the only valid pointer for accessing this memory now.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must denote a block of memory currently allocated via this allocator.
    /// * `old_layout` must be the same [`Layout`] that block was originally allocated with.
    /// * `new_layout.size()` must be greater than or equal to `old_layout.size()`.
    ///
    /// # Panics
    ///
    /// Panics / aborts if reserving space for `new_layout` fails.
    #[inline(always)]
    unsafe fn grow(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> NonNull<u8> {
        // SAFETY: This is UNSOUND (see comment on `get_stats_ref`). But usage is gated behind
        // `track_allocations` feature, so should never be compiled in production code.
        #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
        unsafe {
            crate::tracking::get_stats_ref(self).record_reallocation();
        }

        // SAFETY: Safety requirements of `Allocator::grow` are the same as for this method
        let res = unsafe { Allocator::grow(&self, ptr, old_layout, new_layout) };
        match res {
            Ok(new_ptr) => new_ptr.cast::<u8>(),
            Err(_) => handle_alloc_error(new_layout), // panic/abort
        }
    }

    /// Shrink an existing allocation to new [`Layout`].
    ///
    /// If the allocation cannot be shrunk in place, the `layout.new()` bytes of data
    /// from the old allocation are copied to the new allocation.
    ///
    /// If the allocation is shrunk in place, no memory copying will occur.
    ///
    /// Either way, the pointer returned points to the new allocation.
    ///
    /// Any access to the old `ptr` is Undefined Behavior, even if the allocation was shrunk in-place.
    /// The newly returned pointer is the only valid pointer for accessing this memory now.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must denote a block of memory currently allocated via this allocator.
    /// * `old_layout` must be the same [`Layout`] that block was originally allocated with.
    /// * `new_layout.size()` must be smaller than or equal to `old_layout.size()`.
    ///
    /// # Panics
    ///
    /// Panics / aborts if reserving space for `new_layout` fails.
    #[inline(always)]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> NonNull<u8> {
        // SAFETY: Safety requirements of `Allocator::shrink` are the same as for this method
        let res = unsafe { Allocator::shrink(&self, ptr, old_layout, new_layout) };
        match res {
            Ok(new_ptr) => new_ptr.cast::<u8>(),
            Err(_) => handle_alloc_error(new_layout), // panic/abort
        }
    }
}
