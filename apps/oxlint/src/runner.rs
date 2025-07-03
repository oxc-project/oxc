use std::io::Write;

use oxc_linter::ExternalLinter;

use crate::cli::CliRunResult;

/// A trait for exposing functionality to the CLI.
pub trait Runner {
    type Options;

    fn new(matches: Self::Options, external_linter: Option<ExternalLinter>) -> Self;

    /// Executes the runner, providing some result to the CLI.
    fn run(self, stdout: &mut dyn Write) -> CliRunResult;
}
