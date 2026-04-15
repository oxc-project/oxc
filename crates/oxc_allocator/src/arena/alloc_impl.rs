//! Main implementation of allocation methods.
//!
//! Public allocation methods are in `alloc.rs`.
//! They all ultimately call into `alloc_layout` or `try_alloc_layout`, defined in this module.
//! (`alloc_layout` and `try_alloc_layout` are also public methods)

use std::{
    alloc::Layout,
    cmp::Ordering,
    iter,
    ptr::{self, NonNull},
};

use allocator_api2::alloc::{AllocError, Allocator};

use super::{
    Arena, CHUNK_FOOTER_SIZE, DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER,
    bumpalo_alloc::{Alloc as BumpaloAlloc, AllocErr},
    utils::{
        is_pointer_aligned_to, layout_from_size_align, oom, round_down_to, round_mut_ptr_down_to,
        round_mut_ptr_up_to_unchecked, round_up_to, round_up_to_unchecked,
    },
};

impl<const MIN_ALIGN: usize> Arena<MIN_ALIGN> {
    /// Allocate space for an object with the given `Layout`.
    ///
    /// The returned pointer points at uninitialized memory, and should be initialized with [`std::ptr::write`].
    ///
    /// # Panics
    ///
    /// Panics if reserving space matching `layout` fails.
    #[inline(always)]
    pub fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        self.try_alloc_layout(layout).unwrap_or_else(|_| oom())
    }

    /// Attempt to allocate space for an object with the given `Layout`.
    ///
    /// If allocation fails, returns `Err`.
    ///
    /// The returned pointer points at uninitialized memory, and should be initialized with [`std::ptr::write`].
    ///
    /// # Errors
    ///
    /// Errors if reserving space matching `layout` fails.
    #[inline(always)]
    pub fn try_alloc_layout(&self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        let res = if let Some(p) = self.try_alloc_layout_fast(layout) {
            Ok(p)
        } else {
            self.alloc_layout_slow(layout).ok_or(AllocErr)
        };

        #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
        if res.is_ok() {
            self.stats.record_allocation();
        }

        res
    }

    #[inline(always)]
    fn try_alloc_layout_fast(&self, layout: Layout) -> Option<NonNull<u8>> {
        // We don't need to check for ZSTs here since they will automatically be handled properly:
        // the pointer will be bumped by zero bytes, modulo alignment.
        // This keeps the fast path optimized for non-ZSTs, which are much more common.
        unsafe {
            let footer_ptr = self.current_chunk_footer.get();
            let footer = footer_ptr.as_ref();

            let ptr = footer.ptr.get().as_ptr();
            let start = footer.data.as_ptr();
            debug_assert!(
                start <= ptr,
                "start pointer {start:#p} should be less than or equal to bump pointer {ptr:#p}"
            );
            debug_assert!(
                ptr <= footer_ptr.cast::<u8>().as_ptr(),
                "bump pointer {ptr:#p} should be less than or equal to footer pointer {footer_ptr:#p}"
            );
            debug_assert!(
                is_pointer_aligned_to(ptr, MIN_ALIGN),
                "bump pointer {ptr:#p} should be aligned to the minimum alignment of {MIN_ALIGN:#x}"
            );
            // This `match` should be boiled away by LLVM: `MIN_ALIGN` is a constant and the layout's alignment
            // is also constant in practice after inlining
            let aligned_ptr = match layout.align().cmp(&MIN_ALIGN) {
                Ordering::Less => {
                    // We need to round the size up to a multiple of `MIN_ALIGN` to preserve the minimum alignment.
                    // This might overflow since we cannot rely on `Layout`'s guarantees.
                    let aligned_size = round_up_to(layout.size(), MIN_ALIGN)?;

                    let capacity = (ptr as usize) - (start as usize);
                    if aligned_size > capacity {
                        return None;
                    }

                    ptr.wrapping_sub(aligned_size)
                }
                Ordering::Equal => {
                    // `Layout` guarantees that rounding the size up to its align cannot overflow
                    // (but does not guarantee that the size is initially a multiple of the alignment,
                    // which is why we need to do this rounding)
                    let aligned_size = round_up_to_unchecked(layout.size(), layout.align());

                    let capacity = (ptr as usize) - (start as usize);
                    if aligned_size > capacity {
                        return None;
                    }

                    ptr.wrapping_sub(aligned_size)
                }
                Ordering::Greater => {
                    // `Layout` guarantees that rounding the size up to its align cannot overflow
                    // (but does not guarantee that the size is initially a multiple of the alignment,
                    // which is why we need to do this rounding)
                    let aligned_size = round_up_to_unchecked(layout.size(), layout.align());

                    let aligned_ptr = round_mut_ptr_down_to(ptr, layout.align());
                    let capacity = (aligned_ptr as usize).wrapping_sub(start as usize);
                    if aligned_ptr < start || aligned_size > capacity {
                        return None;
                    }

                    aligned_ptr.wrapping_sub(aligned_size)
                }
            };

            debug_assert!(
                is_pointer_aligned_to(aligned_ptr, layout.align()),
                "pointer {aligned_ptr:#p} should be aligned to layout alignment of {:#}",
                layout.align()
            );
            debug_assert!(
                is_pointer_aligned_to(aligned_ptr, MIN_ALIGN),
                "pointer {aligned_ptr:#p} should be aligned to minimum alignment of {MIN_ALIGN:#}"
            );
            debug_assert!(
                start <= aligned_ptr && aligned_ptr <= ptr,
                "pointer {aligned_ptr:#p} should be in range {start:#p}..{ptr:#p}"
            );

            debug_assert!(!aligned_ptr.is_null());
            let aligned_ptr = NonNull::new_unchecked(aligned_ptr);

            footer.ptr.set(aligned_ptr);
            Some(aligned_ptr)
        }
    }

    /// Slow path allocation for when we need to allocate a new chunk, because there isn't enough room
    /// in our current chunk.
    #[inline(never)]
    #[cold]
    fn alloc_layout_slow(&self, layout: Layout) -> Option<NonNull<u8>> {
        unsafe {
            if !self.can_grow {
                return None;
            }

            // Get a new chunk from the global allocator
            let current_footer = self.current_chunk_footer.get();
            let current_layout = current_footer.as_ref().layout;

            // By default, we want our new chunk to be about twice as big as the previous chunk.
            // If the global allocator refuses it, we try to divide it by half until it works
            // or the requested size is smaller than the default footer size.
            let min_new_chunk_size = layout.size().max(DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER);
            let mut base_size =
                (current_layout.size() - CHUNK_FOOTER_SIZE).checked_mul(2)?.max(min_new_chunk_size);
            let mut chunk_memory_details = iter::from_fn(|| {
                if base_size >= min_new_chunk_size {
                    let size = base_size;
                    base_size /= 2;
                    Self::new_chunk_memory_details(Some(size), layout)
                } else {
                    None
                }
            });

            let new_footer = chunk_memory_details.find_map(|new_chunk_memory_details| {
                Self::new_chunk(new_chunk_memory_details, layout, current_footer)
            })?;

            debug_assert_eq!(new_footer.as_ref().data.as_ptr() as usize % layout.align(), 0);

            // Set the new chunk as our new current chunk
            self.current_chunk_footer.set(new_footer);

            // And then we can rely on `try_alloc_layout_fast` to allocate space within this chunk
            let ptr = self.try_alloc_layout_fast(layout);
            debug_assert!(ptr.is_some());
            ptr
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        // If the pointer is the last allocation we made, we can reuse the bytes,
        // otherwise they are simply leaked - at least until somebody calls `reset()`
        unsafe {
            if self.is_last_allocation(ptr) {
                let ptr = self.current_chunk_footer.get().as_ref().ptr.get();
                let ptr = ptr.as_ptr().add(layout.size());

                let ptr = round_mut_ptr_up_to_unchecked(ptr, MIN_ALIGN);
                debug_assert!(
                    is_pointer_aligned_to(ptr, MIN_ALIGN),
                    "bump pointer {ptr:#p} should be aligned to the minimum alignment of {MIN_ALIGN:#x}"
                );
                let ptr = NonNull::new_unchecked(ptr);
                self.current_chunk_footer.get().as_ref().ptr.set(ptr);
            }
        }
    }

    #[inline]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<u8>, AllocErr> {
        // If the new layout demands greater alignment than the old layout has, then either:
        //
        // 1. the pointer happens to satisfy the new layout's alignment, so we got lucky
        //    and can return the pointer as-is, or
        //
        // 2. the pointer is not aligned to the new layout's demanded alignment, and we are unlucky.
        //
        // In the case of (2), to successfully "shrink" the allocation, we have to allocate a whole new region
        // for the new layout.
        if old_layout.align() < new_layout.align() {
            return if is_pointer_aligned_to(ptr.as_ptr(), new_layout.align()) {
                Ok(ptr)
            } else {
                let new_ptr = self.try_alloc_layout(new_layout)?;

                #[cfg(all(
                    feature = "track_allocations",
                    not(feature = "disable_track_allocations")
                ))]
                self.stats.record_reallocation_after_allocation();

                // We know that these regions are nonoverlapping because `new_ptr` is a fresh allocation
                unsafe {
                    ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), new_layout.size());
                }

                Ok(new_ptr)
            };
        }

        debug_assert!(is_pointer_aligned_to(ptr.as_ptr(), new_layout.align()));

        let old_size = old_layout.size();
        let new_size = new_layout.size();

        // This is how much space we would *actually* reclaim while satisfying the requested alignment
        let delta = round_down_to(old_size - new_size, new_layout.align().max(MIN_ALIGN));

        if unsafe { self.is_last_allocation(ptr) }
                // Only reclaim the excess space (which requires a copy) if it is worth it:
                // we are actually going to recover "enough" space and we can do a non-overlapping copy.
                //
                // We do `old_size.div_ceil(2)` so division rounds up rather than down. Consider when:
                //
                //     old_size = 5
                //     new_size = 3
                //
                // If we do not take care to round up, this will result in:
                //
                //     delta = 2
                //     (old_size / 2) = (5 / 2) = 2
                //
                // And the the check will succeed even though we are have overlapping ranges:
                //
                //     |--------old-allocation-------|
                //     |------from-------|
                //                 |-------to--------|
                //     +-----+-----+-----+-----+-----+
                //     |  a  |  b  |  c  |  .  |  .  |
                //     +-----+-----+-----+-----+-----+
                //
                // But we MUST NOT have overlapping ranges because we use `copy_nonoverlapping` below.
                // Therefore, we round the division up to avoid this issue.
                && delta >= old_size.div_ceil(2)
        {
            unsafe {
                let footer = self.current_chunk_footer.get();
                let footer = footer.as_ref();

                // Note: `new_ptr` is aligned, because ptr *has to* be aligned, and we made sure delta is aligned
                let new_ptr = NonNull::new_unchecked(footer.ptr.get().as_ptr().add(delta));
                debug_assert!(
                    is_pointer_aligned_to(new_ptr.as_ptr(), MIN_ALIGN),
                    "bump pointer {new_ptr:#p} should be aligned to the minimum alignment of {MIN_ALIGN:#x}"
                );
                footer.ptr.set(new_ptr);

                // Note: We know it is non-overlapping because of the size check in the `if` condition
                ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), new_size);

                #[cfg(all(
                    feature = "track_allocations",
                    not(feature = "disable_track_allocations")
                ))]
                self.stats.record_reallocation();

                return Ok(new_ptr);
            }
        }

        // If this wasn't the last allocation, or shrinking wasn't worth it, simply return the old pointer as is
        Ok(ptr)
    }

    #[inline]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<u8>, AllocErr> {
        let old_size = old_layout.size();

        let new_size = new_layout.size();
        let new_size = round_up_to(new_size, MIN_ALIGN).ok_or(AllocErr)?;

        let align_is_compatible = old_layout.align() >= new_layout.align();

        if align_is_compatible && unsafe { self.is_last_allocation(ptr) } {
            // Try to allocate the delta size within this same block so we can reuse the currently allocated space
            let delta = new_size - old_size;
            if let Some(p) =
                self.try_alloc_layout_fast(layout_from_size_align(delta, old_layout.align())?)
            {
                unsafe { ptr::copy(ptr.as_ptr(), p.as_ptr(), old_size) };

                #[cfg(all(
                    feature = "track_allocations",
                    not(feature = "disable_track_allocations")
                ))]
                self.stats.record_reallocation();

                return Ok(p);
            }
        }

        // Fallback: Do a fresh allocation and copy the existing data into it
        let new_ptr = self.try_alloc_layout(new_layout)?;
        unsafe { ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), old_size) };

        #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
        self.stats.record_reallocation_after_allocation();

        Ok(new_ptr)
    }

    #[inline]
    unsafe fn is_last_allocation(&self, ptr: NonNull<u8>) -> bool {
        let footer = self.current_chunk_footer.get();
        let footer = unsafe { footer.as_ref() };
        footer.ptr.get() == ptr
    }
}

unsafe impl<const MIN_ALIGN: usize> BumpaloAlloc for &Arena<MIN_ALIGN> {
    #[inline(always)]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        self.try_alloc_layout(layout)
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { Arena::<MIN_ALIGN>::dealloc(self, ptr, layout) };
    }

    #[inline]
    unsafe fn realloc(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<NonNull<u8>, AllocErr> {
        let old_size = layout.size();

        if old_size == 0 {
            return self.try_alloc_layout(layout);
        }

        let new_layout = layout_from_size_align(new_size, layout.align())?;
        if new_size <= old_size {
            unsafe { Arena::shrink(self, ptr, layout, new_layout) }
        } else {
            unsafe { Arena::grow(self, ptr, layout, new_layout) }
        }
    }
}

unsafe impl<const MIN_ALIGN: usize> Allocator for &Arena<MIN_ALIGN> {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.try_alloc_layout(layout)
            .map(|p| unsafe {
                NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(p.as_ptr(), layout.size()))
            })
            .map_err(|_| AllocError)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { Arena::<MIN_ALIGN>::dealloc(self, ptr, layout) };
    }

    #[inline]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { Arena::<MIN_ALIGN>::shrink(self, ptr, old_layout, new_layout) }
            .map(|p| unsafe {
                NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(p.as_ptr(), new_layout.size()))
            })
            .map_err(|_| AllocError)
    }

    #[inline]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { Arena::<MIN_ALIGN>::grow(self, ptr, old_layout, new_layout) }
            .map(|p| unsafe {
                NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(p.as_ptr(), new_layout.size()))
            })
            .map_err(|_| AllocError)
    }

    #[inline]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let mut ptr = unsafe { self.grow(ptr, old_layout, new_layout) }?;
        (unsafe { ptr.as_mut() })[old_layout.size()..].fill(0);
        Ok(ptr)
    }
}
