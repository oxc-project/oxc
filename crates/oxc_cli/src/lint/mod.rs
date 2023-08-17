mod command;
mod error;
mod isolated_handler;

use clap::{ArgMatches, Command};
use std::{collections::BTreeMap, env, path::PathBuf};
use std::{io::BufWriter, sync::Arc, time::Duration};

use self::command::lint_command;
pub use self::{error::Error, isolated_handler::IsolatedLintHandler};

use oxc_index::assert_impl_all;
use oxc_linter::{AllowWarnDeny, LintOptions, Linter, RuleCategory, RuleEnum, RULES};
use rustc_hash::FxHashSet;

use crate::{CliRunResult, Runner};

pub struct LintRunner {
    options: Arc<LintOptions>,
    linter: Arc<Linter>,
}
assert_impl_all!(LintRunner: Send, Sync);

impl Runner for LintRunner {
    const ABOUT: &'static str = "Lint this repository.";
    const NAME: &'static str = "lint";

    fn new(matches: &ArgMatches) -> Self {
        let options = parse_arg_matches(matches);
        let linter = Linter::from_rules(Self::derive_rules(&options))
            .with_fix(options.fix)
            .with_print_execution_times(options.print_execution_times);
        Self { options: Arc::new(options), linter: Arc::new(linter) }
    }

    fn init_command() -> Command {
        lint_command()
    }

    fn run(&self) -> CliRunResult {
        if self.options.list_rules {
            Self::print_rules();
            return CliRunResult::None;
        }

        let result =
            IsolatedLintHandler::new(Arc::clone(&self.options), Arc::clone(&self.linter)).run();

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

fn parse_arg_matches(matches: &ArgMatches) -> LintOptions {
    let list_rules = matches.get_flag("rules");
    LintOptions {
        paths: matches.get_many("path").map_or_else(
            || if list_rules { vec![] } else { vec![PathBuf::from(".")] },
            |paths| paths.into_iter().cloned().collect(),
        ),
        rules: get_rules(matches),
        fix: matches.get_flag("fix"),
        quiet: matches.get_flag("quiet"),
        ignore_path: matches
            .get_one::<PathBuf>("ignore-path")
            .map_or_else(|| PathBuf::from(".eslintignore"), Clone::clone),
        no_ignore: matches.get_flag("no-ignore"),
        ignore_pattern: matches
            .get_many::<String>("ignore-pattern")
            .map(|patterns| patterns.into_iter().cloned().collect())
            .unwrap_or_default(),
        max_warnings: matches.get_one("max-warnings").copied(),
        list_rules,
        print_execution_times: matches!(env::var("TIMING"), Ok(x) if x == "true" || x == "1"),
    }
}

/// Get all rules in order, e.g.
/// `-A all -D no-var -D -eqeqeq` => [("allow", "all"), ("deny", "no-var"), ("deny", "eqeqeq")]
/// Defaults to [("deny", "correctness")];
fn get_rules(matches: &ArgMatches) -> Vec<(AllowWarnDeny, String)> {
    let mut map: BTreeMap<usize, (AllowWarnDeny, String)> = BTreeMap::new();
    for key in ["allow", "deny"] {
        let allow_warn_deny = AllowWarnDeny::from(key);
        if let Some(values) = matches.get_many::<String>(key) {
            let indices = matches.indices_of(key).unwrap();
            let zipped =
                values.zip(indices).map(|(value, i)| (i, (allow_warn_deny, value.clone())));
            map.extend(zipped);
        }
    }
    if map.is_empty() {
        vec![(AllowWarnDeny::Deny, "correctness".into())]
    } else {
        map.into_values().collect()
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::{lint_command, parse_arg_matches, AllowWarnDeny, LintOptions};

    #[test]
    fn verify_command() {
        lint_command().debug_assert();
    }

    fn get_lint_options(arg: &str) -> LintOptions {
        let matches = lint_command().try_get_matches_from(arg.split(' ')).unwrap();
        parse_arg_matches(&matches)
    }

    #[test]
    fn default() {
        let options = get_lint_options("lint .");
        assert_eq!(options.paths, vec![PathBuf::from(".")]);
        assert!(!options.fix);
        assert!(!options.quiet);
        assert_eq!(options.ignore_path, PathBuf::from(".eslintignore"));
        assert!(!options.no_ignore);
        assert!(options.ignore_pattern.is_empty());
        assert_eq!(options.max_warnings, None);
    }

    #[test]
    fn multiple_paths() {
        let options = get_lint_options("lint foo bar baz");
        assert_eq!(
            options.paths,
            [PathBuf::from("foo"), PathBuf::from("bar"), PathBuf::from("baz")]
        );
    }

    #[test]
    fn rules_with_deny_and_allow() {
        let options = get_lint_options(
            "lint src -D suspicious --deny pedantic -A no-debugger --allow no-var",
        );
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
        let options = get_lint_options("lint foo.js --quiet");
        assert!(options.quiet);
    }

    #[test]
    fn fix_true() {
        let options = get_lint_options("lint foo.js --fix");
        assert!(options.fix);
    }

    #[test]
    fn max_warnings() {
        let options = get_lint_options("lint --max-warnings 10 foo.js");
        assert_eq!(options.max_warnings, Some(10));
    }

    #[test]
    fn ignore_path() {
        let options = get_lint_options("lint --ignore-path .xxx foo.js");
        assert_eq!(options.ignore_path, PathBuf::from(".xxx"));
    }

    #[test]
    fn no_ignore() {
        let options = get_lint_options("lint --no-ignore foo.js");
        assert!(options.no_ignore);
    }

    #[test]
    fn single_ignore_pattern() {
        let options = get_lint_options("lint --ignore-pattern ./test foo.js");
        assert_eq!(options.ignore_pattern, vec![String::from("./test")]);
    }

    #[test]
    fn multiple_ignore_pattern() {
        let options =
            get_lint_options("lint --ignore-pattern ./test --ignore-pattern bar.js foo.js");
        assert_eq!(options.ignore_pattern, vec![String::from("./test"), String::from("bar.js")]);
    }

    #[test]
    fn list_rules_true() {
        let options = get_lint_options("lint --rules");
        assert!(options.paths.is_empty());
        assert!(options.list_rules);
    }
}
