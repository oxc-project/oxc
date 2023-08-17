// mod git;
mod lint;
mod runner;
mod type_check;
mod walk;

use clap::{Arg, Command};

pub use crate::{
    lint::LintRunner,
    runner::{CliRunResult, Runner},
    type_check::TypeCheckRunner,
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
        .arg(
            Arg::new("threads")
                .long("threads")
                .value_parser(clap::value_parser!(usize))
                .help("Number of threads to use. Set to 1 for using only 1 CPU core."),
        )
}
