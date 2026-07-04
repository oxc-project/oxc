//! Port of typescript-go's `internal/compiler` package.
//!
//! Builds a [`Program`] from a set of root files by reading, parsing, and binding each one —
//! the port of tsgo's `NewProgram` -> `processAllProgramFiles` root-file parse pass. File
//! reading and path normalization reuse `oxc_resolver` (tsgo's `internal/vfs` + `internal/tspath`).

mod fileloader;
mod filesparser;
mod host;
mod program;
mod source_file;

pub use host::CompilerHost;
pub use program::Program;
pub use source_file::{FileId, SourceFile};
