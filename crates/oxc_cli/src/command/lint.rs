use std::path::PathBuf;

use bpaf::Bpaf;
use oxc_linter::AllowWarnDeny;

use super::{
    ignore::{ignore_options, IgnoreOptions},
    misc_options, CliCommand, MiscOptions, VERSION,
};

// To add a header or footer, see
// <https://docs.rs/bpaf/latest/bpaf/struct.OptionParser.html#method.descr>
/// Linter for the JavaScript Oxidation Compiler
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version(VERSION))]
pub struct LintCommand {
    #[bpaf(external(lint_options))]
    pub lint_options: LintOptions,
}

impl LintCommand {
    pub fn handle_threads(&self) {
        CliCommand::set_rayon_threads(self.lint_options.misc_options.threads);
    }
}

#[derive(Debug, Clone, Bpaf)]
pub struct LintOptions {
    #[bpaf(external(lint_filter), map(LintFilter::into_tuple), many)]
    pub filter: Vec<(AllowWarnDeny, String)>,

    #[bpaf(external)]
    pub enable_plugins: EnablePlugins,

    #[bpaf(external)]
    pub fix_options: FixOptions,

    #[bpaf(external)]
    pub ignore_options: IgnoreOptions,

    #[bpaf(external)]
    pub warning_options: WarningOptions,

    #[bpaf(external)]
    pub misc_options: MiscOptions,

    /// ESLint configuration file (experimental)
    ///
    /// * only `.json` extension is supported
    #[bpaf(long("config"), short('c'), argument("PATH"))]
    pub config: Option<PathBuf>,

    /// Single file, single path or list of paths
    #[bpaf(positional("PATH"), many, guard(validate_paths, PATHS_ERROR_MESSAGE))]
    pub paths: Vec<PathBuf>,
}

fn validate_paths(paths: &Vec<PathBuf>) -> bool {
    if paths.is_empty() {
        true
    } else {
        paths.iter().all(|p| p.components().next() != Some(std::path::Component::ParentDir))
    }
}

const PATHS_ERROR_MESSAGE: &str = "PATH cannot start with \"..\"";

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

/// Handle Warnings
#[derive(Debug, Clone, Bpaf)]
pub struct WarningOptions {
    /// Disable reporting on warnings, only errors are reported
    #[bpaf(switch, hide_usage)]
    pub quiet: bool,

    /// Ensure warnings produce a non-zero exit code
    #[bpaf(switch, hide_usage)]
    pub deny_warnings: bool,

    /// Specify a warning threshold,
    /// which can be used to force exit with an error status if there are too many warning-level rule violations in your project
    #[bpaf(argument("INT"), hide_usage)]
    pub max_warnings: Option<usize>,
}

/// Enable Plugins
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Bpaf)]
pub struct EnablePlugins {
    /// Enable the experimental import plugin and detect ESM problems
    #[bpaf(switch, hide_usage)]
    pub import_plugin: bool,

    /// Enable the Jest plugin and detect test problems
    #[bpaf(switch, hide_usage)]
    pub jest_plugin: bool,

    /// Enable the JSX-a11y plugin and detect accessibility problems
    #[bpaf(switch, hide_usage)]
    pub jsx_a11y_plugin: bool,

    /// Enable the Next.js plugin and detect Next.js problems
    #[bpaf(switch, hide_usage)]
    pub nextjs_plugin: bool,

    /// Enable the React performance plugin and detect rendering performance problems
    #[bpaf(switch, hide_usage)]
    pub react_perf_plugin: bool,
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
    use std::path::PathBuf;

    use oxc_linter::AllowWarnDeny;

    use super::{lint_command, LintOptions};

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
    fn no_parent_path() {
        match lint_command().run_inner(&["../parent_dir"]) {
            Ok(_) => panic!("Should not allow parent dir"),
            Err(err) => match err {
                bpaf::ParseFailure::Stderr(doc) => {
                    assert_eq!("`../parent_dir`: PATH cannot start with \"..\"", format!("{doc}"));
                }
                _ => unreachable!(),
            },
        }
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
