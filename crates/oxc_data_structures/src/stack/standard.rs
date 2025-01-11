#![expect(clippy::unnecessary_safety_comment)]

use std::{
    mem::size_of,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use super::{StackCapacity, StackCommon};

/// A simple stack.
///
/// If a non-empty stack is viable for your use case, prefer [`NonEmptyStack`], which is cheaper for
/// all operations.
///
/// [`NonEmptyStack`] is usually the better choice, unless either:
///
/// 1. The stack will likely never have anything pushed to it.
///    [`NonEmptyStack::new`] always allocates, whereas [`Stack::new`] does not.
///    So if stack usually starts empty and remains empty, [`Stack`] will avoid an allocation.
///    This is the same as how [`Vec`] does not allocate until you push a value into it.
///
/// 2. The type the stack holds is large or expensive to construct, so there's a high cost in having to
///    create an initial dummy value (which [`NonEmptyStack`] requires, but [`Stack`] doesn't).
///
/// To simplify implementation, zero size types are not supported (`Stack<()>`).
///
/// ## Design
/// Designed for maximally efficient [`push`], [`pop`], and reading/writing the last value on stack
/// ([`last`] / [`last_mut`]). Although, unlike [`NonEmptyStack`], [`last`] and [`last_mut`] are
/// fallible, and not branchless. So [`Stack::last`] and [`Stack::last_mut`] are a bit more expensive
/// than [`NonEmptyStack`]'s equivalents.
///
/// The alternative would likely be to use a [`Vec`]. But `Vec` is optimized for indexing into at
/// arbitrary positions, not for `push` and `pop`. `Vec` stores `len` and `capacity` as integers,
/// so requires pointer maths on every operation: `let entry_ptr = base_ptr + index * size_of::<T>();`.
///
/// In comparison, [`Stack`] uses a `cursor` pointer, so avoids these calculations.
/// This is similar to how [`std`'s slice iterators] work.
///
/// [`push`]: Stack::push
/// [`pop`]: Stack::pop
/// [`last`]: Stack::last
/// [`last_mut`]: Stack::last_mut
/// [`NonEmptyStack`]: super::NonEmptyStack
/// [`NonEmptyStack::new`]: super::NonEmptyStack::new
/// [`std`'s slice iterators]: std::slice::Iter
pub struct Stack<T> {
    // Pointer to *after* last entry on stack.
    cursor: NonNull<T>,
    // Pointer to start of allocation containing stack
    start: NonNull<T>,
    // Pointer to end of allocation containing stack
    end: NonNull<T>,
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> StackCapacity<T> for Stack<T> {}

impl<T> StackCommon<T> for Stack<T> {
    #[inline]
    fn start(&self) -> NonNull<T> {
        self.start
    }

    #[inline]
    fn end(&self) -> NonNull<T> {
        self.end
    }

    #[inline]
    fn cursor(&self) -> NonNull<T> {
        self.cursor
    }

    #[inline]
    fn set_start(&mut self, start: NonNull<T>) {
        self.start = start;
    }

    #[inline]
    fn set_end(&mut self, end: NonNull<T>) {
        self.end = end;
    }

    #[inline]
    fn set_cursor(&mut self, cursor: NonNull<T>) {
        self.cursor = cursor;
    }

    fn len(&self) -> usize {
        // SAFETY: `self.start` and `self.cursor` are both derived from same pointer.
        // `self.cursor` is always >= `self.start`.
        // Distance between pointers is always a multiple of `size_of::<T>()`.
        unsafe { self.cursor_offset() }
    }
}

impl<T> Stack<T> {
    /// Maximum capacity.
    ///
    /// Effectively unlimited on 64-bit systems.
    pub const MAX_CAPACITY: usize = <Self as StackCapacity<T>>::MAX_CAPACITY;

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
    /// # Safety
    ///
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
        let (start, end) = Self::allocate(capacity_bytes);

        // `cursor` is positioned at start
        Self { cursor: start, start, end }
    }

    /// Get reference to last value on stack.
    #[inline]
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
    /// # Safety
    ///
    /// * Stack must not be empty.
    #[inline]
    pub unsafe fn last_unchecked(&self) -> &T {
        debug_assert!(self.end > self.start);
        debug_assert!(self.cursor > self.start);
        debug_assert!(self.cursor <= self.end);
        // SAFETY: All methods ensure `self.cursor` is always in bounds, is aligned for `T`,
        // and `self.current.sub(1)` points to a valid initialized `T`, if stack is not empty.
        // Caller guarantees stack is not empty.
        self.cursor.sub(1).as_ref()
    }

    /// Get mutable reference to last value on stack.
    #[inline]
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
    /// # Safety
    ///
    /// * Stack must not be empty.
    #[inline]
    pub unsafe fn last_mut_unchecked(&mut self) -> &mut T {
        debug_assert!(self.end > self.start);
        debug_assert!(self.cursor > self.start);
        debug_assert!(self.cursor <= self.end);
        // SAFETY: All methods ensure `self.cursor` is always in bounds, is aligned for `T`,
        // and `self.current.sub(1)` points to a valid initialized `T`, if stack is not empty.
        // Caller guarantees stack is not empty.
        self.cursor.sub(1).as_mut()
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
            self.cursor = unsafe { self.cursor.add(1) };
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
        #[expect(clippy::if_not_else)]
        if self.end != self.start {
            // Stack was already allocated. Grow capacity.
            // SAFETY: Checked above that is already allocated.
            self.grow();
        } else {
            // Stack was not allocated yet.
            // SAFETY: `DEFAULT_CAPACITY_BYTES` satisfies requirements.
            let (start, end) = Self::allocate(Self::DEFAULT_CAPACITY_BYTES);
            self.start = start;
            self.cursor = start;
            self.end = end;
        }

        // Write value + increment cursor.
        // SAFETY: We just allocated additional capacity, so `self.cursor` is in bounds.
        // `self.cursor` is aligned for `T`.
        unsafe { self.cursor.as_ptr().write(value) }
        // SAFETY: Cursor is not at end, so advancing by a `T` cannot be out of bounds
        self.cursor = unsafe { self.cursor.add(1) };
    }

    /// Pop value from stack.
    #[inline]
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
    /// # Safety
    ///
    /// * Stack must not be empty.
    #[inline]
    pub unsafe fn pop_unchecked(&mut self) -> T {
        debug_assert!(self.end > self.start);
        debug_assert!(self.cursor > self.start);
        debug_assert!(self.cursor <= self.end);
        // SAFETY: Caller guarantees stack is not empty, so subtracting 1 cannot be out of bounds
        self.cursor = self.cursor.sub(1);
        // SAFETY: All methods ensure `self.cursor` is always in bounds, is aligned for `T`,
        // and points to a valid initialized `T`, if stack is not empty.
        // Caller guarantees stack was not empty.
        self.cursor.read()
    }

    /// Get number of entries on stack.
    #[inline]
    pub fn len(&self) -> usize {
        <Self as StackCommon<T>>::len(self)
    }

    /// Get if stack is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cursor == self.start
    }

    /// Get capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        <Self as StackCommon<T>>::capacity(self)
    }

    /// Get contents of stack as a slice `&[T]`.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        <Self as StackCommon<T>>::as_slice(self)
    }

    /// Get contents of stack as a mutable slice `&mut [T]`.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        <Self as StackCommon<T>>::as_mut_slice(self)
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        // Nothing to drop if stack never allocated
        if self.end == self.start {
            return;
        }

        if !self.is_empty() {
            // SAFETY: Checked above that stack is allocated.
            // Stack contains `self.len()` initialized entries, starting at `self.start`
            unsafe { self.drop_contents() };
        }

        // Drop the memory
        // SAFETY: Checked above that stack is allocated.
        unsafe { self.deallocate() };
    }
}

impl<T> Deref for Stack<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> DerefMut for Stack<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
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

        assert_eq!(stack.pop(), Some(40));
        assert_len_cap_last!(stack, 3, 4, Some(&31));
        assert_eq!(stack.capacity_bytes(), 32);

        stack.push(41);
        assert_len_cap_last!(stack, 4, 4, Some(&41));
        assert_eq!(stack.capacity_bytes(), 32);

        stack.push(50);
        assert_len_cap_last!(stack, 5, 8, Some(&50));
        assert_eq!(stack.capacity_bytes(), 64);

        assert_eq!(stack.pop(), Some(50));
        assert_len_cap_last!(stack, 4, 8, Some(&41));
        assert_eq!(stack.pop(), Some(41));
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
