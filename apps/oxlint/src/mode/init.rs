use std::{fs, path::Path};

use serde_json::json;

use crate::{DEFAULT_OXLINTRC_NAME, cli::CliRunResult, lint::print_and_flush_stdout};

pub fn run_init(cwd: &Path, stdout: &mut dyn std::io::Write) -> CliRunResult {
    let mut config = serde_json::Map::new();

    config.insert("$schema".to_string(), json!("./node_modules/oxlint/configuration_schema.json"));

    config.insert("plugins".to_string(), json!(["typescript", "unicorn", "oxc"]));
    config.insert("categories".to_string(), json!({ "correctness": "error" }));
    config.insert("rules".to_string(), json!({}));
    config.insert("env".to_string(), json!({ "builtin": true }));

    let configuration = serde_json::to_string_pretty(&serde_json::Value::Object(config)).unwrap();

    if fs::write(cwd.join(DEFAULT_OXLINTRC_NAME), configuration).is_ok() {
        print_and_flush_stdout(stdout, "Configuration file created\n");
        return CliRunResult::ConfigFileInitSucceeded;
    }

    print_and_flush_stdout(stdout, "Failed to create configuration file\n");
    CliRunResult::ConfigFileInitFailed
}
