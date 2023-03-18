// mod git;
mod lint;
mod result;
mod walk;

use clap::Command;

use crate::lint::lint_command;
pub use crate::{
    lint::{LintOptions, LintRunner},
    result::CliRunResult,
    walk::Walk,
};

#[must_use]
pub fn command() -> Command {
    Command::new("oxc")
        .bin_name("oxc")
        .version("alpha")
        .author("Boshen")
        .about("The JavaScript Oxidation Compiler")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(lint_command())
}
