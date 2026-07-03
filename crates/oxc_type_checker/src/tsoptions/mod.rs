//! Port of typescript-go's `internal/tsoptions` package.
//!
//! Command-line and (later) `tsconfig.json` option parsing.

mod commandlineparser;
mod tsconfigparsing;

pub use commandlineparser::{TypeCheckCommand, parse_command_line};
pub use tsconfigparsing::{get_file_names, parse_config_file};
