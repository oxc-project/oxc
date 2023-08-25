use std::{
    io::BufWriter,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use oxc_diagnostics::DiagnosticService;
use oxc_linter::{LintOptions, LintService, Linter, PathWork};

use crate::{command::LintOptions as CliLintOptions, walk::Walk, CliRunResult, Runner};

pub struct LintRunner {
    options: CliLintOptions,
}

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

        let lint_service = LintService::new(Arc::clone(&linter));
        let tx_path = lint_service.tx_path.clone();
        lint_service.run(&diagnostic_service.sender().clone());

        rayon::spawn({
            let number_of_files = Arc::clone(&number_of_files);
            let walk = Walk::new(&paths, &ignore_options);
            move || {
                let mut count = 0;
                for path in walk.iter() {
                    count += 1;
                    tx_path.send(PathWork::Begin(path)).unwrap();
                }
                tx_path.send(PathWork::Done).unwrap();
                number_of_files.store(count, Ordering::SeqCst);
            }
        });

        diagnostic_service.run();
        linter.print_execution_times_if_enable();

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

#[cfg(all(test, not(target_os = "windows")))]
mod test {
    use super::LintRunner;
    use crate::{lint_command, CliRunResult, Runner};

    fn test(args: &[&str]) -> CliRunResult {
        let options = lint_command().run_inner(args).unwrap().lint_options;
        LintRunner::new(options).run()
    }

    #[test]
    fn dir() {
        let args = &["--quiet", "fixtures"];
        let CliRunResult::LintResult {
            number_of_rules,
            number_of_files,
            number_of_warnings,
            number_of_errors,
            ..
        } = test(args)
        else {
            unreachable!()
        };
        assert!(number_of_rules > 0);
        assert_eq!(number_of_files, 1);
        assert_eq!(number_of_warnings, 1);
        assert_eq!(number_of_errors, 0);
    }

    #[test]
    fn file() {
        let args = &["--quiet", "fixtures/debugger.js"];
        let CliRunResult::LintResult {
            number_of_rules,
            number_of_files,
            number_of_warnings,
            number_of_errors,
            ..
        } = test(args)
        else {
            unreachable!()
        };
        assert!(number_of_rules > 0);
        assert_eq!(number_of_files, 1);
        assert_eq!(number_of_warnings, 1);
        assert_eq!(number_of_errors, 0);
    }
}
