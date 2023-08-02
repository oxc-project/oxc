use std::{
    fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc,
    },
};

use miette::NamedSource;
use oxc_allocator::Allocator;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
    Error, GraphicalReportHandler, Severity,
};
use oxc_linter::{Fixer, LintContext, Linter};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use super::options::LintOptions;
use crate::{CliRunResult, Walk};

pub struct IsolatedLintHandler {
    options: Arc<LintOptions>,

    linter: Arc<Linter>,
}

#[derive(Debug, Error, Diagnostic)]
#[error("File is too long to fit on the screen")]
#[diagnostic(help("{0:?} seems like a minified file"))]
pub struct MinifiedFileError(pub PathBuf);

impl IsolatedLintHandler {
    pub(super) fn new(options: Arc<LintOptions>, linter: Arc<Linter>) -> Self {
        Self { options, linter }
    }

    /// # Panics
    ///
    /// * When `mpsc::channel` fails to send.
    pub(super) fn run(&self) -> CliRunResult {
        let now = std::time::Instant::now();

        let number_of_files = Arc::new(AtomicUsize::new(0));
        let (tx_error, rx_error) = mpsc::channel::<(PathBuf, Vec<Error>)>();

        self.process_paths(&number_of_files, tx_error);
        let (number_of_warnings, number_of_errors) = self.process_diagnostics(&rx_error);

        CliRunResult::LintResult {
            duration: now.elapsed(),
            number_of_rules: self.linter.number_of_rules(),
            number_of_files: number_of_files.load(Ordering::Relaxed),
            number_of_warnings,
            number_of_errors,
            max_warnings_exceeded: self
                .options
                .max_warnings
                .map_or(false, |max_warnings| number_of_warnings > max_warnings),
        }
    }

    fn process_paths(
        &self,
        number_of_files: &Arc<AtomicUsize>,
        tx_error: mpsc::Sender<(PathBuf, Vec<Error>)>,
    ) {
        let (tx_path, rx_path) = mpsc::channel::<Box<Path>>();

        let walk = Walk::new(&self.options);
        let number_of_files = Arc::clone(number_of_files);
        rayon::spawn(move || {
            let mut count = 0;
            walk.iter().for_each(|path| {
                count += 1;
                tx_path.send(path).unwrap();
            });
            number_of_files.store(count, Ordering::Relaxed);
        });

        let linter = Arc::clone(&self.linter);
        rayon::spawn(move || {
            while let Ok(path) = rx_path.recv() {
                let tx_error = tx_error.clone();
                let linter = Arc::clone(&linter);
                rayon::spawn(move || {
                    if let Some(diagnostics) = Self::lint_path(&linter, &path) {
                        tx_error.send(diagnostics).unwrap();
                    }
                    drop(tx_error);
                });
            }
        });
    }

    fn process_diagnostics(
        &self,
        rx_error: &mpsc::Receiver<(PathBuf, Vec<Error>)>,
    ) -> (usize, usize) {
        let mut number_of_warnings = 0;
        let mut number_of_errors = 0;
        let mut buf_writer = BufWriter::new(std::io::stdout());
        let handler = GraphicalReportHandler::new();

        while let Ok((path, diagnostics)) = rx_error.recv() {
            let mut output = String::new();
            for diagnostic in diagnostics {
                let severity = diagnostic.severity();
                let is_warning = severity == Some(Severity::Warning);
                let is_error = severity.is_none() || severity == Some(Severity::Error);
                if is_warning || is_error {
                    if is_warning {
                        number_of_warnings += 1;
                    }
                    if is_error {
                        number_of_errors += 1;
                    }
                    // The --quiet flag follows ESLint's --quiet behavior as documented here: https://eslint.org/docs/latest/use/command-line-interface#--quiet
                    // Note that it does not disable ALL diagnostics, only Warning diagnostics
                    if self.options.quiet {
                        continue;
                    }

                    if let Some(max_warnings) = self.options.max_warnings {
                        if number_of_warnings > max_warnings {
                            continue;
                        }
                    }
                }

                let mut err = String::new();
                handler.render_report(&mut err, diagnostic.as_ref()).unwrap();
                // Skip large output and print only once
                if err.lines().any(|line| line.len() >= 400) {
                    let minified_diagnostic = Error::new(MinifiedFileError(path.clone()));
                    err = format!("{minified_diagnostic:?}");
                    output = err;
                    break;
                }
                output.push_str(&err);
            }
            buf_writer.write_all(output.as_bytes()).unwrap();
        }

        buf_writer.flush().unwrap();
        (number_of_warnings, number_of_errors)
    }

    fn lint_path(linter: &Linter, path: &Path) -> Option<(PathBuf, Vec<Error>)> {
        let source_text =
            fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {path:?}"));
        let allocator = Allocator::default();
        let source_type =
            SourceType::from_path(path).unwrap_or_else(|_| panic!("Incorrect {path:?}"));
        let ret = Parser::new(&allocator, &source_text, source_type)
            .allow_return_outside_function(true)
            .parse();

        if !ret.errors.is_empty() {
            return Some(Self::wrap_diagnostics(path, &source_text, ret.errors));
        };

        let program = allocator.alloc(ret.program);
        let semantic_ret = SemanticBuilder::new(&source_text, source_type)
            .with_trivias(&ret.trivias)
            .with_check_syntax_error(true)
            .with_module_record_builder(true)
            .build(program);

        if !semantic_ret.errors.is_empty() {
            return Some(Self::wrap_diagnostics(path, &source_text, semantic_ret.errors));
        };

        let lint_ctx = LintContext::new(&Rc::new(semantic_ret.semantic));
        let result = linter.run(lint_ctx);

        if result.is_empty() {
            return None;
        }

        if linter.has_fix() {
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
