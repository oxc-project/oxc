use std::{fs, path::Path};

use oxc_linter::Oxlintrc;
use serde_json::Value;

use crate::{DEFAULT_OXLINTRC, cli::CliRunResult, lint::print_and_flush_stdout};

pub fn run_init(cwd: &Path, stdout: &mut dyn std::io::Write) -> CliRunResult {
    let oxlintrc_for_print = serde_json::to_string_pretty(&Oxlintrc::default()).unwrap();

    let schema_relative_path = "node_modules/oxlint/configuration_schema.json";
    let configuration = if cwd.join(schema_relative_path).is_file() {
        let mut config_json: Value = serde_json::from_str(&oxlintrc_for_print).unwrap();
        if let Value::Object(ref mut obj) = config_json {
            let mut json_object = serde_json::Map::new();
            json_object.insert("$schema".to_string(), format!("./{schema_relative_path}").into());
            json_object.extend(obj.clone());
            *obj = json_object;
        }
        serde_json::to_string_pretty(&config_json).unwrap()
    } else {
        oxlintrc_for_print
    };

    if fs::write(DEFAULT_OXLINTRC, configuration).is_ok() {
        print_and_flush_stdout(stdout, "Configuration file created\n");
        return CliRunResult::ConfigFileInitSucceeded;
    }

    // failed case
    print_and_flush_stdout(stdout, "Failed to create configuration file\n");
    CliRunResult::ConfigFileInitFailed
}
