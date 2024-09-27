use super::NonEmptyStack;

/// Stack which is sparsely filled.
///
/// Functionally equivalent to a stack implemented as `Vec<Option<T>>`, but more memory-efficient
/// in cases where majority of entries in the stack will be empty (`None`).
///
/// Stack is initialized with a single entry which can never be popped off.
/// If `Program` has a entry on the stack, can use this initial entry for it. Get value for `Program`
/// in `exit_program` visitor with `SparseStack::take_last` instead of `SparseStack::pop`.
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
/// * `SparseStack<T>` is 4 bytes per entry.
///
/// When the stack grows and reallocates, `SparseStack` has less memory to copy, which is a performance
/// win too.
pub struct SparseStack<T> {
    has_values: NonEmptyStack<bool>,
    values: Vec<T>,
}

impl<T> SparseStack<T> {
    /// Create new `SparseStack`.
    pub fn new() -> Self {
        // `has_values` starts with a single empty entry, which will never be popped off.
        // This means `take_last`, `last_or_init`, and `last_mut_or_init` can all be infallible,
        // as there's always an entry on the stack to read.
        Self { has_values: NonEmptyStack::new(false), values: vec![] }
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
            let value = unsafe { self.values.pop().unwrap_unchecked() };
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
            let value = unsafe { self.values.last().unwrap_unchecked() };
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
            let value = unsafe { self.values.pop().unwrap_unchecked() };
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
        unsafe { self.values.last().unwrap_unchecked() }
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

        // SAFETY: Last `self.has_values` is only `true` if there's a corresponding value in `self.values`.
        // This invariant is maintained in `push`, `pop`, `take_last`, and `last_or_init`.
        // Here either last `self.has_values` was already `true`, or it's just been set to `true`
        // and a value pushed to `self.values` above.
        unsafe { self.values.last_mut().unwrap_unchecked() }
    }

    /// Get number of entries on the stack.
    ///
    /// Number of entries is always at least 1. Stack is never empty.
    #[inline]
    pub fn len(&self) -> usize {
        self.has_values.len()
    }
}
