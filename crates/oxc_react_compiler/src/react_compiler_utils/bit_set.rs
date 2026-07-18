//! Bit set keyed by a typed index.
//!
//! [`BitSet<I>`] is a replacement for `FxHashSet<I>` where `I` is a dense [`oxc_index`]
//! index type (e.g. the HIR IDs defined with `define_nonmax_u32_index_type!`).
//!
//! Prefer it over a hash set when the key space is a dense, bounded universe of IDs
//! (per-function or per-program index spaces):
//!
//! - Membership is a single shift + mask — no hashing.
//! - Memory is 1 bit per possible ID, so even a mostly-empty set over a universe of
//!   thousands of IDs costs a few hundred bytes.
//! - All methods forward to a shared non-generic implementation, so each additional
//!   `I` costs almost no binary size, unlike hash sets which monomorphize several KiB
//!   of probe/grow machinery per element type.
//! - Iteration yields IDs in ascending index order, which is deterministic across runs.
//!
//! Stick with a hash set when the key space is unbounded or extremely sparse relative
//! to its maximum index (e.g. string keys, or IDs sampled from a huge universe).
//!
//! The API is intentionally limited to what current callers need (`insert` / `contains`
//! / `iter`); grow it as the `FxHashSet` -> `BitSet` migration reaches more passes.

use std::{fmt, marker::PhantomData};

use oxc_index::Idx;

const BITS: usize = usize::BITS as usize;

/// Non-generic implementation backing [`BitSet`].
///
/// All real code lives here so that `BitSet<I>` instantiations share it.
#[derive(Clone, Default)]
struct RawBitSet {
    words: Vec<usize>,
}

impl RawBitSet {
    fn insert(&mut self, index: usize) -> bool {
        let word_index = index / BITS;
        if word_index >= self.words.len() {
            self.words.resize(word_index + 1, 0);
        }
        let bit = 1 << (index % BITS);
        let word = &mut self.words[word_index];
        let is_new = *word & bit == 0;
        *word |= bit;
        is_new
    }

    fn contains(&self, index: usize) -> bool {
        match self.words.get(index / BITS) {
            Some(word) => word & (1 << (index % BITS)) != 0,
            None => false,
        }
    }

    fn iter(&self) -> RawIter<'_> {
        RawIter {
            words: &self.words,
            word_index: 0,
            current: self.words.first().copied().unwrap_or(0),
        }
    }
}

/// Iterator over set bit indices, ascending.
struct RawIter<'a> {
    words: &'a [usize],
    word_index: usize,
    current: usize,
}

impl Iterator for RawIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        while self.current == 0 {
            self.word_index += 1;
            self.current = *self.words.get(self.word_index)?;
        }
        let bit = self.current.trailing_zeros() as usize;
        // Clear the lowest set bit.
        self.current &= self.current - 1;
        Some(self.word_index * BITS + bit)
    }
}

/// A set of typed indices backed by a bit vector. See the [module docs](self) for
/// when to prefer this over `FxHashSet<I>`.
#[derive(Clone)]
pub struct BitSet<I: Idx> {
    raw: RawBitSet,
    marker: PhantomData<I>,
}

impl<I: Idx> BitSet<I> {
    /// Create an empty set.
    #[inline]
    pub fn new() -> Self {
        Self { raw: RawBitSet::default(), marker: PhantomData }
    }

    /// Add `id` to the set. Returns `true` if it was not already present.
    #[inline]
    pub fn insert(&mut self, id: I) -> bool {
        self.raw.insert(id.index())
    }

    /// Returns `true` if `id` is in the set.
    #[inline]
    pub fn contains(&self, id: I) -> bool {
        self.raw.contains(id.index())
    }

    /// Iterate over the IDs in the set in ascending index order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = I> + '_ {
        self.raw.iter().map(I::from_usize)
    }
}

impl<I: Idx> Default for BitSet<I> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Idx> Extend<I> for BitSet<I> {
    fn extend<T: IntoIterator<Item = I>>(&mut self, iter: T) {
        for id in iter {
            self.insert(id);
        }
    }
}

impl<I: Idx> FromIterator<I> for BitSet<I> {
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let mut set = Self::new();
        set.extend(iter);
        set
    }
}

impl<I: Idx> fmt::Debug for BitSet<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}
