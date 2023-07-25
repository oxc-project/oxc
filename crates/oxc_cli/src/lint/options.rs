use std::{collections::BTreeMap, env, path::PathBuf};

use clap::ArgMatches;

use super::command::lint_command;
pub use super::{
    error::Error, isolated_handler::IsolatedLintHandler, module_tree_handler::ModuleTreeLintHandler,
};
use crate::runner::RunnerOptions;

#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct LintOptions {
    pub paths: Vec<PathBuf>,
    /// Allow / Deny rules in order. [("allow" / "deny", rule name)]
    /// Defaults to [("deny", "correctness")]
    pub rules: Vec<(AllowWarnDeny, String)>,
    pub list_rules: bool,
    pub fix: bool,
    pub quiet: bool,
    pub ignore_path: PathBuf,
    pub no_ignore: bool,
    pub ignore_pattern: Vec<String>,
    pub max_warnings: Option<usize>,
    pub print_execution_times: bool,
}

impl Default for LintOptions {
    fn default() -> Self {
        let default_matches = ArgMatches::default();
        Self::from(&default_matches)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AllowWarnDeny {
    Allow,
    // Warn,
    Deny,
}

impl From<&'static str> for AllowWarnDeny {
    fn from(s: &'static str) -> Self {
        match s {
            "allow" => Self::Allow,
            "deny" => Self::Deny,
            _ => unreachable!(),
        }
    }
}

impl<'a> From<&'a ArgMatches> for LintOptions {
    fn from(matches: &'a ArgMatches) -> Self {
        let list_rules = matches.get_flag("rules");

        Self {
            paths: matches.get_many("path").map_or_else(
                || if list_rules { vec![] } else { vec![PathBuf::from(".")] },
                |paths| paths.into_iter().cloned().collect(),
            ),
            rules: Self::get_rules(matches),
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
}

impl LintOptions {
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
}

impl RunnerOptions for LintOptions {
    #[inline]
    fn build_args(cmd: clap::Command) -> clap::Command {
        lint_command(cmd)
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use clap::Command;

    use super::{AllowWarnDeny, LintOptions};
    use crate::runner::RunnerOptions;

    #[test]
    fn verify_command() {
        LintOptions::build_args(Command::new("oxc")).debug_assert();
    }

    fn get_lint_options(arg: &str) -> LintOptions {
        let matches = LintOptions::build_args(Command::new("oxc"))
            .try_get_matches_from(arg.split(' '))
            .unwrap();
        LintOptions::from(&matches)
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
