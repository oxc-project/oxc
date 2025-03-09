use oxc_allocator::Allocator;
use oxc_semantic::Scoping;

use super::TraverseCtx;

/// Wrapper around [`TraverseCtx`], allowing its reuse.
///
/// We cannot expose ability to obtain an owned [`TraverseCtx`], as it's then possible to circumvent
/// the safety invariants of [`TraverseAncestry`].
///
/// This wrapper type can safely be passed to user code as only ways it can be used are to:
///
/// * Call `traverse_mut_with_ctx`, which maintains safety invariants.
/// * Unwrap it to [`Scoping`], which discards the sensitive [`TraverseAncestry`] in the process.
///
/// [`TraverseAncestry`]: super::TraverseAncestry
#[repr(transparent)]
pub struct ReusableTraverseCtx<'a>(TraverseCtx<'a>);

// Public methods
impl<'a> ReusableTraverseCtx<'a> {
    /// Create new [`ReusableTraverseCtx`].
    pub fn new(scoping: Scoping, allocator: &'a Allocator) -> Self {
        Self(TraverseCtx::new(scoping, allocator))
    }

    /// Consume [`ReusableTraverseCtx`] and return [`Scoping`].
    pub fn into_scoping(self) -> Scoping {
        self.0.scoping.into_scoping()
    }

    /// Unwrap [`TraverseCtx`] in a [`ReusableTraverseCtx`].
    ///
    /// Only for use in tests. Allows circumventing the safety invariants of [`TraverseAncestry`].
    ///
    /// # SAFETY
    /// Caller must ensure [`TraverseCtx`] returned by this method is not used unsoundly.
    /// See [`TraverseAncestry`] for details of the invariants.
    ///
    /// [`TraverseAncestry`]: super::TraverseAncestry
    #[inline]
    pub unsafe fn unwrap(self) -> TraverseCtx<'a> {
        self.0
    }
}

// Internal methods
impl<'a> ReusableTraverseCtx<'a> {
    /// Mutably borrow [`TraverseCtx`] from a [`ReusableTraverseCtx`].
    #[inline] // because this function is a no-op at run time
    pub(crate) fn get_mut(&mut self) -> &mut TraverseCtx<'a> {
        &mut self.0
    }
}
