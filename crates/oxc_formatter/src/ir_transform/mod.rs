//! This module contains IR transforms for sorting and aesthetic features.
//! `sort_common` holds the target-independent engine (group vocabulary + matcher, permutation
//! helpers); `sort_imports` is its first consumer. Further sorting targets are tracked in
//! <https://github.com/oxc-project/oxc/issues/22521>.
//!
//! Several approaches were considered:
//! - 1. Sort at the AST level.
//! - 2. Sort the entire IR after AST→IR conversion is fully done.
//! - 3. Sort the IR for each run of `ImportDeclaration`s during AST→IR conversion (current approach).
//!
//! Sorting at the AST level (1) requires referencing comments and blank lines while traversing the AST,
//! which needs frequent lookups, with no dramatic speed improvement.
//! Additionally, comments are printed in a top-down manner, and sorting that disrupts this order
//! would complicate the code.
//!
//! Sorting the whole IR after the fact (2) avoids impacting AST→IR conversion,
//! but requires a full second pass over the entire IR,
//! and splicing reordered elements forces copying everything that follows the imports.
//!
//! The current approach (3) is a middle ground:
//! import sorting is invoked when a run of consecutive `ImportDeclaration`s ends.
//! Because the run sits at the tail of the buffer,
//! splicing the sorted result is just popping and pushing, no full IR copy required.
//!
//! For more details, refer to the related PRs below.
//! - <https://github.com/oxc-project/oxc/pull/14647> (initial experiment, AST level)
//! - <https://github.com/oxc-project/oxc/pull/14651> (initial experiment, IR level)
//! - <https://github.com/oxc-project/oxc/pull/22065> (move to during-IR-construction)

mod sort_common;
mod sort_imports;

pub mod options;

// First consumer lands with `sort.namedImports` (a later PR); remove this allowance then.
#[allow(dead_code, clippy::allow_attributes)]
pub mod sorted_list;

pub use sort_imports::*;
