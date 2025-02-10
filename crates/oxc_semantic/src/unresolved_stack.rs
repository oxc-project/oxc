use assert_unchecked::assert_unchecked;
use rustc_hash::FxHashMap;

use oxc_span::Atom;
use oxc_syntax::reference::ReferenceId;

/// Unlike `ScopeTree`'s `UnresolvedReferences`, this type uses `Atom` as the key,
/// and uses a heap-allocated hashmap (not arena-allocated)
type TempUnresolvedReferences<'a> = FxHashMap<Atom<'a>, Vec<ReferenceId>>;

// Stack used to accumulate unresolved refs while traversing scopes.
// Indexed by scope depth. We recycle `UnresolvedReferences` instances during traversal
// to reduce allocations, so the stack grows to maximum scope depth, but never shrinks.
// See: <https://github.com/oxc-project/oxc/issues/4169>
// This stack abstraction uses the invariant that stack only grows to avoid bounds checks.
pub(crate) struct UnresolvedReferencesStack<'a> {
    stack: Vec<TempUnresolvedReferences<'a>>,
    /// Current scope depth.
    /// 0 is global scope. 1 is `Program`.
    /// Incremented on entering a scope, and decremented on exit.
    current_scope_depth: usize,
}

impl<'a> UnresolvedReferencesStack<'a> {
    /// Initial scope depth.
    /// Start on 1 (`Program` scope depth).
    /// SAFETY: Must be >= 1 to ensure soundness of `current_and_parent_mut`.
    const INITIAL_DEPTH: usize = 1;

    /// Most programs will have at least 1 place where scope depth reaches 16,
    /// so initialize `stack` with this length, to reduce reallocations as it grows.
    /// This is just an estimate of a good initial size, but certainly better than
    /// `Vec`'s default initial capacity of 4.
    /// SAFETY: Must be >= 2 to ensure soundness of `current_and_parent_mut`.
    const INITIAL_SIZE: usize = 16;

    /// Assert invariants
    const ASSERT_INVARIANTS: () = {
        assert!(Self::INITIAL_DEPTH >= 1);
        assert!(Self::INITIAL_SIZE >= 2);
        assert!(Self::INITIAL_SIZE > Self::INITIAL_DEPTH);
    };

    pub(crate) fn new() -> Self {
        // Invoke `ASSERT_INVARIANTS` assertions. Without this line, the assertions are ignored.
        const { Self::ASSERT_INVARIANTS };

        let mut stack = vec![];
        stack.resize_with(Self::INITIAL_SIZE, TempUnresolvedReferences::default);
        Self { stack, current_scope_depth: Self::INITIAL_DEPTH }
    }

    pub(crate) fn increment_scope_depth(&mut self) {
        self.current_scope_depth += 1;

        // Grow stack if required to ensure `self.stack[self.current_scope_depth]` is in bounds
        if self.stack.len() <= self.current_scope_depth {
            self.stack.push(TempUnresolvedReferences::default());
        }
    }

    pub(crate) fn decrement_scope_depth(&mut self) {
        self.current_scope_depth -= 1;
        // This assertion is required to ensure depth does not go below 1.
        // If it did, would cause UB in `current_and_parent_mut`, which relies on
        // `current_scope_depth - 1` always being a valid index into `self.stack`.
        assert!(self.current_scope_depth > 0);
    }

    #[inline]
    pub(crate) fn scope_depth(&self) -> usize {
        self.current_scope_depth
    }

    /// Get unresolved references hash map for current scope
    #[inline]
    pub(crate) fn root(&self) -> &TempUnresolvedReferences<'a> {
        // SAFETY: `stack.len() > current_scope_depth` initially.
        // Thereafter, `stack` never shrinks, only grows.
        // `current_scope_depth` is only increased in `increment_scope_depth`,
        // and it grows `stack` to ensure `stack.len()` always exceeds `current_scope_depth`.
        // So this read is always guaranteed to be in bounds.
        unsafe { self.stack.get_unchecked(0) }
    }

    /// Get unresolved references hash map for current scope
    #[inline]
    pub(crate) fn current_mut(&mut self) -> &mut TempUnresolvedReferences<'a> {
        // SAFETY: `stack.len() > current_scope_depth` initially.
        // Thereafter, `stack` never shrinks, only grows.
        // `current_scope_depth` is only increased in `increment_scope_depth`,
        // and it grows `stack` to ensure `stack.len()` always exceeds `current_scope_depth`.
        // So this read is always guaranteed to be in bounds.
        unsafe { self.stack.get_unchecked_mut(self.current_scope_depth) }
    }

    /// Get unresolved references hash maps for current scope, and parent scope
    #[inline]
    pub(crate) fn current_and_parent_mut(
        &mut self,
    ) -> (&mut TempUnresolvedReferences<'a>, &mut TempUnresolvedReferences<'a>) {
        // Assert invariants to remove bounds checks in code below.
        // https://godbolt.org/z/vv5Wo5csv
        // SAFETY: `current_scope_depth` starts at 1, and is only decremented
        // in `decrement_scope_depth` which checks it doesn't go below 1.
        unsafe { assert_unchecked!(self.current_scope_depth > 0) };
        // SAFETY: `stack.len() > current_scope_depth` initially.
        // Thereafter, `stack` never shrinks, only grows.
        // `current_scope_depth` is only increased in `increment_scope_depth`,
        // and it grows `stack` to ensure `stack.len()` always exceeds `current_scope_depth`.
        unsafe { assert_unchecked!(self.stack.len() > self.current_scope_depth) };

        let mut iter = self.stack.iter_mut();
        let parent = iter.nth(self.current_scope_depth - 1).unwrap();
        let current = iter.next().unwrap();
        (current, parent)
    }

    #[inline]
    pub(crate) fn into_root(self) -> TempUnresolvedReferences<'a> {
        // SAFETY: Stack starts with a non-zero size and never shrinks.
        // This assertion removes bounds check in `.next()`.
        unsafe { assert_unchecked!(!self.stack.is_empty()) };
        self.stack.into_iter().next().unwrap()
    }
}
