//! Main public allocation methods.
//!
//! All ultimately call into `alloc_layout` or `try_alloc_layout`, defined in `alloc_impl.rs`.

use std::{
    alloc::Layout,
    ptr::{self},
    slice, str,
};

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
    #[inline(always)]
    pub fn alloc<T>(&self, val: T) -> &mut T {
        self.alloc_with(|| val)
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
    #[inline(always)]
    pub fn try_alloc<T>(&self, val: T) -> Result<&mut T, AllocErr> {
        self.try_alloc_with(|| val)
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
        #[inline(always)]
        unsafe fn inner_writer<T, F>(ptr: *mut T, f: F)
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
            unsafe { ptr::write(ptr, f()) };
        }

        let layout = Layout::new::<T>();

        unsafe {
            let ptr = self.alloc_layout(layout);
            let ptr = ptr.as_ptr().cast::<T>();
            inner_writer(ptr, f);
            &mut *ptr
        }
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
        #[inline(always)]
        unsafe fn inner_writer<T, F>(ptr: *mut T, f: F)
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
            unsafe { ptr::write(ptr, f()) };
        }

        // Self-contained: `ptr` is allocated for `T` and then a `T` is written.
        let layout = Layout::new::<T>();
        let ptr = self.try_alloc_layout(layout)?;
        let ptr = ptr.as_ptr().cast::<T>();

        unsafe {
            inner_writer(ptr, f);
            Ok(&mut *ptr)
        }
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
            slice::from_raw_parts_mut(dst_ptr.as_ptr(), src.len())
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
    ///     name: String,
    /// }
    ///
    /// let originals = [
    ///     Sheep { name: "Alice".into() },
    ///     Sheep { name: "Bob".into() },
    ///     Sheep { name: "Cathy".into() },
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
                ptr::write(dst_ptr.as_ptr().add(i), val);
            }

            slice::from_raw_parts_mut(dst_ptr.as_ptr(), src.len())
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
                ptr::write(dst_ptr.as_ptr().add(i), f(i));
            }

            let result = slice::from_raw_parts_mut(dst_ptr.as_ptr(), len);
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
    /// let arena = Arena::new();
    /// let s: String = "Hello Arena!".to_string();
    /// let x: &[String] = arena.alloc_slice_fill_clone(2, &s);
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
