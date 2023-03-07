use std::{
    hash::Hash,
    num::NonZeroUsize,
    ops::{Index, IndexMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(NonZeroUsize);

impl Default for SymbolId {
    fn default() -> Self {
        Self::new(1)
    }
}

impl<T> Index<SymbolId> for Vec<T> {
    type Output = T;

    fn index(&self, id: SymbolId) -> &Self::Output {
        &self[id.index0()]
    }
}

impl<T> IndexMut<SymbolId> for Vec<T> {
    fn index_mut(&mut self, id: SymbolId) -> &mut T {
        &mut self[id.index0()]
    }
}

impl SymbolId {
    #[must_use]
    pub fn new(n: usize) -> Self {
        unsafe { Self(NonZeroUsize::new_unchecked(n)) }
    }

    #[must_use]
    pub(crate) fn index0(self) -> usize {
        self.0.get() - 1
    }
}
