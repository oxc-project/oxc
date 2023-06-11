#![feature(let_chains)]

#[cfg(feature = "serde")]
mod serialize;

pub mod hir;
mod hir_builder;
mod hir_kind;
pub mod hir_util;
pub mod precedence;
mod visit;
mod visit_mut;

use oxc_index::define_index_type;

pub use crate::{hir_builder::HirBuilder, hir_kind::HirKind, visit::Visit, visit_mut::VisitMut};

define_index_type! {
    pub struct HirId = usize;
}
