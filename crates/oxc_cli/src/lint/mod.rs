mod error;
mod isolated_handler;

use std::{io::BufWriter, sync::Arc};

pub use self::{error::Error, isolated_handler::IsolatedLintHandler};

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
        let walk = Walk::new(&paths, &ignore_options);
        let filter = if filter.is_empty() { LintOptions::default().filter } else { filter };
        let lint_options =
            LintOptions { filter, fix: fix_options.fix, timing: misc_options.timing };
        let linter = Arc::new(Linter::from_options(lint_options));
        let result =
            IsolatedLintHandler::new(Arc::new(warning_options), Arc::clone(&linter)).run(walk);

        linter.print_execution_times_if_enable();

        result
    }
}
