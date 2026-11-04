use std::{
    fs,
    io::{self, ErrorKind, Write},
    path::Path,
};

use oxc_config::ConfigDiscovery;
use serde_json::json;

use crate::{DEFAULT_OXLINTRC_NAME, cli::CliRunResult, lint::print_and_flush_stdout};

pub fn run_init(cwd: &Path, stdout: &mut dyn Write) -> CliRunResult {
    match create_config(cwd) {
        Ok(()) => {
            print_and_flush_stdout(stdout, "Configuration file created\n");
            CliRunResult::ConfigFileInitSucceeded
        }
        Err(error) => {
            let message = if error.kind() == ErrorKind::AlreadyExists {
                "Oxlint config file already exists.\nRemove it and re-run.\n"
            } else {
                "Failed to create configuration file\n"
            };
            print_and_flush_stdout(stdout, message);
            CliRunResult::ConfigFileInitFailed
        }
    }
}

fn create_config(cwd: &Path) -> io::Result<()> {
    for name in ConfigDiscovery::oxlint().config_file_names() {
        match fs::symlink_metadata(cwd.join(name)) {
            Ok(_) => return Err(ErrorKind::AlreadyExists.into()),
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }

    let mut config = serde_json::Map::new();

    config.insert("$schema".to_string(), json!("./node_modules/oxlint/configuration_schema.json"));

    config.insert("plugins".to_string(), json!(["typescript", "unicorn", "oxc"]));
    config.insert("categories".to_string(), json!({ "correctness": "error" }));
    config.insert("rules".to_string(), json!({}));
    config.insert("env".to_string(), json!({ "builtin": true }));

    let configuration = serde_json::to_string_pretty(&serde_json::Value::Object(config)).unwrap();

    fs::File::create_new(cwd.join(DEFAULT_OXLINTRC_NAME))?.write_all(configuration.as_bytes())
}
