mod error;
mod isolated_handler;

use std::{io::BufWriter, sync::Arc, time::Duration};

pub use self::{error::Error, isolated_handler::IsolatedLintHandler};

use oxc_index::assert_impl_all;
use oxc_linter::{LintOptions, Linter};

use crate::{
    command::{CliOptions, LintOptions as CliLintOptions, WalkOptions},
    CliRunResult, Runner,
};

pub struct LintRunner {
    walk_options: Arc<WalkOptions>,
    cli_options: Arc<CliOptions>,
    list_rules: bool,
    linter: Arc<Linter>,
}
assert_impl_all!(LintRunner: Send, Sync);

impl Runner for LintRunner {
    type Options = CliLintOptions;

    fn new(options: Self::Options) -> Self {
        let list_rules = options.rules;
        let filter =
            if options.filter.is_empty() { LintOptions::default().filter } else { options.filter };
        let lint_options = LintOptions { filter, fix: options.fix, timing: options.timing };
        let linter = Linter::from_options(lint_options);
        Self {
            cli_options: Arc::new(options.cli),
            walk_options: Arc::new(options.walk),
            list_rules,
            linter: Arc::new(linter),
        }
    }

    fn run(&self) -> CliRunResult {
        if self.list_rules {
            Self::print_rules();
            return CliRunResult::None;
        }

        let result = IsolatedLintHandler::new(
            Arc::clone(&self.cli_options),
            Arc::clone(&self.walk_options),
            Arc::clone(&self.linter),
        )
        .run();

        if self.linter.options().timing {
            self.print_execution_times();
        }

        result
    }
}

impl LintRunner {
    fn print_rules() {
        let mut stdout = BufWriter::new(std::io::stdout());
        Linter::print_rules(&mut stdout);
    }

    fn print_execution_times(&self) {
        let mut timings = self
            .linter
            .rules()
            .iter()
            .map(|rule| (rule.name(), rule.execute_time()))
            .collect::<Vec<_>>();

        timings.sort_by_key(|x| x.1);
        let total = timings.iter().map(|x| x.1).sum::<Duration>().as_secs_f64();

        println!("Rule timings in milliseconds:");
        println!("Total: {:.2}ms", total * 1000.0);
        println!("{:>7} | {:>5} | Rule", "Time", "%");
        for (name, duration) in timings.iter().rev() {
            let millis = duration.as_secs_f64() * 1000.0;
            let relative = duration.as_secs_f64() / total * 100.0;
            println!("{millis:>7.2} | {relative:>4.1}% | {name}");
        }
    }
}
