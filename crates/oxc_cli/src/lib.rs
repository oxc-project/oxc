// mod git;
mod lint;
mod result;
mod type_check;
mod walk;

use clap::{Arg, Command};

use crate::{lint::lint_command, type_check::type_check_command};
pub use crate::{
    lint::{LintOptions, LintRunner, LintRunnerWithModuleTree},
    result::CliRunResult,
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
        .subcommand(lint_command())
        .subcommand(type_check_command())
        .arg(
            Arg::new("threads")
                .long("threads")
                .value_parser(clap::value_parser!(usize))
                .help("Number of threads to use. Set to 1 for using only 1 CPU core."),
        )
}
