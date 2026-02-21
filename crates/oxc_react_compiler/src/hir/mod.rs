mod hir_types;

pub mod assertions;
pub mod build_hir;
pub mod build_reactive_scope_terminals_hir;
pub mod compute_unconditional_blocks;
pub mod dominator;
pub mod environment;
pub mod find_context_identifiers;
pub mod globals;
pub mod hir_builder;
pub mod memoize_fbt_operands;
pub mod merge_consecutive_blocks;
pub mod merge_overlapping_reactive_scopes_hir;
pub mod object_shape;
pub mod print_hir;
pub mod propagate_scope_dependencies_hir;
pub mod prune_unused_labels_hir;
pub mod scope_dependency_utils;
pub mod types;
pub mod visitors;

pub use hir_types::*;
