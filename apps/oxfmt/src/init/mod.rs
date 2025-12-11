use std::fs;
use std::io::BufWriter;
use std::path::Path;

use oxc_formatter::Oxfmtrc;

use crate::cli::CliRunResult;
use crate::core::utils;

/// Run the `--init` command to scaffold a default configuration file.
pub fn run_init() -> CliRunResult {
    let mut stdout = BufWriter::new(std::io::stdout());
    let mut stderr = BufWriter::new(std::io::stderr());

    // Check if config file already exists
    if Path::new(".oxfmtrc.json").exists() || Path::new(".oxfmtrc.jsonc").exists() {
        utils::print_and_flush(&mut stderr, "Configuration file already exists.\n");
        return CliRunResult::InitAborted;
    }

    // NOTE: Use `Oxfmtrc` struct to prevent typos and ensure field consistency
    // All other fields are default `None` = not set
    let config = Oxfmtrc {
        // To make visible that this field exists
        ignore_patterns: Some(vec![]),
        ..Oxfmtrc::default()
    };
    let Ok(mut json) = serde_json::to_value(&config) else {
        utils::print_and_flush(&mut stderr, "Failed to generate configuration.\n");
        return CliRunResult::InitFailed;
    };

    // NOTE: `serde_json::Map` does not support inserting at the beginning,
    // so we rebuild the object to place `$schema` at the top.
    let schema_path = "./node_modules/oxfmt/configuration_schema.json";
    if Path::new(schema_path).is_file()
        && let Some(obj) = json.as_object_mut()
    {
        let mut new_obj = serde_json::Map::new();
        new_obj.insert("$schema".to_string(), serde_json::Value::String(schema_path.to_string()));
        for (k, v) in std::mem::take(obj) {
            new_obj.insert(k, v);
        }
        json = serde_json::Value::Object(new_obj);
    }

    let Ok(json_str) = serde_json::to_string_pretty(&json) else {
        utils::print_and_flush(&mut stderr, "Failed to serialize configuration.\n");
        return CliRunResult::InitFailed;
    };

    if fs::write(".oxfmtrc.jsonc", format!("{json_str}\n")).is_ok() {
        utils::print_and_flush(&mut stdout, "Created `.oxfmtrc.jsonc`.\n");
        CliRunResult::InitSucceeded
    } else {
        utils::print_and_flush(&mut stderr, "Failed to write `.oxfmtrc.jsonc`.\n");
        CliRunResult::InitFailed
    }
}
