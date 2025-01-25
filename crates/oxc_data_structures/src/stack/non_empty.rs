#![expect(clippy::unnecessary_safety_comment)]

use std::{
    mem::size_of,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use super::{StackCapacity, StackCommon};

/// A stack which can never be empty.
///
/// [`NonEmptyStack`] is created initially with 1 entry, and [`pop`] does not allow removing it
/// (though that initial entry can be mutated with [`last_mut`]).
///
/// The fact that the stack is never empty makes all operations except [`pop`] infallible.
/// [`last`] and [`last_mut`] are branchless.
///
/// The trade-off is that you cannot create a [`NonEmptyStack`] without allocating,
/// and you must create an initial value for the "dummy" initial entry.
/// If that is not a good trade-off for your use case, prefer [`Stack`], which can be empty.
///
/// [`NonEmptyStack`] is usually a better choice than [`Stack`], unless either:
///
/// 1. The stack will likely never have anything pushed to it.
///    [`NonEmptyStack::new`] always allocates, whereas [`Stack::new`] does not.
///    So if stack usually starts empty and remains empty, [`Stack`] will avoid an allocation.
///    This is the same as how [`Vec`] does not allocate until you push a value into it.
///
/// 2. The type the stack holds is large or expensive to construct, so there's a high cost in having to
///    create an initial dummy value (which [`NonEmptyStack`] requires, but [`Stack`] doesn't).
///
/// [`SparseStack`] may be preferable if the type you're storing is an `Option`.
///
/// To simplify implementation, zero size types are not supported (e.g. `NonEmptyStack<()>`).
///
/// ## Design
/// Designed for maximally efficient [`push`], [`pop`], and reading/writing the last value on stack
/// ([`last`] / [`last_mut`]).
///
/// The alternative would likely be to use a [`Vec`]. But `Vec` is optimized for indexing into at
/// arbitrary positions, not for `push` and `pop`. `Vec` stores `len` and `capacity` as integers,
/// so requires pointer maths on every operation: `let entry_ptr = base_ptr + index * size_of::<T>();`.
///
/// In comparison, [`NonEmptyStack`] contains a `cursor` pointer, which always points to last entry
/// on stack, so it can be read/written with a minimum of operations.
///
/// This design is similar to [`std`'s slice iterators].
///
/// Comparison to [`Vec`]:
/// * [`last`] and [`last_mut`] are 1 instruction, instead of `Vec`'s 4.
/// * [`pop`] is 1 instruction shorter than `Vec`'s equivalent.
/// * [`push`] is 1 instruction shorter than `Vec`'s equivalent, and uses 1 less register.
///
/// ### Possible alternative designs
/// 1. `cursor` could point to *after* last entry, rather than *to* it. This has advantage that [`pop`]
///    uses 1 less register, but disadvantage that [`last`] and [`last_mut`] are 2 instructions, not 1.
///    <https://godbolt.org/z/xnx7YP5de>
///
/// 2. Stack could grow downwards, like `bumpalo` allocator does. This would probably make [`pop`] use
///    1 less register, but at the cost that: (a) the stack can never grow in place, which would incur
///    more memory copies when the stack grows, and (b) [`as_slice`] would have the entries in
///    reverse order.
///
/// [`push`]: NonEmptyStack::push
/// [`pop`]: NonEmptyStack::pop
/// [`last`]: NonEmptyStack::last
/// [`last_mut`]: NonEmptyStack::last_mut
/// [`as_slice`]: NonEmptyStack::as_slice
/// [`Stack`]: super::Stack
/// [`Stack::new`]: super::Stack::new
/// [`SparseStack`]: super::SparseStack
/// [`std`'s slice iterators]: std::slice::Iter
pub struct NonEmptyStack<T> {
    /// Pointer to last entry on stack.
    /// Points *to* last entry, not *after* last entry.
    cursor: NonNull<T>,
    /// Pointer to start of allocation (first entry)
    start: NonNull<T>,
    /// Pointer to end of allocation
    end: NonNull<T>,
}

impl<T> StackCapacity<T> for NonEmptyStack<T> {}

impl<T> StackCommon<T> for NonEmptyStack<T> {
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

    #[inline]
    fn len(&self) -> usize {
        // SAFETY: `self.start` and `self.cursor` are both derived from same pointer.
        // `self.cursor` is always >= `self.start`.
        // Distance between pointers is always a multiple of `size_of::<T>()`.
        let offset = unsafe { self.cursor_offset() };

        // When stack has 1 entry, `start - cursor == 0`, so add 1 to get number of entries.
        // SAFETY: Capacity cannot exceed `Self::MAX_CAPACITY`, which is `<= isize::MAX`,
        // and offset can't exceed capacity, so `+ 1` cannot wrap around.
        // `checked_add(1).unwrap_unchecked()` instead of just `+ 1` to hint to compiler
        // that return value can never be zero.
        unsafe { offset.checked_add(1).unwrap_unchecked() }
    }
}

impl<T> NonEmptyStack<T> {
    /// Maximum capacity.
    ///
    /// Effectively unlimited on 64-bit systems.
    pub const MAX_CAPACITY: usize = <Self as StackCapacity<T>>::MAX_CAPACITY;

    /// Create new [`NonEmptyStack`] with default pre-allocated capacity, and initial value `initial_value`.
    ///
    /// # Panics
    /// Panics if `T` is a zero-sized type.
    #[inline]
    pub fn new(initial_value: T) -> Self {
        // SAFETY: `DEFAULT_CAPACITY_BYTES` satisfies requirements
        unsafe {
            Self::new_with_capacity_bytes_unchecked(Self::DEFAULT_CAPACITY_BYTES, initial_value)
        }
    }

    /// Create new [`NonEmptyStack`] with pre-allocated capacity for `capacity` entries,
    /// and initial value `initial_value`.
    ///
    /// `capacity` cannot be 0.
    ///
    /// # Panics
    /// Panics if any of these requirements are not satisfied:
    /// * `T` must not be a zero-sized type.
    /// * `capacity` must not be 0.
    /// * `capacity` must not exceed [`Self::MAX_CAPACITY`].
    #[inline]
    pub fn with_capacity(capacity: usize, initial_value: T) -> Self {
        assert!(capacity > 0, "`capacity` cannot be zero");
        assert!(capacity <= Self::MAX_CAPACITY, "`capacity` must not exceed `Self::MAX_CAPACITY`");
        // SAFETY: Assertions above ensure `capacity` satisfies requirements
        unsafe { Self::with_capacity_unchecked(capacity, initial_value) }
    }

    /// Create new [`NonEmptyStack`] with pre-allocated capacity for `capacity` entries,
    /// and initial value `initial_value`, without checks.
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
    pub unsafe fn with_capacity_unchecked(capacity: usize, initial_value: T) -> Self {
        debug_assert!(capacity > 0);
        debug_assert!(capacity <= Self::MAX_CAPACITY);
        // Cannot overflow if `capacity <= MAX_CAPACITY`
        let capacity_bytes = capacity * size_of::<T>();
        // SAFETY: Safety invariants which caller must satisfy guarantee that `capacity_bytes`
        // satisfies requirements
        Self::new_with_capacity_bytes_unchecked(capacity_bytes, initial_value)
    }

    /// Create new [`NonEmptyStack`] with provided capacity in bytes, and initial value `initial_value`,
    /// without checks.
    ///
    /// # Panics
    /// Panics if `T` is a zero-sized type.
    ///
    /// # SAFETY
    /// * `capacity_bytes` must not be 0.
    /// * `capacity_bytes` must be a multiple of `mem::size_of::<T>()`.
    /// * `capacity_bytes` must not exceed [`Self::MAX_CAPACITY_BYTES`].
    #[inline]
    unsafe fn new_with_capacity_bytes_unchecked(capacity_bytes: usize, initial_value: T) -> Self {
        // ZSTs are not supported for simplicity
        assert!(size_of::<T>() > 0, "Zero sized types are not supported");

        // SAFETY: Caller guarantees `capacity_bytes` satisfies requirements
        let (start, end) = Self::allocate(capacity_bytes);

        // Write initial value to start of allocation.
        // SAFETY: Allocation was created with alignment of `T`, and with capacity for at least 1 entry,
        // so `start` is valid for writing a `T`.
        start.as_ptr().write(initial_value);

        // `cursor` is positioned at start i.e. pointing at initial value
        Self { cursor: start, start, end }
    }

    /// Get reference to last value on stack.
    #[inline]
    pub fn last(&self) -> &T {
        // SAFETY: All methods ensure `self.cursor` is always in bounds, is aligned for `T`,
        // and points to a valid initialized `T`
        unsafe { self.cursor.as_ref() }
    }

    /// Get mutable reference to last value on stack.
    #[inline]
    pub fn last_mut(&mut self) -> &mut T {
        // SAFETY: All methods ensure `self.cursor` is always in bounds, is aligned for `T`,
        // and points to a valid initialized `T`
        unsafe { self.cursor.as_mut() }
    }

    /// Push value to stack.
    ///
    /// # Panics
    /// Panics if stack is already filled to maximum capacity.
    #[inline]
    pub fn push(&mut self, value: T) {
        // SAFETY: Stack is never empty and `self.cursor` is always less than `self.end`, which is end
        // of allocation. So advancing by a `T` cannot be out of bounds.
        // The distance between `self.cursor` and `self.end` is always a multiple of `size_of::<T>()`,
        // so `==` check is sufficient to detect when full to capacity.
        let new_cursor = unsafe { self.cursor.add(1) };
        if new_cursor == self.end {
            // Needs to grow
            // SAFETY: Stack is full to capacity
            unsafe { self.push_slow(value) };
        } else {
            // Capacity for at least 1 more entry
            // SAFETY: We checked there is capacity for 1 more entry, so `self.cursor` is in bounds.
            // `self.cursor` was aligned for `T`, and we added `size_of::<T>()` to pointer.
            // `size_of::<T>()` is always a multiple of `T`'s alignment, so `self.cursor` must still be
            // aligned for `T`.
            unsafe { new_cursor.as_ptr().write(value) };
            self.cursor = new_cursor;
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
    /// Stack must be full to capacity. i.e. `self.cursor.add(1) == self.end`.
    #[cold]
    #[inline(never)]
    unsafe fn push_slow(&mut self, value: T) {
        // Grow allocation.
        // SAFETY: Stack is always allocated.
        self.grow();

        // Write value.
        // SAFETY: We just allocated additional capacity, so `self.cursor` is in bounds.
        // `self.cursor` is aligned for `T`.
        unsafe { self.cursor.as_ptr().write(value) }
    }

    /// Pop value from stack.
    ///
    /// # Panics
    /// Panics if the stack has only 1 entry on it.
    #[inline]
    pub fn pop(&mut self) -> T {
        // Panic if trying to remove last entry from stack.
        //
        // Putting the panic in an `#[inline(never)]` + `#[cold]` function removes a 6-byte `lea`
        // instruction vs `assert!(self.cursor != self.start, "Cannot pop all entries")`.
        // This reduces this function on x86_64 from 32 bytes to 26 bytes.
        // This function is commonly used, and we want it to be inlined, so every byte counts.
        // https://godbolt.org/z/5587z99rM
        #[inline(never)]
        #[cold]
        fn error() -> ! {
            panic!("Cannot pop all entries");
        }

        if self.cursor == self.start {
            error();
        }

        // SAFETY: Assertion above ensures stack has at least 2 entries
        unsafe { self.pop_unchecked() }
    }

    /// Pop value from stack, without checking that stack isn't empty.
    ///
    /// # Safety
    ///
    /// * Stack must have at least 2 entries, so that after pop, it still has at least 1.
    #[inline]
    pub unsafe fn pop_unchecked(&mut self) -> T {
        debug_assert!(self.cursor > self.start);
        debug_assert!(self.cursor < self.end);
        // SAFETY: All methods ensure `self.cursor` is always in bounds, is aligned for `T`,
        // and points to a valid initialized `T`
        let value = self.cursor.read();
        // SAFETY: Caller guarantees there's at least 2 entries on stack, so subtracting 1
        // cannot be out of bounds
        self.cursor = self.cursor.sub(1);
        value
    }

    /// Get number of values on stack.
    ///
    /// Number of entries is always at least 1. Stack is never empty.
    #[inline]
    pub fn len(&self) -> usize {
        <Self as StackCommon<T>>::len(self)
    }

    /// Get if stack is empty. Always returns `false`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        // This method is pointless, as the stack is never empty. But provide it to override
        // the default method from `slice::is_empty` which is inherited via `Deref`
        false
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

impl<T> Drop for NonEmptyStack<T> {
    fn drop(&mut self) {
        // Drop contents.
        // SAFETY: Stack is always allocated, and contains `self.len()` initialized entries,
        // starting at `self.start`.
        unsafe { self.drop_contents() };

        // Drop the memory
        // SAFETY: Stack is always allocated.
        unsafe { self.deallocate() };
    }
}

impl<T> Deref for NonEmptyStack<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> DerefMut for NonEmptyStack<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: Default> Default for NonEmptyStack<T> {
    fn default() -> Self {
        Self::new(T::default())
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
        let stack = NonEmptyStack::new(true);
        assert_len_cap_last!(stack, 1, 16, &true);
        assert_eq!(stack.capacity_bytes(), 16);

        let stack = NonEmptyStack::new(10u64);
        assert_len_cap_last!(stack, 1, 4, &10);
        assert_eq!(stack.capacity_bytes(), 32);

        let stack = NonEmptyStack::new([10u8; 1024]);
        assert_len_cap_last!(stack, 1, 4, &[10; 1024]);
        assert_eq!(stack.capacity_bytes(), 4096);

        let stack = NonEmptyStack::new([10u8; 1025]);
        assert_len_cap_last!(stack, 1, 1, &[10; 1025]);
        assert_eq!(stack.capacity_bytes(), 1025);
    }

    #[test]
    fn with_capacity() {
        let stack = NonEmptyStack::with_capacity(16, 10u64);
        assert_len_cap_last!(stack, 1, 16, &10);
        assert_eq!(stack.capacity_bytes(), 128);
    }

    #[test]
    #[should_panic(expected = "`capacity` cannot be zero")]
    fn with_capacity_zero() {
        NonEmptyStack::with_capacity(0, 10u64);
    }

    #[test]
    fn push_then_pop() {
        let mut stack = NonEmptyStack::new(10u64);
        assert_len_cap_last!(stack, 1, 4, &10);
        assert_eq!(stack.capacity_bytes(), 32);

        stack.push(20);
        assert_len_cap_last!(stack, 2, 4, &20);
        stack.push(30);
        assert_len_cap_last!(stack, 3, 4, &30);

        stack.push(40);
        assert_len_cap_last!(stack, 4, 4, &40);
        assert_eq!(stack.capacity_bytes(), 32);
        stack.push(50);
        assert_len_cap_last!(stack, 5, 8, &50);
        assert_eq!(stack.capacity_bytes(), 64);

        stack.push(60);
        assert_len_cap_last!(stack, 6, 8, &60);
        stack.push(70);
        assert_len_cap_last!(stack, 7, 8, &70);

        stack.push(80);
        assert_len_cap_last!(stack, 8, 8, &80);
        assert_eq!(stack.capacity_bytes(), 64);

        stack.push(90);
        assert_len_cap_last!(stack, 9, 16, &90);
        assert_eq!(stack.capacity_bytes(), 128);

        assert_eq!(stack.pop(), 90);
        assert_len_cap_last!(stack, 8, 16, &80);
        assert_eq!(stack.pop(), 80);
        assert_len_cap_last!(stack, 7, 16, &70);
        assert_eq!(stack.pop(), 70);
        assert_len_cap_last!(stack, 6, 16, &60);
        assert_eq!(stack.pop(), 60);
        assert_len_cap_last!(stack, 5, 16, &50);
        assert_eq!(stack.pop(), 50);
        assert_len_cap_last!(stack, 4, 16, &40);
        assert_eq!(stack.pop(), 40);
        assert_len_cap_last!(stack, 3, 16, &30);
        assert_eq!(stack.pop(), 30);
        assert_len_cap_last!(stack, 2, 16, &20);
        assert_eq!(stack.pop(), 20);
        assert_len_cap_last!(stack, 1, 16, &10);
        assert_eq!(stack.capacity_bytes(), 128);
    }

    #[test]
    fn push_and_pop_mixed() {
        let mut stack = NonEmptyStack::new(10u64);
        assert_len_cap_last!(stack, 1, 4, &10);
        assert_eq!(stack.capacity_bytes(), 32);

        stack.push(20);
        assert_len_cap_last!(stack, 2, 4, &20);
        stack.push(30);
        assert_len_cap_last!(stack, 3, 4, &30);

        assert_eq!(stack.pop(), 30);
        assert_len_cap_last!(stack, 2, 4, &20);

        stack.push(31);
        assert_len_cap_last!(stack, 3, 4, &31);
        stack.push(40);
        assert_len_cap_last!(stack, 4, 4, &40);
        stack.push(50);
        assert_len_cap_last!(stack, 5, 8, &50);

        assert_eq!(stack.pop(), 50);
        assert_len_cap_last!(stack, 4, 8, &40);
        assert_eq!(stack.pop(), 40);
        assert_len_cap_last!(stack, 3, 8, &31);
        assert_eq!(stack.pop(), 31);
        assert_len_cap_last!(stack, 2, 8, &20);

        stack.push(32);
        assert_len_cap_last!(stack, 3, 8, &32);

        assert_eq!(stack.pop(), 32);
        assert_len_cap_last!(stack, 2, 8, &20);
        assert_eq!(stack.pop(), 20);
        assert_len_cap_last!(stack, 1, 8, &10);
    }

    #[test]
    #[should_panic(expected = "Cannot pop all entries")]
    fn pop_panic() {
        let mut stack = NonEmptyStack::new(10u64);
        stack.pop();
    }

    #[test]
    #[should_panic(expected = "Cannot pop all entries")]
    fn pop_panic2() {
        let mut stack = NonEmptyStack::new(10u64);
        stack.push(20);
        stack.push(30);
        stack.pop();
        stack.pop();
        stack.pop();
    }

    #[test]
    fn last_mut() {
        let mut stack = NonEmptyStack::new(10u64);
        assert_len_cap_last!(stack, 1, 4, &10);

        *stack.last_mut() = 11;
        assert_len_cap_last!(stack, 1, 4, &11);
        *stack.last_mut() = 12;
        assert_len_cap_last!(stack, 1, 4, &12);

        stack.push(20);
        assert_len_cap_last!(stack, 2, 4, &20);
        *stack.last_mut() = 21;
        assert_len_cap_last!(stack, 2, 4, &21);
        *stack.last_mut() = 22;
        assert_len_cap_last!(stack, 2, 4, &22);
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
            let mut stack = NonEmptyStack::new(Droppy(10));
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
