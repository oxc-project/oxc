#![feature(let_chains)]

use std::num::NonZeroU32;

pub mod hir;
pub mod hir_builder;
pub mod lower;

#[cfg(feature = "serde")]
use serde::Serialize;

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
