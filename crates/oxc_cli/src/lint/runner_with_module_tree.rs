use std::{
    fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, OnceLock},
};

use crossbeam_channel::{unbounded, Receiver, Sender};
use miette::NamedSource;
use nodejs_resolver::Resource;
use oxc_allocator::Allocator;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
    Error, GraphicalReportHandler, Severity,
};
use oxc_linter::{FixResult, Fixer, Linter, RuleCategory, RuleEnum, RULES};
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use oxc_span::{SourceType, VALID_EXTENSIONS};
use rayon::prelude::*;
use rustc_hash::FxHashSet;

use super::{
    resolver::{ResolveResult, Resolver},
    AllowWarnDeny, LintOptions,
};
use crate::CliRunResult;

#[derive(Clone)]
struct LinterRuntimeData {
    linter: Arc<Linter>,
    visited: Arc<DashSet<PathBuf>>,
    tx_error: Sender<(PathBuf, Vec<Error>)>,
}

pub struct LintRunnerWithModuleTree {
    options: LintOptions,
}
use dashmap::DashSet;

#[derive(Debug, Error, Diagnostic)]
#[error("File is too long to fit on the screen")]
#[diagnostic(help("{0:?} seems like a minified file"))]
pub struct MinifiedFileError(pub PathBuf);

impl LintRunnerWithModuleTree {
    pub fn new(options: LintOptions) -> Self {
        Self { options }
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

        let linter =
            Linter::from_rules(Self::derive_rules(&self.options)).with_fix(self.options.fix);
        let linter = Arc::new(linter);

        let (tx_error, rx_error) = unbounded();

        RESOLVER.set(Resolver::default()).unwrap();

        let visited = Arc::new(DashSet::new());

        process_paths(
            &self.options.paths,
            LinterRuntimeData {
                linter: Arc::clone(&linter),
                visited: Arc::clone(&visited),
                tx_error,
            },
        );

        let (number_of_warnings, number_of_diagnostics) = self.process_diagnostics(&rx_error);

        CliRunResult::LintResult {
            duration: now.elapsed(),
            number_of_rules: linter.number_of_rules(),
            number_of_files: visited.len(),
            number_of_diagnostics,
            number_of_warnings,
            max_warnings_exceeded: self
                .options
                .max_warnings
                .map_or(false, |max_warnings| number_of_warnings > max_warnings),
        }
    }

    fn process_diagnostics(&self, rx_error: &Receiver<(PathBuf, Vec<Error>)>) -> (usize, usize) {
        let mut number_of_warnings = 0;
        let mut number_of_diagnostics = 0;
        let mut buf_writer = BufWriter::new(std::io::stdout());
        let handler = GraphicalReportHandler::new();

        for (path, diagnostics) in rx_error.iter() {
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

                if err.lines().all(|line| line.len() < 400) {
                    output.push_str(&err);
                    continue;
                }

                // If the error is too long, we assume it's a minified file
                let minified_diagnostic = Error::new(MinifiedFileError(path.clone()));
                output = format!("{minified_diagnostic:?}");
                break;
            }

            buf_writer.write_all(output.as_bytes()).unwrap();
        }

        buf_writer.flush().unwrap();
        (number_of_warnings, number_of_diagnostics)
    }
}

fn process_paths(paths: &[PathBuf], runtime_data: LinterRuntimeData) {
    paths.par_iter().for_each(move |path| {
        let path = path.canonicalize().unwrap();

        if path.is_file() {
            run_for_file(&path, &runtime_data);
        } else if path.is_dir() {
            run_for_dir(&path, &runtime_data);
        }
    });
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

fn run_for_dir(path: &Path, runtime_data: &LinterRuntimeData) {
    fs::read_dir(path)
        .expect("Can't read directory")
        .par_bridge()
        .map(|entry| entry.expect("Can't read directory entry").path())
        .filter(|path| !runtime_data.visited.contains(path))
        .for_each(|path| {
            if path.is_file() {
                run_for_file(&path, runtime_data);
            } else if path.is_dir() {
                run_for_dir(&path, runtime_data);
            }
        });
}

static RESOLVER: OnceLock<Resolver> = OnceLock::new();

fn run_for_file(path: &Path, runtime_data: &LinterRuntimeData) {
    let LinterRuntimeData { linter, visited, tx_error } = &runtime_data;

    if visited.contains(path) {
        return;
    }

    let Ok(source_type) = SourceType::from_path(path) else {
        eprintln!("File {} is not supported, skipping.", path.display());
        return;
    };

    visited.insert(path.to_path_buf());

    let source = fs::read_to_string(path).unwrap_or_else(|_| panic!("{path:?} not found"));

    let allocator = Allocator::default();

    let (program, trivias) = {
        let ParserReturn { program, errors, trivias, .. } =
            Parser::new(&allocator, &source, source_type).parse();

        if !errors.is_empty() {
            tx_error.send(wrap_diagnostics(path, &source, errors)).unwrap();
            return;
        };

        (allocator.alloc(program), trivias)
    };

    let semantic = {
        let SemanticBuilderReturn { errors, semantic } = SemanticBuilder::new(&source, source_type)
            .with_trivias(&trivias)
            .with_check_syntax_error(true)
            .with_module_record_builder(true)
            .build(program);

        if !errors.is_empty() {
            tx_error.send(wrap_diagnostics(path, &source, errors)).unwrap();
            return;
        };

        semantic
    };

    let resolver = RESOLVER.get().unwrap();
    let resolve_path = path.parent().expect("File path always has a parent");

    let imported_modules = semantic.module_record().module_requests.keys();

    imported_modules.par_bridge().for_each(|name| {
        if !name.starts_with('.') {
            return;
        }

        let Ok(resolve_result) = resolver.resolve(resolve_path, name) else {
            eprintln!("Couldn't resolve '{name}' in '{}'.", resolve_path.display());
            return; 
        };

        let ResolveResult::Resource(Resource{ path, .. }) = resolve_result else { return; };

        if !path.extension().is_some_and(|ext| VALID_EXTENSIONS.contains(&ext.to_str().unwrap())) {
            return;
        }

        if visited.contains(&path) {
            return;
        }

        run_for_file(&path, runtime_data);
    });

    let result = linter.run(&Rc::new(semantic));

    if result.is_empty() {
        return;
    }

    let messages = if linter.has_fix() {
        let FixResult { messages, fixed_code, .. } = Fixer::new(&source, result).fix();
        fs::write(path, fixed_code.as_bytes()).unwrap();
        messages
    } else {
        result
    };

    let errors = messages.into_iter().map(|m| m.error).collect();
    let diagnostic = wrap_diagnostics(path, &source, errors);
    tx_error.send(diagnostic).unwrap();
}
