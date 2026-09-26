use std::{env, path::Path};

use serde_json::Value;

use crate::{
    cli::{ExplicitFileConfig, command::ConfigOptions, resolve_explicit_file},
    core::{JsLoadJsConfigCb, create_js_config_loader, utils},
};

pub struct ApiResolveConfigResult {
    pub config: Value,
    pub ignored: bool,
}

/// `resolveConfig()` implementation for the NAPI API.
///
/// Resolves the config for `filename` the same way `oxfmt --stdin-filepath` run from `cwd` would,
/// so the returned config can be passed to `format()` to get the same result.
///
/// # Errors
/// Returns error if config loading, parsing, or validation fails.
///
/// # Panics
/// Panics if `cwd` is not given and the current working directory cannot be determined.
pub fn run(
    filename: &str,
    cwd: Option<&str>,
    load_js_config_cb: JsLoadJsConfigCb,
) -> Result<ApiResolveConfigResult, String> {
    let current_dir = env::current_dir().expect("Failed to get current working directory");
    let cwd = match cwd {
        Some(cwd) => utils::normalize_relative_path(&current_dir, Path::new(cwd)),
        None => current_dir,
    };
    let filepath = utils::normalize_relative_path(&cwd, Path::new(filename));
    let js_config_loader = create_js_config_loader(load_js_config_cb);

    // Same as running the CLI without `--config`, `--disable-nested-config`, or `--ignore-path`
    let config_options = ConfigOptions { config: None, disable_nested_config: false };
    let ExplicitFileConfig { config_resolver, ignored } =
        resolve_explicit_file(&cwd, &filepath, &config_options, &[], &js_config_loader)?;

    let format_config = config_resolver.resolve_format_config(&filepath)?;
    let config = serde_json::to_value(&*format_config).map_err(|err| err.to_string())?;

    Ok(ApiResolveConfigResult { config, ignored })
}
