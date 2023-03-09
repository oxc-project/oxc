use std::path::PathBuf;

use argh::FromArgs;
use glob::Pattern;

#[derive(Debug, PartialEq, FromArgs)]
/// The JavaScript Oxidation Compiler
pub struct Command {
    #[argh(subcommand)]
    pub inner: Subcommand,
}

#[derive(Debug, PartialEq, FromArgs)]
#[argh(subcommand)]
pub enum Subcommand {
    Lint(LintCommand),
}

#[derive(Debug, PartialEq, FromArgs)]
#[argh(subcommand, name = "lint")]
/// Lint this repository.
pub struct LintCommand {
    #[argh(switch)]
    /// this option allows you to disable reporting on warnings. If you enable this option, only errors are reported by oxc_lint.
    pub quiet: bool,

    #[argh(switch)]
    /// this option allows you to enable oxc to fix as many issues as possible. If enabled, only unfixed issues are reported in the output
    pub fix: bool,

    #[argh(option)]
    /// this option allows you to specify a warning threshold, which can be used to force oxc_lint to exit with an error status if there are too many warning-level rule violations in your project.
    pub max_warnings: Option<usize>,

    #[argh(positional)]
    /// file or directory paths to scan. Directories are scanned recursively.
    pub paths: Vec<PathBuf>,

    #[argh(option, default = "String::from(\".eslintignore\")")]
    /// this option allows you to specify the file to use as your .eslintignore.
    pub ignore_path: String,

    #[argh(switch)]
    /// disables excluding of files from .eslintignore files, --ignore-path flags, --ignore-pattern flags.
    pub no_ignore: bool,

    #[argh(option)]
    /// this option allows you to specify patterns of files to ignore (in addition to those in .eslintignore).
    pub ignore_pattern: Vec<Pattern>,
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, str::FromStr};

    use argh::FromArgs;

    use super::{Command, LintCommand, Subcommand};

    fn get_lint_matches(arg: &str) -> LintCommand {
        let args = arg.split(' ').collect::<Vec<&str>>();

        let cmd: &[&str] = &args[0..1];
        let args: &[&str] = &args[1..];

        let command = Command::from_args(cmd, args).unwrap();

        match command.inner {
            Subcommand::Lint(lint_cmd) => lint_cmd,
        }
    }

    #[test]
    fn test_lint_path() {
        let matches = get_lint_matches("oxc lint .");
        assert_eq!(matches.paths, [PathBuf::from(".")]);
    }

    #[test]
    fn test_lint_multiple_paths() {
        let matches = get_lint_matches("oxc lint foo bar baz");
        assert_eq!(
            matches.paths,
            [PathBuf::from("foo"), PathBuf::from("bar"), PathBuf::from("baz")]
        );
    }

    #[test]
    // `argh` doesn't support aliases yet
    #[ignore]
    fn test_check_path() {
        let matches = get_lint_matches("oxc check /path/to/dir");
        assert_eq!(matches.paths, [PathBuf::from("/path/to/dir")]);
    }

    #[test]
    fn test_quiet_true() {
        let matches = get_lint_matches("oxc lint foo.js --quiet");
        assert!(matches.quiet);
    }

    #[test]
    fn test_quiet_false() {
        let matches = get_lint_matches("oxc lint foo.js");
        assert!(!matches.quiet);
    }

    #[test]
    fn test_fix_true() {
        let matches = get_lint_matches("oxc lint foo.js --fix");
        assert!(matches.fix);
    }

    #[test]
    fn test_fix_false() {
        let matches = get_lint_matches("oxc lint foo.js");
        assert!(!matches.fix);
    }

    #[test]
    fn test_max_warnings_none() {
        let arg = "oxc lint foo.js";
        let matches = get_lint_matches(arg);
        assert!(matches.max_warnings.is_none());
    }

    #[test]
    fn test_max_warnings_some() {
        let arg = "oxc lint --max-warnings 10 foo.js";
        let matches = get_lint_matches(arg);
        assert_eq!(matches.max_warnings, Some(10));
    }

    #[test]
    fn test_ignore_path() {
        let matches = get_lint_matches("oxc lint --ignore-path .gitignore foo.js");
        assert_eq!(matches.ignore_path, ".gitignore".to_string());
    }

    #[test]
    fn test_no_ignore() {
        let matches = get_lint_matches("oxc lint --no-ignore foo.js");
        assert!(matches.no_ignore);
    }

    #[test]
    fn test_single_ignore_pattern() {
        let matches = get_lint_matches("oxc lint --ignore-pattern \"./test\" foo.js");
        assert_eq!(matches.ignore_pattern, [glob::Pattern::from_str("\"./test\"").unwrap()]);
    }

    #[test]
    fn test_multiple_ignore_pattern() {
        let matches = get_lint_matches(
            "oxc lint --ignore-pattern \"./test\" --ignore-pattern \"bar.js\" foo.js",
        );

        assert_eq!(
            matches.ignore_pattern,
            [
                glob::Pattern::from_str("\"./test\"").unwrap(),
                glob::Pattern::from_str("\"bar.js\"").unwrap()
            ]
        );
    }
}
