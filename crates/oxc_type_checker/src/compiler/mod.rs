//! Port of typescript-go's `internal/compiler` package.
//!
//! Builds a [`Program`] from a project's root files: parse each file, follow its imports and
//! module augmentations to load the dependent files (in parallel via rayon), and collect them —
//! in tsgo's dependency-first program order — into an index-vec keyed by [`FileId`]. File
//! reading, path normalization, and module resolution reuse `oxc_resolver`. Binding and type
//! checking are later steps, as are lib files, type reference directives, and project references.

mod fileloader;
mod filesparser;
mod host;
mod program;
mod references;
mod source_file;

pub use program::{FileId, Program, ProgramOptions};
pub use source_file::SourceFile;
