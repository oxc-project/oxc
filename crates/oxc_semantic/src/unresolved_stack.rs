use oxc_str::Ident;
use oxc_syntax::reference::ReferenceId;

/// Flat list of unresolved references collected during AST traversal.
///
/// Instead of maintaining per-scope hashmaps and merging them on scope exit (bubble-up),
/// references are collected flat and resolved in a single pass after traversal (walk-up).
/// This eliminates all hashmap drain+insert operations during scope exit.
pub struct UnresolvedReferences<'a> {
    /// Flat list of (name, reference_id) pairs collected during traversal.
    references: Vec<(Ident<'a>, ReferenceId)>,
}

impl<'a> UnresolvedReferences<'a> {
    pub(crate) fn new() -> Self {
        Self { references: Vec::new() }
    }

    /// Push an unresolved reference to the flat list.
    #[inline]
    pub(crate) fn push(&mut self, name: Ident<'a>, reference_id: ReferenceId) {
        self.references.push((name, reference_id));
    }

    /// Get the current length, used as a checkpoint for early resolution.
    #[inline]
    pub(crate) fn checkpoint(&self) -> usize {
        self.references.len()
    }

    /// Take all collected references, leaving the list empty. O(1) pointer swap.
    #[inline]
    pub(crate) fn take(&mut self) -> Vec<(Ident<'a>, ReferenceId)> {
        std::mem::take(&mut self.references)
    }

    /// Get a slice of references from `start` to end and the length for truncation.
    #[inline]
    pub(crate) fn slice_from(&self, start: usize) -> &[(Ident<'a>, ReferenceId)] {
        &self.references[start..]
    }

    /// Truncate the list to `len`, removing references at the end.
    #[inline]
    pub(crate) fn truncate(&mut self, len: usize) {
        self.references.truncate(len);
    }
}
