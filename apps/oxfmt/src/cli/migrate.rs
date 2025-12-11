use std::fs;
use std::io::BufWriter;
use std::path::Path;

use napi::bindgen_prelude::FnArgs;
use serde_json::Value;

use oxc_formatter::Oxfmtrc;

use crate::cli::CliRunResult;
use crate::core::{read_prettierignore, utils};

#[cfg(feature = "napi")]
use crate::core::JsGetPrettierConfigCb;

/// Run the `--migrate` command for pure Rust builds (no NAPI)
pub fn run_migrate(source: &str) -> CliRunResult {
    let mut stderr = BufWriter::new(std::io::stderr());

    if source != "prettier" {
        utils::print_and_flush(
            &mut stderr,
            &format!("Error: Unknown migration source '{source}'\n"),
        );
        utils::print_and_flush(&mut stderr, "Supported sources: prettier\n");
        return CliRunResult::MigrateFailed;
    }

    utils::print_and_flush(
        &mut stderr,
        "Error: Migration is only available in Node.js environment.\n",
    );
    utils::print_and_flush(&mut stderr, "Please use 'npx oxfmt --migrate prettier' instead.\n");
    CliRunResult::MigrateFailed
}

/// Run the `--migrate` command with NAPI support
#[cfg(feature = "napi")]
pub async fn run_migrate_napi(
    source: &str,
    get_prettier_config_cb: JsGetPrettierConfigCb,
) -> CliRunResult {
    let mut stdout = BufWriter::new(std::io::stdout());
    let mut stderr = BufWriter::new(std::io::stderr());

    // Validate source
    if source != "prettier" {
        utils::print_and_flush(
            &mut stderr,
            &format!("Error: Unknown migration source '{source}'\n"),
        );
        utils::print_and_flush(&mut stderr, "Supported sources: prettier\n");
        return CliRunResult::MigrateFailed;
    }

    // Check if config already exists
    if Path::new(".oxfmtrc.json").exists() || Path::new(".oxfmtrc.jsonc").exists() {
        utils::print_and_flush(&mut stderr, "Error: Configuration file already exists.\n");
        utils::print_and_flush(
            &mut stderr,
            "Remove .oxfmtrc.json or .oxfmtrc.jsonc manually before migrating.\n",
        );
        return CliRunResult::MigrateAborted;
    }

    // Get current working directory
    let cwd = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            utils::print_and_flush(&mut stderr, "Error: Could not determine current directory\n");
            return CliRunResult::MigrateFailed;
        }
    };

    // Call JS to get Prettier config
    // Pass cwd path - Prettier will search upward from there
    let config_json = {
        let status =
            get_prettier_config_cb.call_async(FnArgs::from((cwd.display().to_string(),))).await;
        match status {
            Ok(promise) => promise.await,
            Err(err) => {
                Err(napi::Error::from_reason(format!("Failed to get Prettier config: {err}")))
            }
        }
    };

    let config_json = match config_json {
        Ok(Some(json)) => json,
        Ok(None) => {
            utils::print_and_flush(&mut stderr, "Error: No Prettier configuration found\n");
            return CliRunResult::MigrateFailed;
        }
        Err(err) => {
            utils::print_and_flush(&mut stderr, &format!("Error getting Prettier config: {err}\n"));
            return CliRunResult::MigrateFailed;
        }
    };

    // Parse Prettier config JSON
    let prettier_config: Value = match serde_json::from_str(&config_json) {
        Ok(v) => v,
        Err(err) => {
            utils::print_and_flush(&mut stderr, &format!("Error parsing Prettier config: {err}\n"));
            return CliRunResult::MigrateFailed;
        }
    };

    // Convert Prettier config to Oxfmt
    let (mut oxfmtrc, unsupported) = match prettier_to_oxfmtrc(&prettier_config) {
        Ok(result) => result,
        Err(err) => {
            utils::print_and_flush(&mut stderr, &format!("Error converting config: {err}\n"));
            return CliRunResult::MigrateFailed;
        }
    };

    // Read and parse .prettierignore from current directory
    let ignore_patterns = read_prettierignore(&cwd);
    // Always set ignore patterns (even if empty array)
    oxfmtrc.ignore_patterns = Some(ignore_patterns);

    // Write config file
    if let Err(err) = write_migrated_config(&oxfmtrc) {
        utils::print_and_flush(&mut stderr, &format!("{err}\n"));
        return CliRunResult::MigrateFailed;
    }

    utils::print_and_flush(&mut stdout, "✓ Created .oxfmtrc.jsonc\n");

    // Warn about unsupported options
    if !unsupported.is_empty() {
        utils::print_and_flush(
            &mut stdout,
            "\n⚠ Warning: The following Prettier options were not migrated:\n",
        );
        for item in &unsupported {
            utils::print_and_flush(&mut stdout, &format!("  - {item}\n"));
        }
    }

    CliRunResult::MigrateSucceeded
}

/// Convert Prettier config JSON to Oxfmtrc
/// Returns (Oxfmtrc, unsupported_options)
fn prettier_to_oxfmtrc(prettier_json: &Value) -> Result<(Oxfmtrc, Vec<String>), String> {
    let mut unsupported = Vec::new();

    let obj = prettier_json.as_object().ok_or("Invalid Prettier config: not an object")?;

    // Track unsupported options before deserializing
    let config_only_unsupported = [
        ("experimentalOperatorPosition", "experimental feature not yet supported"),
        ("experimentalTernaries", "experimental feature not yet supported"),
        ("proseWrap", "markdown-specific option"),
        ("htmlWhitespaceSensitivity", "HTML-specific option"),
        ("vueIndentScriptAndStyle", "Vue-specific option"),
    ];

    for (key, reason) in &config_only_unsupported {
        if let Some(value) = obj.get(*key) {
            unsupported.push(format!("{key}: {value} ({reason})"));
        }
    }

    // Deprecated option
    if let Some(value) = obj.get("jsxBracketSameLine") {
        unsupported
            .push(format!("jsxBracketSameLine: {value} (deprecated, use bracketSameLine instead)"));
    }

    // Handle endOfLine: "auto" specially
    if obj.get("endOfLine").and_then(|v| v.as_str()) == Some("auto") {
        unsupported.push("endOfLine: \"auto\" (not supported, using default \"lf\")".to_string());
    }

    // Deserialize into Oxfmtrc - this handles all the field mapping
    // Serde will skip invalid values and use defaults
    let oxfmtrc: Oxfmtrc = serde_json::from_value(prettier_json.clone()).unwrap_or_default();

    Ok((oxfmtrc, unsupported))
}

/// Write Oxfmtrc to .oxfmtrc.jsonc file
fn write_migrated_config(oxfmtrc: &Oxfmtrc) -> Result<(), String> {
    let mut json =
        serde_json::to_value(oxfmtrc).map_err(|e| format!("Failed to serialize config: {e}"))?;

    // Add $schema if available
    let schema_path = "./node_modules/oxfmt/configuration_schema.json";
    if Path::new(schema_path).is_file() {
        if let Some(obj) = json.as_object_mut() {
            let mut new_obj = serde_json::Map::new();
            new_obj.insert("$schema".to_string(), Value::String(schema_path.to_string()));
            for (k, v) in std::mem::take(obj) {
                new_obj.insert(k, v);
            }
            json = Value::Object(new_obj);
        }
    }

    let json_str =
        serde_json::to_string_pretty(&json).map_err(|e| format!("Failed to format config: {e}"))?;

    fs::write(".oxfmtrc.jsonc", format!("{json_str}\n"))
        .map_err(|_| "Failed to write .oxfmtrc.jsonc".to_string())?;

    Ok(())
}
