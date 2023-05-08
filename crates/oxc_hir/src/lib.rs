#![feature(let_chains)]

#[cfg(feature = "serde")]
mod serialize;

pub mod hir;
mod hir_builder;
mod visit_mut;

use oxc_index::Idx;

pub use crate::hir_builder::HirBuilder;
pub use crate::visit_mut::VisitMut;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HirId(usize);

impl Idx for HirId {
    fn new(idx: usize) -> Self {
        Self(idx)
    }

    fn index(self) -> usize {
        self.0
    }
}
