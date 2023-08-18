use bpaf::Bpaf;
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
    pub fn cli_options(&self) -> &CliOptions {
        match self {
            Self::Lint(options) => &options.cli,
            Self::Check(options) => &options.cli,
        }
    }

    pub fn handle_threads(&self) {
        Self::set_rayon_threads(self.cli_options().threads);
    }

    fn set_rayon_threads(threads: Option<usize>) {
        if let Some(threads) = threads {
            rayon::ThreadPoolBuilder::new().num_threads(threads).build_global().unwrap();
        }
    }
}

/// Linter for the JavaScript Oxidation Compiler
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct LintCommand {
    #[bpaf(external(lint_options))]
    pub lint_options: LintOptions,
}

impl LintCommand {
    pub fn handle_threads(&self) {
        CliCommand::set_rayon_threads(self.lint_options.cli.threads);
    }
}

#[derive(Debug, Clone, Bpaf)]
pub struct CliOptions {
    /// Disable reporting on warnings, only errors are reported
    #[bpaf(switch, hide_usage)]
    pub quiet: bool,

    /// Specify a warning threshold,
    /// which can be used to force exit with an error status if there are too many warning-level rule violations in your project
    #[bpaf(argument("NUMBER"), hide_usage)]
    pub max_warnings: Option<usize>,

    /// Number of threads to use. Set to 1 for using only 1 CPU core.
    #[bpaf(argument("NUMBER"), hide_usage)]
    pub threads: Option<usize>,
}

#[derive(Debug, Clone, Bpaf)]
pub struct WalkOptions {
    /// Disables excluding of files from .eslintignore files
    #[bpaf(switch)]
    pub no_ignore: bool,

    /// Specify the file to use as your .eslintignore
    #[bpaf(argument("PATH"), fallback(".eslintignore".into()))]
    pub ignore_path: OsString,

    /// Specify patterns of files to ignore (in addition to those in .eslintignore)
    #[bpaf(argument("PATTERN"), many)]
    pub ignore_pattern: Vec<String>,

    /// Single file, single path or list of paths
    #[bpaf(positional("PATH"), many)]
    pub paths: Vec<PathBuf>,
}

static FILTER_HELP: &str = r#"
To allow or deny a rule, use multiple -A <NAME> or -D <NAME>.

For example "-D correctness -A no-debugger" or "-A all -D no-debugger".

The categories are:
  * correctness - code that is outright wrong or useless
  * suspicious  - code that is most likely wrong or useless
  * pedantic    - lints which are rather strict or have occasional false positives
  * style       - code that should be written in a more idiomatic way
  * nursery     - new lints that are still under development
  * restriction - lints which prevent the use of language and library features
  * all         - all the categories listed above

The default category is "-D correctness".
"#;

#[derive(Debug, Clone, Bpaf)]
pub struct LintOptions {
    #[bpaf(external(lint_filter), map(LintFilter::into_tuple), many, group_help(FILTER_HELP))]
    pub filter: Vec<(AllowWarnDeny, String)>,

    /// Fix as many issues as possible. Only unfixed issues are reported in the output
    #[bpaf(switch)]
    pub fix: bool,

    /// Display the execution time of each lint rule
    #[bpaf(switch, env("TIMING"), hide_usage)]
    pub timing: bool,

    /// list all the rules that are currently registered
    #[bpaf(switch, hide_usage)]
    pub rules: bool,

    #[bpaf(external(cli_options), hide_usage)]
    pub cli: CliOptions,

    #[bpaf(external(walk_options), hide_usage)]
    pub walk: WalkOptions,
}

#[derive(Debug, Clone, Bpaf)]
pub enum LintFilter {
    Allow(
        /// Allow the rule or category
        #[bpaf(short('A'), long("allow"), argument("RULE|CATEGORY"))]
        String,
    ),
    Deny(
        /// Deny the rule or category
        #[bpaf(short('D'), long("deny"), argument("RULE|CATEGORY"))]
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

#[derive(Debug, Clone, Bpaf)]
pub struct CheckOptions {
    #[bpaf(external(cli_options), hide_usage)]
    pub cli: CliOptions,

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

#[cfg(test)]
mod cli_options {
    use super::{lint_command, CliOptions};

    fn get_cli_options(arg: &str) -> CliOptions {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        lint_command().run_inner(args.as_slice()).unwrap().lint_options.cli
    }

    #[test]
    fn default() {
        let options = get_cli_options(".");
        assert!(!options.quiet);
        assert_eq!(options.max_warnings, None);
    }

    #[test]
    fn quiet() {
        let options = get_cli_options("--quiet .");
        assert!(options.quiet);
    }

    #[test]
    fn max_warnings() {
        let options = get_cli_options("--max-warnings 10 .");
        assert_eq!(options.max_warnings, Some(10));
    }
}

#[cfg(test)]
mod lint_options {
    use super::{lint_command, LintOptions};
    use oxc_linter::AllowWarnDeny;

    fn get_lint_options(arg: &str) -> LintOptions {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        lint_command().run_inner(args.as_slice()).unwrap().lint_options
    }

    #[test]
    fn default() {
        let options = get_lint_options(".");
        assert!(!options.fix);
        assert!(!options.rules);
    }

    #[test]
    fn list_rules() {
        let options = get_lint_options("--rules");
        assert!(options.rules);
    }

    #[test]
    fn fix() {
        let options = get_lint_options("--fix test.js");
        assert!(options.fix);
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
mod walk_options {
    use super::{lint_command, WalkOptions};
    use std::{ffi::OsString, path::PathBuf};

    fn get_walk_options(arg: &str) -> WalkOptions {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        lint_command().run_inner(args.as_slice()).unwrap().lint_options.walk
    }

    #[test]
    fn default() {
        let options = get_walk_options(".");
        assert_eq!(options.paths, vec![PathBuf::from(".")]);
        assert_eq!(options.ignore_path, OsString::from(".eslintignore"));
        assert!(!options.no_ignore);
        assert!(options.ignore_pattern.is_empty());
    }

    #[test]
    fn multiple_paths() {
        let options = get_walk_options("foo bar baz");
        assert_eq!(
            options.paths,
            [PathBuf::from("foo"), PathBuf::from("bar"), PathBuf::from("baz")]
        );
    }

    #[test]
    fn ignore_path() {
        let options = get_walk_options("--ignore-path .xxx foo.js");
        assert_eq!(options.ignore_path, PathBuf::from(".xxx"));
    }

    #[test]
    fn no_ignore() {
        let options = get_walk_options("--no-ignore foo.js");
        assert!(options.no_ignore);
    }

    #[test]
    fn single_ignore_pattern() {
        let options = get_walk_options("--ignore-pattern ./test foo.js");
        assert_eq!(options.ignore_pattern, vec![String::from("./test")]);
    }

    #[test]
    fn multiple_ignore_pattern() {
        let options = get_walk_options("--ignore-pattern ./test --ignore-pattern bar.js foo.js");
        assert_eq!(options.ignore_pattern, vec![String::from("./test"), String::from("bar.js")]);
    }
}
