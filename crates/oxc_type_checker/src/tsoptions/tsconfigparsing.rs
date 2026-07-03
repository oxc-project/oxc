//! Port of typescript-go's `internal/tsoptions/tsconfigparsing.go`.
//!
//! Reads and parses a `tsconfig.json`. The JSONC parse and the `extends`/`references`
//! search are delegated to `oxc_resolver`.

use std::{path::Path, sync::Arc};

use oxc_resolver::{ResolveOptions, Resolver, TsConfig};

/// Parse the `tsconfig.json` at `config_file`, resolving its `extends` chain.
///
/// Mirrors tsgo's `GetParsedCommandLineOfConfigFile`, but delegates the JSONC parse and the
/// `extends`/`references` search to `oxc_resolver`. `config_file` may be a path to a config
/// file or to a directory (in which case `tsconfig.json` is assumed).
///
/// Expanding `files`/`include`/`exclude` into the concrete root file list is a later step.
///
/// # Errors
///
/// Returns the `oxc_resolver` error text when the file is missing, or when the tsconfig (or
/// any config it `extends`) is invalid.
pub fn parse_config_file(config_file: &Path) -> Result<Arc<TsConfig>, String> {
    let resolver = Resolver::new(ResolveOptions::default());
    resolver.resolve_tsconfig(config_file).map_err(|error| error.to_string())
}
