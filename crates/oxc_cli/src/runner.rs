use std::{
    path::PathBuf,
    process::{ExitCode, Termination},
};

use clap::Command;

pub trait RunnerOptions: for<'a> From<&'a clap::ArgMatches> {
    /// Add CLI arguments to a [`Command`], usually created from a [`Runner`].
    fn build_args(cmd: Command) -> Command;
}

/// A trait for exposing Oxide functionality to the CLI.
pub trait Runner: Send + Sync {
    /// The name of the runner. Used to create a subcommand.
    const NAME: &'static str;
    /// A short description about what the runner does
    const ABOUT: &'static str;
    type Options: RunnerOptions;

    fn command() -> Command {
        let cmd = Command::new(Self::NAME).about(Self::ABOUT);
        Self::Options::build_args(cmd)
    }
    fn new(options: Self::Options) -> Self;

    /// Executes the runner, providing some result to the CLI.
    fn run(&self) -> CliRunResult;
}

#[derive(Debug)]
pub enum CliRunResult {
    None,
    IOError(crate::lint::Error),
    PathNotFound {
        paths: Vec<PathBuf>,
    },
    LintResult {
        duration: std::time::Duration,
        number_of_rules: usize,
        number_of_files: usize,
        number_of_warnings: usize,
        number_of_errors: usize,
        max_warnings_exceeded: bool,
    },
    TypeCheckResult {
        duration: std::time::Duration,
        number_of_diagnostics: usize,
    },
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None => ExitCode::from(0),
            Self::PathNotFound { paths } => {
                println!("Path {paths:?} does not exist.");
                ExitCode::from(1)
            }
            Self::IOError(e) => {
                println!("IO Error: {e}");
                ExitCode::from(1)
            }
            Self::LintResult {
                duration,
                number_of_rules,
                number_of_files,
                number_of_warnings,
                number_of_errors,
                max_warnings_exceeded,
            } => {
                let ms = duration.as_millis();
                let threads = rayon::current_num_threads();
                let number_of_diagnostics = number_of_warnings + number_of_errors;

                if number_of_diagnostics > 0 {
                    println!();
                }

                println!(
                    "Finished in {ms}ms on {number_of_files} files with {number_of_rules} rules using {threads} threads."
                );

                if max_warnings_exceeded {
                    println!("Exceeded maximum number of warnings. Found {number_of_warnings}.");
                    return ExitCode::from(1);
                }

                if number_of_diagnostics > 0 {
                    let warnings = if number_of_warnings == 1 { "warning" } else { "warnings" };
                    let errors = if number_of_errors == 1 { "error" } else { "errors" };
                    println!(
                        "Found {number_of_warnings} {warnings} and {number_of_errors} {errors}."
                    );
                    return ExitCode::from(1);
                }

                // eslint does not print anything after success, so we do the same.
                // It is also standard to not print anything after success in the *nix world.
                ExitCode::from(0)
            }
            Self::TypeCheckResult { duration, number_of_diagnostics } => {
                let ms = duration.as_millis();
                println!("Finished in {ms}ms.");

                if number_of_diagnostics > 0 {
                    println!("Found {number_of_diagnostics} errors.");
                    return ExitCode::from(1);
                }

                ExitCode::from(0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction, ArgMatches, Command};

    use super::*;

    #[derive(Debug)]
    struct TestRunnerOptions {
        foo: bool,
    }

    #[derive(Debug)]
    struct TestRunner {
        options: TestRunnerOptions,
    }

    impl<'a> From<&'a ArgMatches> for TestRunnerOptions {
        fn from(value: &'a ArgMatches) -> Self {
            let foo = value.get_flag("foo");
            Self { foo }
        }
    }

    impl RunnerOptions for TestRunnerOptions {
        fn build_args(cmd: Command) -> Command {
            cmd.arg(Arg::new("foo").short('f').action(ArgAction::SetTrue).required(false))
        }
    }

    impl Runner for TestRunner {
        type Options = TestRunnerOptions;

        const ABOUT: &'static str = "some description";
        const NAME: &'static str = "test";

        fn new(options: Self::Options) -> Self {
            Self { options }
        }

        fn run(&self) -> CliRunResult {
            CliRunResult::None
        }
    }

    #[test]
    fn check_cmd_validity() {
        TestRunner::command().debug_assert();
    }

    #[test]
    fn smoke_test_runner() {
        let cmd = TestRunner::command();
        let matches = cmd.get_matches_from("test -f".split(' '));
        let opts = TestRunnerOptions::from(&matches);
        let runner = TestRunner::new(opts);
        assert!(runner.options.foo);
        let result = runner.run();
        assert!(matches!(result, CliRunResult::None));
    }
}
