use std::{
    ffi::OsStr,
    fs,
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, OnceLock},
};

use crossbeam_channel::{unbounded, Receiver, Sender};
use miette::NamedSource;
use oxc_allocator::Allocator;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
    Error, GraphicalReportHandler, Severity,
};
use oxc_linter::{Calculate, FixResult, Fixer, Linter, RuleCategory, RuleEnum, RULES};
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use oxc_span::{SourceType, VALID_EXTENSIONS};
use rayon::prelude::*;
use rustc_hash::FxHashSet;

use super::{
    error::{ErrorWithPath, Result},
    resolver::{ResolveResult, Resolver, Resource},
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
        let mut stdout = BufWriter::new(io::stdout());
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

    pub fn run(&self) -> CliRunResult {
        let now = std::time::Instant::now();

        let linter =
            Linter::from_rules(Self::derive_rules(&self.options)).with_fix(self.options.fix);
        let linter = Arc::new(linter);

        // Unless other panic happens, calling `Sender::send` can't fail, because we hold the
        // receiver until all senders are dropped. This allows us to safely unwrap all calls to send.
        let (tx_error, rx_error) = unbounded();

        // we can ignore the result because nothing bad happens if the resolver is already set
        // TODO: make sure this is still true once we allow options to be set
        // during runtime (config file, args, etc.)
        let _ = RESOLVER.set(Resolver::default());

        let visited = Arc::new(DashSet::new());

        // TODO: try to process as many files as possible even if some of them fail
        let result = process_paths(
            &self.options.paths,
            LinterRuntimeData {
                linter: Arc::clone(&linter),
                visited: Arc::clone(&visited),
                tx_error,
            },
        );

        let (number_of_warnings, number_of_diagnostics) = self.process_diagnostics(&rx_error);

        if let Err(err) = result {
            return CliRunResult::IOError(err);
        }

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
        let mut buf_writer = BufWriter::new(io::stdout());
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
                handler
                    .render_report(&mut err, diagnostic.as_ref())
                    .expect("Writing to a string can't fail");

                if err.lines().all(|line| line.len() < 400) {
                    output.push_str(&err);
                    continue;
                }

                // If the error is too long, we assume it's a minified file and print it as only error
                output = format!("{:?}", Error::new(MinifiedFileError(path.clone())));
                break;
            }

            // write operations on stdout can't fail according to RFC 1014
            // https://rust-lang.github.io/rfcs/1014-stdout-existential-crisis.html
            buf_writer.write_all(output.as_bytes()).expect("Writing to stdout can't fail");
        }

        // see comment above
        buf_writer.flush().expect("Flushing stdout can't fail");
        (number_of_warnings, number_of_diagnostics)
    }
}

fn process_paths(paths: &[PathBuf], runtime_data: LinterRuntimeData) -> Result<()> {
    paths.par_iter().try_for_each(move |path| {
        let path = path.canonicalize().with_path(path)?;

        if path.is_file() {
            run_for_file(&path, &runtime_data)
        } else if path.is_dir() {
            run_for_dir(&path, &runtime_data)
        } else {
            Ok(())
        }
    })
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

fn run_for_dir(path: &Path, runtime_data: &LinterRuntimeData) -> Result<()> {
    fs::read_dir(path).with_path(path)?.par_bridge().try_for_each(|entry| {
        let path = entry.with_path(path)?.path();

        if path.is_file() {
            if !runtime_data.visited.contains(&path) {
                run_for_file(&path, runtime_data)?;
            }
        } else if path.is_dir() {
            run_for_dir(&path, runtime_data)?;
        }

        Ok(())
    })
}

static RESOLVER: OnceLock<Resolver> = OnceLock::new();

fn run_for_file(path: &Path, runtime_data: &LinterRuntimeData) -> Result<()> {
    let LinterRuntimeData { linter, visited, tx_error } = &runtime_data;

    if visited.contains(path) {
        return Ok(());
    }

    visited.insert(path.to_path_buf());

    let Ok(source_type) = SourceType::from_path(path) else {
        // skip unsupported file types (e.g. .css or .json)
        return Ok(());
    };

    let source = fs::read_to_string(path).with_path(path)?;

    let allocator = Allocator::default();

    let (program, trivias) = {
        let ParserReturn { program, errors, trivias, .. } =
            Parser::new(&allocator, &source, source_type).parse();

        if !errors.is_empty() {
            tx_error.send(wrap_diagnostics(path, &source, errors)).unwrap();
            return Ok(());
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
            return Ok(());
        };

        semantic
    };

    // this is ok to unwrap because we know that the resolver is initialized, otherwise this function wouldn't be called
    let resolver = RESOLVER.get().unwrap();

    let resolve_path = path.parent().expect("Absolute file path always has a parent");

    let imported_modules = semantic.module_record().module_requests.keys();

    imported_modules
        .par_bridge()
        .filter(|name| name.starts_with('.'))
        .filter_map(|name| {
            resolver.resolve(resolve_path, name).map_or_else(
                |_| {
                    eprintln!("Couldn't resolve '{name}' in '{}'.", resolve_path.display());
                    None
                },
                Some,
            )
        })
        .filter_map(|resolved| match resolved {
            ResolveResult::Resource(Resource { path, .. }) => Some(path),
            ResolveResult::Ignored => None,
        })
        .filter(|path| {
            path.extension()
                .and_then(OsStr::to_str)
                .is_some_and(|ext| VALID_EXTENSIONS.contains(&ext))
        })
        .filter(|path| !visited.contains(path))
        .try_for_each(|path| run_for_file(&path, runtime_data))?;

    let result = linter.run(&Rc::new(semantic), path.related_to(Path::new(".")).unwrap());

    if result.is_empty() {
        return Ok(());
    }

    let messages = if linter.has_fix() {
        let FixResult { messages, fixed_code, .. } = Fixer::new(&source, result).fix();
        fs::write(path, fixed_code.as_bytes()).with_path(path)?;
        messages
    } else {
        result
    };

    let errors = messages.into_iter().map(|m| m.error).collect();
    let diagnostic = wrap_diagnostics(path, &source, errors);
    tx_error.send(diagnostic).unwrap();

    Ok(())
}
