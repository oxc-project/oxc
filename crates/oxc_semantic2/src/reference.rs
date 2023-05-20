use oxc_index::{Idx, NonZeroIdx};
use oxc_span::Atom;

use crate::symbol::SymbolId;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ReferenceId(NonZeroIdx);

impl Idx for ReferenceId {
    fn new(idx: usize) -> Self {
        Self(NonZeroIdx::new(idx))
    }

    fn index(self) -> usize {
        self.0.index()
    }
}
#[cfg(target_pointer_width = "64")]
mod size_asserts {
    oxc_index::static_assert_size!(Option<super::ReferenceId>, 8);
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub name: Atom,
    pub symbol_id: Option<SymbolId>,
}

impl Reference {
    pub fn new(name: Atom) -> Self {
        Self { name, symbol_id: None }
    }
}
