// Allocation methods are small and should always be inlined
#![expect(clippy::inline_always)]

use std::{
    alloc::Layout,
    ptr::{self, NonNull},
};

use crate::alloc::Alloc;

use super::{Arena, config::ArenaConfigExt};

impl<Config: ArenaConfigExt> Alloc for Arena<Config> {
    /// TODO: Docs
    #[inline(always)]
    fn alloc(&self, layout: Layout) -> NonNull<u8> {
        self.alloc_layout(layout)
    }

    /// # SAFETY
    /// TODO
    #[inline(always)]
    #[expect(unused_variables)]
    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        // No-op. We don't attempt to reclaim memory.
    }

    /// # SAFETY
    /// TODO
    #[inline(always)]
    #[expect(unused_variables)]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> NonNull<u8> {
        // No-op. We don't attempt to reclaim memory.
        ptr
    }

    /// # SAFETY
    /// TODO
    #[inline(always)]
    unsafe fn grow(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> NonNull<u8> {
        // Allocate new layout
        let new_ptr = self.alloc_layout(new_layout);

        // Copy data from old allocation to new one
        // SAFETY: TODO
        unsafe { ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), old_layout.size()) };

        // We don't attempt to reclaim memory for the old allocation

        new_ptr
    }
}
