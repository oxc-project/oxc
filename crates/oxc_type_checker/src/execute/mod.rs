//! Port of typescript-go's `internal/execute` package.
//!
//! Drives the compiler from the command line.

mod tsc;

pub use tsc::command_line;
