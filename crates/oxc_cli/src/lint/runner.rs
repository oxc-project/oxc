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
use oxc_ast::SourceType;
use oxc_diagnostics::{Error, GraphicalReportHandler, MinifiedFileError, Severity};
use oxc_linter::{Fixer, Linter, RuleCategory, RuleEnum, RULES};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use rustc_hash::FxHashSet;

use super::{AllowWarnDeny, LintOptions};
use crate::{CliRunResult, Walk};

pub struct LintRunner {
    options: LintOptions,

    linter: Arc<Linter>,
}

impl LintRunner {
    #[must_use]
    pub fn new(options: LintOptions) -> Self {
        let linter = Linter::from_rules(Self::derive_rules(&options)).with_fix(options.fix);
        Self { options, linter: Arc::new(linter) }
    }

    fn derive_rules(options: &LintOptions) -> Vec<RuleEnum> {
        let mut rules: FxHashSet<RuleEnum> = FxHashSet::default();

        for (allow_warn_deny, name_or_category) in &options.rules {
            let maybe_category = RuleCategory::from(name_or_category.as_str());
            match allow_warn_deny {
                AllowWarnDeny::Deny => {
                    match maybe_category {
                        Some(category) => rules.extend(
                            RULES.iter().filter(|rule| rule.category() == category).cloned(),
                        ),
                        None => {
                            if name_or_category == "all" {
                                rules.extend(RULES.iter().cloned());
                            } else {
                                rules.extend(
                                    RULES
                                        .iter()
                                        .filter(|rule| rule.name() == name_or_category)
                                        .cloned(),
                                );
                            }
                        }
                    };
                }
                AllowWarnDeny::Allow => {
                    match maybe_category {
                        Some(category) => rules.retain(|rule| rule.category() != category),
                        None => {
                            if name_or_category == "all" {
                                rules.drain();
                            } else {
                                rules.retain(|rule| rule.name() == name_or_category);
                            }
                        }
                    };
                }
            }
        }

        let mut rules = rules.into_iter().collect::<Vec<_>>();
        // for stable diagnostics output ordering
        rules.sort_unstable_by_key(|rule| rule.name());
        rules
    }

    /// # Panics
    ///
    /// * When `mpsc::channel` fails to send.
    #[must_use]
    pub fn run(&self) -> CliRunResult {
        let now = std::time::Instant::now();

        let number_of_files = Arc::new(AtomicUsize::new(0));
        let (tx_error, rx_error) = mpsc::channel::<(PathBuf, Vec<Error>)>();

        self.process_paths(&number_of_files, tx_error);
        let (number_of_warnings, number_of_diagnostics) = self.process_diagnostics(&rx_error);

        CliRunResult::LintResult {
            duration: now.elapsed(),
            number_of_rules: self.linter.number_of_rules(),
            number_of_files: number_of_files.load(Ordering::Relaxed),
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
        let mut number_of_diagnostics = 0;
        let mut buf_writer = BufWriter::new(std::io::stdout());
        let handler = GraphicalReportHandler::new();

        while let Ok((path, diagnostics)) = rx_error.recv() {
            number_of_diagnostics += diagnostics.len();
            let mut output = String::new();
            for diagnostic in diagnostics {
                if diagnostic.severity() == Some(Severity::Warning) {
                    number_of_warnings += 1;
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
        (number_of_warnings, number_of_diagnostics)
    }

    fn lint_path(linter: &Linter, path: &Path) -> Option<(PathBuf, Vec<Error>)> {
        let source_text = fs::read_to_string(path).unwrap_or_else(|_| panic!("{path:?} not found"));
        let allocator = Allocator::default();
        let source_type =
            SourceType::from_path(path).unwrap_or_else(|_| panic!("incorrect {path:?}"));
        let ret = Parser::new(&allocator, &source_text, source_type).parse();

        if !ret.errors.is_empty() {
            return Some(Self::wrap_diagnostics(path, &source_text, ret.errors));
        };

        let program = allocator.alloc(ret.program);
        let semantic_ret = SemanticBuilder::new(&source_text, source_type, &ret.trivias)
            .with_check_syntax_error(true)
            .build(program);

        if !semantic_ret.errors.is_empty() {
            return Some(Self::wrap_diagnostics(path, &source_text, semantic_ret.errors));
        };

        let result = linter.run(&Rc::new(semantic_ret.semantic));

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
