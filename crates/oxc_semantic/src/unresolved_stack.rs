use oxc_str::Ident;
use oxc_syntax::{reference::ReferenceId, scope::ScopeId};

/// Flat list of unresolved references collected during AST traversal.
///
/// Instead of maintaining per-scope hashmaps and merging them on scope exit (bubble-up),
/// references are collected flat and resolved in a single pass after traversal (walk-up).
/// This eliminates all hashmap drain+insert operations during scope exit.
pub struct UnresolvedReferences<'a> {
    /// Flat list of `(name, reference_id, resolution_start_scope)` entries
    /// collected during traversal. Most references use their recorded scope;
    /// the override is only set for unresolved function-parameter references
    /// which must skip declarations in their own function body.
    references: Vec<(Ident<'a>, ReferenceId, Option<ScopeId>)>,
}

impl<'a> UnresolvedReferences<'a> {
    pub(crate) fn new() -> Self {
        Self { references: Vec::new() }
    }

    /// Reserve exactly `additional` more slots in the underlying `Vec`.
    /// Avoids growth reallocations when the expected count is known up-front
    /// (typically from [`crate::Stats::count`]).
    #[inline]
    pub(crate) fn reserve_exact(&mut self, additional: usize) {
        self.references.reserve_exact(additional);
    }

    /// Push an unresolved reference to the flat list.
    #[inline]
    pub(crate) fn push(&mut self, name: Ident<'a>, reference_id: ReferenceId) {
        self.references.push((name, reference_id, None));
    }

    /// Get the current length, used as a checkpoint for early resolution.
    #[inline]
    pub(crate) fn checkpoint(&self) -> usize {
        self.references.len()
    }

    /// Take all collected references, leaving the list empty. O(1) pointer swap.
    #[inline]
    pub(crate) fn take(&mut self) -> Vec<(Ident<'a>, ReferenceId, Option<ScopeId>)> {
        std::mem::take(&mut self.references)
    }

    /// Current number of unresolved references.
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.references.len()
    }

    /// Read a reference by index, by value.
    ///
    /// Used by [`crate::SemanticBuilder::resolve_references_for_current_scope`]
    /// to process the list in-place without allocating a temporary `Vec`. All
    /// entry fields are `Copy`, so this hands the caller an owned tuple that's
    /// detached from the underlying borrow.
    ///
    /// # Panics
    /// Panics if `idx >= self.len()`.
    #[inline]
    pub(crate) fn get(&self, idx: usize) -> (Ident<'a>, ReferenceId, Option<ScopeId>) {
        self.references[idx]
    }

    /// Overwrite the reference at `idx` (write-cursor support for in-place
    /// processing).
    ///
    /// # Panics
    /// Panics if `idx >= self.len()`.
    #[inline]
    pub(crate) fn set(
        &mut self,
        idx: usize,
        name: Ident<'a>,
        reference_id: ReferenceId,
        resolution_start_scope_id: Option<ScopeId>,
    ) {
        self.references[idx] = (name, reference_id, resolution_start_scope_id);
    }

    /// Truncate the list to `len`, removing references at the end.
    #[inline]
    pub(crate) fn truncate(&mut self, len: usize) {
        self.references.truncate(len);
    }
}
