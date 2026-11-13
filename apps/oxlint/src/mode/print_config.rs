use oxc_linter::{Config, ExternalPluginStore};

use crate::{cli::CliRunResult, lint::print_and_flush_stdout};

pub fn run_print_config(
    config: &Config,
    external_plugin_store: &ExternalPluginStore,
    stdout: &mut dyn std::io::Write,
) -> CliRunResult {
    let config_file = config.to_config_file(external_plugin_store);
    print_and_flush_stdout(stdout, &config_file);
    print_and_flush_stdout(stdout, "\n");

    CliRunResult::PrintConfigResult
}
