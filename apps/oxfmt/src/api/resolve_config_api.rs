use std::{
    env,
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::core::{
    ConfigResolver, JsLoadJsConfigCb, create_js_config_loader, resolve_editorconfig_path,
};

/// `resolveConfig()` implementation for the NAPI API.
///
/// Returns `None` when no `.oxfmtrc*`, JS config, `vite.config.ts` with `fmt`,
/// or `.editorconfig` can be found for the target file.
///
/// # Panics
/// Panics if the current working directory cannot be determined.
pub fn run(file_name: &str, load_js_config_cb: JsLoadJsConfigCb) -> Result<Option<Value>, String> {
    let cwd = env::current_dir().expect("Failed to get current working directory");
    let target_path = normalize_target_path(&cwd, file_name);
    let discovery_cwd = target_path.parent().unwrap_or(&cwd);
    let js_config_loader = create_js_config_loader(load_js_config_cb);
    let editorconfig_path = resolve_editorconfig_path(discovery_cwd);

    let mut resolver = ConfigResolver::from_config(
        discovery_cwd,
        None,
        editorconfig_path.as_deref(),
        Some(&js_config_loader),
    )?;
    resolver.build_and_validate()?;

    if resolver.config_dir().is_none() && !resolver.has_editorconfig() {
        return Ok(None);
    }

    serde_json::to_value(resolver.resolve_format_config(&target_path))
        .map(Some)
        .map_err(|err| err.to_string())
}

fn normalize_target_path(cwd: &Path, file_name: &str) -> PathBuf {
    let path = PathBuf::from(file_name);
    if path.is_absolute() { path } else { cwd.join(path) }
}
