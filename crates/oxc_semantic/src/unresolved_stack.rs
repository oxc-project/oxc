use oxc_str::Ident;
use oxc_syntax::{reference::ReferenceId, scope::ScopeId};

#[derive(Clone, Copy)]
pub struct UnresolvedReference<'a> {
    pub name: Ident<'a>,
    pub reference_id: ReferenceId,
    /// Scope where the next lookup should begin.
    pub lookup_scope_id: ScopeId,
}

/// Flat list of unresolved references collected during AST traversal.
///
/// Instead of maintaining per-scope hashmaps and merging them on scope exit (bubble-up),
/// references are collected flat and resolved in a single pass after traversal (walk-up).
/// This eliminates all hashmap drain+insert operations during scope exit.
#[derive(Default)]
pub struct UnresolvedReferences<'a> {
    /// The lookup scope initially matches the reference's recorded scope and advances past
    /// function bodies which are not visible to parameter references.
    references: Vec<UnresolvedReference<'a>>,
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
    pub(crate) fn push(&mut self, reference: UnresolvedReference<'a>) {
        self.references.push(reference);
    }

    /// Take all collected references, leaving the list empty. O(1) pointer swap.
    #[inline]
    pub(crate) fn take(&mut self) -> Vec<UnresolvedReference<'a>> {
        std::mem::take(&mut self.references)
    }

    /// Current number of unresolved references.
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.references.len()
    }

    /// Retain only the references in `self.references[start..]` for which `keep` returns `true`.
    /// Like [`Vec::retain_mut`], but restricted to the suffix starting at `start`.
    /// `keep` may modify the reference in place.
    #[inline]
    pub(crate) fn retain_from(
        &mut self,
        start: usize,
        mut keep: impl FnMut(&mut UnresolvedReference<'a>) -> bool,
    ) {
        self.references.extract_if(start.., |reference| !keep(reference)).for_each(drop);
    }
}
