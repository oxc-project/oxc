mod error;
mod isolated_handler;

use std::{io::BufWriter, sync::Arc, time::Duration};

pub use self::{error::Error, isolated_handler::IsolatedLintHandler};

use oxc_index::assert_impl_all;
use oxc_linter::{AllowWarnDeny, LintOptions, Linter, RuleCategory, RuleEnum, RULES};
use rustc_hash::FxHashSet;

use crate::{
    command::{LintFilter, LintOptions as CliLintOptions, WalkOptions},
    CliRunResult, Runner,
};

pub struct LintRunner {
    walk_options: Arc<WalkOptions>,
    options: Arc<LintOptions>,
    linter: Arc<Linter>,
}
assert_impl_all!(LintRunner: Send, Sync);

impl Runner for LintRunner {
    type Options = CliLintOptions;

    fn new(options: Self::Options) -> Self {
        let lint_options = parse_cli_options(&options);
        let linter = Linter::from_rules(Self::derive_rules(&lint_options))
            .with_fix(lint_options.fix)
            .with_print_execution_times(lint_options.print_execution_times);
        Self {
            walk_options: Arc::new(options.walk),
            options: Arc::new(lint_options),
            linter: Arc::new(linter),
        }
    }

    fn run(&self) -> CliRunResult {
        if self.options.list_rules {
            Self::print_rules();
            return CliRunResult::None;
        }

        let result = IsolatedLintHandler::new(
            Arc::clone(&self.walk_options),
            Arc::clone(&self.options),
            Arc::clone(&self.linter),
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

fn parse_cli_options(options: &CliLintOptions) -> LintOptions {
    let rules = get_rules(options);
    LintOptions {
        rules,
        fix: options.fix,
        quiet: options.cli.quiet,
        max_warnings: options.cli.max_warnings,
        list_rules: options.rules,
        print_execution_times: options.timing,
    }
}

/// Get all rules in order, e.g.
/// `-A all -D no-var -D -eqeqeq` => [("allow", "all"), ("deny", "no-var"), ("deny", "eqeqeq")]
/// Defaults to [("deny", "correctness")];
fn get_rules(options: &CliLintOptions) -> Vec<(AllowWarnDeny, String)> {
    if options.filter.is_empty() {
        vec![(AllowWarnDeny::Deny, "correctness".into())]
    } else {
        options
            .filter
            .iter()
            .map(|f| match f {
                LintFilter::Allow(allow) => (AllowWarnDeny::Allow, allow.clone()),
                LintFilter::Deny(deny) => (AllowWarnDeny::Deny, deny.clone()),
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::{parse_cli_options, AllowWarnDeny, LintOptions};
    use crate::lint_command;

    fn get_lint_options(arg: &str) -> LintOptions {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<String>>();
        let options = lint_command().run_inner(args.as_slice()).unwrap();
        parse_cli_options(&options.lint_options)
    }

    #[test]
    fn default() {
        let options = get_lint_options(".");
        assert!(!options.fix);
        assert!(!options.quiet);
        assert_eq!(options.max_warnings, None);
    }

    #[test]
    fn rules_with_deny_and_allow() {
        let options =
            get_lint_options("-D suspicious --deny pedantic -A no-debugger --allow no-var src");
        assert_eq!(
            options.rules,
            vec![
                (AllowWarnDeny::Deny, "suspicious".into()),
                (AllowWarnDeny::Deny, "pedantic".into()),
                (AllowWarnDeny::Allow, "no-debugger".into()),
                (AllowWarnDeny::Allow, "no-var".into())
            ]
        );
    }

    #[test]
    fn quiet_true() {
        let options = get_lint_options("foo.js --quiet");
        assert!(options.quiet);
    }

    #[test]
    fn fix_true() {
        let options = get_lint_options("foo.js --fix");
        assert!(options.fix);
    }

    #[test]
    fn max_warnings() {
        let options = get_lint_options("--max-warnings 10 foo.js");
        assert_eq!(options.max_warnings, Some(10));
    }

    #[test]
    fn list_rules_true() {
        let options = get_lint_options("--rules");
        assert!(options.list_rules);
    }
}
