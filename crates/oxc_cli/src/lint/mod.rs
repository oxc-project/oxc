mod command;
mod error;
mod isolated_handler;
mod options;

use std::{io::BufWriter, sync::Arc, time::Duration};

use oxc_index::assert_impl_all;
use oxc_linter::{Linter, RuleCategory, RuleEnum, RULES};
use oxc_query::schema;
use rustc_hash::FxHashSet;

pub use self::{error::Error, options::LintOptions};
use self::{isolated_handler::IsolatedLintHandler, options::AllowWarnDeny};
use crate::{plugin::LinterPlugin, CliRunResult, Runner};

pub struct LintRunner {
    options: Arc<LintOptions>,
    linter: Arc<Linter>,
}
assert_impl_all!(LintRunner: Send, Sync);

impl Default for LintRunner {
    fn default() -> Self {
        Self::new(LintOptions::default())
    }
}

impl Runner for LintRunner {
    type Options = LintOptions;

    const ABOUT: &'static str = "Lint this repository.";
    const NAME: &'static str = "lint";

    fn new(options: LintOptions) -> Self {
        let linter = Linter::from_rules(Self::derive_rules(&options))
            .with_fix(options.fix)
            .with_print_execution_times(options.print_execution_times);
        Self { options: Arc::new(options), linter: Arc::new(linter) }
    }

    fn run(&self) -> CliRunResult {
        if self.options.list_rules {
            Self::print_rules();
            return CliRunResult::None;
        }

        let result = IsolatedLintHandler::new(
            Arc::clone(&self.options),
            Arc::clone(&self.linter),
            Arc::new(LinterPlugin::new(schema(), self.options.plugin_path.clone()).unwrap()), // TODO: Propagate the unwrap
        )
        .run();

        if self.options.print_execution_times {
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
