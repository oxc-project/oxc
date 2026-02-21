mod hir_types;

pub mod assertions;
pub mod compute_unconditional_blocks;
pub mod dominator;
pub mod environment;
pub mod globals;
pub mod hir_builder;
pub mod merge_consecutive_blocks;
pub mod object_shape;
pub mod print_hir;
pub mod prune_unused_labels_hir;
pub mod types;
pub mod visitors;

pub use hir_types::*;
