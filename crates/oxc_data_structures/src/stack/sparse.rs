use super::{NonEmptyStack, Stack};

/// Stack which is sparsely filled.
///
/// Functionally equivalent to [`NonEmptyStack<Option<T>>`], but more memory-efficient
/// in cases where majority of entries in the stack will be empty (`None`).
///
/// It has the same advantages as [`NonEmptyStack`] in terms of [`last`] and [`last_mut`] being
/// infallible and branchless, and with very fast lookup (without any pointer maths).
/// [`SparseStack`]'s advantage over [`NonEmptyStack`] is less memory usage for empty entries (`None`).
///
/// Stack is initialized with a single entry which can never be popped off.
/// If `Program` has a entry on the stack, can use this initial entry for it. Get value for `Program`
/// in `exit_program` visitor with [`take_last`] instead of [`pop`].
///
/// The stack is stored as 2 arrays:
/// 1. `has_values` - Records whether an entry on the stack has a value or not (`Some` or `None`).
/// 2. `values` - Where the stack entry *does* have a value, it's stored in this array.
///
/// Memory is only consumed for values where values exist.
///
/// Where value (`T`) is large, and most entries have no value, this will be a significant memory saving.
///
/// e.g. if `T` is 24 bytes, and 90% of stack entries have no values:
/// * `Vec<Option<T>>` is 24 bytes per entry (or 32 bytes if `T` has no niche).
/// * `NonEmptyStack<Option<T>>` is same.
/// * `SparseStack<T>` is 4 bytes per entry.
///
/// When the stack grows and reallocates, `SparseStack` has less memory to copy, which is a performance
/// win too.
///
/// To simplify implementation, zero size types are not supported (`SparseStack<()>`).
///
/// [`last`]: SparseStack::last
/// [`last_mut`]: SparseStack::last_mut
/// [`take_last`]: SparseStack::take_last
/// [`pop`]: SparseStack::pop
/// [`NonEmptyStack<Option<T>>`]: NonEmptyStack
pub struct SparseStack<T> {
    has_values: NonEmptyStack<bool>,
    values: Stack<T>,
}

impl<T> Default for SparseStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SparseStack<T> {
    /// Maximum capacity for filled entries (`Some`).
    ///
    /// Unless `size_of::<T>() == 1`, `MAX_FILLED_CAPACITY` is lower than [`MAX_TOTAL_CAPACITY`].
    ///
    /// Both are effectively unlimited on 64-bit systems.
    ///
    /// [`MAX_TOTAL_CAPACITY`]: Self::MAX_TOTAL_CAPACITY
    pub const MAX_FILLED_CAPACITY: usize = Stack::<T>::MAX_CAPACITY;

    /// Maximum capacity for entries (either `Some` or `None`).
    ///
    /// Effectively unlimited on 64-bit systems.
    pub const MAX_TOTAL_CAPACITY: usize = NonEmptyStack::<bool>::MAX_CAPACITY;

    /// Create new `SparseStack`.
    ///
    /// # Panics
    /// Panics if `T` is a zero-sized type.
    pub fn new() -> Self {
        // `has_values` starts with a single empty entry, which will never be popped off.
        // This means `take_last`, `last_or_init`, and `last_mut_or_init` can all be infallible,
        // as there's always an entry on the stack to read.
        Self { has_values: NonEmptyStack::new(false), values: Stack::new() }
    }

    /// Create new `SparseStack` with pre-allocated capacity.
    ///
    /// * `total_capacity` is capacity for any entries (either `Some` or `None`). Cannot be 0.
    /// * `filled_capacity` is capacity for full entries (`Some`).
    ///
    /// # Panics
    /// Panics if any of these requirements are not satisfied:
    /// * `T` must not be a zero-sized type.
    /// * `total_capacity` must not be 0.
    /// * `total_capacity` must not exceed `Self::MAX_TOTAL_CAPACITY`.
    /// * `filled_capacity` must not exceed `Self::MAX_FILLED_CAPACITY`.
    #[inline]
    pub fn with_capacity(total_capacity: usize, filled_capacity: usize) -> Self {
        Self {
            has_values: NonEmptyStack::with_capacity(total_capacity, false),
            values: Stack::with_capacity(filled_capacity),
        }
    }

    /// Push an entry to the stack.
    #[inline]
    pub fn push(&mut self, value: Option<T>) {
        let has_value = if let Some(value) = value {
            self.values.push(value);
            true
        } else {
            false
        };
        self.has_values.push(has_value);
    }

    /// Pop last entry from the stack.
    ///
    /// # Panics
    /// Panics if the stack has only 1 entry on it.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        let has_value = self.has_values.pop();
        if has_value {
            debug_assert!(!self.values.is_empty());
            // SAFETY: Last `self.has_values` is only `true` if there's a corresponding value in `self.values`.
            // This invariant is maintained in `push`, `take_last`, `last_or_init`, and `last_mut_or_init`.
            // We maintain it here too because we just popped from `self.has_values`, so that `true`
            // has been consumed at the same time we consume its corresponding value from `self.values`.
            let value = unsafe { self.values.pop_unchecked() };
            Some(value)
        } else {
            None
        }
    }

    /// Get value of last entry on the stack.
    #[inline]
    pub fn last(&self) -> Option<&T> {
        let has_value = *self.has_values.last();
        if has_value {
            debug_assert!(!self.values.is_empty());
            // SAFETY: Last `self.has_values` is only `true` if there's a corresponding value in `self.values`.
            // This invariant is maintained in `push`, `pop`, `take_last`, `last_or_init`, and `last_mut_or_init`.
            let value = unsafe { self.values.last_unchecked() };
            Some(value)
        } else {
            None
        }
    }

    /// Get value of last entry on the stack.
    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        let has_value = *self.has_values.last();
        if has_value {
            debug_assert!(!self.values.is_empty());
            // SAFETY: Last `self.has_values` is only `true` if there's a corresponding value in `self.values`.
            // This invariant is maintained in `push`, `pop`, `take_last`, `last_or_init`, and `last_mut_or_init`.
            let value = unsafe { self.values.last_mut_unchecked() };
            Some(value)
        } else {
            None
        }
    }

    /// Take value from last entry on the stack, leaving last entry empty.
    #[inline]
    pub fn take_last(&mut self) -> Option<T> {
        let has_value = self.has_values.last_mut();
        if *has_value {
            *has_value = false;

            debug_assert!(!self.values.is_empty());
            // SAFETY: Last `self.has_values` is only `true` if there's a corresponding value in `self.values`.
            // This invariant is maintained in `push`, `pop`, `last_or_init`, and `last_mut_or_init`.
            // We maintain it here too because we just set last `self.has_values` to `false`
            // at the same time as we consume the corresponding value from `self.values`.
            let value = unsafe { self.values.pop_unchecked() };
            Some(value)
        } else {
            None
        }
    }

    /// Initialize the value for last entry on the stack, if it has no value already.
    /// Return reference to value.
    #[inline]
    pub fn last_or_init<I: FnOnce() -> T>(&mut self, init: I) -> &T {
        let has_value = self.has_values.last_mut();
        if !*has_value {
            *has_value = true;
            self.values.push(init());
        }

        debug_assert!(!self.values.is_empty());
        // SAFETY: Last `self.has_values` is only `true` if there's a corresponding value in `self.values`.
        // This invariant is maintained in `push`, `pop`, `take_last`, and `last_mut_or_init`.
        // Here either last `self.has_values` was already `true`, or it's just been set to `true`
        // and a value pushed to `self.values` above.
        unsafe { self.values.last_unchecked() }
    }

    /// Initialize the value for last entry on the stack, if it has no value already.
    /// Return mutable reference to value.
    #[inline]
    pub fn last_mut_or_init<I: FnOnce() -> T>(&mut self, init: I) -> &mut T {
        let has_value = self.has_values.last_mut();
        if !*has_value {
            *has_value = true;
            self.values.push(init());
        }

        debug_assert!(!self.values.is_empty());
        // SAFETY: Last `self.has_values` is only `true` if there's a corresponding value in `self.values`.
        // This invariant is maintained in `push`, `pop`, `take_last`, and `last_or_init`.
        // Here either last `self.has_values` was already `true`, or it's just been set to `true`
        // and a value pushed to `self.values` above.
        unsafe { self.values.last_mut_unchecked() }
    }

    /// Get number of entries on the stack.
    ///
    /// Number of entries is always at least 1. Stack is never empty.
    #[inline]
    #[expect(clippy::len_without_is_empty)] // `is_empty` method is pointless. It's never empty.
    pub fn len(&self) -> usize {
        self.has_values.len()
    }

    /// Get number of filled entries on the stack.
    #[inline]
    pub fn filled_len(&self) -> usize {
        self.values.len()
    }

    /// Get capacity of stack for any entries (either `Some` or `None`).
    ///
    /// Capacity is always at least 1. Stack is never empty.
    #[inline]
    pub fn total_capacity(&self) -> usize {
        self.has_values.capacity()
    }

    /// Get capacity of stack for filled entries (`Some`).
    ///
    /// The capacity can be zero (unlike [`total_capacity`]).
    ///
    /// [`total_capacity`]: Self::total_capacity
    #[inline]
    pub fn filled_capacity(&self) -> usize {
        self.values.capacity()
    }

    /// Get filled entries of stack as a slice `&[T]`.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.values.as_slice()
    }

    /// Get filled entries of stack as a mutable slice `&mut [T]`.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.values.as_mut_slice()
    }
}
