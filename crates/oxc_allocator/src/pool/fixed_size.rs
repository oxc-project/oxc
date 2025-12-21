use std::{
    alloc::{GlobalAlloc, Layout, System},
    mem::{self, ManuallyDrop},
    ptr::NonNull,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

#[cfg(target_os = "windows")]
use std::sync::Condvar;

use oxc_ast_macros::ast;
use oxc_data_structures::stack::Stack;

use crate::{
    Allocator,
    generated::fixed_size_constants::{BLOCK_ALIGN, BLOCK_SIZE, RAW_METADATA_SIZE},
};

const TWO_GIB: usize = 1 << 31;
const FOUR_GIB: usize = 1 << 32;

/// A thread-safe pool for reusing [`Allocator`] instances, that uses fixed-size allocators,
/// suitable for use with raw transfer.
///
/// Has different behavior depending on platform.
/// Implementation for Windows is different, due to Windows' lack of virtual memory overcommit.
///
/// # Design
/// This pool is designed for the linter to use with JS plugins.
///
/// There are 3 possible scenarios:
///
/// ## 1. JS plugins not in use
///
/// Linter will use `StandardAllocatorPool`, so not relevant here.
///
/// ## 2. JS plugins in use, but no `import` plugin
///
/// Parse + lint happen one after the other, on a single thread.
/// Files are parsed directly into fixed-size allocators. After linting, the allocator is returned to the pool.
///
/// Therefore, no more than `thread_count` fixed-size allocators exist concurrently.
///
/// ## 3. Both JS plugins and `import` plugin in use
///
/// Multi-file analysis is enabled. Many ASTs may be parsed and held in memory - many more than `thread_count`.
/// Linter parses ASTs into standard allocators.
/// When it is time to lint AST with JS plugins, AST is copied into a fixed-size allocator.
/// After linting, the fixed-size allocator is returned to the pool.
/// The last step happens on maximum `thread_count` threads simultaneously.
///
/// Therefore, no more than `thread_count` fixed-size allocators exist concurrently.
///
/// ## All scenarios
///
/// In all scenarios, `thread_count` fixed-size allocators is sufficient for the workload.
///
/// We handle both scenarios (2) and (3) the same way, but implemented differently on Windows and Linux/Mac.
///
/// * Linux/Mac: No problem creating `thread_count` allocators.
///   That is sufficient for the workload. Pool will not need to grow.
///
/// * Windows: There may not be enough memory for `thread_count` allocators.
///   We create a pool of as close to `thread_count` allocators as possible.
///   If an allocator is unavailable in pool, thread blocks until another thread is finished with one.
///
/// On all platforms, the pool is not growable after it's created, and that is sufficient for the workload.
pub struct FixedSizeAllocatorPool {
    /// Allocators currently in the pool.
    /// We use a `Stack` because it's faster than `Vec` for `push` and `pop`,
    /// and those are the operations we do while `Mutex` lock is held.
    /// The shorter the time lock is held, the less contention there is.
    allocators: Mutex<Stack<FixedSizeAllocator>>,

    /// Condition variable used to signal when an allocator is returned to the pool.
    /// Threads blocked in [`Self::get`] waiting for an allocator will be woken up
    /// when [`Self::add`] returns an allocator to the pool.
    ///
    /// Only used on Windows. On *nix systems, we don't need this synchronization.
    #[cfg(target_os = "windows")]
    available: Condvar,
}

impl FixedSizeAllocatorPool {
    /// Create a new [`FixedSizeAllocatorPool`] containing `thread_count` allocators.
    ///
    /// Linux/Mac implementation.
    ///
    /// Linux/Mac systems that we support overcommit virtual memory, so there is plenty of virtual memory available.
    /// Creating a number of allocators equal to the number of threads should be easily possible,
    /// and they'll primarily consume only virtual memory, not physical memory.
    ///
    /// The pool is not growable. Calling [`get`] on this pool when it's empty will panic.
    ///
    /// # Panics
    /// Panics if cannot create `thread_count` allocators. This should be impossible if `thread_count` is accurate.
    ///
    /// [`get`]: Self::get
    #[cfg(not(target_os = "windows"))]
    pub fn new(thread_count: usize) -> Self {
        let mut allocators = Stack::with_capacity(thread_count);

        // Create `thread_count` allocators
        for i in 0..thread_count {
            // It's impossible to create more than `u32::MAX` allocators, as `u32::MAX` x 4 GiB allocations would
            // consume almost the entirety of a 64-bit address space. No platform has such a large address space.
            // Typically they use 48 bit address space, or 53 bit at most.
            #[expect(clippy::cast_possible_truncation)]
            let allocator = FixedSizeAllocator::try_new(i as u32).unwrap();
            allocators.push(allocator);
        }

        Self { allocators: Mutex::new(allocators) }
    }

    /// Create a new [`FixedSizeAllocatorPool`] containing *up to* `thread_count` allocators.
    ///
    /// Windows implementation.
    ///
    /// Windows doesn't overcommit virtual memory, so it's easy to hit OOM.
    /// We want to use as many allocators as we can up to `thread_count`, but without exhausting memory,
    /// and without leaving the system starved of memory for *other* allocations.
    ///
    /// So we create as many allocators as we can, up to `thread_count + 1`, and then discard the last one.
    /// This should guarantee that there's at least 4 GiB of memory left for other allocations.
    ///
    /// If we didn't return one allocator's memory back to the system, then it'd be pot luck whether you might get OOM,
    /// depending on exactly how much memory the system has available in total.
    ///
    /// Each allocator is 4 GiB in size, so if system has 16.01 GiB of memory available, we could succeed in creating
    /// 4 x 4 GiB allocators, but that'd only leave 10 MiB of memory free. Likely then some other allocation
    /// (e.g. creating a normal `Allocator`, or even allocating a heap `String`) would fail due to OOM later on.
    ///
    /// Note that "memory available" on Windows does not mean "how much RAM the system has".
    /// It includes the swap file, the size of which depends on how much free disk space the system has.
    /// So numbers like 16.01 GiB are not at all out of the question.
    ///
    /// The pool is not growable. Calling [`get`] on this pool when it's empty will block the calling thread
    /// until another thread returns an allocator to the pool.
    ///
    /// # Panics
    /// Panics if cannot create at least 1 allocator.
    ///
    /// [`get`]: Self::get
    #[cfg(target_os = "windows")]
    pub fn new(thread_count: usize) -> Self {
        // Capacity is `thread_count + 1`, because we want to give 1 allocator back to system
        let capacity = thread_count + 1;

        let mut allocators = Stack::with_capacity(capacity);

        // Get as many allocators as possible, up to `capacity`
        for i in 0..capacity {
            // It's impossible to create more than `u32::MAX` allocators, as `u32::MAX` x 4 GiB allocations would
            // consume almost the entirety of a 64-bit address space. No platform has such a large address space.
            // Typically they use 48 bit address space, or 53 bit at most.
            #[expect(clippy::cast_possible_truncation)]
            let allocator = FixedSizeAllocator::try_new(i as u32);
            let Ok(allocator) = allocator else { break };
            allocators.push(allocator);
        }

        // Discard last allocator if we have more than 1.
        // This leaves pool containing between 1 and `thread_count` allocators.
        match allocators.len() {
            // If we can't create even 1 allocator, panic
            0 => panic!("Insufficient memory to create fixed-size allocator pool"),
            // If we only got 1, keep it.
            // If system has just over 4 GiB memory available in total, OOM is possible later.
            // But what else can we do in this case?
            1 => {}
            // Otherwise, discard the last allocator we got, to leave memory free for other allocations
            _ => {
                allocators.pop();
            }
        }

        Self { allocators: Mutex::new(allocators), available: Condvar::new() }
    }

    /// Retrieve an [`Allocator`] from the pool.
    ///
    /// Linux/Mac implementation.
    ///
    /// # Panics
    ///
    /// * Panics if the pool is empty.
    /// * Panics if the underlying mutex is poisoned.
    #[cfg(not(target_os = "windows"))]
    pub fn get(&self) -> Allocator {
        // Try to get an allocator from the pool.
        // This is in a block, so that `Mutex` lock is held for the shortest possible time.
        let maybe_allocator = {
            let mut allocators_guard = self.allocators.lock().unwrap();
            allocators_guard.pop()
        };

        // Panic if pool is empty. Should never happen if the pool was created with the correct `thread_count`.
        if let Some(allocator) = maybe_allocator {
            allocator.into_inner()
        } else {
            panic!("Tried to get an allocator from an empty `FixedSizeAllocatorPool`")
        }
    }

    /// Retrieve an [`Allocator`] from the pool, blocking until one becomes available if the pool is currentlyempty.
    ///
    /// Windows implementation.
    ///
    /// # Panics
    /// Panics if the underlying mutex is poisoned.
    #[cfg(target_os = "windows")]
    pub fn get(&self) -> Allocator {
        // Try to get an allocator from the pool.
        // If pool is empty, wait for notification that the pool isn't empty any more.
        // After receiving a notification, we must still check that pool is not empty,
        // (no `.pop().unwrap_unchecked()` here), because `Condvar` can produce spurious wakeups.
        let mut allocators_guard = self.allocators.lock().unwrap();
        loop {
            if let Some(allocator) = allocators_guard.pop() {
                return allocator.into_inner();
            }
            allocators_guard = self.available.wait(allocators_guard).unwrap();
        }
    }

    /// Add an [`Allocator`] to the pool.
    ///
    /// The `Allocator` is reset by this method, so it's ready to be re-used.
    ///
    /// # SAFETY
    /// The `Allocator` must have been created by a `FixedSizeAllocatorPool` (not `StandardAllocatorPool`).
    ///
    /// # Panics
    /// Panics if the underlying mutex is poisoned.
    pub(super) unsafe fn add(&self, allocator: Allocator) {
        let mut fixed_size_allocator =
            FixedSizeAllocator { allocator: ManuallyDrop::new(allocator) };
        fixed_size_allocator.reset();

        // This is in a block so that lock (`allocators_guard`) is released before notifying the `Condvar`.
        // This avoids waking up a thread, only for it to be immediately blocked again because the `Mutex`
        // is still locked.
        {
            let mut allocators_guard = self.allocators.lock().unwrap();
            allocators_guard.push(fixed_size_allocator);
        }

        // On Windows, notify waiting threads that an allocator is available (see Windows impl of `get`)
        #[cfg(target_os = "windows")]
        self.available.notify_one();
    }
}

/// Metadata about a `FixedSizeAllocator`.
///
/// Is stored in the memory backing the `FixedSizeAllocator`, after `RawTransferMetadata`,
/// which is after the section of the allocation which `Allocator` uses for its chunk.
#[ast]
pub struct FixedSizeAllocatorMetadata {
    /// ID of this allocator
    pub id: u32,
    /// Pointer to start of original allocation backing the `FixedSizeAllocator`
    pub(crate) alloc_ptr: NonNull<u8>,
    /// `true` if both Rust and JS currently hold references to this `FixedSizeAllocator`.
    ///
    /// * `false` initially.
    /// * Set to `true` when buffer is shared with JS.
    /// * When JS garbage collector collects the buffer, set back to `false` again.
    ///   Memory will be freed when the `FixedSizeAllocator` is dropped on Rust side.
    /// * Also set to `false` if `FixedSizeAllocator` is dropped on Rust side.
    ///   Memory will be freed in finalizer when JS garbage collector collects the buffer.
    pub is_double_owned: AtomicBool,
}

// What we ideally want is an allocation 2 GiB in size, aligned on 4 GiB.
// But system allocator on Mac OS refuses allocations with 4 GiB alignment.
// https://github.com/rust-lang/rust/blob/556d20a834126d2d0ac20743b9792b8474d6d03c/library/std/src/sys/alloc/unix.rs#L16-L27
// https://github.com/rust-lang/rust/issues/30170
//
// So we instead allocate 4 GiB with 2 GiB alignment, and then use either the 1st or 2nd half
// of the allocation, one of which is guaranteed to be on a 4 GiB boundary.
//
// TODO: We could use this workaround only on Mac OS, and just allocate what we actually want on Linux.
// Windows OS allocator also doesn't support high alignment allocations, so Rust contains a workaround
// which over-allocates (6 GiB in this case).
// https://github.com/rust-lang/rust/blob/556d20a834126d2d0ac20743b9792b8474d6d03c/library/std/src/sys/alloc/windows.rs#L120-L137
// Could just use that built-in workaround, rather than implementing our own, or allocate a 6 GiB chunk
// with alignment 16, to skip Rust's built-in workaround.
// Note: Rust's workaround will likely commit a whole page of memory, just to store the real pointer.
const ALLOC_SIZE: usize = BLOCK_SIZE + TWO_GIB;
const ALLOC_ALIGN: usize = TWO_GIB;

/// Layout of backing allocations for fixed-size allocators.
const ALLOC_LAYOUT: Layout = match Layout::from_size_align(ALLOC_SIZE, ALLOC_ALIGN) {
    Ok(layout) => layout,
    Err(_) => unreachable!(),
};

/// Structure which wraps an [`Allocator`] with fixed size of 2 GiB - 16, and aligned on 4 GiB.
///
/// # Allocation strategy
///
/// To achieve this, we manually allocate memory to back the `Allocator`'s single chunk,
/// and to store other metadata.
///
/// We over-allocate 4 GiB, and then use only half of that allocation - either the 1st half,
/// or the 2nd half, depending on the alignment of the allocation received from `alloc.alloc()`.
/// One of those halves will be aligned on 4 GiB, and that's the one we use.
///
/// Inner `Allocator` is wrapped in `ManuallyDrop` to prevent it freeing the memory itself,
/// and `FixedSizeAllocator` has a custom `Drop` impl which frees the whole of the original allocation.
///
/// We allocate via `System` allocator, bypassing any registered alternative global allocator
/// (e.g. Mimalloc in linter). Mimalloc complains that it cannot serve allocations with high alignment,
/// and presumably it's pointless to try to obtain such large allocations from a thread-local heap,
/// so better to go direct to the system allocator anyway.
///
/// # Regions of the allocated memory
///
/// 2 GiB of the allocated memory is not used at all (see above).
///
/// The remaining 2 GiB - 16 bytes, which *is* used, is split up as follows:
///
/// ```txt
///                                                         WHOLE BLOCK - aligned on 4 GiB
/// <-----------------------------------------------------> Allocated block (`BLOCK_SIZE` bytes)
///
///                                                         ALLOCATOR
/// <----------------------------------------->             `Allocator` chunk (`CHUNK_SIZE` bytes)
///                                      <---->             Bumpalo's `ChunkFooter` (aligned on 16)
/// <----------------------------------->                   `Allocator` chunk data storage (for AST)
///
///                                                         METADATA
///                                            <---->       `RawTransferMetadata`
///                                                  <----> `FixedSizeAllocatorMetadata`
///
///                                                         BUFFER SENT TO JS
/// <----------------------------------------------->       Buffer sent to JS (`BUFFER_SIZE` bytes)
/// ```
///
/// Note that the buffer sent to JS includes both the `Allocator` chunk, and `RawTransferMetadata`,
/// but does NOT include `FixedSizeAllocatorMetadata`.
///
/// The end of the region used for `Allocator` chunk must be aligned on `Allocator::RAW_MIN_ALIGN` (16),
/// due to the requirements of Bumpalo. We manage that by:
/// * `BLOCK_SIZE` is a multiple of 16.
/// * `RawTransferMetadata` is 16 bytes.
/// * Size of `FixedSizeAllocatorMetadata` is rounded up to a multiple of 16.
#[repr(transparent)]
struct FixedSizeAllocator {
    /// `Allocator` which utilizes part of the original allocation
    allocator: ManuallyDrop<Allocator>,
}

impl FixedSizeAllocator {
    /// Try to create a new [`FixedSizeAllocator`].
    ///
    /// Returns `Err` if memory allocation fails.
    #[expect(clippy::items_after_statements)]
    fn try_new(id: u32) -> Result<Self, ()> {
        // Only support little-endian systems. `Allocator::from_raw_parts` includes this same assertion.
        // This module is only compiled on 64-bit little-endian systems, so it should be impossible for
        // this panic to occur. But we want to make absolutely sure that if there's a mistake elsewhere,
        // `Allocator::from_raw_parts` cannot panic, as that'd result in a large memory leak.
        // Compiler will optimize this out.
        #[expect(clippy::manual_assert)]
        if cfg!(target_endian = "big") {
            panic!("`FixedSizeAllocator` is not supported on big-endian systems.");
        }

        // Allocate block of memory.
        // SAFETY: `ALLOC_LAYOUT` does not have zero size.
        let alloc_ptr = unsafe { System.alloc(ALLOC_LAYOUT) };
        let alloc_ptr = NonNull::new(alloc_ptr).ok_or(())?;

        // All code in the rest of this function is infallible, so the allocation will always end up
        // owned by a `FixedSizeAllocator`, which takes care of freeing the memory correctly on drop

        // Get pointer to use for allocator chunk, aligned to 4 GiB.
        // `alloc_ptr` is aligned on 2 GiB, so `alloc_ptr % FOUR_GIB` is either 0 or `TWO_GIB`.
        //
        // * If allocation is already aligned on 4 GiB, `offset == 0`.
        //   Chunk occupies 1st half of the allocation.
        // * If allocation is not aligned on 4 GiB, `offset == TWO_GIB`.
        //   Adding `offset` to `alloc_ptr` brings it up to 4 GiB alignment.
        //   Chunk occupies 2nd half of the allocation.
        //
        // Either way, `chunk_ptr` is aligned on 4 GiB.
        let offset = alloc_ptr.as_ptr() as usize % FOUR_GIB;
        // SAFETY: We allocated 4 GiB of memory, so adding `offset` to `alloc_ptr` is in bounds
        let chunk_ptr = unsafe { alloc_ptr.add(offset) };

        debug_assert!((chunk_ptr.as_ptr() as usize).is_multiple_of(BLOCK_ALIGN));

        const FIXED_METADATA_SIZE_ROUNDED: usize =
            size_of::<FixedSizeAllocatorMetadata>().next_multiple_of(Allocator::RAW_MIN_ALIGN);
        const FIXED_METADATA_OFFSET: usize = BLOCK_SIZE - FIXED_METADATA_SIZE_ROUNDED;
        const _: () =
            assert!(FIXED_METADATA_OFFSET.is_multiple_of(align_of::<FixedSizeAllocatorMetadata>()));

        const CHUNK_SIZE: usize = FIXED_METADATA_OFFSET - RAW_METADATA_SIZE;
        const _: () = assert!(CHUNK_SIZE.is_multiple_of(Allocator::RAW_MIN_ALIGN));

        // SAFETY: Memory region starting at `chunk_ptr` with `CHUNK_SIZE` bytes is within
        // the allocation we just made.
        // `chunk_ptr` has high alignment (4 GiB). `CHUNK_SIZE` is large and a multiple of 16.
        let allocator = unsafe { Allocator::from_raw_parts(chunk_ptr, CHUNK_SIZE) };
        let allocator = ManuallyDrop::new(allocator);

        // Write `FixedSizeAllocatorMetadata` to after space reserved for `RawTransferMetadata`,
        // which is after the end of the allocator chunk
        let metadata =
            FixedSizeAllocatorMetadata { alloc_ptr, id, is_double_owned: AtomicBool::new(false) };
        // SAFETY: `FIXED_METADATA_OFFSET` is `FIXED_METADATA_SIZE_ROUNDED` bytes before end of
        // the allocation, so there's space for `FixedSizeAllocatorMetadata`.
        // It's sufficiently aligned for `FixedSizeAllocatorMetadata`.
        unsafe {
            let metadata_ptr =
                chunk_ptr.add(FIXED_METADATA_OFFSET).cast::<FixedSizeAllocatorMetadata>();
            metadata_ptr.write(metadata);
        }

        Ok(Self { allocator })
    }

    /// Reset this [`FixedSizeAllocator`].
    fn reset(&mut self) {
        // Set cursor back to end
        self.allocator.reset();

        // Set data pointer back to start.
        // SAFETY: Fixed-size allocators have data pointer originally aligned on `BLOCK_ALIGN`,
        // and size less than `BLOCK_ALIGN`. So we can restore original data pointer by rounding down
        // to next multiple of `BLOCK_ALIGN`.
        // We're restoring the original data pointer, so it cannot break invariants about alignment,
        // being within the chunk's allocation, or being before cursor pointer.
        unsafe {
            let data_ptr = self.allocator.data_ptr();
            let offset = data_ptr.as_ptr() as usize % BLOCK_ALIGN;
            let data_ptr = data_ptr.sub(offset);
            self.allocator.set_data_ptr(data_ptr);
        }
    }

    /// Unwrap a [`FixedSizeAllocator`] into the [`Allocator`] it contains.
    ///
    /// Caller must ensure that the returned `Allocator` is not dropped.
    /// It must be wrapped in another type to ensure that it's dropped correctly.
    #[inline] // Because this function is a no-op
    fn into_inner(self) -> Allocator {
        // SAFETY: `FixedSizeAllocator` is just a wrapper around `ManuallyDrop<Allocator>`,
        // and is `#[repr(transparent)]`, so the 2 are equivalent
        let allocator =
            unsafe { mem::transmute::<FixedSizeAllocator, ManuallyDrop<Allocator>>(self) };
        ManuallyDrop::into_inner(allocator)
    }
}

impl Drop for FixedSizeAllocator {
    fn drop(&mut self) {
        // SAFETY: This `Allocator` was created by this `FixedSizeAllocator`
        unsafe {
            let metadata_ptr = self.allocator.fixed_size_metadata_ptr();
            free_fixed_size_allocator(metadata_ptr);
        }
    }
}

/// Deallocate memory backing a `FixedSizeAllocator` if it's not double-owned
/// (both owned by a `FixedSizeAllocator` on Rust side *and* held as a buffer on JS side).
///
/// If it is double-owned, don't deallocate the memory but set the flag that it's no longer double-owned
/// so next call to this function will deallocate it.
///
/// # SAFETY
///
/// This function must be called only when either:
/// 1. The corresponding `FixedSizeAllocator` is dropped on Rust side. or
/// 2. The buffer on JS side corresponding to this `FixedSizeAllocatorMetadata` is garbage collected.
///
/// Calling this function in any other circumstances would result in a double-free.
///
/// `metadata_ptr` must point to a valid `FixedSizeAllocatorMetadata`.
pub unsafe fn free_fixed_size_allocator(metadata_ptr: NonNull<FixedSizeAllocatorMetadata>) {
    // Get pointer to start of original allocation from `FixedSizeAllocatorMetadata`
    let alloc_ptr = {
        // SAFETY: This `Allocator` was created by the `FixedSizeAllocator`.
        // `&FixedSizeAllocatorMetadata` ref only lives until end of this block.
        let metadata = unsafe { metadata_ptr.as_ref() };

        // * If `is_double_owned` is already `false`, then one of:
        //   1. The `Allocator` was never sent to JS side, or
        //   2. The `FixedSizeAllocator` was already dropped on Rust side, or
        //   3. Garbage collector already collected it on JS side.
        //   We can deallocate the memory.
        //
        // * If `is_double_owned` is `true`, set it to `false` and exit.
        //   Memory will be freed when `FixedSizeAllocator` is dropped on Rust side
        //   or JS garbage collector collects the buffer.
        //
        // Maybe a more relaxed `Ordering` would be OK, but I (@overlookmotel) am not sure,
        // so going with `Ordering::SeqCst` to be on safe side.
        // Deallocation only happens at the end of the whole process, so it shouldn't matter much.
        // TODO: Figure out if can use `Ordering::Relaxed`.
        let is_double_owned = metadata.is_double_owned.swap(false, Ordering::SeqCst);
        if is_double_owned {
            return;
        }

        metadata.alloc_ptr
    };

    // Deallocate the memory backing the `FixedSizeAllocator`.
    // SAFETY: Originally allocated from `System` allocator at `alloc_ptr`, with layout `ALLOC_LAYOUT`.
    unsafe { System.dealloc(alloc_ptr.as_ptr(), ALLOC_LAYOUT) }
}

impl Allocator {
    /// Get pointer to the `FixedSizeAllocatorMetadata` for this [`Allocator`].
    ///
    /// # SAFETY
    /// * This `Allocator` must have been created by a `FixedSizeAllocator`.
    /// * This pointer must not be used to create a mutable reference to the `FixedSizeAllocatorMetadata`,
    ///   only immutable references.
    pub unsafe fn fixed_size_metadata_ptr(&self) -> NonNull<FixedSizeAllocatorMetadata> {
        // SAFETY: Caller guarantees this `Allocator` was created by a `FixedSizeAllocator`.
        //
        // `FixedSizeAllocator::new` writes `FixedSizeAllocatorMetadata` after the end of
        // the chunk owned by the `Allocator`, and `RawTransferMetadata` (see above).
        // `end_ptr` is end of the allocator chunk (after the chunk header).
        // So `end_ptr + RAW_METADATA_SIZE` points to a valid, initialized `FixedSizeAllocatorMetadata`.
        //
        // We never create `&mut` references to `FixedSizeAllocatorMetadata`,
        // and it's not part of the buffer sent to JS, so no danger of aliasing violations.
        unsafe { self.end_ptr().add(RAW_METADATA_SIZE).cast::<FixedSizeAllocatorMetadata>() }
    }
}
