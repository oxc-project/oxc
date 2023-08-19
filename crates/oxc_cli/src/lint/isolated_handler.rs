use std::{
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc,
    },
};

use oxc_allocator::Allocator;
use oxc_diagnostics::{DiagnosticService, Error};
use oxc_linter::{Fixer, LintContext, Linter};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use crate::{CliRunResult, Walk, WarningOptions};

pub struct IsolatedLintHandler {
    linter: Arc<Linter>,
    diagnostic_service: DiagnosticService,
}

impl IsolatedLintHandler {
    pub(super) fn new(warning_options: &WarningOptions, linter: Arc<Linter>) -> Self {
        let diagnostic_service = DiagnosticService::default()
            .with_quiet(warning_options.quiet)
            .with_max_warnings(warning_options.max_warnings);
        Self { linter, diagnostic_service }
    }

    /// # Panics
    ///
    /// * When `mpsc::channel` fails to send.
    pub(super) fn run(&self, walk: Walk) -> CliRunResult {
        let now = std::time::Instant::now();

        let number_of_files = Arc::new(AtomicUsize::new(0));

        self.process_paths(walk, &number_of_files);
        self.diagnostic_service.run();

        CliRunResult::LintResult {
            duration: now.elapsed(),
            number_of_rules: self.linter.number_of_rules(),
            number_of_files: number_of_files.load(Ordering::Relaxed),
            number_of_warnings: self.diagnostic_service.warnings_count(),
            number_of_errors: self.diagnostic_service.errors_count(),
            max_warnings_exceeded: self.diagnostic_service.max_warnings_exceeded(),
        }
    }

    fn process_paths(&self, walk: Walk, number_of_files: &Arc<AtomicUsize>) {
        let (tx_path, rx_path) = mpsc::channel::<Box<Path>>();

        let number_of_files = Arc::clone(number_of_files);
        rayon::spawn(move || {
            let mut count = 0;
            walk.iter().for_each(|path| {
                count += 1;
                tx_path.send(path).unwrap();
            });
            number_of_files.store(count, Ordering::Relaxed);
        });

        let mut processing = 0;
        let linter = Arc::clone(&self.linter);
        let tx_error = self.diagnostic_service.sender().clone();
        rayon::spawn(move || {
            while let Ok(path) = rx_path.recv() {
                processing += 1;
                let tx_error = tx_error.clone();
                let linter = Arc::clone(&linter);
                rayon::spawn(move || {
                    if let Some(diagnostics) = Self::lint_path(&linter, &path) {
                        tx_error.send(Some(diagnostics)).unwrap();
                    }
                    processing -= 1;
                    if processing == 0 {
                        tx_error.send(None).unwrap();
                    }
                });
            }
        });
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
            return Some(DiagnosticService::wrap_diagnostics(path, &source_text, ret.errors));
        };

        let program = allocator.alloc(ret.program);
        let semantic_ret = SemanticBuilder::new(&source_text, source_type)
            .with_trivias(ret.trivias)
            .with_check_syntax_error(true)
            .with_module_record_builder(true)
            .build(program);

        if !semantic_ret.errors.is_empty() {
            return Some(DiagnosticService::wrap_diagnostics(
                path,
                &source_text,
                semantic_ret.errors,
            ));
        };

        let lint_ctx = LintContext::new(&Rc::new(semantic_ret.semantic));
        let result = linter.run(lint_ctx);

        if result.is_empty() {
            return None;
        }

        if linter.options().fix {
            let fix_result = Fixer::new(&source_text, result).fix();
            fs::write(path, fix_result.fixed_code.as_bytes()).unwrap();
            let errors = fix_result.messages.into_iter().map(|m| m.error).collect();
            return Some(DiagnosticService::wrap_diagnostics(path, &source_text, errors));
        }

        let errors = result.into_iter().map(|diagnostic| diagnostic.error).collect();
        Some(DiagnosticService::wrap_diagnostics(path, &source_text, errors))
    }
}
