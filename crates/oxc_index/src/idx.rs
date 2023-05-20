use std::{fmt::Debug, hash::Hash, num::NonZeroUsize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NonZeroIdx(NonZeroUsize);

impl Default for NonZeroIdx {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Idx for NonZeroIdx {
    fn new(idx: usize) -> Self {
        Self(unsafe { NonZeroUsize::new_unchecked(idx + 1) })
    }

    fn index(self) -> usize {
        self.0.get() - 1
    }
}

/// Represents some newtyped `usize` wrapper.
///
/// Purpose: avoid mixing indexes for different bitvector domains.
pub trait Idx: 'static + Copy + Eq + PartialEq + Debug + Hash {
    fn new(idx: usize) -> Self;

    fn index(self) -> usize;

    #[inline]
    fn increment(&mut self) {
        *self = self.plus(1);
    }

    #[inline]
    fn increment_by(&mut self, amount: usize) {
        *self = self.plus(amount);
    }

    #[inline]
    #[must_use = "Use `increment_by` if you wanted to update the index in-place"]
    fn plus(self, amount: usize) -> Self {
        Self::new(self.index() + amount)
    }
}

impl Idx for usize {
    #[inline]
    fn new(idx: usize) -> Self {
        idx
    }

    #[inline]
    fn index(self) -> usize {
        self
    }
}

impl Idx for u32 {
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn new(idx: usize) -> Self {
        debug_assert!(Self::try_from(idx).is_ok());
        idx as Self
    }

    #[inline]
    fn index(self) -> usize {
        self as usize
    }
}
