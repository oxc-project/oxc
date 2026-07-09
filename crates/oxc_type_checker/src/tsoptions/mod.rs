//! Port of typescript-go's `internal/tsoptions` package.
//!
//! Command-line and (later) `tsconfig.json` option parsing.

mod commandlineparser;
mod tsconfigparsing;

pub use commandlineparser::{TypeCheckCommand, parse_command_line};
pub(crate) use tsconfigparsing::{
    SUPPORTED_TS_EXTENSIONS_WITH_JSON_FLAT, get_allow_js, get_resolve_json_module,
    get_supported_extensions, get_supported_extensions_with_json_flat,
};
pub use tsconfigparsing::{get_file_names, parse_config_file};
