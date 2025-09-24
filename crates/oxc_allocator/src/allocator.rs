use std::{
    alloc::Layout,
    ptr::{self, NonNull},
    slice, str,
};

#[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
use std::mem::offset_of;

use bumpalo::Bump;

use oxc_data_structures::assert_unchecked;

#[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
use crate::tracking::AllocationStats;

/// A bump-allocated memory arena.
///
/// # Anatomy of an Allocator
///
/// [`Allocator`] is flexibly sized. It grows as required as you allocate data into it.
///
/// To do that, an [`Allocator`] consists of multiple memory chunks.
///
/// [`Allocator::new`] creates a new allocator without any chunks. When you first allocate an object
/// into it, it will lazily create an initial chunk, the size of which is determined by the size of that
/// first allocation.
///
/// As more data is allocated into the [`Allocator`], it will likely run out of capacity. At that point,
/// a new memory chunk is added, and further allocations will use this new chunk (until it too runs out
/// of capacity, and *another* chunk is added).
///
/// The data from the 1st chunk is not copied into the 2nd one. It stays where it is, which means
/// `&` or `&mut` references to data in the first chunk remain valid. This is unlike e.g. `Vec` which
/// copies all existing data when it grows.
///
/// Each chunk is at least double the size of the last one, so growth in capacity is exponential.
///
/// [`Allocator::reset`] keeps only the last chunk (the biggest one), and discards any other chunks,
/// returning their memory to the global allocator. The last chunk has its cursor rewound back to
/// the start, so it's empty, ready to be re-used for allocating more data.
///
/// # Recycling allocators
///
/// For good performance, it's ideal to create an [`Allocator`], and re-use it over and over, rather than
/// repeatedly creating and dropping [`Allocator`]s.
///
/// ```
/// // This is good!
/// use oxc_allocator::Allocator;
/// let mut allocator = Allocator::new();
///
/// # fn do_stuff(_n: usize, _allocator: &Allocator) {}
/// for i in 0..100 {
///     do_stuff(i, &allocator);
///     // Reset the allocator, freeing the memory used by `do_stuff`
///     allocator.reset();
/// }
/// ```
///
/// ```
/// // DON'T DO THIS!
/// # use oxc_allocator::Allocator;
/// # fn do_stuff(_n: usize, _allocator: &Allocator) {}
/// for i in 0..100 {
///     let allocator = Allocator::new();
///     do_stuff(i, &allocator);
/// }
/// ```
///
/// ```
/// // DON'T DO THIS EITHER!
/// # use oxc_allocator::Allocator;
/// # let allocator = Allocator::new();
/// # fn do_stuff(_n: usize, _allocator: &Allocator) {}
/// for i in 0..100 {
///     do_stuff(i, &allocator);
///     // We haven't reset the allocator, so we haven't freed the memory used by `do_stuff`.
///     // The allocator will grow and grow, consuming more and more memory.
/// }
/// ```
///
/// ## Why is re-using an [`Allocator`] good for performance?
///
/// 3 reasons:
///
/// #### 1. Avoid expensive system calls
///
/// Creating an [`Allocator`] is a fairly expensive operation as it involves a call into global allocator,
/// which in turn will likely make a system call. Ditto when the [`Allocator`] is dropped.
/// Re-using an existing [`Allocator`] avoids these costs.
///
/// #### 2. CPU cache
///
/// Re-using an existing allocator means you're re-using the same block of memory. If that memory was
/// recently accessed, it's likely to be warm in the CPU cache, so memory accesses will be much faster
/// than accessing "cold" sections of main memory.
///
/// This can have a very significant positive impact on performance.
///
/// #### 3. Capacity stabilization
///
/// The most efficient [`Allocator`] is one with only 1 chunk which has sufficient capacity for
/// everything you're going to allocate into it.
///
/// Why?
///
/// 1. Every allocation will occur without the allocator needing to grow.
///
/// 2. This makes the "is there sufficient capacity to allocate this?" check in [`alloc`] completely
///    predictable (the answer is always "yes"). The CPU's branch predictor swiftly learns this,
///    speeding up operation.
///
/// 3. When the [`Allocator`] is reset, there are no excess chunks to discard, so no system calls.
///
/// Because [`reset`] keeps only the biggest chunk (see above), re-using the same [`Allocator`]
/// for multiple similar workloads will result in the [`Allocator`] swiftly stabilizing at a capacity
/// which is sufficient to service those workloads with a single chunk.
///
/// If workload is completely uniform, it reaches stable state on the 3rd round.
///
/// ```
/// # use oxc_allocator::Allocator;
/// let mut allocator = Allocator::new();
///
/// fn workload(allocator: &Allocator) {
///     // Allocate 4 MB of data in small chunks
///     for i in 0..1_000_000u32 {
///         allocator.alloc(i);
///     }
/// }
///
/// // 1st round
/// workload(&allocator);
///
/// // `allocator` has capacity for 4 MB data, but split into many chunks.
/// // `reset` throws away all chunks except the last one which will be approx 2 MB.
/// allocator.reset();
///
/// // 2nd round
/// workload(&allocator);
///
/// // `workload` filled the 2 MB chunk, so a 2nd chunk was created of double the size (4 MB).
/// // `reset` discards the smaller chunk, leaving only a single 4 MB chunk.
/// allocator.reset();
///
/// // 3rd round
/// // `allocator` now has sufficient capacity for all allocations in a single 4 MB chunk.
/// workload(&allocator);
///
/// // `reset` has no chunks to discard. It keeps the single 4 MB chunk. No system calls.
/// allocator.reset();
///
/// // More rounds
/// // All serviced without needing to grow the allocator, and with no system calls.
/// for _ in 0..100 {
///   workload(&allocator);
///   allocator.reset();
/// }
/// ```
///
/// # No `Drop`s
///
/// Objects allocated into Oxc memory arenas are never [`Dropped`](Drop).
/// Memory is released in bulk when the allocator is dropped, without dropping the individual
/// objects in the arena.
///
/// Therefore, it would produce a memory leak if you allocated [`Drop`] types into the arena
/// which own memory allocations outside the arena.
///
/// Static checks make this impossible to do. [`Allocator::alloc`], [`Box::new_in`], [`Vec::new_in`],
/// [`HashMap::new_in`], and all other methods which store data in the arena will refuse to compile
/// if called with a [`Drop`] type.
///
/// ```compile_fail
/// use oxc_allocator::{Allocator, Box};
/// let allocator = Allocator::new();
///
/// struct Foo {
///     pub a: i32
/// }
///
/// impl std::ops::Drop for Foo {
///     fn drop(&mut self) {}
/// }
///
/// // This will fail to compile because `Foo` implements `Drop`
/// let foo = Box::new_in(Foo { a: 0 }, &allocator);
/// ```
///
/// ```compile_fail
/// use oxc_allocator::{Allocator, Box};
/// let allocator = Allocator::new();
///
/// struct Bar {
///     v: std::vec::Vec<u8>,
/// }
///
/// // This will fail to compile because `Bar` contains a `std::vec::Vec`, and it implements `Drop`
/// let bar = Box::new_in(Bar { v: vec![1, 2, 3] }, &allocator);
/// ```
///
/// # Examples
///
/// Consumers of the [`oxc` umbrella crate](https://crates.io/crates/oxc) pass
/// [`Allocator`] references to other tools.
///
/// ```ignore
/// use oxc::{allocator::Allocator, parser::Parser, span::SourceType};
///
/// let allocator = Allocator::default();
/// let parsed = Parser::new(&allocator, "let x = 1;", SourceType::default());
/// assert!(parsed.errors.is_empty());
/// ```
///
/// [`reset`]: Allocator::reset
/// [`alloc`]: Allocator::alloc
/// [`Box::new_in`]: crate::Box::new_in
/// [`Vec::new_in`]: crate::Vec::new_in
/// [`HashMap::new_in`]: crate::HashMap::new_in
#[derive(Default)]
pub struct Allocator {
    bump: Bump,
    /// Used to track number of allocations made in this allocator when `track_allocations` feature is enabled
    #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
    pub(crate) stats: AllocationStats,
}

/// Offset of `stats` field, relative to `bump` field.
/// Used in `tracking` module for allocation tracking.
#[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
#[expect(clippy::cast_possible_wrap)]
pub const STATS_FIELD_OFFSET: isize =
    (offset_of!(Allocator, stats) as isize) - (offset_of!(Allocator, bump) as isize);

impl Allocator {
    /// Create a new [`Allocator`] with no initial capacity.
    ///
    /// This method does not reserve any memory to back the allocator. Memory for allocator's initial
    /// chunk will be reserved lazily, when you make the first allocation into this [`Allocator`]
    /// (e.g. with [`Allocator::alloc`], [`Box::new_in`], [`Vec::new_in`], [`HashMap::new_in`]).
    ///
    /// If you can estimate the amount of memory the allocator will require to fit what you intend to
    /// allocate into it, it is generally preferable to create that allocator with [`with_capacity`],
    /// which reserves that amount of memory upfront. This will avoid further system calls to allocate
    /// further chunks later on. This point is less important if you're re-using the allocator multiple
    /// times.
    ///
    /// See [`Allocator`] docs for more information on efficient use of [`Allocator`].
    ///
    /// [`with_capacity`]: Allocator::with_capacity
    /// [`Box::new_in`]: crate::Box::new_in
    /// [`Vec::new_in`]: crate::Vec::new_in
    /// [`HashMap::new_in`]: crate::HashMap::new_in
    //
    // `#[inline(always)]` because just delegates to `bumpalo` method
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            bump: Bump::new(),
            #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
            stats: AllocationStats::default(),
        }
    }

    /// Create a new [`Allocator`] with specified capacity.
    ///
    /// See [`Allocator`] docs for more information on efficient use of [`Allocator`].
    //
    // `#[inline(always)]` because just delegates to `bumpalo` method
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bump: Bump::with_capacity(capacity),
            #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
            stats: AllocationStats::default(),
        }
    }

    /// Allocate an object in this [`Allocator`] and return an exclusive reference to it.
    ///
    /// # Panics
    /// Panics if reserving space for `T` fails.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::Allocator;
    ///
    /// let allocator = Allocator::default();
    /// let x = allocator.alloc([1u8; 20]);
    /// assert_eq!(x, &[1u8; 20]);
    /// ```
    //
    // `#[inline(always)]` because this is a very hot path and `Bump::alloc` is a very small function.
    // We always want it to be inlined.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn alloc<T>(&self, val: T) -> &mut T {
        const { assert!(!std::mem::needs_drop::<T>(), "Cannot allocate Drop type in arena") };

        #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
        self.stats.record_allocation();

        self.bump.alloc(val)
    }

    /// Copy a string slice into this [`Allocator`] and return a reference to it.
    ///
    /// # Panics
    /// Panics if reserving space for the string fails.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::Allocator;
    /// let allocator = Allocator::default();
    /// let hello = allocator.alloc_str("hello world");
    /// assert_eq!(hello, "hello world");
    /// ```
    //
    // `#[inline(always)]` because this is a hot path and `Bump::alloc_str` is a very small function.
    // We always want it to be inlined.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn alloc_str<'alloc>(&'alloc self, src: &str) -> &'alloc str {
        #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
        self.stats.record_allocation();

        self.bump.alloc_str(src)
    }

    /// `Copy` a slice into this `Bump` and return an exclusive reference to the copy.
    ///
    /// # Panics
    /// Panics if reserving space for the slice fails.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::Allocator;
    /// let allocator = Allocator::default();
    /// let x = allocator.alloc_slice_copy(&[1, 2, 3]);
    /// assert_eq!(x, &[1, 2, 3]);
    /// ```
    // `#[inline(always)]` because this is a hot path and `Bump::alloc_slice_copy` is a very small function.
    // We always want it to be inlined.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn alloc_slice_copy<T: Copy>(&self, src: &[T]) -> &mut [T] {
        #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
        self.stats.record_allocation();

        self.bump.alloc_slice_copy(src)
    }

    /// Allocate space for an object with the given [`Layout`].
    ///
    /// The returned pointer points at uninitialized memory, and should be initialized with
    /// [`std::ptr::write`].
    ///
    /// # Panics
    ///
    /// Panics if reserving space matching `layout` fails.
    pub fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
        self.stats.record_allocation();

        self.bump.alloc_layout(layout)
    }

    /// Create new `&str` from a fixed-size array of `&str`s concatenated together,
    /// allocated in the given `allocator`.
    ///
    /// # Panics
    ///
    /// Panics if the sum of length of all strings exceeds `isize::MAX`.
    ///
    /// # Example
    /// ```
    /// use oxc_allocator::Allocator;
    ///
    /// let allocator = Allocator::new();
    /// let s = allocator.alloc_concat_strs_array(["hello", " ", "world", "!"]);
    /// assert_eq!(s, "hello world!");
    /// ```
    // `#[inline(always)]` because want compiler to be able to remove checked addition where some of
    // `strings` are statically known.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn alloc_concat_strs_array<'a, const N: usize>(&'a self, strings: [&str; N]) -> &'a str {
        // Calculate total length of all the strings concatenated.
        //
        // We have to use `checked_add` here to guard against additions wrapping around
        // if some of the input `&str`s are very long, or there's many of them.
        //
        // However, `&str`s have max length of `isize::MAX`.
        // https://users.rust-lang.org/t/does-str-reliably-have-length-isize-max/126777
        // Use `assert_unchecked!` to communicate this invariant to compiler, which allows it to
        // optimize out the overflow checks where some of `strings` are static, so their size is known.
        //
        // e.g. `allocator.from_strs_array_in(["__vite_ssr_import_", str, "__"])`, for example,
        // requires no checks at all, because the static parts have total length of 20 bytes,
        // and `str` has max length of `isize::MAX`. `isize::MAX as usize + 20` cannot overflow `usize`.
        // Compiler can see that, and removes the overflow check.
        // https://godbolt.org/z/MGh44Yz5d
        #[expect(clippy::checked_conversions)]
        let total_len = strings.iter().fold(0usize, |total_len, s| {
            let len = s.len();
            // SAFETY: `&str`s have maximum length of `isize::MAX`
            unsafe { assert_unchecked!(len <= (isize::MAX as usize)) };
            total_len.checked_add(len).unwrap()
        });
        assert!(
            isize::try_from(total_len).is_ok(),
            "attempted to create a string longer than `isize::MAX` bytes"
        );

        #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
        self.stats.record_allocation();

        // Create actual `&str` in a separate function, to ensure that `alloc_concat_strs_array`
        // is inlined, so that compiler has knowledge to remove the overflow checks above.
        // When some of `strings` are static, this function is usually only a few instructions.
        // Compiler can choose whether or not to inline `alloc_concat_strs_array_with_total_len_in`.
        // SAFETY: `total_len` has been calculated correctly above.
        // `total_len` is `<= isize::MAX`.
        unsafe { self.alloc_concat_strs_array_with_total_len_in(strings, total_len) }
    }

    /// Create a new `&str` from a fixed-size array of `&str`s concatenated together,
    /// allocated in the given `allocator`, with provided `total_len`.
    ///
    /// # SAFETY
    /// * `total_len` must be the total length of all `strings` concatenated.
    /// * `total_len` must be `<= isize::MAX`.
    unsafe fn alloc_concat_strs_array_with_total_len_in<'a, const N: usize>(
        &'a self,
        strings: [&str; N],
        total_len: usize,
    ) -> &'a str {
        if total_len == 0 {
            return "";
        }

        // Allocate `total_len` bytes.
        // SAFETY: Caller guarantees `total_len <= isize::MAX`.
        let layout = unsafe { Layout::from_size_align_unchecked(total_len, 1) };
        let start_ptr = self.bump().alloc_layout(layout);

        let mut end_ptr = start_ptr;
        for str in strings {
            let src_ptr = str.as_ptr();
            let len = str.len();

            // SAFETY:
            // `src` is obtained from a `&str` with length `len`, so is valid for reading `len` bytes.
            // `end_ptr` is within bounds of the allocation. So is `end_ptr + len`.
            // `u8` has no alignment requirements, so `src_ptr` and `end_ptr` are sufficiently aligned.
            // No overlapping, because we're copying from an existing `&str` to a newly allocated buffer.
            unsafe { ptr::copy_nonoverlapping(src_ptr, end_ptr.as_ptr(), len) };

            // SAFETY: We allocated sufficient capacity for all the strings concatenated.
            // So `end_ptr.add(len)` cannot go out of bounds.
            end_ptr = unsafe { end_ptr.add(len) };
        }

        debug_assert_eq!(end_ptr.as_ptr() as usize - start_ptr.as_ptr() as usize, total_len);

        // SAFETY: We have allocated and filled `total_len` bytes starting at `start_ptr`.
        // Concatenating multiple `&str`s results in a valid UTF-8 string.
        unsafe {
            let slice = slice::from_raw_parts(start_ptr.as_ptr(), total_len);
            str::from_utf8_unchecked(slice)
        }
    }

    /// Reset this allocator.
    ///
    /// Performs mass deallocation on everything allocated in this arena by resetting the pointer
    /// into the underlying chunk of memory to the start of the chunk.
    /// Does not run any `Drop` implementations on deallocated objects.
    ///
    /// If this arena has allocated multiple chunks to bump allocate into, then the excess chunks
    /// are returned to the global allocator.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::Allocator;
    ///
    /// let mut allocator = Allocator::default();
    ///
    /// // Allocate a bunch of things.
    /// {
    ///     for i in 0..100 {
    ///         allocator.alloc(i);
    ///     }
    /// }
    ///
    /// // Reset the arena.
    /// allocator.reset();
    ///
    /// // Allocate some new things in the space previously occupied by the
    /// // original things.
    /// for j in 200..400 {
    ///     allocator.alloc(j);
    /// }
    /// ```
    //
    // `#[inline(always)]` because it just delegates to `bumpalo`
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn reset(&mut self) {
        #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
        self.stats.reset();

        self.bump.reset();
    }

    /// Calculate the total capacity of this [`Allocator`] including all chunks, in bytes.
    ///
    /// Note: This is the total amount of memory the [`Allocator`] owns NOT the total size of data
    /// that's been allocated in it. If you want the latter, use [`used_bytes`] instead.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::Allocator;
    ///
    /// let capacity = 64 * 1024; // 64 KiB
    /// let mut allocator = Allocator::with_capacity(capacity);
    /// allocator.alloc(123u64); // 8 bytes
    ///
    /// // Result is the capacity (64 KiB), not the size of allocated data (8 bytes).
    /// // `Allocator::with_capacity` may allocate a bit more than requested.
    /// assert!(allocator.capacity() >= capacity);
    /// ```
    ///
    /// [`used_bytes`]: Allocator::used_bytes
    //
    // `#[inline(always)]` because it just delegates to `bumpalo`
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.bump.allocated_bytes()
    }

    /// Calculate the total size of data used in this [`Allocator`], in bytes.
    ///
    /// This is the total amount of memory that has been *used* in the [`Allocator`], NOT the amount of
    /// memory the [`Allocator`] owns. If you want the latter, use [`capacity`] instead.
    ///
    /// The result includes:
    ///
    /// 1. Padding bytes between objects which have been allocated to preserve alignment of types
    ///    where they have different alignments or have larger-than-typical alignment.
    /// 2. Excess capacity in [`Vec`]s, [`StringBuilder`]s and [`HashMap`]s.
    /// 3. Objects which were allocated but later dropped. [`Allocator`] does not re-use allocations,
    ///    so anything which is allocated into arena continues to take up "dead space", even after it's
    ///    no longer referenced anywhere.
    /// 4. "Dead space" left over where a [`Vec`], [`StringBuilder`] or [`HashMap`] has grown and had to
    ///    make a new allocation to accommodate its new larger size. Its old allocation continues to
    ///    take up "dead" space in the allocator, unless it was the most recent allocation.
    ///
    /// In practice, this almost always means that the result returned from this function will be an
    /// over-estimate vs the amount of "live" data in the arena.
    ///
    /// However, if you are using the result of this method to create a new `Allocator` to clone
    /// an AST into, it is theoretically possible (though very unlikely) that it may be a slight
    /// under-estimate of the capacity required in new allocator to clone the AST into, depending
    /// on the order that `&str`s were allocated into arena in parser vs the order they get allocated
    /// during cloning. The order allocations are made in affects the amount of padding bytes required.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let capacity = 64 * 1024; // 64 KiB
    /// let mut allocator = Allocator::with_capacity(capacity);
    ///
    /// allocator.alloc(1u8); // 1 byte with alignment 1
    /// allocator.alloc(2u8); // 1 byte with alignment 1
    /// allocator.alloc(3u64); // 8 bytes with alignment 8
    ///
    /// // Only 10 bytes were allocated, but 16 bytes were used, in order to align `3u64` on 8
    /// assert_eq!(allocator.used_bytes(), 16);
    ///
    /// allocator.reset();
    ///
    /// let mut vec = Vec::<u64>::with_capacity_in(2, &allocator);
    ///
    /// // Allocate something else, so `vec`'s allocation is not the most recent
    /// allocator.alloc(123u64);
    ///
    /// // `vec` has to grow beyond it's initial capacity
    /// vec.extend([1, 2, 3, 4]);
    ///
    /// // `vec` takes up 32 bytes, and `123u64` takes up 8 bytes = 40 total.
    /// // But there's an additional 16 bytes consumed for `vec`'s original capacity of 2,
    /// // which is still using up space
    /// assert_eq!(allocator.used_bytes(), 56);
    /// ```
    ///
    /// [`capacity`]: Allocator::capacity
    /// [`Vec`]: crate::Vec
    /// [`StringBuilder`]: crate::StringBuilder
    /// [`HashMap`]: crate::HashMap
    pub fn used_bytes(&self) -> usize {
        let mut bytes = 0;
        // SAFETY: No allocations are made while `chunks_iter` is alive. No data is read from the chunks.
        let chunks_iter = unsafe { self.bump.iter_allocated_chunks_raw() };
        for (_, size) in chunks_iter {
            bytes += size;
        }
        bytes
    }

    /// Get inner [`bumpalo::Bump`].
    ///
    /// This method is not public. We don't want to expose `Bump` to user.
    /// The fact that we're using `bumpalo` is an internal implementation detail.
    //
    // `#[inline(always)]` because it's a no-op
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(crate) fn bump(&self) -> &Bump {
        &self.bump
    }

    /// Create [`Allocator`] from a [`bumpalo::Bump`].
    ///
    /// This method is not public. Only used by [`Allocator::from_raw_parts`].
    //
    // `#[inline(always)]` because it's a no-op
    #[cfg(feature = "from_raw_parts")]
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(crate) fn from_bump(bump: Bump) -> Self {
        Self {
            bump,
            #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
            stats: AllocationStats::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Allocator;

    #[test]
    fn test_api() {
        let mut allocator = Allocator::default();
        {
            let array = allocator.alloc([123; 10]);
            assert_eq!(array, &[123; 10]);
            let str = allocator.alloc_str("hello");
            assert_eq!(str, "hello");
        }
        allocator.reset();
    }

    #[test]
    fn string_from_array_len_1() {
        let allocator = Allocator::default();
        let s = allocator.alloc_concat_strs_array(["hello"]);
        assert_eq!(s, "hello");
    }

    #[test]
    fn string_from_array_len_2() {
        let allocator = Allocator::default();
        let s = allocator.alloc_concat_strs_array(["hello", "world!"]);
        assert_eq!(s, "helloworld!");
    }

    #[test]
    fn string_from_array_len_3() {
        let hello = "hello";
        let world = std::string::String::from("world");
        let allocator = Allocator::default();
        let s = allocator.alloc_concat_strs_array([hello, &world, "!"]);
        assert_eq!(s, "helloworld!");
    }

    #[test]
    fn string_from_empty_array() {
        let allocator = Allocator::default();
        let s = allocator.alloc_concat_strs_array([]);
        assert_eq!(s, "");
    }

    #[test]
    fn string_from_array_of_empty_strs() {
        let allocator = Allocator::default();
        let s = allocator.alloc_concat_strs_array(["", "", ""]);
        assert_eq!(s, "");
    }

    #[test]
    fn string_from_array_containing_some_empty_strs() {
        let allocator = Allocator::default();
        let s = allocator.alloc_concat_strs_array(["", "hello", ""]);
        assert_eq!(s, "hello");
    }
}
