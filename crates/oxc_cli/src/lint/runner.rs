use std::{
    fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Sender},
        Arc, OnceLock,
    },
};

use miette::NamedSource;
use oxc_allocator::Allocator;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
    Error, GraphicalReportHandler, Severity,
};
use oxc_linter::{Fixer, Linter, RuleCategory, RuleEnum, RULES};
use oxc_parser::Parser;
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use oxc_span::SourceType;
use rustc_hash::FxHashSet;

use super::{
    resolver::{ResolveResult, Resolver},
    AllowWarnDeny, LintOptions,
};
use crate::CliRunResult;

pub struct LintRunner {
    options: LintOptions,

    linter: Arc<Linter>,
}
use dashmap::DashSet;

#[derive(Debug, Error, Diagnostic)]
#[error("File is too long to fit on the screen")]
#[diagnostic(help("{0:?} seems like a minified file"))]
pub struct MinifiedFileError(pub PathBuf);

impl LintRunner {
    pub fn new(options: LintOptions) -> Self {
        let linter = Linter::from_rules(Self::derive_rules(&options)).with_fix(options.fix);
        Self { options, linter: Arc::new(linter) }
    }

    pub fn print_rules() {
        let mut stdout = BufWriter::new(std::io::stdout());
        Linter::print_rules(&mut stdout);
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
                                rules.clear();
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
    pub fn run(&self) -> CliRunResult {
        let now = std::time::Instant::now();

        let number_of_files = Arc::new(AtomicUsize::new(0));
        let (tx_error, rx_error) = mpsc::channel::<(PathBuf, Vec<Error>)>();

        RESOLVER.set(Resolver::default()).unwrap();

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
        tx_error: Sender<(PathBuf, Vec<Error>)>,
    ) {
        let visited = Arc::new(DashSet::new());

        for path in &self.options.paths {
            let path = path.canonicalize().unwrap();
            let linter = Arc::clone(&self.linter);
            let tx_error = tx_error.clone();
            let number_of_files = Arc::clone(number_of_files);
            let visited = Arc::clone(&visited);

            rayon::spawn(move || {
                if path.is_file() {
                    run_for_file(&path, &linter, &tx_error, &number_of_files, &visited);
                } else if path.is_dir() {
                    run_for_dir(&path, &linter, &tx_error, &number_of_files, &visited);
                }
            });
        }

        drop(tx_error);
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

fn run_for_dir(
    path: &Path,
    linter: &Arc<Linter>,
    tx_error: &Sender<(PathBuf, Vec<Error>)>,
    number_of_files: &Arc<AtomicUsize>,
    visited: &Arc<DashSet<PathBuf>>,
) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();

        if visited.contains(&path) {
            continue;
        }

        if path.is_file() {
            run_for_file(&path, linter, tx_error, number_of_files, visited);
        } else if path.is_dir() {
            run_for_dir(&path, linter, tx_error, number_of_files, visited);
        }
    }
}

static RESOLVER: OnceLock<Resolver> = OnceLock::new();

fn run_for_file(
    path: &Path,
    linter: &Arc<Linter>,
    tx_error: &Sender<(PathBuf, Vec<Error>)>,
    number_of_files: &Arc<AtomicUsize>,
    visited: &Arc<DashSet<PathBuf>>,
) {
    number_of_files.fetch_add(1, Ordering::Relaxed);
    visited.insert(path.to_path_buf());

    let source = fs::read_to_string(path).unwrap_or_else(|_| panic!("{path:?} not found"));

    let allocator = Allocator::default();

    let source_type = SourceType::from_path(path).unwrap_or_else(|_| panic!("incorrect {path:?}"));

    let parser_return = Parser::new(&allocator, &source, source_type).parse();

    if !parser_return.errors.is_empty() {
        tx_error.send(wrap_diagnostics(path, &source, parser_return.errors)).unwrap();
    };

    let program = allocator.alloc(parser_return.program);

    let SemanticBuilderReturn { errors, semantic } = SemanticBuilder::new(&source, source_type)
        .with_trivias(&parser_return.trivias)
        .with_check_syntax_error(true)
        .with_module_record_builder(true)
        .build(program);

    if !errors.is_empty() {
        tx_error.send(wrap_diagnostics(path, &source, errors)).unwrap();
    };

    let resolver = RESOLVER.get().unwrap();
    let resolve_path = path.parent().expect("File path always has a parent");

    for name in semantic.module_record().module_requests.keys() {
        if !name.starts_with('.') {
            continue;
        }

        let resolve_result = resolver.resolve(resolve_path, name).unwrap_or_else(|_| {
            panic!("Couldn't resolve '{name}' in '{}'.", resolve_path.display())
        });

        let ResolveResult::Resource(resource) = resolve_result else { continue; };
        let path = resource.path;

        if visited.contains(&path) {
            continue;
        }

        let linter = Arc::clone(linter);
        let tx_error = tx_error.clone();
        let number_of_files = Arc::clone(number_of_files);
        let visited = Arc::clone(visited);

        rayon::spawn(move || {
            run_for_file(&path, &linter, &tx_error, &number_of_files, &visited);
        });
    }

    let result = linter.run(&Rc::new(semantic));

    if result.is_empty() {
        return;
    }

    let diagnostic = if linter.has_fix() {
        let fix_result = Fixer::new(&source, result).fix();
        fs::write(path, fix_result.fixed_code.as_bytes()).unwrap();
        let errors = fix_result.messages.into_iter().map(|m| m.error).collect();
        wrap_diagnostics(path, &source, errors)
    } else {
        let errors = result.into_iter().map(|diagnostic| diagnostic.error).collect();
        wrap_diagnostics(path, &source, errors)
    };

    tx_error.send(diagnostic).unwrap();
}
