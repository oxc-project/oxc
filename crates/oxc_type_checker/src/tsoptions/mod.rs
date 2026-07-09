//! Port of typescript-go's `internal/tsoptions` package.
//!
//! Command-line parsing, raw `tsconfig.json` parsing (with `extends` merging), and the
//! expansion of a config's file specs into the project's root file list.

mod commandlineparser;
mod parsinghelpers;
mod tsconfigparsing;

pub use commandlineparser::{TypeCheckCommand, parse_command_line};
pub use tsconfigparsing::{ParsedCommandLine, parse_config_file};
pub(crate) use tsconfigparsing::{
    SUPPORTED_TS_EXTENSIONS_WITH_JSON_FLAT, get_supported_extensions,
    get_supported_extensions_with_json_flat,
};
