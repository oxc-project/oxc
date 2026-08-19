//! Main public allocation methods.
//!
//! All ultimately call into `alloc_layout` or `try_alloc_layout`, defined in `alloc_impl.rs`.

use std::{
    alloc::Layout,
    mem::MaybeUninit,
    ptr::{self, NonNull},
    str,
};

use oxc_data_structures::assert_unchecked;

use super::{Arena, bumpalo_alloc::AllocErr, utils::oom};

impl<const MIN_ALIGN: usize> Arena<MIN_ALIGN> {
    /// Allocate an object in this `Arena`. Returns an exclusive reference to it.
    ///
    /// # Panics
    ///
    /// Panics if reserving space for `T` fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// let arena = Arena::new();
    /// let x = arena.alloc("hello");
    /// assert_eq!(*x, "hello");
    /// ```
    #[expect(clippy::mut_from_ref)]
    #[inline(always)]
    pub fn alloc<T>(&self, val: T) -> &mut T {
        // SAFETY: The closure only moves out `val` (already constructed by the caller).
        // It does not allocate from this arena or otherwise touch `Arena`, so `IMPURE_CLOSURE: false` is valid.
        unsafe { self.alloc_with_impl::<false, _, T>(|| val) }
    }

    /// Try to allocate an object in this `Arena`. Returns an exclusive reference to it, if the allocation succeeds.
    ///
    /// # Errors
    ///
    /// Errors if reserving space for `T` fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// let arena = Arena::new();
    /// let x = arena.try_alloc("hello");
    /// assert_eq!(x, Ok(&mut "hello"));
    /// ```
    #[expect(clippy::mut_from_ref)]
    #[inline(always)]
    pub fn try_alloc<T>(&self, val: T) -> Result<&mut T, AllocErr> {
        // SAFETY: The closure only moves out `val` (already constructed by the caller).
        // It does not allocate from this arena or otherwise touch `Arena`, so `IMPURE_CLOSURE: false` is valid.
        unsafe { self.try_alloc_with_impl::<false, _, T>(|| val) }
    }

    /// Pre-allocate space for an object in this `Arena`, and initialize it using the closure.
    /// Returns an exclusive reference to it.
    ///
    /// See [The `_with` Method Suffix](#initializer-functions-the-_with-method-suffix) for a discussion
    /// of the differences between the `_with` suffixed methods and those methods without it,
    /// their performance characteristics, and when you might or might not choose a `_with` suffixed method.
    ///
    /// # Panics
    ///
    /// Panics if reserving space for `T` fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// let arena = Arena::new();
    /// let x = arena.alloc_with(|| "hello");
    /// assert_eq!(*x, "hello");
    /// ```
    #[expect(clippy::mut_from_ref)]
    #[inline(always)]
    pub fn alloc_with<F, T>(&self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        // SAFETY: `IMPURE_CLOSURE: true` imposes no requirement on `f`
        unsafe { self.alloc_with_impl::<true, F, T>(f) }
    }

    /// Implementation of [`alloc`] and [`alloc_with`].
    ///
    /// When `IMPURE_CLOSURE` is `false` ([`alloc`]), sets `cursor_ptr` *after* calling the closure
    /// and writing the result, and asserts to compiler that the write left `start_ptr` unchanged.
    /// This is only valid if `f` closure does not perform further allocations in this arena,
    /// or alter `cursor_ptr` / `start_ptr` by any other means.
    ///
    /// This operation ordering and the assertion lets the compiler keep both `cursor_ptr` and `start_ptr` in registers
    /// across consecutive allocations, instead of reloading them after each value write (which it otherwise cannot
    /// prove does not alias the pointer fields). It also allows compiler to skip alignment calculations on consecutive
    /// allocations which share the same alignment, as it can prove the bump cursor is already aligned.
    ///
    /// When `IMPURE_CLOSURE` is `true` ([`alloc_with`]), sets `cursor_ptr` *before* calling the closure.
    /// The "`start_ptr` is unchanged" assertion (and the snapshot read that feeds it) are compiled away.
    /// This makes it valid for the closure to perform further allocations.
    ///
    /// # SAFETY
    ///
    /// If `IMPURE_CLOSURE` is `false`, `f` must not allocate from this `Arena` or otherwise mutate
    /// `cursor_ptr` or `start_ptr`.
    ///
    /// * [`alloc`]'s `|| val` closure satisfies this, so it passes `IMPURE_CLOSURE: false`.
    /// * An arbitrary [`alloc_with`] closure does not, so it passes `IMPURE_CLOSURE: true`.
    ///
    /// [`alloc`]: Self::alloc
    /// [`alloc_with`]: Self::alloc_with
    #[expect(clippy::mut_from_ref)]
    #[inline(always)]
    unsafe fn alloc_with_impl<const IMPURE_CLOSURE: bool, F, T>(&self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        #[inline(always)]
        fn inner_writer<T, F>(slot: &mut MaybeUninit<T>, f: F) -> &mut T
        where
            F: FnOnce() -> T,
        {
            // This function is translated as:
            // - Allocate space for a T on the stack.
            // - Call `f()` with the return value being put onto this stack space.
            // - memcpy from the stack to the heap.
            //
            // Ideally we want LLVM to always realize that doing a stack allocation is unnecessary and optimize
            // the code so it writes directly into the heap instead. It seems we get it to realize this most
            // consistently if we put this critical line into it's own function instead of inlining it into the
            // surrounding code.
            slot.write(f())
        }

        let layout = Layout::new::<T>();

        // For a pure closure, defer committing `cursor_ptr` until *after* the value is written.
        // This keeps the cursor in a register across consecutive allocations - no store-then-reload after
        // the write, and the redundant alignment mask is dropped on subsequent same-alignment allocations.
        // It is sound because a pure closure does not allocate, so nothing observes the arena between
        // computing the slot and committing the cursor.
        //
        // For an impure closure, commit `cursor_ptr` up-front, so a re-entrant `f` (which may allocate)
        // sees the updated pointer.
        let ptr = self.alloc_layout_impl::<IMPURE_CLOSURE>(layout);

        // Snapshot `start_ptr` as the (fast or slow) layout path left it, so the assertion below can pin it.
        // `start_ptr` is read-only on the fast path, but the value write may alias it as far as compiler can tell,
        // so without this it would be reloaded on the next allocation.
        // Unused, and removed by the optimizer, when `IMPURE_CLOSURE` is `true`.
        let start_ptr = self.start_ptr.get();

        // SAFETY: `ptr` was allocated with `T`'s layout, so it's correctly aligned and sized for `MaybeUninit<T>`
        // (same layout as `T`). The memory was just allocated, so no other reference aliases it,
        // and it lives for as long as the returned reference (tied to `&self`).
        // `MaybeUninit<T>` has no validity invariant, so `&mut` to this uninitialized memory is sound.
        let slot = unsafe { ptr.cast::<MaybeUninit<T>>().as_mut() };
        let value = inner_writer(slot, f);

        if !IMPURE_CLOSURE {
            // Commit the cursor now, *after* the write. `ptr` is the just-allocated slot,
            // and the arena bumps down, so `cursor_ptr` points at the object just allocated.
            self.cursor_ptr.set(ptr.cast::<u8>());

            // SAFETY: The caller guarantees `f` did not mutate `start_ptr`,
            // so `start_ptr` still equals the value snapshotted above
            unsafe { assert_unchecked!(self.start_ptr.get() == start_ptr) };
        }

        value
    }

    /// Try to pre-allocate space for an object in this `Arena`, and initialize it using the closure.
    /// Returns an exclusive reference to it, if the allocation succeeds.
    ///
    /// See [The `_with` Method Suffix](#initializer-functions-the-_with-method-suffix) for a discussion
    /// of the differences between the `_with` suffixed methods and those methods without it,
    /// their performance characteristics, and when you might or might not choose a `_with` suffixed method.
    ///
    /// # Errors
    ///
    /// Errors if reserving space for `T` fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// let arena = Arena::new();
    /// let x = arena.try_alloc_with(|| "hello");
    /// assert_eq!(x, Ok(&mut "hello"));
    /// ```
    #[expect(clippy::mut_from_ref)]
    #[inline(always)]
    pub fn try_alloc_with<F, T>(&self, f: F) -> Result<&mut T, AllocErr>
    where
        F: FnOnce() -> T,
    {
        // SAFETY: `IMPURE_CLOSURE: true` imposes no requirement on `f`
        unsafe { self.try_alloc_with_impl::<true, F, T>(f) }
    }

    /// Implementation of [`try_alloc`] and [`try_alloc_with`].
    ///
    /// When `IMPURE_CLOSURE` is `false` ([`try_alloc`]), sets `cursor_ptr` *after* calling the closure
    /// and writing the result, and asserts to compiler that the write left `start_ptr` unchanged.
    /// This is only valid if `f` closure does not perform further allocations in this arena,
    /// or alter `cursor_ptr` / `start_ptr` by any other means.
    ///
    /// This operation ordering and the assertion lets the compiler keep both `cursor_ptr` and `start_ptr` in registers
    /// across consecutive allocations, instead of reloading them after each value write (which it otherwise cannot
    /// prove does not alias the pointer fields). It also allows compiler to skip alignment calculations on consecutive
    /// allocations which share the same alignment, as it can prove the bump cursor is already aligned.
    ///
    /// When `IMPURE_CLOSURE` is `true` ([`try_alloc_with`]), sets `cursor_ptr` *before* calling the closure.
    /// The "`start_ptr` is unchanged" assertion (and the snapshot read that feeds it) are compiled away.
    /// This makes it valid for the closure to perform further allocations.
    ///
    /// # SAFETY
    ///
    /// If `IMPURE_CLOSURE` is `false`, `f` must not allocate from this `Arena` or otherwise mutate
    /// `cursor_ptr` or `start_ptr`.
    ///
    /// * [`try_alloc`]'s `|| val` closure satisfies this, so it passes `IMPURE_CLOSURE: false`.
    /// * An arbitrary [`try_alloc_with`] closure does not, so it passes `IMPURE_CLOSURE: true`.
    ///
    /// [`try_alloc`]: Self::try_alloc
    /// [`try_alloc_with`]: Self::try_alloc_with
    #[expect(clippy::mut_from_ref)]
    #[inline(always)]
    unsafe fn try_alloc_with_impl<const IMPURE_CLOSURE: bool, F, T>(
        &self,
        f: F,
    ) -> Result<&mut T, AllocErr>
    where
        F: FnOnce() -> T,
    {
        #[inline(always)]
        fn inner_writer<T, F>(slot: &mut MaybeUninit<T>, f: F) -> &mut T
        where
            F: FnOnce() -> T,
        {
            // This function is translated as:
            // - Allocate space for a T on the stack.
            // - Call `f()` with the return value being put onto this stack space.
            // - memcpy from the stack to the heap.
            //
            // Ideally we want LLVM to always realize that doing a stack allocation is unnecessary and optimize
            // the code so it writes directly into the heap instead. It seems we get it to realize this most
            // consistently if we put this critical line into it's own function instead of inlining it into the
            // surrounding code.
            slot.write(f())
        }

        let layout = Layout::new::<T>();

        // For a pure closure, defer committing `cursor_ptr` until *after* the value is written.
        // This keeps the cursor in a register across consecutive allocations - no store-then-reload after
        // the write, and the redundant alignment mask is dropped on subsequent same-alignment allocations.
        // It is sound because a pure closure does not allocate, so nothing observes the arena between
        // computing the slot and committing the cursor.
        //
        // For an impure closure, commit `cursor_ptr` up-front, so a re-entrant `f` (which may allocate)
        // sees the updated pointer.
        let ptr = self.try_alloc_layout_impl::<IMPURE_CLOSURE>(layout)?;

        // Snapshot `start_ptr` as the (fast or slow) layout path left it, so the assertion below can pin it.
        // `start_ptr` is read-only on the fast path, but the value write may alias it as far as compiler can tell,
        // so without this it would be reloaded on the next allocation.
        // Unused, and removed by the optimizer, when `IMPURE_CLOSURE` is `true`.
        let start_ptr = self.start_ptr.get();

        // SAFETY: `ptr` was allocated with `T`'s layout, so it's correctly aligned and sized for `MaybeUninit<T>`
        // (same layout as `T`). The memory was just allocated, so no other reference aliases it,
        // and it lives for as long as the returned reference (tied to `&self`).
        // `MaybeUninit<T>` has no validity invariant, so `&mut` to this uninitialized memory is sound.
        let slot = unsafe { ptr.cast::<MaybeUninit<T>>().as_mut() };
        let value = inner_writer(slot, f);

        if !IMPURE_CLOSURE {
            // Commit the cursor now, *after* the write. `ptr` is the just-allocated slot,
            // and the arena bumps down, so `cursor_ptr` points at the object just allocated.
            self.cursor_ptr.set(ptr.cast::<u8>());

            // SAFETY: The caller guarantees `f` did not mutate `start_ptr`,
            // so `start_ptr` still equals the value snapshotted above
            unsafe { assert_unchecked!(self.start_ptr.get() == start_ptr) };
        }

        Ok(value)
    }

    /// `Copy` a slice into this `Arena` and return an exclusive reference to the copy.
    ///
    /// # Panics
    ///
    /// Panics if reserving space for the slice fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// let arena = Arena::new();
    /// let x = arena.alloc_slice_copy(&[1, 2, 3]);
    /// assert_eq!(x, &[1, 2, 3]);
    /// ```
    #[expect(clippy::mut_from_ref)]
    #[inline(always)]
    pub fn alloc_slice_copy<T>(&self, src: &[T]) -> &mut [T]
    where
        T: Copy,
    {
        let layout = Layout::for_value(src);
        let dst_ptr = self.alloc_layout(layout).cast::<T>();

        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), dst_ptr.as_ptr(), src.len());
            NonNull::slice_from_raw_parts(dst_ptr, src.len()).as_mut()
        }
    }

    /// `Clone` a slice into this `Arena`. Returns an exclusive reference to the clone.
    /// Prefer [`alloc_slice_copy`](#method.alloc_slice_copy) if `T` is `Copy`.
    ///
    /// # Panics
    ///
    /// Panics if reserving space for the slice fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// #[derive(Clone, Debug, Eq, PartialEq)]
    /// struct Sheep {
    ///     name: &'static str,
    /// }
    ///
    /// let originals = [
    ///     Sheep { name: "Alice" },
    ///     Sheep { name: "Bob" },
    ///     Sheep { name: "Cathy" },
    /// ];
    ///
    /// let arena = Arena::new();
    /// let clones = arena.alloc_slice_clone(&originals);
    /// assert_eq!(originals, clones);
    /// ```
    #[expect(clippy::mut_from_ref)]
    #[inline(always)]
    pub fn alloc_slice_clone<T>(&self, src: &[T]) -> &mut [T]
    where
        T: Clone,
    {
        let layout = Layout::for_value(src);
        let dst_ptr = self.alloc_layout(layout).cast::<T>();

        unsafe {
            for (i, val) in src.iter().cloned().enumerate() {
                dst_ptr.add(i).write(val);
            }

            NonNull::slice_from_raw_parts(dst_ptr, src.len()).as_mut()
        }
    }

    /// `Copy` a string slice into this `Arena`. Returns an exclusive reference to it.
    ///
    /// # Panics
    ///
    /// Panics if reserving space for the string fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// let arena = Arena::new();
    /// let hello = arena.alloc_str("hello world");
    /// assert_eq!("hello world", hello);
    /// ```
    #[expect(clippy::mut_from_ref)]
    #[inline(always)]
    pub fn alloc_str(&self, src: &str) -> &mut str {
        let buffer = self.alloc_slice_copy(src.as_bytes());
        unsafe {
            // This is OK, because it already came in as str, so it is guaranteed to be UTF-8
            str::from_utf8_unchecked_mut(buffer)
        }
    }

    /// Allocate a new slice of size `len` into this `Arena`. Returns an exclusive reference to the slice.
    ///
    /// The elements of the slice are initialized using the supplied closure.
    /// The closure argument is the position in the slice.
    ///
    /// # Panics
    ///
    /// Panics if reserving space for the slice fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// let arena = Arena::new();
    /// let x = arena.alloc_slice_fill_with(5, |i| 5 * (i + 1));
    /// assert_eq!(x, &[5, 10, 15, 20, 25]);
    /// ```
    #[expect(clippy::mut_from_ref)]
    #[inline(always)]
    pub fn alloc_slice_fill_with<T, F>(&self, len: usize, mut f: F) -> &mut [T]
    where
        F: FnMut(usize) -> T,
    {
        let layout = Layout::array::<T>(len).unwrap_or_else(|_| oom());
        let dst_ptr = self.alloc_layout(layout).cast::<T>();

        unsafe {
            for i in 0..len {
                dst_ptr.add(i).write(f(i));
            }

            let result = NonNull::slice_from_raw_parts(dst_ptr, len).as_mut();
            debug_assert_eq!(Layout::for_value(result), layout);
            result
        }
    }

    /// Allocate a new slice of size `len` into this `Arena`. Returns an exclusive reference to the copy.
    ///
    /// All elements of the slice are initialized to `value`.
    ///
    /// # Panics
    ///
    /// Panics if reserving space for the slice fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// let arena = Arena::new();
    /// let x = arena.alloc_slice_fill_copy(5, 42);
    /// assert_eq!(x, &[42, 42, 42, 42, 42]);
    /// ```
    #[inline(always)]
    pub fn alloc_slice_fill_copy<T: Copy>(&self, len: usize, value: T) -> &mut [T] {
        self.alloc_slice_fill_with(len, |_| value)
    }

    /// Allocate a new slice of size `len` into this `Arena`. Return an exclusive reference to the clone.
    ///
    /// All elements of the slice are initialized to `value.clone()`.
    ///
    /// # Panics
    ///
    /// Panics if reserving space for the slice fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// #[derive(Clone, Debug, Eq, PartialEq)]
    /// struct Sheep {
    ///     name: &'static str,
    /// }
    ///
    /// let arena = Arena::new();
    /// let s = Sheep { name: "Flossy" };
    /// let x: &[Sheep] = arena.alloc_slice_fill_clone(2, &s);
    /// assert_eq!(x.len(), 2);
    /// assert_eq!(&x[0], &s);
    /// assert_eq!(&x[1], &s);
    /// ```
    #[inline(always)]
    pub fn alloc_slice_fill_clone<T: Clone>(&self, len: usize, value: &T) -> &mut [T] {
        self.alloc_slice_fill_with(len, |_| value.clone())
    }

    /// Allocate a new slice of size `len` into this `Arena`. Returns an exclusive reference to the slice.
    ///
    /// The elements are initialized using the supplied iterator.
    ///
    /// # Panics
    ///
    /// Panics if reserving space for the slice fails, or if the supplied iterator returns fewer elements than
    /// it promised.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// let arena = Arena::new();
    /// let x: &[i32] = arena.alloc_slice_fill_iter([2, 3, 5].iter().cloned().map(|i| i * i));
    /// assert_eq!(x, [4, 9, 25]);
    /// ```
    #[inline(always)]
    pub fn alloc_slice_fill_iter<T, I>(&self, iter: I) -> &mut [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut iter = iter.into_iter();
        self.alloc_slice_fill_with(iter.len(), |_| {
            iter.next().expect("Iterator supplied too few elements")
        })
    }

    /// Allocate a new slice of size `len` into this `Arena`. Returns an exclusive reference to the slice.
    ///
    /// All elements of the slice are initialized to [`T::default()`].
    ///
    /// [`T::default()`]: https://doc.rust-lang.org/std/default/trait.Default.html#tymethod.default
    ///
    /// # Panics
    ///
    /// Panics if reserving space for the slice fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// let arena = Arena::new();
    /// let x = arena.alloc_slice_fill_default::<u32>(5);
    /// assert_eq!(x, &[0, 0, 0, 0, 0]);
    /// ```
    #[inline(always)]
    pub fn alloc_slice_fill_default<T: Default>(&self, len: usize) -> &mut [T] {
        self.alloc_slice_fill_with(len, |_| T::default())
    }
}
