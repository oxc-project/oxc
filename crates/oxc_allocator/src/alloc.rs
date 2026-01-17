use std::{alloc::Layout, ptr::NonNull};

/// Trait describing an allocator.
///
/// It's a simpler version of `allocator_api2`'s [`Allocator`] trait.
///
/// The difference between these methods and [`Allocator`]'s versions of them are:
///
/// * `shrink` and `grow` return a pointer and panic/abort if allocation fails,
///   instead of returning `Result::Err`.
/// * All methods return a `NonNull<u8>`, instead of `NonNull<[u8]>`.
///
/// [`Allocator`]: allocator_api2::alloc::Allocator
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
