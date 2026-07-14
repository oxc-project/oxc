//! A reusable scratch stack for building AST node lists.
//!
//! The parser builds many lists of AST nodes (statements, array/object elements,
//! call arguments, class members, ...). Building each list directly in the arena
//! is expensive: an arena [`Vec`] starts empty and grows `1 → 2 → 4 → 8 → …`, and
//! **every** grow copies the existing contents (arena reallocation always
//! `memmove`s, even when the buffer is the last allocation).
//!
//! [`ScratchStack`] avoids this. It is a single, persistent, heap-backed buffer,
//! reused for every list in a parse. A caller records a [`mark`], pushes the
//! list's elements, then [`drain_into`] moves them into the arena in one
//! exact-size allocation and rewinds the stack to the mark. Because the buffer is
//! reused, after a brief warm-up it stops growing, so its own growth cost
//! amortizes to ~zero across the parse — and no list ever `memmove`s while being
//! built.
//!
//! One buffer serves every element type: elements are stored as raw bytes, packed
//! by their natural size/alignment. This is sound because the buffer is used
//! strictly LIFO (recursive-descent nesting marks above its parent's run) and all
//! AST list-element types are non-[`Drop`] with alignment `<= SCRATCH_ALIGN`.
//!
//! [`Vec`]: oxc_allocator::Vec
//! [`mark`]: ScratchStack::mark
//! [`drain_into`]: ScratchStack::drain_into

use std::{
    alloc::{self, Layout},
    cmp,
    marker::PhantomData,
    mem::{align_of, needs_drop, size_of},
    ptr::{self, NonNull},
};

use oxc_allocator::{Allocator, ArenaVec};

/// Alignment of the scratch buffer.
///
/// All AST node types have alignment `<= 8`; 16 is used to be safe and
/// future-proof. [`ScratchStack::push`] and [`ScratchStack::drain_into`] assert
/// `align_of::<T>() <= SCRATCH_ALIGN` at compile time.
const SCRATCH_ALIGN: usize = 16;

/// Initial capacity of the scratch buffer, in bytes.
///
/// Sized so that the buffer rarely (if ever) needs to grow during a parse. Growth
/// is handled gracefully regardless; this only affects warm-up.
const INITIAL_CAPACITY: usize = 8192;

const _: () = assert!(INITIAL_CAPACITY > 0 && INITIAL_CAPACITY <= isize::MAX as usize);

/// A reusable, persistent, byte-addressed LIFO stack for accumulating AST node
/// lists before moving them into the arena.
///
/// See the [module documentation](self) for the rationale.
///
/// # Usage invariant
///
/// Used strictly LIFO. Between a [`mark`] and the matching [`drain_into`], all
/// pushes must be of the same type `T` (the element type of the list being
/// built). Nested lists must mark above the parent's run and drain (rewinding to
/// their own mark) before the parent pushes again. Recursive-descent parsing
/// satisfies this naturally.
///
/// [`mark`]: ScratchStack::mark
/// [`drain_into`]: ScratchStack::drain_into
pub struct ScratchStack {
    /// Start of the allocation. Dangling (and never dereferenced) when `capacity == 0`.
    ptr: NonNull<u8>,
    /// Number of bytes currently in use (the stack top).
    len: usize,
    /// Number of bytes allocated.
    capacity: usize,
}

// `ScratchStack` owns its heap buffer exclusively, like `Vec<u8>`, so it is safe
// to move across threads. The buffer only ever holds plain bytes.
// SAFETY: The buffer is owned exclusively and contains no thread-affine state.
unsafe impl Send for ScratchStack {}

impl ScratchStack {
    /// Create a new [`ScratchStack`] with the default initial capacity.
    pub fn new() -> Self {
        let layout = Self::layout(INITIAL_CAPACITY);
        // SAFETY: `layout` has non-zero size (`INITIAL_CAPACITY > 0`).
        let ptr = unsafe { alloc::alloc(layout) };
        let Some(ptr) = NonNull::new(ptr) else { alloc::handle_alloc_error(layout) };
        Self { ptr, len: 0, capacity: INITIAL_CAPACITY }
    }

    #[inline]
    fn layout(capacity: usize) -> Layout {
        debug_assert!(capacity > 0);
        // `SCRATCH_ALIGN` is a valid power-of-two alignment. This only panics if
        // `capacity` rounded up to `SCRATCH_ALIGN` overflows `isize::MAX`, which
        // does not happen for the capacities used here.
        Layout::from_size_align(capacity, SCRATCH_ALIGN).unwrap_or_else(|_| capacity_overflow())
    }

    /// Record the current stack top. Pass the returned mark to [`drain_into`]
    /// after pushing this list's elements.
    ///
    /// [`drain_into`]: ScratchStack::drain_into
    #[inline]
    pub fn mark(&self) -> usize {
        self.len
    }

    /// Number of `T` elements pushed since `mark` (without draining).
    ///
    /// Used to compute a list-relative index while a list is still being built.
    #[inline]
    pub fn count_since<T>(&self, mark: usize) -> usize {
        let offset = align_up(mark, align_of::<T>());
        if self.len > offset { (self.len - offset) / size_of::<T>() } else { 0 }
    }

    /// Push a value onto the stack.
    #[inline]
    pub fn push<T>(&mut self, value: T) {
        const {
            assert!(align_of::<T>() <= SCRATCH_ALIGN);
            assert!(size_of::<T>() > 0, "`ScratchStack` does not support zero-sized types");
            // Non-`Drop` guarantees rewinding / abandoning bytes never leaks a resource.
            assert!(!needs_drop::<T>(), "`ScratchStack` only holds non-`Drop` types");
        }
        let offset = align_up(self.len, align_of::<T>());
        let new_len = offset + size_of::<T>();
        if new_len > self.capacity {
            self.grow(new_len);
        }
        // SAFETY: `grow` guarantees `capacity >= new_len`, so `[offset, offset + size_of::<T>())`
        // is in bounds. `self.ptr` is aligned to `SCRATCH_ALIGN >= align_of::<T>()` and `offset`
        // is aligned to `align_of::<T>()`, so `self.ptr + offset` is correctly aligned for `T`.
        unsafe {
            self.ptr.as_ptr().add(offset).cast::<T>().write(value);
        }
        self.len = new_len;
    }

    /// Move the elements pushed since `mark` into `allocator` as a single
    /// exact-size arena [`Vec`], then rewind the stack to `mark`.
    ///
    /// `T` must be the type that was pushed since `mark`.
    ///
    /// [`Vec`]: oxc_allocator::Vec
    #[inline]
    pub fn drain_into<'a, T>(
        &mut self,
        mark: usize,
        allocator: &'a Allocator,
    ) -> ArenaVec<'a, T> {
        const { assert!(align_of::<T>() <= SCRATCH_ALIGN) };
        let offset = align_up(mark, align_of::<T>());
        let count = if self.len > offset { (self.len - offset) / size_of::<T>() } else { 0 };

        let vec = if count == 0 {
            ArenaVec::new_in(&allocator)
        } else {
            // SAFETY: `[offset, offset + count * size_of::<T>())` was written by `push::<T>`
            // as `count` initialized, aligned `T`s. `ScratchDrain` reads each exactly once,
            // moving it out; the source bytes are abandoned by the rewind below (`T` is
            // non-`Drop`, so this leaks nothing).
            let iter = unsafe {
                ScratchDrain {
                    ptr: self.ptr.as_ptr().add(offset).cast::<T>(),
                    remaining: count,
                    marker: PhantomData,
                }
            };
            ArenaVec::from_iter_in(iter, &allocator)
        };

        self.len = mark;
        vec
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self, required: usize) {
        let new_capacity = cmp::max(self.capacity * 2, required);
        let new_layout = Self::layout(new_capacity);
        let old_layout = Self::layout(self.capacity);
        // SAFETY: `self.ptr` was allocated with `old_layout` (capacity is always `> 0`),
        // `new_layout` has the same alignment, and `new_layout.size() > old_layout.size()`.
        let new_ptr = unsafe { alloc::realloc(self.ptr.as_ptr(), old_layout, new_layout.size()) };
        let Some(new_ptr) = NonNull::new(new_ptr) else { alloc::handle_alloc_error(new_layout) };
        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }
}

impl Drop for ScratchStack {
    fn drop(&mut self) {
        // SAFETY: `self.ptr` was allocated with this layout and has not been freed.
        // `capacity` is always `> 0` (allocated in `new`, only ever grown).
        unsafe {
            alloc::dealloc(self.ptr.as_ptr(), Self::layout(self.capacity));
        }
    }
}

/// Iterator that moves `remaining` values out of a raw pointer, one at a time.
struct ScratchDrain<T> {
    ptr: *const T,
    remaining: usize,
    marker: PhantomData<T>,
}

impl<T> Iterator for ScratchDrain<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        if self.remaining == 0 {
            return None;
        }
        // SAFETY: `remaining` elements starting at `ptr` are initialized `T`s. Each is read
        // exactly once (we advance `ptr` and decrement `remaining`), moving it out.
        let value = unsafe { ptr::read(self.ptr) };
        // SAFETY: after reading, advancing by one stays within the initialized run (or reaches
        // its one-past-the-end, which is never dereferenced because `remaining` hits 0).
        self.ptr = unsafe { self.ptr.add(1) };
        self.remaining -= 1;
        Some(value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for ScratchDrain<T> {}

/// Round `offset` up to a multiple of `align` (a power of two).
#[inline]
const fn align_up(offset: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (offset + (align - 1)) & !(align - 1)
}

#[cold]
#[inline(never)]
fn capacity_overflow() -> ! {
    panic!("`ScratchStack` capacity overflow");
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use super::ScratchStack;

    #[test]
    fn push_and_drain_single_type() {
        let allocator = Allocator::default();
        let mut scratch = ScratchStack::new();

        let mark = scratch.mark();
        for i in 0..10u32 {
            scratch.push(i);
        }
        assert_eq!(scratch.count_since::<u32>(mark), 10);

        let vec = scratch.drain_into::<u32>(mark, &allocator);
        assert_eq!(&*vec, &(0..10).collect::<Vec<_>>()[..]);
        // Stack is rewound.
        assert_eq!(scratch.mark(), mark);
    }

    #[test]
    fn empty_run_drains_to_empty_vec() {
        let allocator = Allocator::default();
        let mut scratch = ScratchStack::new();

        let mark = scratch.mark();
        let vec = scratch.drain_into::<u32>(mark, &allocator);
        assert!(vec.is_empty());
        assert_eq!(scratch.mark(), mark);
    }

    #[test]
    fn nested_lifo_different_types() {
        let allocator = Allocator::default();
        let mut scratch = ScratchStack::new();

        // Outer list of `u64`.
        let outer = scratch.mark();
        scratch.push(1u64);
        scratch.push(2u64);

        // Nested list of `(u8, u32)` built above the outer run, then drained.
        let inner = scratch.mark();
        scratch.push((7u8, 700u32));
        scratch.push((8u8, 800u32));
        let inner_vec = scratch.drain_into::<(u8, u32)>(inner, &allocator);
        assert_eq!(&*inner_vec, &[(7u8, 700u32), (8u8, 800u32)][..]);
        // Draining the inner list rewound the stack to the outer run.
        assert_eq!(scratch.mark(), inner);

        // Outer list continues, unaffected.
        scratch.push(3u64);
        let outer_vec = scratch.drain_into::<u64>(outer, &allocator);
        assert_eq!(&*outer_vec, &[1u64, 2, 3][..]);
        assert_eq!(scratch.mark(), outer);
    }

    #[test]
    fn grows_past_initial_capacity() {
        let allocator = Allocator::default();
        let mut scratch = ScratchStack::new();

        let mark = scratch.mark();
        let n = super::INITIAL_CAPACITY; // far more elements than the initial byte capacity
        for i in 0..n {
            scratch.push(i);
        }
        let vec = scratch.drain_into::<usize>(mark, &allocator);
        assert_eq!(vec.len(), n);
        assert!(vec.iter().copied().eq(0..n));
    }

    #[test]
    fn reuse_across_lists_rewinds() {
        let allocator = Allocator::default();
        let mut scratch = ScratchStack::new();

        for round in 0..100u32 {
            let mark = scratch.mark();
            assert_eq!(mark, 0, "stack must fully rewind between top-level lists");
            for i in 0..round {
                scratch.push(i);
            }
            let vec = scratch.drain_into::<u32>(mark, &allocator);
            assert_eq!(vec.len(), round as usize);
        }
    }
}
