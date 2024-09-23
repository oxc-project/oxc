/// Stack which is sparsely filled.
///
/// Functionally equivalent to a stack implemented as `Vec<Option<T>>`, but more memory-efficient
/// in cases where majority of entries in the stack will be empty (`None`).
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
    has_values: Vec<bool>,
    values: Vec<T>,
}

impl<T> SparseStack<T> {
    /// Create new `SparseStack`.
    pub fn new() -> Self {
        Self { has_values: vec![], values: vec![] }
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
    /// Panics if the stack is empty.
    pub fn pop(&mut self) -> Option<T> {
        let has_value = self.has_values.pop().unwrap();
        if has_value {
            debug_assert!(!self.values.is_empty());
            // SAFETY: Last `self.has_values` is only `true` if there's a corresponding value in `self.values`.
            // This invariant is maintained in `push`, `take`, and `get_or_init`.
            // We maintain it here too because we just popped from `self.has_values`, so that `true`
            // has been consumed at the same time we consume its corresponding value from `self.values`.
            let value = unsafe { self.values.pop().unwrap_unchecked() };
            Some(value)
        } else {
            None
        }
    }

    /// Take value from last entry on the stack.
    ///
    /// # Panics
    /// Panics if the stack is empty.
    pub fn take(&mut self) -> Option<T> {
        let has_value = self.has_values.last_mut().unwrap();
        if *has_value {
            *has_value = false;

            debug_assert!(!self.values.is_empty());
            // SAFETY: Last `self.has_values` is only `true` if there's a corresponding value in `self.values`.
            // This invariant is maintained in `push`, `pop`, and `get_or_init`.
            // We maintain it here too because we just set last `self.has_values` to `false`
            // at the same time as we consume the corresponding value from `self.values`.
            let value = unsafe { self.values.pop().unwrap_unchecked() };
            Some(value)
        } else {
            None
        }
    }

    /// Initialize the value for top entry on the stack, if it has no value already.
    /// Return reference to value.
    ///
    /// # Panics
    /// Panics if the stack is empty.
    pub fn get_or_init<I: FnOnce() -> T>(&mut self, init: I) -> &T {
        let has_value = self.has_values.last_mut().unwrap();
        if !*has_value {
            *has_value = true;
            self.values.push(init());
        }

        debug_assert!(!self.values.is_empty());
        // SAFETY: Last `self.has_values` is only `true` if there's a corresponding value in `self.values`.
        // This invariant is maintained in `push`, `pop`, and `take`.
        // Here either last `self.has_values` was already `true`, or it's just been set to `true`
        // and a value pushed to `self.values` above.
        unsafe { self.values.last().unwrap_unchecked() }
    }

    /// Get number of entries on the stack.
    #[inline]
    pub fn len(&self) -> usize {
        self.has_values.len()
    }

    /// Returns `true` if stack is empty.
    #[inline]
    #[expect(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.has_values.is_empty()
    }
}
