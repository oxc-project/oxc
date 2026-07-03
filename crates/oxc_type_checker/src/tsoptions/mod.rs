//! Port of typescript-go's `internal/tsoptions` package.
//!
//! Command-line and (later) `tsconfig.json` option parsing.

mod commandlineparser;

pub use commandlineparser::{TypeCheckCommand, parse_command_line};
