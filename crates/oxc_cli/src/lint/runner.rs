use std::{
    fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{mpsc, Arc},
};

use miette::NamedSource;
use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_diagnostics::{Error, MinifiedFileError, Severity};
use oxc_linter::{Fixer, Linter};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;

use super::LintOptions;
use crate::{CliRunResult, Walk};

pub struct LintRunner {
    options: LintOptions,
}

impl LintRunner {
    #[must_use]
    pub fn new(options: LintOptions) -> Self {
        Self { options }
    }

    /// # Panics
    ///
    /// * When `mpsc::channel` fails to send.
    #[must_use]
    pub fn run(&self) -> CliRunResult {
        let now = std::time::Instant::now();

        let mut number_of_files = 0;
        let mut number_of_warnings = 0;
        let mut number_of_diagnostics = 0;

        let (tx_error, rx_error) = mpsc::channel::<(PathBuf, Vec<Error>)>();

        self.process_paths(&mut number_of_files, tx_error);
        self.process_diagnostics(&mut number_of_warnings, &mut number_of_diagnostics, &rx_error);

        CliRunResult::LintResult {
            duration: now.elapsed(),
            number_of_files,
            number_of_diagnostics,
            number_of_warnings,
            max_warnings_exceeded: self
                .options
                .max_warnings
                .map_or(false, |max_warnings| number_of_warnings > max_warnings),
        }
    }

    fn process_paths(
        &self,
        number_of_files: &mut usize,
        tx_error: mpsc::Sender<(PathBuf, Vec<Error>)>,
    ) {
        let (tx_path, rx_path) = mpsc::channel::<Box<Path>>();
        let options = &self.options;
        let fix = options.fix;
        rayon::join(
            move || {
                Walk::new(options).iter().for_each(|path| {
                    *number_of_files += 1;
                    tx_path.send(path).unwrap();
                });
            },
            move || {
                while let Ok(path) = rx_path.recv() {
                    let tx_error = tx_error.clone();
                    rayon::spawn(move || {
                        if let Some(diagnostics) = Self::lint_path(&path, fix) {
                            tx_error.send(diagnostics).unwrap();
                        }
                        drop(tx_error);
                    });
                }
            },
        );
    }

    fn process_diagnostics(
        &self,
        number_of_warnings: &mut usize,
        number_of_diagnostics: &mut usize,
        rx_error: &mpsc::Receiver<(PathBuf, Vec<Error>)>,
    ) {
        let mut buf_writer = BufWriter::new(std::io::stdout());

        while let Ok((path, diagnostics)) = rx_error.recv() {
            *number_of_diagnostics += diagnostics.len();
            for diagnostic in diagnostics {
                if diagnostic.severity() == Some(Severity::Warning) {
                    *number_of_warnings += 1;
                    // The --quiet flag follows ESLint's --quiet behavior as documented here: https://eslint.org/docs/latest/use/command-line-interface#--quiet
                    // Note that it does not disable ALL diagnostics, only Warning diagnostics
                    if self.options.quiet {
                        continue;
                    }

                    if let Some(max_warnings) = self.options.max_warnings {
                        if *number_of_warnings > max_warnings {
                            continue;
                        }
                    }
                }

                let output = format!("{diagnostic:?}");
                // Skip large output and print only once
                if output.lines().any(|line| line.len() >= 400) {
                    let minified_diagnostic = Error::new(MinifiedFileError(path.clone()));
                    buf_writer.write_all(format!("{minified_diagnostic:?}").as_bytes()).unwrap();
                    break;
                }
                buf_writer.write_all(output.as_bytes()).unwrap();
            }
        }

        buf_writer.flush().unwrap();
    }

    fn lint_path(path: &Path, fix: bool) -> Option<(PathBuf, Vec<Error>)> {
        let source_text = fs::read_to_string(path).unwrap_or_else(|_| panic!("{path:?} not found"));
        let allocator = Allocator::default();
        let source_type =
            SourceType::from_path(path).unwrap_or_else(|_| panic!("incorrect {path:?}"));
        let parser_source_text = source_text.clone();
        let ret = Parser::new(&allocator, &parser_source_text, source_type).parse();

        if !ret.errors.is_empty() {
            return Some(Self::wrap_diagnostics(path, &source_text, ret.errors));
        };

        let program = allocator.alloc(ret.program);
        let trivias = Rc::new(ret.trivias);
        let semantic_ret = SemanticBuilder::new(source_type).build(program, &trivias);

        if !semantic_ret.errors.is_empty() {
            return Some(Self::wrap_diagnostics(path, &source_text, semantic_ret.errors));
        };

        let result = Linter::new().with_fix(fix).run(&Rc::new(semantic_ret.semantic), &source_text);

        if result.is_empty() {
            return None;
        }

        if fix {
            let fix_result = Fixer::new(&source_text, result).fix();
            fs::write(path, fix_result.fixed_code.as_bytes()).unwrap();
            let errors = fix_result.messages.into_iter().map(|m| m.error).collect();
            return Some(Self::wrap_diagnostics(path, &source_text, errors));
        }

        let errors = result.into_iter().map(|diagnostic| diagnostic.error).collect();
        Some(Self::wrap_diagnostics(path, &source_text, errors))
    }

    fn wrap_diagnostics(
        path: &Path,
        source_text: &str,
        diagnostics: Vec<Error>,
    ) -> (PathBuf, Vec<Error>) {
        let source = Arc::new(NamedSource::new(path.to_string_lossy(), source_text.to_owned()));
        let diagnostics = diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.with_source_code(Arc::clone(&source)))
            .collect();
        (path.to_path_buf(), diagnostics)
    }
}
