mod command;
mod runner;

use std::{collections::BTreeMap, path::PathBuf};

use clap::ArgMatches;

pub use self::{command::lint_command, runner::LintRunner};

#[derive(Debug)]
pub struct LintOptions {
    pub paths: Vec<PathBuf>,
    /// Allow / Deny rules in order. [("allow" / "deny", rule name)]
    /// Defaults to [("deny", "correctness")]
    pub rules: Vec<(AllowWarnDeny, String)>,
    pub fix: bool,
    pub quiet: bool,
    pub ignore_path: PathBuf,
    pub no_ignore: bool,
    pub ignore_pattern: Vec<String>,
    pub max_warnings: Option<usize>,
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
        Self {
            paths: matches.get_many("path").map_or_else(
                || vec![PathBuf::from(".")],
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

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::{lint_command, AllowWarnDeny, LintOptions};

    #[test]
    fn verify_command() {
        lint_command().debug_assert();
    }

    fn get_lint_options(arg: &str) -> LintOptions {
        let matches = lint_command().try_get_matches_from(arg.split(' ')).unwrap();
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
}
