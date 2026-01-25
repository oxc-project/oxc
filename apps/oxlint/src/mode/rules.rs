use oxc_linter::{Config, table::RuleTable};
use rustc_hash::FxHashSet;

use crate::{
    cli::CliRunResult,
    lint::print_and_flush_stdout,
    output_formatter::{OutputFormat, OutputFormatter},
};

/// If the user requested `--rules`, print a CLI-specific table that
/// includes an "Enabled?" column based on the resolved configuration.
pub fn run_rules(
    lint_config: &Config,
    output_format: OutputFormat,
    output_formatter: &OutputFormatter,
    stdout: &mut dyn std::io::Write,
) -> CliRunResult {
    // Preserve previous behavior of `--rules` output when `-f` is set
    if output_format == OutputFormat::Default {
        // Build the set of enabled builtin rule names from the resolved config.
        let enabled: FxHashSet<&str> =
            lint_config.rules().iter().map(|(rule, _)| rule.name()).collect();

        let table = RuleTable::default();
        for section in &table.sections {
            let md = section.render_markdown_table_cli(&enabled);
            print_and_flush_stdout(stdout, &md);
            print_and_flush_stdout(stdout, "\n");
        }

        print_and_flush_stdout(
            stdout,
            format!("Default: {}\n", table.turned_on_by_default_count).as_str(),
        );
        print_and_flush_stdout(stdout, format!("Total: {}\n", table.total).as_str());
    } else if let Some(output) = output_formatter.all_rules() {
        print_and_flush_stdout(stdout, &output);
    }

    CliRunResult::None
}
