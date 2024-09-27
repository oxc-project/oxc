#![expect(clippy::unnecessary_safety_comment)]

use std::{
    alloc::{self, Layout},
    mem::{align_of, size_of, ManuallyDrop},
    ptr::{self, NonNull},
};

use assert_unchecked::assert_unchecked;

use super::StackCapacity;

/// A simple stack.
///
/// If a non-empty stack is viable for your use case, prefer [`NonEmptyStack`], which is cheaper for
/// all operations.
///
/// [`NonEmptyStack`] is usually the better choice, unless:
/// 1. You want `new()` not to allocate.
/// 2. Creating initial value for `NonEmptyStack::new()` is expensive.
///
/// To simplify implementation, zero size types are not supported (`Stack<()>`).
///
/// ## Design
/// Designed for maximally efficient `push`, `pop`, and reading/writing the last value on stack
/// (although, unlike [`NonEmptyStack`], `last` and `last_mut` are fallible, and not branchless).
///
/// The alternative would likely be to use a `Vec`. But `Vec` is optimized for indexing into at
/// arbitrary positions, not for `push` and `pop`. `Vec` stores `len` and `capacity` as integers,
/// so requires pointer maths on every operation: `let entry_ptr = base_ptr + index * size_of::<T>();`.
///
/// In comparison, `Stack` uses a `cursor` pointer, so avoids these calculations.
/// This is similar to how `std`'s slice iterators work.
///
/// [`NonEmptyStack`]: super::NonEmptyStack
pub struct Stack<T> {
    // Pointer to *after* last entry on stack.
    cursor: NonNull<T>,
    // Pointer to start of allocation containing stack
    start: NonNull<T>,
    // Pointer to end of allocation containing stack
    end: NonNull<T>,
}

impl<T> StackCapacity for Stack<T> {
    type Item = T;
}

impl<T> Stack<T> {
    /// Maximum capacity.
    ///
    /// Effectively unlimited on 64-bit systems.
    pub const MAX_CAPACITY: usize = <Self as StackCapacity>::MAX_CAPACITY;

    /// Create new empty `Stack`.
    ///
    /// # Panics
    /// Panics if `T` is a zero-sized type.
    #[inline]
    pub const fn new() -> Self {
        // ZSTs are not supported for simplicity
        assert!(size_of::<T>() > 0, "Zero sized types are not supported");

        // Create stack with equal `start` and `end`
        let dangling = NonNull::dangling();
        Self { cursor: dangling, start: dangling, end: dangling }
    }

    /// Create new `Stack` with pre-allocated capacity for `capacity` entries.
    ///
    /// # Panics
    /// Panics if any of these requirements are not satisfied:
    /// * `T` must not be a zero-sized type.
    /// * `capacity` must not exceed [`Self::MAX_CAPACITY`].
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            Self::new()
        } else {
            assert!(
                capacity <= Self::MAX_CAPACITY,
                "`capacity` must not exceed `Self::MAX_CAPACITY`"
            );
            // SAFETY: Assertion above ensures `capacity` satisfies requirements
            unsafe { Self::with_capacity_unchecked(capacity) }
        }
    }

    /// Create new `Stack` with pre-allocated capacity for `capacity` entries, without checks.
    ///
    /// `capacity` cannot be 0.
    ///
    /// # Panics
    /// Panics if `T` is a zero-sized type.
    ///
    /// # SAFETY
    /// * `capacity` must not be 0.
    /// * `capacity` must not exceed [`Self::MAX_CAPACITY`].
    #[inline]
    pub unsafe fn with_capacity_unchecked(capacity: usize) -> Self {
        debug_assert!(capacity > 0);
        debug_assert!(capacity <= Self::MAX_CAPACITY);
        // Cannot overflow if `capacity <= MAX_CAPACITY`
        let capacity_bytes = capacity * size_of::<T>();
        // SAFETY: Safety invariants which caller must satisfy guarantee that `capacity_bytes`
        // satisfies requirements
        Self::new_with_capacity_bytes_unchecked(capacity_bytes)
    }

    /// Create new `Stack` with provided capacity in bytes, without checks.
    ///
    /// # Panics
    /// Panics if `T` is a zero-sized type.
    ///
    /// # SAFETY
    /// * `capacity_bytes` must not be 0.
    /// * `capacity_bytes` must be a multiple of `mem::size_of::<T>()`.
    /// * `capacity_bytes` must not exceed [`Self::MAX_CAPACITY_BYTES`].
    #[inline]
    unsafe fn new_with_capacity_bytes_unchecked(capacity_bytes: usize) -> Self {
        // ZSTs are not supported for simplicity
        assert!(size_of::<T>() > 0, "Zero sized types are not supported");

        // SAFETY: Caller guarantees `capacity_bytes` satisfies requirements
        let layout = Self::layout_for(capacity_bytes);
        let ptr = alloc::alloc(layout);
        if ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }
        // `layout_for` produces a layout with `T`'s alignment, so `ptr` is aligned for `T`
        let ptr = ptr.cast::<T>();

        // SAFETY: We checked `ptr` is non-null
        let start = NonNull::new_unchecked(ptr);
        // SAFETY: We allocated `capacity_bytes` bytes, so `end` is end of allocation
        let end = NonNull::new_unchecked(ptr.byte_add(capacity_bytes));

        // `cursor` is positioned at start
        Self { cursor: start, start, end }
    }

    /// Get layout for allocation of `capacity_bytes` bytes.
    ///
    /// # SAFETY
    /// * `capacity_bytes` must not be 0.
    /// * `capacity_bytes` must be a multiple of `mem::size_of::<T>()`.
    /// * `capacity_bytes` must not exceed [`Self::MAX_CAPACITY_BYTES`].
    #[inline]
    unsafe fn layout_for(capacity_bytes: usize) -> Layout {
        // `capacity_bytes` must not be 0 because cannot make 0-size allocations.
        debug_assert!(capacity_bytes > 0);
        // `capacity_bytes` must be a multiple of `size_of::<T>()` so that `new_cursor == self.end`
        // check in `push` accurately detects when full to capacity
        debug_assert!(capacity_bytes % size_of::<T>() == 0);
        // `capacity_bytes` must not exceed `Self::MAX_CAPACITY_BYTES` to prevent creating an allocation
        // of illegal size
        debug_assert!(capacity_bytes <= Self::MAX_CAPACITY_BYTES);

        // SAFETY: `align_of::<T>()` trivially satisfies alignment requirements.
        // Caller guarantees `capacity_bytes <= MAX_CAPACITY_BYTES`.
        // `MAX_CAPACITY_BYTES` takes into account the rounding-up by alignment requirement.
        Layout::from_size_align_unchecked(capacity_bytes, align_of::<T>())
    }

    /// Get reference to last value on stack.
    #[inline]
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn last(&self) -> Option<&T> {
        #[expect(clippy::if_not_else)]
        if !self.is_empty() {
            // SAFETY: Stack is not empty
            Some(unsafe { self.last_unchecked() })
        } else {
            None
        }
    }

    /// Get reference to last value on stack, without checking stack isn't empty.
    ///
    /// # SAFETY
    /// Stack must not be empty.
    #[inline]
    pub unsafe fn last_unchecked(&self) -> &T {
        // SAFETY: All methods ensure `self.cursor` is always in bounds, is aligned for `T`,
        // and `self.current.sub(1)` points to a valid initialized `T`, if stack is not empty.
        // Caller guarantees stack is not empty.
        NonNull::new_unchecked(self.cursor.as_ptr().sub(1)).as_ref()
    }

    /// Get mutable reference to last value on stack.
    #[inline]
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        #[expect(clippy::if_not_else)]
        if !self.is_empty() {
            // SAFETY: Stack is not empty
            Some(unsafe { self.last_mut_unchecked() })
        } else {
            None
        }
    }

    /// Get mutable reference to last value on stack, without checking stack isn't empty.
    ///
    /// # SAFETY
    /// Stack must not be empty.
    #[inline]
    pub unsafe fn last_mut_unchecked(&mut self) -> &mut T {
        // SAFETY: All methods ensure `self.cursor` is always in bounds, is aligned for `T`,
        // and `self.current.sub(1)` points to a valid initialized `T`, if stack is not empty.
        // Caller guarantees stack is not empty.
        NonNull::new_unchecked(self.cursor.as_ptr().sub(1)).as_mut()
    }

    /// Push value to stack.
    ///
    /// # Panics
    /// Panics if stack is already filled to maximum capacity.
    #[inline]
    pub fn push(&mut self, value: T) {
        // The distance between `self.cursor` and `self.end` is always a multiple of `size_of::<T>()`,
        // so `==` check is sufficient to detect when full to capacity.
        if self.cursor == self.end {
            // Needs to grow
            // SAFETY: Stack is full to capacity
            unsafe { self.push_slow(value) };
        } else {
            // SAFETY: Cursor is not at end, so `self.cursor` is in bounds for writing
            unsafe { self.cursor.as_ptr().write(value) };
            // SAFETY: Cursor is not at end, so advancing by a `T` cannot be out of bounds
            self.cursor = unsafe { NonNull::new_unchecked(self.cursor.as_ptr().add(1)) };
        }
    }

    /// Push value to stack when stack is full to capacity.
    ///
    /// This is the slow branch of `push`, which is rarely taken, so marked as `#[cold]` and
    /// `#[inline(never)]` to make `push` as small as possible, so it can be inlined.
    ///
    /// # Panics
    /// Panics if stack is already at maximum capacity.
    ///
    /// # SAFETY
    /// Stack must be full to capacity. i.e. `self.cursor == self.end`.
    #[cold]
    #[inline(never)]
    unsafe fn push_slow(&mut self, value: T) {
        if self.end == self.start {
            // Stack was not allocated yet.
            // SAFETY: `DEFAULT_CAPACITY_BYTES` satisfies requirements.
            let new = ManuallyDrop::new(Self::new_with_capacity_bytes_unchecked(
                Self::DEFAULT_CAPACITY_BYTES,
            ));
            self.start = new.start;
            self.cursor = new.start;
            self.end = new.end;
        } else {
            // Stack was already allocated. Grow capacity.
            // Get new capacity
            let old_capacity_bytes = self.capacity_bytes();
            // Capacity in bytes cannot be larger than `isize::MAX`, so `* 2` cannot overflow.
            let mut new_capacity_bytes = old_capacity_bytes * 2;
            if new_capacity_bytes > Self::MAX_CAPACITY_BYTES {
                assert!(
                    old_capacity_bytes < Self::MAX_CAPACITY_BYTES,
                    "Cannot grow beyond `Self::MAX_CAPACITY`"
                );
                new_capacity_bytes = Self::MAX_CAPACITY_BYTES;
            }
            debug_assert!(new_capacity_bytes > old_capacity_bytes);

            // Reallocate.
            // SAFETY:
            // Stack is allocated, and `self.start` and `self.end` are boundaries of that allocation.
            // So `self.start` and `old_layout` accurately describe the current allocation.
            // `old_capacity_bytes` was a multiple of `size_of::<T>()`, so double that must be too.
            // `MAX_CAPACITY_BYTES` is also a multiple of `size_of::<T>()`.
            // So `new_capacity_bytes` must be a multiple of `size_of::<T>()`.
            // `new_capacity_bytes` is `<= MAX_CAPACITY_BYTES`, so is a legal allocation size.
            // `layout_for` produces a layout with `T`'s alignment, so `new_ptr` is aligned for `T`.
            let new_ptr = unsafe {
                let old_ptr = self.start.as_ptr().cast::<u8>();
                let old_layout = Self::layout_for(old_capacity_bytes);
                let new_ptr = alloc::realloc(old_ptr, old_layout, new_capacity_bytes);
                if new_ptr.is_null() {
                    let new_layout = Self::layout_for(new_capacity_bytes);
                    alloc::handle_alloc_error(new_layout);
                }
                new_ptr.cast::<T>()
            };

            // Update pointers.
            // Stack was full to capacity, so new last index after push is the old capacity.
            // i.e. `self.cursor - self.start == old_end - old_start`.
            // Note: All pointers need to be updated even if allocation grew in place.
            // From docs for `GlobalAlloc::realloc`:
            // "Any access to the old `ptr` is Undefined Behavior, even if the allocation remained in-place."
            // <https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html#method.realloc>
            // `end` changes whatever happens, so always need to be updated.
            // `cursor` needs to be derived from `start` to make `offset_from` valid, so also needs updating.
            // SAFETY: We checked that `new_ptr` is non-null.
            // `old_capacity_bytes` and `new_capacity_bytes` are both multiples of `size_of::<T>()`.
            // `size_of::<T>()` is always a multiple of `T`'s alignment, and `new_ptr` is aligned for `T`,
            // so new `self.cursor` and `self.end` are aligned for `T`.
            // `old_capacity_bytes` is always `< new_capacity_bytes`, so new `self.cursor` must be in bounds.
            unsafe {
                self.start = NonNull::new_unchecked(new_ptr);
                self.end = NonNull::new_unchecked(new_ptr.byte_add(new_capacity_bytes));
                self.cursor = NonNull::new_unchecked(new_ptr.byte_add(old_capacity_bytes));
            }
        }

        // Write value + increment cursor.
        // SAFETY: We just allocated additional capacity, so `self.cursor` is in bounds.
        // `self.cursor` is aligned for `T`.
        unsafe { self.cursor.as_ptr().write(value) }
        // SAFETY: Cursor is not at end, so advancing by a `T` cannot be out of bounds
        self.cursor = unsafe { NonNull::new_unchecked(self.cursor.as_ptr().add(1)) };
    }

    /// Pop value from stack.
    #[inline]
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn pop(&mut self) -> Option<T> {
        #[expect(clippy::if_not_else)]
        if !self.is_empty() {
            // SAFETY: Just checked stack is not empty
            Some(unsafe { self.pop_unchecked() })
        } else {
            None
        }
    }

    /// Pop value from stack, without checking that stack isn't empty.
    ///
    /// # SAFETY
    /// Stack must not be empty.
    #[inline]
    pub unsafe fn pop_unchecked(&mut self) -> T {
        debug_assert!(self.cursor > self.start);
        debug_assert!(self.cursor < self.end);
        // SAFETY: Caller guarantees stack is not empty, so subtracting 1 cannot be out of bounds
        self.cursor = NonNull::new_unchecked(self.cursor.as_ptr().sub(1));
        // SAFETY: All methods ensure `self.cursor` is always in bounds, is aligned for `T`,
        // and points to a valid initialized `T`, if stack is not empty.
        // Caller guarantees stack was not empty.
        self.cursor.as_ptr().read()
    }

    /// Get number of entries on stack.
    #[inline]
    pub fn len(&self) -> usize {
        // `offset_from` returns offset in units of `T`.
        // SAFETY: `self.start` and `self.cursor` are both derived from same pointer
        // (in `new`, `new_with_capacity_bytes_unchecked` and `push_slow`).
        // Both pointers are always within bounds of a single allocation.
        // Distance between pointers is always a multiple of `size_of::<T>()`.
        // `self.cursor` is always >= `self.start`.
        // `assert_unchecked!` is to help compiler to optimize.
        // See: https://doc.rust-lang.org/std/primitive.pointer.html#method.sub_ptr
        #[expect(clippy::cast_sign_loss)]
        unsafe {
            assert_unchecked!(self.cursor >= self.start);
            self.cursor.as_ptr().offset_from(self.start.as_ptr()) as usize
        }
    }

    /// Get if stack is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cursor == self.start
    }

    /// Get capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        // SAFETY: `self.start` and `self.end` are both derived from same pointer
        // (in `new`, `new_with_capacity_bytes_unchecked` and `push_slow`).
        // Both pointers are always within bounds of single allocation.
        // Distance between pointers is always a multiple of `size_of::<T>()`.
        // `self.end` is always >= `self.start`.
        // `assert_unchecked!` is to help compiler to optimize.
        // See: https://doc.rust-lang.org/std/primitive.pointer.html#method.sub_ptr
        #[expect(clippy::cast_sign_loss)]
        unsafe {
            assert_unchecked!(self.end >= self.start);
            self.end.as_ptr().offset_from(self.start.as_ptr()) as usize
        }
    }

    /// Get capacity in bytes.
    #[inline]
    fn capacity_bytes(&self) -> usize {
        // SAFETY: `self.start` and `self.end` are both derived from same pointer
        // (in `new`, `new_with_capacity_bytes_unchecked` and `push_slow`).
        // Both pointers are always within bounds of single allocation.
        // Distance between pointers is always a multiple of `size_of::<T>()`.
        // `self.end` is always >= `self.start`.
        // `assert_unchecked!` is to help compiler to optimize.
        // See: https://doc.rust-lang.org/std/primitive.pointer.html#method.sub_ptr
        #[expect(clippy::cast_sign_loss)]
        unsafe {
            assert_unchecked!(self.end >= self.start);
            self.end.as_ptr().byte_offset_from(self.start.as_ptr()) as usize
        }
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        // Nothing to drop if stack never allocated
        if self.end == self.start {
            return;
        }

        if !self.is_empty() {
            // Drop contents. This block copied from `std`'s `Vec`.
            // Will be optimized out if `T` is non-drop, as `drop_in_place` calls `std::mem::needs_drop`.
            // SAFETY: Stack contains `self.len()` initialized entries, starting at `self.start`.
            unsafe {
                ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.start.as_ptr(), self.len()));
            }
        }

        // Drop the memory
        // SAFETY: Checked above that stack is allocated.
        // `self.start` and `self.end` are boundaries of that allocation.
        // So `self.start` and `layout` accurately describe the current allocation.
        unsafe {
            let layout = Self::layout_for(self.capacity_bytes());
            alloc::dealloc(self.start.as_ptr().cast::<u8>(), layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_len_cap_last {
        ($stack:ident, $len:expr, $capacity:expr, $last:expr) => {
            assert_eq!($stack.len(), $len);
            assert_eq!($stack.capacity(), $capacity);
            assert_eq!($stack.last(), $last);
        };
    }

    #[test]
    fn new() {
        let stack = Stack::<bool>::new();
        assert_len_cap_last!(stack, 0, 0, None);
        assert_eq!(stack.capacity_bytes(), 0);

        let stack = Stack::<u64>::new();
        assert_len_cap_last!(stack, 0, 0, None);
        assert_eq!(stack.capacity_bytes(), 0);

        let stack = Stack::<[u8; 1024]>::new();
        assert_len_cap_last!(stack, 0, 0, None);
        assert_eq!(stack.capacity_bytes(), 0);

        let stack = Stack::<[u8; 1025]>::new();
        assert_len_cap_last!(stack, 0, 0, None);
        assert_eq!(stack.capacity_bytes(), 0);
    }

    #[test]
    fn with_capacity() {
        let stack = Stack::<u64>::with_capacity(16);
        assert_len_cap_last!(stack, 0, 16, None);
        assert_eq!(stack.capacity_bytes(), 128);
    }

    #[test]
    fn with_capacity_zero() {
        let stack = Stack::<u64>::with_capacity(0);
        assert_len_cap_last!(stack, 0, 0, None);
    }

    #[test]
    fn push_then_pop() {
        let mut stack = Stack::<u64>::new();
        assert_len_cap_last!(stack, 0, 0, None);
        assert_eq!(stack.capacity_bytes(), 0);

        stack.push(10);
        assert_len_cap_last!(stack, 1, 4, Some(&10));
        assert_eq!(stack.capacity_bytes(), 32);

        stack.push(20);
        assert_len_cap_last!(stack, 2, 4, Some(&20));
        stack.push(30);
        assert_len_cap_last!(stack, 3, 4, Some(&30));

        stack.push(40);
        assert_len_cap_last!(stack, 4, 4, Some(&40));
        assert_eq!(stack.capacity_bytes(), 32);
        stack.push(50);
        assert_len_cap_last!(stack, 5, 8, Some(&50));
        assert_eq!(stack.capacity_bytes(), 64);

        stack.push(60);
        assert_len_cap_last!(stack, 6, 8, Some(&60));
        stack.push(70);
        assert_len_cap_last!(stack, 7, 8, Some(&70));

        stack.push(80);
        assert_len_cap_last!(stack, 8, 8, Some(&80));
        assert_eq!(stack.capacity_bytes(), 64);

        stack.push(90);
        assert_len_cap_last!(stack, 9, 16, Some(&90));
        assert_eq!(stack.capacity_bytes(), 128);

        assert_eq!(stack.pop(), Some(90));
        assert_len_cap_last!(stack, 8, 16, Some(&80));
        assert_eq!(stack.pop(), Some(80));
        assert_len_cap_last!(stack, 7, 16, Some(&70));
        assert_eq!(stack.pop(), Some(70));
        assert_len_cap_last!(stack, 6, 16, Some(&60));
        assert_eq!(stack.pop(), Some(60));
        assert_len_cap_last!(stack, 5, 16, Some(&50));
        assert_eq!(stack.pop(), Some(50));
        assert_len_cap_last!(stack, 4, 16, Some(&40));
        assert_eq!(stack.pop(), Some(40));
        assert_len_cap_last!(stack, 3, 16, Some(&30));
        assert_eq!(stack.pop(), Some(30));
        assert_len_cap_last!(stack, 2, 16, Some(&20));
        assert_eq!(stack.pop(), Some(20));
        assert_len_cap_last!(stack, 1, 16, Some(&10));
        assert_eq!(stack.pop(), Some(10));
        assert_len_cap_last!(stack, 0, 16, None);
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.capacity_bytes(), 128);
    }

    #[test]
    fn push_and_pop_mixed() {
        let mut stack = Stack::<u64>::new();
        assert_len_cap_last!(stack, 0, 0, None);
        assert_eq!(stack.capacity_bytes(), 0);

        stack.push(10);
        assert_len_cap_last!(stack, 1, 4, Some(&10));
        assert_eq!(stack.capacity_bytes(), 32);

        stack.push(20);
        assert_len_cap_last!(stack, 2, 4, Some(&20));
        stack.push(30);
        assert_len_cap_last!(stack, 3, 4, Some(&30));

        assert_eq!(stack.pop(), Some(30));
        assert_len_cap_last!(stack, 2, 4, Some(&20));

        stack.push(31);
        assert_len_cap_last!(stack, 3, 4, Some(&31));
        stack.push(40);
        assert_len_cap_last!(stack, 4, 4, Some(&40));
        assert_eq!(stack.capacity_bytes(), 32);
        stack.push(50);
        assert_len_cap_last!(stack, 5, 8, Some(&50));
        assert_eq!(stack.capacity_bytes(), 64);

        assert_eq!(stack.pop(), Some(50));
        assert_len_cap_last!(stack, 4, 8, Some(&40));
        assert_eq!(stack.pop(), Some(40));
        assert_len_cap_last!(stack, 3, 8, Some(&31));
        assert_eq!(stack.pop(), Some(31));
        assert_len_cap_last!(stack, 2, 8, Some(&20));

        stack.push(32);
        assert_len_cap_last!(stack, 3, 8, Some(&32));

        assert_eq!(stack.pop(), Some(32));
        assert_len_cap_last!(stack, 2, 8, Some(&20));
        assert_eq!(stack.pop(), Some(20));
        assert_len_cap_last!(stack, 1, 8, Some(&10));
        assert_eq!(stack.pop(), Some(10));
        assert_len_cap_last!(stack, 0, 8, None);
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.capacity_bytes(), 64);

        stack.push(11);
        assert_len_cap_last!(stack, 1, 8, Some(&11));
        assert_eq!(stack.pop(), Some(11));
        assert_len_cap_last!(stack, 0, 8, None);
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.capacity_bytes(), 64);
    }

    #[test]
    fn last_mut() {
        let mut stack = Stack::<u64>::new();
        assert_len_cap_last!(stack, 0, 0, None);
        assert_eq!(stack.last_mut(), None);

        stack.push(10);
        assert_len_cap_last!(stack, 1, 4, Some(&10));

        *stack.last_mut().unwrap() = 11;
        assert_len_cap_last!(stack, 1, 4, Some(&11));
        *stack.last_mut().unwrap() = 12;
        assert_len_cap_last!(stack, 1, 4, Some(&12));

        stack.push(20);
        assert_len_cap_last!(stack, 2, 4, Some(&20));
        *stack.last_mut().unwrap() = 21;
        assert_len_cap_last!(stack, 2, 4, Some(&21));
        *stack.last_mut().unwrap() = 22;
        assert_len_cap_last!(stack, 2, 4, Some(&22));
    }

    #[test]
    #[expect(clippy::items_after_statements)]
    fn drop() {
        use std::sync::{Mutex, OnceLock};

        static DROPS: OnceLock<Mutex<Vec<u32>>> = OnceLock::new();
        DROPS.get_or_init(|| Mutex::new(vec![]));

        fn drops() -> Vec<u32> {
            std::mem::take(DROPS.get().unwrap().lock().unwrap().as_mut())
        }

        #[derive(PartialEq, Debug)]
        struct Droppy(u32);

        impl Drop for Droppy {
            fn drop(&mut self) {
                DROPS.get().unwrap().lock().unwrap().push(self.0);
            }
        }

        {
            let mut stack = Stack::new();
            stack.push(Droppy(10));
            stack.push(Droppy(20));
            stack.push(Droppy(30));
            assert_eq!(stack.len(), 3);
            assert_eq!(stack.capacity(), 4);

            stack.pop();
            assert_eq!(drops(), &[30]);
            assert!(drops().is_empty());

            stack.push(Droppy(31));
            stack.push(Droppy(40));
            stack.push(Droppy(50));
            assert_eq!(stack.len(), 5);
            assert_eq!(stack.capacity(), 8);
            assert!(drops().is_empty());
        }

        assert_eq!(drops(), &[10, 20, 31, 40, 50]);
    }
}
