use oxc_allocator::{Allocator, Vec as ArenaVec};
use oxc_data_structures::assert_unchecked;
use oxc_span::{ArenaIdentHashMap, Ident};
use oxc_syntax::reference::ReferenceId;

/// Stores reference IDs for a single identifier name within a scope.
pub type ReferenceIds<'alloc> = ArenaVec<'alloc, ReferenceId>;

/// Unlike `ScopeTree`'s `UnresolvedReferences`, this type uses `Ident` as the key.
type TempUnresolvedReferences<'alloc> = ArenaIdentHashMap<'alloc, ReferenceIds<'alloc>>;

// Stack used to accumulate unresolved refs while traversing scopes.
// Indexed by scope depth. We recycle `UnresolvedReferences` instances during traversal
// to reduce allocations, so the stack grows to maximum scope depth, but never shrinks.
// See: <https://github.com/oxc-project/oxc/issues/4169>
// This stack abstraction uses the invariant that stack only grows to avoid bounds checks.
pub struct UnresolvedReferencesStack<'a> {
    stack: ArenaVec<'a, TempUnresolvedReferences<'a>>,
    allocator: &'a Allocator,
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

    pub(crate) fn new(allocator: &'a Allocator) -> Self {
        // Invoke `ASSERT_INVARIANTS` assertions. Without this line, the assertions are ignored.
        const { Self::ASSERT_INVARIANTS };

        let mut stack = ArenaVec::with_capacity_in_scratch(Self::INITIAL_SIZE, allocator);
        for _ in 0..Self::INITIAL_SIZE {
            stack.push(TempUnresolvedReferences::new_in_scratch(allocator));
        }
        Self { stack, allocator, current_scope_depth: Self::INITIAL_DEPTH }
    }

    pub(crate) fn increment_scope_depth(&mut self) {
        self.current_scope_depth += 1;

        // Grow stack if required to ensure `self.stack[self.current_scope_depth]` is in bounds.
        if self.stack.len() <= self.current_scope_depth {
            self.stack.push(TempUnresolvedReferences::new_in_scratch(self.allocator));
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

    #[inline]
    pub(crate) fn add_reference(&mut self, name: Ident<'a>, reference_id: ReferenceId) {
        if let Some(reference_ids) = self.current_mut().get_mut(&name) {
            reference_ids.push(reference_id);
            return;
        }

        let mut reference_ids = ReferenceIds::new_in_scratch(self.allocator);
        reference_ids.push(reference_id);
        self.current_mut().insert(name, reference_ids);
    }

    /// Get unresolved references hash map for current scope.
    #[inline]
    pub(crate) fn current_mut(&mut self) -> &mut TempUnresolvedReferences<'a> {
        // SAFETY: `stack.len() > current_scope_depth` initially.
        // Thereafter, `stack` never shrinks, only grows.
        // `current_scope_depth` is only increased in `increment_scope_depth`,
        // and it grows `stack` to ensure `stack.len()` always exceeds `current_scope_depth`.
        // So this read is always guaranteed to be in bounds.
        unsafe { self.stack.get_unchecked_mut(self.current_scope_depth) }
    }

    /// Get unresolved references hash maps for current scope, and parent scope.
    #[inline]
    pub(crate) fn current_and_parent_mut(
        &mut self,
    ) -> (&mut TempUnresolvedReferences<'a>, &mut TempUnresolvedReferences<'a>) {
        // Assert invariants to remove bounds checks in code below.
        // SAFETY: `current_scope_depth` starts at 1, and is only decremented
        // in `decrement_scope_depth` which checks it doesn't go below 1.
        unsafe { assert_unchecked!(self.current_scope_depth > 0) };
        // SAFETY: `stack.len() > current_scope_depth` initially.
        // Thereafter, `stack` never shrinks, only grows.
        // `current_scope_depth` is only increased in `increment_scope_depth`,
        // and it grows `stack` to ensure `stack.len()` always exceeds `current_scope_depth`.
        unsafe { assert_unchecked!(self.stack.len() > self.current_scope_depth) };

        let (head, tail) = self.stack.split_at_mut(self.current_scope_depth);
        let parent = &mut head[self.current_scope_depth - 1];
        let current = &mut tail[0];
        (current, parent)
    }

    #[inline]
    pub(crate) fn into_root(mut self) -> TempUnresolvedReferences<'a> {
        // SAFETY: Stack starts with a non-zero size and never shrinks.
        // This assertion removes bounds check in `swap_remove`.
        unsafe { assert_unchecked!(!self.stack.is_empty()) };
        // Use `swap_remove(0)` instead of `into_iter().next().unwrap()` to avoid
        // creating an iterator just to get the first element.
        self.stack.swap_remove(0)
    }
}
