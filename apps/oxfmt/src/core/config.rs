use std::path::{Path, PathBuf};

use serde_json::Value;

use oxc_formatter::{FormatOptions, OxfmtOptions, Oxfmtrc};

use super::utils;

/// Resolve config file path from cwd and optional explicit path.
pub fn resolve_config_path(cwd: &Path, config_path: Option<&Path>) -> Option<PathBuf> {
    // If `--config` is explicitly specified, use that path
    if let Some(config_path) = config_path {
        return Some(if config_path.is_absolute() {
            config_path.to_path_buf()
        } else {
            cwd.join(config_path)
        });
    }

    // If `--config` is not specified, search the nearest config file from cwd upwards
    // Support both `.json` and `.jsonc`, but prefer `.json` if both exist
    cwd.ancestors().find_map(|dir| {
        for filename in [".oxfmtrc.json", ".oxfmtrc.jsonc"] {
            let config_path = dir.join(filename);
            if config_path.exists() {
                return Some(config_path);
            }
        }
        None
    })
}

/// # Errors
/// Returns error if:
/// - Config file is specified but not found or invalid
/// - Config file parsing fails
pub fn load_config(
    config_path: Option<&Path>,
) -> Result<(FormatOptions, OxfmtOptions, Value), String> {
    // Read and parse config file, or use empty JSON if not found
    let json_string = match config_path {
        Some(path) => {
            let mut json_string = utils::read_to_string(path)
                // Do not include OS error, it differs between platforms
                .map_err(|_| format!("Failed to read config {}: File not found", path.display()))?;
            // Strip comments (JSONC support)
            json_strip_comments::strip(&mut json_string).map_err(|err| {
                format!("Failed to strip comments from {}: {err}", path.display())
            })?;
            json_string
        }
        None => "{}".to_string(),
    };

    // Parse as raw JSON value (to pass to external formatter)
    let mut raw_config: Value = serde_json::from_str(&json_string)
        .map_err(|err| format!("Failed to parse config: {err}"))?;

    // NOTE: Field validation for `enum` are done here
    let oxfmtrc: Oxfmtrc = serde_json::from_str(&json_string)
        .map_err(|err| format!("Failed to deserialize config: {err}"))?;

    // NOTE: Other validation based on it's field values are done here
    let (format_options, oxfmt_options) =
        oxfmtrc.into_options().map_err(|err| format!("Failed to parse configuration.\n{err}"))?;

    // Populate `raw_config` with resolved options to apply our defaults
    Oxfmtrc::populate_prettier_config(&format_options, &mut raw_config);

    Ok((format_options, oxfmt_options, raw_config))
}
