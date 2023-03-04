mod command;
mod git;
mod options;
mod result;
mod walk;

use std::io::{BufWriter, Write};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::{fs, path::Path, rc::Rc, sync::Arc};

use git::{Git, GitResult};
use miette::NamedSource;
use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_diagnostics::{Error, Severity};
use oxc_linter::{LintRunResult, Linter};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

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
        let now = std::time::Instant::now();

        let paths = &self.cli_options.paths();
        let git = Git::new(paths);
        match git.verify() {
            Err(GitResult::MultipleRepos) => {
                println!("Found multiple Git repositories.");
                return CliRunResult::None;
            }
            Err(GitResult::NoRepo) => {
                println!("No valid Git repository found.");
                return CliRunResult::None;
            }
            Err(GitResult::UncommittedChanges) => {
                println!("Please commit changes before linting.");
                return CliRunResult::None;
            }
            _ => {}
        }

        let fix = self.cli_options.fix;

        let (_, (warnings, diagnostics)): (_, (usize, usize)) = rayon::join(
            move || {
                paths.par_iter().for_each(|path| {
                    let diagnostics = Self::lint_path(path, fix);

                    for d in diagnostics {
                        sender.send(d).unwrap();
                    }
                });
            },
            move || {
                let mut buf_writer = BufWriter::new(std::io::stdout());
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

                    buf_writer
                        .write_all(format!("{diagnostic:?}").as_bytes())
                        .expect("Failed to write diagnostic.");
                }

                buf_writer.flush().expect("Failed to flush.");

                (number_of_warnings, number_of_diagnostics)
            },
        );

        CliRunResult::LintResult {
            duration: now.elapsed(),
            number_of_files: paths.len(),
            number_of_diagnostics: diagnostics,
            number_of_warnings: warnings,
            max_warnings_exceeded: self
                .cli_options
                .max_warnings
                .map_or(false, |max_warnings| warnings > max_warnings),
        }
    }

    fn lint_path(path: &Path, fix: bool) -> Vec<Error> {
        let source_text = fs::read_to_string(path).expect("{name} not found");
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).expect("incorrect {path:?}");
        let parser_source_text = source_text.clone();
        let ret = Parser::new(&allocator, &parser_source_text, source_type).parse();
        let result = if ret.errors.is_empty() {
            let program = allocator.alloc(ret.program);
            let semantic = SemanticBuilder::new().build(program, ret.trivias);
            Linter::new().run(&Rc::new(semantic), &source_text, fix)
        } else {
            LintRunResult { fixed_source: source_text.clone().into(), diagnostics: ret.errors }
        };

        if result.diagnostics.is_empty() {
            return vec![];
        }

        let path = path.to_string_lossy();
        let source = Arc::new(NamedSource::new(path, source_text.clone()));
        result
            .diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.with_source_code(source.clone()))
            .collect()
    }
}
