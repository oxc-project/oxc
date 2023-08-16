// mod git;
mod lint;
mod lint_plugin_tester;
mod plugin;
mod runner;
mod type_check;
mod walk;

use clap::{Arg, Command};
use lint_plugin_tester::LintPluginTestRunner;

pub use crate::{
    lint::{LintOptions, LintRunner},
    runner::{CliRunResult, Runner, RunnerOptions},
    type_check::{TypeCheckOptions, TypeCheckRunner},
    walk::Walk,
};

pub fn command() -> Command {
    Command::new("oxc")
        .bin_name("oxc")
        .version("alpha")
        .author("Boshen")
        .about("The JavaScript Oxidation Compiler")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(LintRunner::command())
        .subcommand(TypeCheckRunner::command())
        .subcommand(LintPluginTestRunner::command())
        .arg(
            Arg::new("threads")
                .long("threads")
                .value_parser(clap::value_parser!(usize))
                .help("Number of threads to use. Set to 1 for using only 1 CPU core."),
        )
}
