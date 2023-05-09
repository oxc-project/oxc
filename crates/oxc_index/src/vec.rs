//! Adapted from <https://github.com/rust-lang/rust/blob/master/compiler/rustc_index/src/vec.rs>

use std::{
    fmt,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::Idx;

/// An owned contiguous collection of `T`s, indexed by `I` rather than by `usize`.
#[derive(Clone, Default)]
#[repr(transparent)]
pub struct IndexVec<I: Idx, T> {
    raw: Vec<T>,
    _marker: PhantomData<fn(&I)>,
}

impl<I: Idx, T> IndexVec<I, T> {
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self { raw: Vec::new(), _marker: PhantomData }
    }

    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    #[inline]
    #[must_use = "Use the returned Idx"]
    pub fn push(&mut self, d: T) -> I {
        let idx = self.next_index();
        self.raw.push(d);
        idx
    }

    /// Gives the next index that will be assigned when `push` is called.
    ///
    /// Manual bounds checks can be done using `idx < slice.next_index()`
    /// (as opposed to `idx.index() < slice.len()`).
    #[inline]
    #[must_use]
    pub fn next_index(&self) -> I {
        I::new(self.len())
    }
}

impl<I: Idx, T> Index<I> for IndexVec<I, T> {
    type Output = T;

    #[inline]
    fn index(&self, index: I) -> &T {
        &self.raw[index.index()]
    }
}

impl<I: Idx, T> IndexMut<I> for IndexVec<I, T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut T {
        &mut self.raw[index.index()]
    }
}

impl<I: Idx, T: fmt::Debug> fmt::Debug for IndexVec<I, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.raw, fmt)
    }
}
