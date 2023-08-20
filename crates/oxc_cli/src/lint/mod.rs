mod error;

use std::{
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
use oxc_linter::{LintOptions, LintService, Linter};

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

        let lint_options = LintOptions::default()
            .with_filter(filter)
            .with_fix(fix_options.fix)
            .with_timing(misc_options.timing);

        let linter = Arc::new(Linter::from_options(lint_options));

        let diagnostic_service = DiagnosticService::default()
            .with_quiet(warning_options.quiet)
            .with_max_warnings(warning_options.max_warnings);

        let number_of_files = Arc::new(AtomicUsize::new(0));

        let (tx_path, rx_path) = mpsc::channel::<Box<Path>>();

        let tx_error = diagnostic_service.sender().clone();
        rayon::scope(|s| {
            let lint_service = LintService::new(Arc::clone(&linter));
            s.spawn(move |_| {
                while let Ok(path) = rx_path.recv() {
                    lint_service.run_path(path, &tx_error);
                }
            });

            let number_of_files = Arc::clone(&number_of_files);
            let walk = Walk::new(&paths, &ignore_options);
            s.spawn(move |_| {
                let mut count = 0;
                walk.iter().for_each(|path| {
                    count += 1;
                    tx_path.send(path).unwrap();
                });
                number_of_files.store(count, Ordering::SeqCst);
            });
        });

        diagnostic_service.run();

        CliRunResult::LintResult {
            duration: now.elapsed(),
            number_of_rules: linter.number_of_rules(),
            number_of_files: number_of_files.load(Ordering::SeqCst),
            number_of_warnings: diagnostic_service.warnings_count(),
            number_of_errors: diagnostic_service.errors_count(),
            max_warnings_exceeded: diagnostic_service.max_warnings_exceeded(),
        }
    }
}
