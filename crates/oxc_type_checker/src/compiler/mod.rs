//! Port of typescript-go's `internal/compiler` package.
//!
//! Collects a program's root files — expanded from the tsconfig or given on the command line —
//! into an index-vec keyed by [`FileId`], normalizing and deduplicating them. Path normalization
//! reuses `oxc_resolver` (tsgo's `internal/tspath`). Parsing and binding those files is a later
//! step.

mod fileloader;
mod program;

pub use program::{FileId, Program};
