use std::path::PathBuf;
mod command;
mod runner;

use clap::ArgMatches;

pub use self::{command::lint_command, runner::LintRunner};

pub struct LintOptions {
    pub paths: Vec<PathBuf>,
    pub fix: bool,
    pub quiet: bool,
    pub ignore_path: PathBuf,
    pub no_ignore: bool,
    pub ignore_pattern: Vec<String>,
    pub max_warnings: Option<usize>,
}

impl<'a> From<&'a ArgMatches> for LintOptions {
    fn from(matches: &'a ArgMatches) -> Self {
        Self {
            paths: matches.get_many("path").map_or_else(
                || vec![PathBuf::from(".")],
                |paths| paths.into_iter().cloned().collect(),
            ),
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

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::{lint_command, LintOptions};

    #[test]
    fn verify_command() {
        lint_command().debug_assert();
    }

    fn get_lint_options(arg: &str) -> LintOptions {
        let matches = lint_command().try_get_matches_from(arg.split(' ')).unwrap();
        LintOptions::from(&matches)
    }

    #[test]
    fn test_default() {
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
    fn test_multiple_paths() {
        let options = get_lint_options("lint foo bar baz");
        assert_eq!(
            options.paths,
            [PathBuf::from("foo"), PathBuf::from("bar"), PathBuf::from("baz")]
        );
    }

    #[test]
    fn test_quiet_true() {
        let options = get_lint_options("lint foo.js --quiet");
        assert!(options.quiet);
    }

    #[test]
    fn test_fix_true() {
        let options = get_lint_options("lint foo.js --fix");
        assert!(options.fix);
    }

    #[test]
    fn test_max_warnings() {
        let options = get_lint_options("lint --max-warnings 10 foo.js");
        assert_eq!(options.max_warnings, Some(10));
    }

    #[test]
    fn test_ignore_path() {
        let options = get_lint_options("lint --ignore-path .xxx foo.js");
        assert_eq!(options.ignore_path, PathBuf::from(".xxx"));
    }

    #[test]
    fn test_no_ignore() {
        let options = get_lint_options("lint --no-ignore foo.js");
        assert!(options.no_ignore);
    }

    #[test]
    fn test_single_ignore_pattern() {
        let options = get_lint_options("lint --ignore-pattern ./test foo.js");
        assert_eq!(options.ignore_pattern, vec![String::from("./test")]);
    }

    #[test]
    fn test_multiple_ignore_pattern() {
        let options =
            get_lint_options("lint --ignore-pattern ./test --ignore-pattern bar.js foo.js");
        assert_eq!(options.ignore_pattern, vec![String::from("./test"), String::from("bar.js")]);
    }
}
