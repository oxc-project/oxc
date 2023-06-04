#![feature(let_chains)]

#[cfg(feature = "serde")]
mod serialize;

pub mod hir;
mod hir_builder;
pub mod hir_util;
pub mod precedence;
mod visit_mut;

use oxc_index::define_index_type;

pub use crate::{hir_builder::HirBuilder, visit_mut::VisitMut};

define_index_type! {
    pub struct HirId = usize;
}
