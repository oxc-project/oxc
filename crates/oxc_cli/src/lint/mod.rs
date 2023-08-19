mod error;

use std::{
    fs,
    io::BufWriter,
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc,
    },
};

pub use self::error::Error;

use oxc_diagnostics::DiagnosticService;
use oxc_index::assert_impl_all;
use oxc_linter::{LintOptions, Linter};

use crate::{command::LintOptions as CliLintOptions, walk::Walk, CliRunResult, Runner};

pub struct LintRunner {
    options: CliLintOptions,
}
assert_impl_all!(LintRunner: Send, Sync);

impl Runner for LintRunner {
    type Options = CliLintOptions;

    fn new(options: Self::Options) -> Self {
        Self { options }
    }

    fn run(self) -> CliRunResult {
        if self.options.misc_options.rules {
            let mut stdout = BufWriter::new(std::io::stdout());
            Linter::print_rules(&mut stdout);
            return CliRunResult::None;
        }

        let CliLintOptions {
            paths,
            filter,
            warning_options,
            ignore_options,
            fix_options,
            misc_options,
        } = self.options;

        let now = std::time::Instant::now();

        let filter = if filter.is_empty() { LintOptions::default().filter } else { filter };

        let lint_options =
            LintOptions { filter, fix: fix_options.fix, timing: misc_options.timing };

        let linter = Arc::new(Linter::from_options(lint_options));

        let diagnostic_service = DiagnosticService::default()
            .with_quiet(warning_options.quiet)
            .with_max_warnings(warning_options.max_warnings);

        let number_of_files = Arc::new(AtomicUsize::new(0));
        let (tx_path, rx_path) = mpsc::channel::<Box<Path>>();

        rayon::spawn({
            let walk = Walk::new(&paths, &ignore_options);
            let number_of_files = Arc::clone(&number_of_files);
            move || {
                let mut count = 0;
                walk.iter().for_each(|path| {
                    count += 1;
                    tx_path.send(path).unwrap();
                });
                number_of_files.store(count, Ordering::Relaxed);
            }
        });

        let processing = Arc::new(AtomicUsize::new(0));
        rayon::spawn({
            let linter = Arc::clone(&linter);
            let tx_error = diagnostic_service.sender().clone();
            let processing = Arc::clone(&processing);
            move || {
                while let Ok(path) = rx_path.recv() {
                    processing.fetch_add(1, Ordering::Relaxed);
                    let tx_error = tx_error.clone();
                    let linter = Arc::clone(&linter);
                    let processing = Arc::clone(&processing);
                    rayon::spawn(move || {
                        let source_text = fs::read_to_string(&path)
                            .unwrap_or_else(|_| panic!("Failed to read {path:?}"));

                        let diagnostics = oxc_linter::LintService::new(linter)
                            .run(&path, &source_text)
                            .map(|errors| {
                                DiagnosticService::wrap_diagnostics(&path, &source_text, errors)
                            });

                        if let Some(diagnostics) = diagnostics {
                            tx_error.send(Some(diagnostics)).unwrap();
                        }
                        processing.fetch_sub(1, Ordering::Relaxed);
                        if processing.load(Ordering::Relaxed) == 0 {
                            tx_error.send(None).unwrap();
                        }
                    });
                }
            }
        });

        diagnostic_service.run();

        CliRunResult::LintResult {
            duration: now.elapsed(),
            number_of_rules: linter.number_of_rules(),
            number_of_files: number_of_files.load(Ordering::Relaxed),
            number_of_warnings: diagnostic_service.warnings_count(),
            number_of_errors: diagnostic_service.errors_count(),
            max_warnings_exceeded: diagnostic_service.max_warnings_exceeded(),
        }
    }
}
