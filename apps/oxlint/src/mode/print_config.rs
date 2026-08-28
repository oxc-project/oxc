use oxc_linter::{ConfigStoreBuilder, ExternalPluginStore, Oxlintrc};

use crate::{cli::CliRunResult, lint::print_and_flush_stdout};

pub fn run_print_config(
    store_builder: &ConfigStoreBuilder,
    oxlintrc: Oxlintrc,
    external_plugin_store: &ExternalPluginStore,
    stdout: &mut dyn std::io::Write,
) -> CliRunResult {
    let config_file = store_builder.resolve_final_config_file(oxlintrc, external_plugin_store);
    print_and_flush_stdout(stdout, &config_file);
    print_and_flush_stdout(stdout, "\n");

    CliRunResult::PrintConfigResult
}
