use std::{
    fs,
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
};

use dashmap::DashSet;
use oxc_allocator::Allocator;
use oxc_diagnostics::{
    miette::{self, Diagnostic, NamedSource},
    thiserror::Error,
    Error, GraphicalReportHandler, Severity,
};
use oxc_parser::{Parser, ParserReturn};
use oxc_resolver::{ResolveResult, Resolver};
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use oxc_span::SourceType;
use rayon::prelude::*;

use crate::{context::ModuleMap, LintContext, LintOptions, Linter};

#[derive(Debug, Error, Diagnostic)]
#[error("File is too long to fit on the screen")]
#[diagnostic(help("{0:?} seems like a minified file"))]
pub struct MinifiedFileError(pub PathBuf);

#[derive(Default, Clone)]
struct RuntimeData {
    linter: Arc<Linter>,
    resolver: Arc<Resolver>,
    visited: Arc<DashSet<PathBuf>>,
    module_map: Arc<ModuleMap>,
}

impl RuntimeData {
    pub fn new(linter: &Arc<Linter>) -> Self {
        Self { linter: Arc::clone(linter), ..Default::default() }
    }
}

pub struct Runner {
    options: LintOptions,
    linter: Arc<Linter>,
}

impl Runner {
    pub fn new(options: LintOptions) -> Self {
        let linter = Linter::from_options(&options).with_fix(options.fix);
        Self { options, linter: Arc::new(linter) }
    }

    #[must_use]
    pub fn with_linter(mut self, linter: Linter) -> Self {
        self.linter = Arc::new(linter);
        self
    }

    pub fn linter(&self) -> &Linter {
        &self.linter
    }

    pub fn lint_options(&self) -> &LintOptions {
        &self.options
    }

    /// # Panics
    ///
    /// Fails to send
    pub fn run_path(&self, path: Box<Path>, tx_error: &Sender<(PathBuf, Vec<Error>)>) {
        let runtime = RuntimeData::new(&self.linter);
        let tx_error = tx_error.clone();
        rayon::spawn(move || {
            if let Some(diagnostics) = process_path(&path, &runtime.clone()) {
                tx_error.send(diagnostics).unwrap();
            }
            drop(tx_error);
        });
    }

    /// # Panics
    ///
    /// Fails to send
    pub fn run_source(
        self,
        source: String,
        source_type: SourceType,
        path: PathBuf,
        tx_error: &Sender<(PathBuf, Vec<Error>)>,
    ) {
        let runtime = RuntimeData::new(&self.linter);
        let tx_error = tx_error.clone();
        rayon::spawn(move || {
            let output = process_source(&path, &source, source_type, &runtime.clone());
            if !output.1.is_empty() {
                tx_error.send(output).unwrap();
            }
            drop(tx_error);
        });
    }

    pub fn process_diagnostics(
        &self,
        rx_error: &Receiver<(PathBuf, Vec<Error>)>,
    ) -> (usize, usize) {
        let options = &self.options;
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
                    if options.quiet {
                        continue;
                    }

                    if let Some(max_warnings) = options.max_warnings {
                        if number_of_warnings > max_warnings {
                            continue;
                        }
                    }
                }

                let mut err = String::new();
                handler
                    .render_report(&mut err, diagnostic.as_ref())
                    .expect("Writing to a string can't fail");

                if err.lines().any(|line| line.len() >= 400) {
                    // If the error is too long, we assume it's a minified file and print it as only error
                    output = format!("{:?}", Error::new(MinifiedFileError(path.clone())));
                    break;
                }

                output.push_str(&err);
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

fn process_path(path: &Path, runtime: &RuntimeData) -> Option<(PathBuf, Vec<Error>)> {
    let Ok(source_type) = SourceType::from_path(path) else {
        return None;
    };

    if runtime.visited.contains(path) {
        return None;
    }

    runtime.visited.insert(path.to_path_buf());

    let source = fs::read_to_string(path).unwrap();
    Some(process_source(path, &source, source_type, runtime))
}

fn process_source(
    path: &Path,
    source: &str,
    source_type: SourceType,
    runtime: &RuntimeData,
) -> (PathBuf, Vec<Error>) {
    let allocator = Allocator::default();

    let ParserReturn { program, errors, trivias, .. } =
        Parser::new(&allocator, source, source_type).allow_return_outside_function(true).parse();

    if !errors.is_empty() {
        return wrap_diagnostics(path, source, errors);
    };

    let program = allocator.alloc(program);

    let SemanticBuilderReturn { errors, semantic } = SemanticBuilder::new(source, source_type)
        .with_trivias(&trivias)
        // .with_check_syntax_error(true)
        .with_module_record_builder(true)
        .build(program);

    runtime
        .module_map
        .insert(path.to_path_buf().into_boxed_path(), Arc::clone(semantic.module_record()));

    if !errors.is_empty() {
        return wrap_diagnostics(path, source, errors);
    };

    let parent_dir = path.parent().unwrap();

    let name_paths = semantic
        .module_record()
        .module_requests
        .keys()
        .par_bridge()
        .filter_map(|name| {
            let resolved = runtime.resolver.resolve(parent_dir, name).ok()?;
            let path = match resolved {
                ResolveResult::Resource(r) => r.path,
                ResolveResult::Ignored => return None,
            };
            process_path(&path, &runtime.clone());
            Some((name.clone(), path.into_boxed_path()))
        })
        .collect::<Vec<_>>();

    *semantic.module_record().resolved_absolute_path.write().unwrap() = path.to_path_buf();

    // Resolve `star_export_entries` from remote modules and
    // clone them into `tar_export_bindings`

    {
        let module_record = semantic.module_record();

        module_record.resolved_module_requests.write().unwrap().extend(name_paths);

        for start_export_entry in &module_record.star_export_entries {
            let Some(module_request) = &start_export_entry.module_request else { continue };
            let resolved_module_requests = module_record.resolved_module_requests.read().unwrap();
            let Some(module_request_path) = resolved_module_requests.get(module_request.name())
            else {
                continue;
            };
            let Some(remote_module_record_ref) = runtime.module_map.get(module_request_path) else {
                continue;
            };
            drop(resolved_module_requests);
            let remote_module_record = remote_module_record_ref.value();
            let remote_exported_bindings = remote_module_record.exported_bindings.keys();

            let mut star_export_entries_write = module_record.star_export_bindings.write().unwrap();
            let resolved_absolute_path_read =
                remote_module_record.resolved_absolute_path.read().unwrap();

            let star_export_bindings = star_export_entries_write
                .entry(resolved_absolute_path_read.clone().into_boxed_path())
                .or_default();
            star_export_bindings.extend(remote_exported_bindings.clone().cloned());

            let remote_star_export_bindings_read =
                remote_module_record.star_export_bindings.read().unwrap();
            let remote_star_export_bindings =
                remote_star_export_bindings_read.iter().flat_map(|(_path, name)| name.clone());
            star_export_bindings.extend(remote_star_export_bindings);
        }
    }

    let lint_context = LintContext::new(&Rc::new(semantic)).with_module_map(&runtime.module_map);
    let messages = runtime.linter.run(lint_context);
    let errors = messages.into_iter().map(|m| m.error).collect();
    wrap_diagnostics(path, source, errors)
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
