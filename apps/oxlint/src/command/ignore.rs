use std::ffi::OsString;

use bpaf::{doc::Style, Bpaf};

pub const NO_IGNORE_HELP: &[(&str, Style)] = &[
    ("Disables excluding of files from .eslintignore files, ", Style::Text),
    ("--ignore-path", Style::Literal),
    (" flags and ", Style::Text),
    ("--ignore-pattern", Style::Literal),
    (" flags", Style::Text),
];

/// Ignore Files
#[derive(Debug, Clone, Bpaf)]
pub struct IgnoreOptions {
    /// Specify the file to use as your .eslintignore
    #[bpaf(argument("PATH"), fallback(".eslintignore".into()), hide_usage)]
    pub ignore_path: OsString,

    /// Specify patterns of files to ignore (in addition to those in .eslintignore)
    ///
    /// The supported syntax is the same as for .eslintignore and .gitignore files
    /// You should quote your patterns in order to avoid shell interpretation of glob patterns
    #[bpaf(argument("PAT"), many, hide_usage)]
    pub ignore_pattern: Vec<String>,

    #[bpaf(switch, hide_usage, help(NO_IGNORE_HELP))]
    pub no_ignore: bool,

    /// Follow symbolic links. Oxlint ignores symbolic links by default.
    #[bpaf(switch, hide_usage)]
    pub symlinks: bool,
}

#[cfg(test)]
mod ignore_options {
    use std::{ffi::OsString, path::PathBuf};

    use super::{super::lint::lint_command, IgnoreOptions};

    fn get_ignore_options(arg: &str) -> IgnoreOptions {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        lint_command().run_inner(args.as_slice()).unwrap().ignore_options
    }

    #[test]
    fn default() {
        let options = get_ignore_options(".");
        assert_eq!(options.ignore_path, OsString::from(".eslintignore"));
        assert!(!options.no_ignore);
        assert!(options.ignore_pattern.is_empty());
    }

    #[test]
    fn ignore_path() {
        let options = get_ignore_options("--ignore-path .xxx foo.js");
        assert_eq!(options.ignore_path, PathBuf::from(".xxx"));
    }

    #[test]
    fn no_ignore() {
        let options = get_ignore_options("--no-ignore foo.js");
        assert!(options.no_ignore);
    }

    #[test]
    fn single_ignore_pattern() {
        let options = get_ignore_options("--ignore-pattern ./test foo.js");
        assert_eq!(options.ignore_pattern, vec![String::from("./test")]);
    }

    #[test]
    fn multiple_ignore_pattern() {
        let options = get_ignore_options("--ignore-pattern ./test --ignore-pattern bar.js foo.js");
        assert_eq!(options.ignore_pattern, vec![String::from("./test"), String::from("bar.js")]);
    }
}
