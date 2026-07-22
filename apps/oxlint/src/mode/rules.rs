use oxc_linter::Config;
use rustc_hash::FxHashSet;

use crate::{cli::CliRunResult, lint::print_and_flush_stdout, output_formatter::OutputFormatter};

/// If the user requested `--rules`, print a CLI-specific table that
/// includes an "Enabled?" column based on the resolved configuration.
pub fn run_rules(
    lint_config: &Config,
    output_formatter: &OutputFormatter,
    stdout: &mut dyn std::io::Write,
    type_aware: bool,
) -> CliRunResult {
    // Build the set of enabled builtin rule names from the resolved config.
    let enabled: FxHashSet<&str> = lint_config
        .rules()
        .iter()
        .filter(|(rule, _)| type_aware || !rule.is_tsgolint_rule())
        .map(|(rule, _)| rule.name())
        .collect();

    if let Some(output) = output_formatter.all_rules(enabled) {
        print_and_flush_stdout(stdout, &output);
    }

    CliRunResult::None
}
