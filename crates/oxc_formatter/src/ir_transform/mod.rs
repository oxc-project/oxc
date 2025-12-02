//! This module contains all IR transforms for sorting and aesthetically features.
//! Currently, it only includes the `SortImportsTransform`.
//!
//! There were several approaches to achieve sorting.
//! - 1. Sort at the AST level.
//! - 2. Sort after converting AST to IR (current approach).
//!
//! At first glance, the former seems simpler and faster, but in practice, there are inconvenient aspects.
//!
//! Sorting requires referencing comments and blank lines while traversing the AST.
//! Frequent lookups are necessary, and there wasn't a dramatic improvement in speed.
//!
//! Additionally, comments are printed in a top-down manner,
//! and sorting that disrupts this order would complicate the code.
//!
//! Despite some overhead, sorting the IR allows us to avoid impacting the existing complex code that converts AST to IR.
//!
//! For more details, refer to the experimental PR below.
//! - <https://github.com/oxc-project/oxc/pull/14647>
//! - <https://github.com/oxc-project/oxc/pull/14651>

mod sort_imports;

pub use sort_imports::*;
