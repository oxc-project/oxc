use bpaf::{doc::Style, Bpaf};
use oxc_linter::AllowWarnDeny;
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub enum CliCommand {
    /// Lint this repository
    #[bpaf(command)]
    Lint(#[bpaf(external(lint_options))] LintOptions),

    /// Use Ezno to type check source code (experimental and work in progress)
    #[bpaf(command)]
    Check(#[bpaf(external(check_options))] CheckOptions),
}

impl CliCommand {
    pub fn handle_threads(&self) {
        match self {
            Self::Lint(options) => {
                Self::set_rayon_threads(options.misc_options.threads);
            }
            Self::Check(_) => {}
        }
    }

    fn set_rayon_threads(threads: Option<usize>) {
        if let Some(threads) = threads {
            rayon::ThreadPoolBuilder::new().num_threads(threads).build_global().unwrap();
        }
    }
}

// To add a header or footer, see
// <https://docs.rs/bpaf/latest/bpaf/struct.OptionParser.html#method.descr>
/// Linter for the JavaScript Oxidation Compiler
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct LintCommand {
    #[bpaf(external(lint_options))]
    pub lint_options: LintOptions,
}

impl LintCommand {
    pub fn handle_threads(&self) {
        CliCommand::set_rayon_threads(self.lint_options.misc_options.threads);
    }
}

/// Miscellaneous
#[derive(Debug, Clone, Bpaf)]
pub struct MiscOptions {
    /// Display the execution time of each lint rule
    #[bpaf(switch, env("TIMING"), hide_usage)]
    pub timing: bool,

    /// list all the rules that are currently registered
    #[bpaf(switch, hide_usage)]
    pub rules: bool,

    /// Number of threads to use. Set to 1 for using only 1 CPU core
    #[bpaf(argument("INT"), hide_usage)]
    pub threads: Option<usize>,
}

#[derive(Debug, Clone, Bpaf)]
pub struct LintOptions {
    #[bpaf(external(lint_filter), map(LintFilter::into_tuple), many)]
    pub filter: Vec<(AllowWarnDeny, String)>,

    /// Use the experimental import plugin and detect ESM problems
    #[bpaf(switch, hide_usage)]
    pub import_plugin: bool,

    #[bpaf(external)]
    pub fix_options: FixOptions,

    #[bpaf(external)]
    pub ignore_options: IgnoreOptions,

    #[bpaf(external)]
    pub warning_options: WarningOptions,

    #[bpaf(external)]
    pub misc_options: MiscOptions,

    /// Single file, single path or list of paths
    #[bpaf(positional("PATH"), many)]
    pub paths: Vec<PathBuf>,
}

// This is formatted according to
// <https://docs.rs/bpaf/latest/bpaf/params/struct.NamedArg.html#method.help>
/// Allowing / Denying Multiple Lints
/// For example `-D correctness -A no-debugger` or `-A all -D no-debugger`.
/// ㅤ
///  The default category is "-D correctness".
///  Use "--rules" for rule names.
///  Use "--help --help" for rule categories.
///
/// The categories are:
///  * correctness - code that is outright wrong or useless
///  * suspicious  - code that is most likely wrong or useless
///  * pedantic    - lints which are rather strict or have occasional false positives
///  * style       - code that should be written in a more idiomatic way
///  * nursery     - new lints that are still under development
///  * restriction - lints which prevent the use of language and library features
///  * all         - all the categories listed above
#[derive(Debug, Clone, Bpaf)]
pub enum LintFilter {
    Allow(
        /// Allow the rule or category (suppress the lint)
        #[bpaf(short('A'), long("allow"), argument("NAME"))]
        String,
    ),
    Deny(
        /// Deny the rule or category (emit an error)
        #[bpaf(short('D'), long("deny"), argument("NAME"))]
        String,
    ),
}

impl LintFilter {
    fn into_tuple(self) -> (AllowWarnDeny, String) {
        match self {
            Self::Allow(s) => (AllowWarnDeny::Allow, s),
            Self::Deny(s) => (AllowWarnDeny::Deny, s),
        }
    }
}

/// Fix Problems
#[derive(Debug, Clone, Bpaf)]
pub struct FixOptions {
    /// Fix as many issues as possible. Only unfixed issues are reported in the output
    #[bpaf(switch)]
    pub fix: bool,
}

const NO_IGNORE_HELP: &[(&str, Style)] = &[
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

    ///
    #[bpaf(switch, hide_usage, help(NO_IGNORE_HELP))]
    pub no_ignore: bool,
}

/// Handle Warnings
#[derive(Debug, Clone, Bpaf)]
pub struct WarningOptions {
    /// Disable reporting on warnings, only errors are reported
    #[bpaf(switch, hide_usage)]
    pub quiet: bool,

    /// Specify a warning threshold,
    /// which can be used to force exit with an error status if there are too many warning-level rule violations in your project
    #[bpaf(argument("INT"), hide_usage)]
    pub max_warnings: Option<usize>,
}

#[derive(Debug, Clone, Bpaf)]
pub struct CheckOptions {
    /// Print called functions
    #[bpaf(switch, hide_usage)]
    pub print_called_functions: bool,

    /// Print types of expressions
    #[bpaf(switch, hide_usage)]
    pub print_expression_mappings: bool,

    /// File to type check
    #[bpaf(positional("PATH"))]
    pub path: PathBuf,
}

// windows binary has an`.exe` extension, which invalidates the snapshots
#[cfg(all(test, not(target_os = "windows")))]
mod snapshot {
    use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};
    use std::process::Command;

    fn test(name: &str, args: &[&str]) {
        let bin = get_cargo_bin("oxlint");
        let mut command = Command::new(bin);
        command.args(args);
        assert_cmd_snapshot!(name, command);
    }

    #[test]
    fn default() {
        test("default", &[]);
    }

    #[test]
    fn help_help() {
        test("help_help", &["--help", "--help"]);
    }
}

#[cfg(test)]
mod misc_options {
    use super::{lint_command, MiscOptions};

    fn get_misc_options(arg: &str) -> MiscOptions {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        lint_command().run_inner(args.as_slice()).unwrap().lint_options.misc_options
    }

    #[test]
    fn default() {
        let options = get_misc_options(".");
        assert!(!options.timing);
        assert!(!options.rules);
        assert!(options.threads.is_none());
    }

    #[test]
    fn timing() {
        let options = get_misc_options("--timing .");
        assert!(options.timing);
    }

    #[test]
    fn threads() {
        let options = get_misc_options("--threads 4 .");
        assert_eq!(options.threads, Some(4));
    }

    #[test]
    fn list_rules() {
        let options = get_misc_options("--rules");
        assert!(options.rules);
    }
}

#[cfg(test)]
mod warning_options {
    use super::{lint_command, WarningOptions};

    fn get_warning_options(arg: &str) -> WarningOptions {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        lint_command().run_inner(args.as_slice()).unwrap().lint_options.warning_options
    }

    #[test]
    fn default() {
        let options = get_warning_options(".");
        assert!(!options.quiet);
        assert_eq!(options.max_warnings, None);
    }

    #[test]
    fn quiet() {
        let options = get_warning_options("--quiet .");
        assert!(options.quiet);
    }

    #[test]
    fn max_warnings() {
        let options = get_warning_options("--max-warnings 10 .");
        assert_eq!(options.max_warnings, Some(10));
    }
}

#[cfg(test)]
mod lint_options {
    use super::{lint_command, LintOptions};
    use oxc_linter::AllowWarnDeny;
    use std::path::PathBuf;

    fn get_lint_options(arg: &str) -> LintOptions {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        lint_command().run_inner(args.as_slice()).unwrap().lint_options
    }

    #[test]
    fn default() {
        let options = get_lint_options(".");
        assert_eq!(options.paths, vec![PathBuf::from(".")]);
        assert!(!options.fix_options.fix);
    }

    #[test]
    fn multiple_paths() {
        let options = get_lint_options("foo bar baz");
        assert_eq!(
            options.paths,
            [PathBuf::from("foo"), PathBuf::from("bar"), PathBuf::from("baz")]
        );
    }

    #[test]
    fn fix() {
        let options = get_lint_options("--fix test.js");
        assert!(options.fix_options.fix);
    }

    #[test]
    fn filter() {
        let options =
            get_lint_options("-D suspicious --deny pedantic -A no-debugger --allow no-var src");
        assert_eq!(
            options.filter,
            [
                (AllowWarnDeny::Deny, "suspicious".into()),
                (AllowWarnDeny::Deny, "pedantic".into()),
                (AllowWarnDeny::Allow, "no-debugger".into()),
                (AllowWarnDeny::Allow, "no-var".into())
            ]
        );
    }
}

#[cfg(test)]
mod ignore_options {
    use super::{lint_command, IgnoreOptions};
    use std::{ffi::OsString, path::PathBuf};

    fn get_ignore_options(arg: &str) -> IgnoreOptions {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        lint_command().run_inner(args.as_slice()).unwrap().lint_options.ignore_options
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
