use std::{io::BufWriter, path::PathBuf, sync::mpsc};

use oxc_diagnostics::Error;
use oxc_linter::{LintOptions, Linter, Runner};

use crate::{CliRunResult, Walk};

pub struct LintRunner;

impl LintRunner {
    pub fn print_rules() {
        let mut stdout = BufWriter::new(std::io::stdout());
        Linter::print_rules(&mut stdout);
    }

    /// # Panics
    ///
    /// Fails to canonicalize the input path
    pub fn run(options: LintOptions) -> CliRunResult {
        let runner = Runner::new(options);
        let options = runner.lint_options();
        let now = std::time::Instant::now();
        let mut number_of_files = 0;
        let (tx_error, rx_error) = mpsc::channel::<(PathBuf, Vec<Error>)>();

        for path in Walk::new(options).iter() {
            number_of_files += 1;
            let mut path = path;
            if !path.is_absolute() {
                path = path.canonicalize().unwrap().into_boxed_path();
            }
            runner.run_path(path.clone(), &tx_error);
        }

        drop(tx_error);

        let (number_of_warnings, number_of_diagnostics) = runner.process_diagnostics(&rx_error);
        let number_of_rules = runner.linter().number_of_rules();
        let max_warnings_exceeded =
            options.max_warnings.map_or(false, |max_warnings| number_of_warnings > max_warnings);

        CliRunResult::LintResult {
            duration: now.elapsed(),
            number_of_rules,
            number_of_files,
            number_of_diagnostics,
            number_of_warnings,
            max_warnings_exceeded,
        }
    }
}
