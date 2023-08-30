use crate::CliRunResult;

/// A trait for exposing functionality to the CLI.
pub trait Runner {
    type Options;

    fn new(matches: Self::Options) -> Self;

    /// Executes the runner, providing some result to the CLI.
    fn run(self) -> CliRunResult;
}
