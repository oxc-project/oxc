mod command;
mod options;
mod result;
mod walk;

use std::{fs, path::Path, rc::Rc};

use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_diagnostics::{Error, Severity};
use oxc_linter::Linter;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walk::Walk;

pub use crate::{command::Command, options::CliOptions, result::CliRunResult};

pub struct Cli {
    pub cli_options: CliOptions,
}

#[allow(clippy::missing_const_for_fn)]
impl Cli {
    #[must_use]
    pub fn new(cli_options: CliOptions) -> Self {
        Self { cli_options }
    }

    /// Runs the linter on the specified paths and returns a `CliRunResult`.
    ///
    /// # Panics
    ///
    /// This function may panic if the `fs::read_to_string` function in `lint_path` fails to read a file.
    #[must_use]
    pub fn lint(&self) -> CliRunResult {
        let paths = &self
            .cli_options
            .paths
            .iter()
            .flat_map(|path| Walk::new(path).iter().collect::<Vec<_>>())
            .collect::<Vec<_>>();

        let number_of_diagnostics = paths
            .par_iter()
            .map(|path| {
                let diagnostics = Self::lint_path(path);
                diagnostics
                    .iter()
                    .filter(|d| match d.severity() {
                        // The --quiet flag follows ESLint's --quiet behavior as documented here: https://eslint.org/docs/latest/use/command-line-interface#--quiet
                        // Note that it does not disable ALL diagnostics, only Warning diagnostics
                        Some(Severity::Warning) => !self.cli_options.quiet,
                        _ => true,
                    })
                    .for_each(|diagnostic| {
                        println!("{diagnostic:?}");
                    });

                diagnostics.len()
            })
            .sum();

        CliRunResult::LintResult { number_of_files: paths.len(), number_of_diagnostics }
    }

    fn lint_path(path: &Path) -> Vec<Error> {
        let source_text = fs::read_to_string(path).expect("{name} not found");
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).expect("incorrect {path:?}");
        let ret = Parser::new(&allocator, &source_text, source_type).parse();
        let diagnostics = if ret.errors.is_empty() {
            let program = allocator.alloc(ret.program);
            let semantic = SemanticBuilder::new().build(program, ret.trivias);
            Linter::new().run(&Rc::new(semantic))
        } else {
            ret.errors
        };

        diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.with_source_code(source_text.clone()))
            .collect()
    }
}
