mod command;
mod options;
mod result;
mod walk;

use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::{fs, path::Path, rc::Rc};

use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_common::PaddedStringView;
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
        let (sender, receiver): (SyncSender<Error>, Receiver<Error>) = sync_channel(32);

        let paths = &self
            .cli_options
            .paths
            .iter()
            .flat_map(|path| {
                Walk::new(path, &self.cli_options)
                    .iter()
                    .filter(|path| {
                        if self.cli_options.no_ignore {
                            return true;
                        }

                        let ignore_pattern = &self.cli_options.ignore_pattern;
                        for pattern in ignore_pattern {
                            if pattern.matches_path(path) {
                                return false;
                            }
                        }

                        true
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let (_, (warnings, diagnostics)): (_, (usize, usize)) = rayon::join(
            move || {
                paths.par_iter().for_each(|path| {
                    let diagnostics = Self::lint_path(path);

                    for d in diagnostics {
                        sender.send(d).unwrap();
                    }
                });
            },
            move || {
                let mut number_of_warnings = 0;
                let mut number_of_diagnostics = 0;

                while let Ok(diagnostic) = receiver.recv() {
                    number_of_diagnostics += 1;

                    if diagnostic.severity() == Some(Severity::Warning) {
                        number_of_warnings += 1;
                        // The --quiet flag follows ESLint's --quiet behavior as documented here: https://eslint.org/docs/latest/use/command-line-interface#--quiet
                        // Note that it does not disable ALL diagnostics, only Warning diagnostics
                        if self.cli_options.quiet {
                            continue;
                        }

                        if let Some(max_warnings) = self.cli_options.max_warnings {
                            if number_of_warnings > max_warnings {
                                continue;
                            }
                        }
                    }

                    println!("{diagnostic:?}");
                }

                (number_of_warnings, number_of_diagnostics)
            },
        );

        CliRunResult::LintResult {
            number_of_files: paths.len(),
            number_of_diagnostics: diagnostics,
            number_of_warnings: warnings,
            max_warnings_exceeded: self
                .cli_options
                .max_warnings
                .map_or(false, |max_warnings| warnings > max_warnings),
        }
    }

    fn lint_path(path: &Path) -> Vec<Error> {
        let source_text = PaddedStringView::read_from_file(path).expect("{name} not found");
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
            .map(|diagnostic| diagnostic.with_source_code((*source_text).clone()))
            .collect()
    }
}
