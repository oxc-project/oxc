#![feature(let_chains)]

use std::num::NonZeroU32;

#[cfg(feature = "serde")]
mod serialize;

pub mod hir;
mod hir_builder;
mod visit_mut;

#[cfg(feature = "serde")]
use serde::Serialize;

pub use crate::hir_builder::HirBuilder;
pub use crate::visit_mut::VisitMut;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct HirId(NonZeroU32);

impl Default for HirId {
    fn default() -> Self {
        Self(unsafe { NonZeroU32::new_unchecked(1) })
    }
}

impl HirId {
    #[must_use]
    pub fn increment(&self) -> Self {
        Self(self.0.saturating_add(1))
    }
}
