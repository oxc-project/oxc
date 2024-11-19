use std::{path::PathBuf, str::FromStr};

use bpaf::Bpaf;
use oxc_linter::{AllowWarnDeny, FixKind, LintPlugins};

use super::{
    expand_glob,
    ignore::{ignore_options, IgnoreOptions},
    misc_options, validate_paths, MiscOptions, PATHS_ERROR_MESSAGE, VERSION,
};

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version(VERSION))]
pub struct LintCommand {
    #[bpaf(external)]
    pub basic_options: BasicOptions,

    #[bpaf(external(lint_filter), map(LintFilter::into_tuple), many, hide_usage)]
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
    pub output_options: OutputOptions,

    /// list all the rules that are currently registered
    #[bpaf(long("rules"), switch, hide_usage)]
    pub list_rules: bool,

    #[bpaf(external)]
    pub misc_options: MiscOptions,

    /// Single file, single path or list of paths
    #[bpaf(positional("PATH"), many, guard(validate_paths, PATHS_ERROR_MESSAGE), map(expand_glob))]
    pub paths: Vec<PathBuf>,
}

impl LintCommand {
    pub fn handle_threads(&self) {
        Self::set_rayon_threads(self.misc_options.threads);
    }

    fn set_rayon_threads(threads: Option<usize>) {
        if let Some(threads) = threads {
            rayon::ThreadPoolBuilder::new().num_threads(threads).build_global().unwrap();
        }
    }
}
/// Basic Configuration
#[derive(Debug, Clone, Bpaf)]
pub struct BasicOptions {
    /// Oxlint configuration file (experimental)
    ///  * only `.json` extension is supported
    ///  * tries to be compatible with the ESLint v8's format
    #[bpaf(long, short, argument("./oxlintrc.json"))]
    pub config: Option<PathBuf>,

    /// TypeScript `tsconfig.json` path for reading path alias and project references for import plugin
    #[bpaf(argument("./tsconfig.json"), hide_usage)]
    pub tsconfig: Option<PathBuf>,
}

// This is formatted according to
// <https://docs.rs/bpaf/latest/bpaf/params/struct.NamedArg.html#method.help>
/// Allowing / Denying Multiple Lints
///
/// Accumulate rules and categories from left to right on the command-line.
///   For example `-D correctness -A no-debugger` or `-A all -D no-debugger`.
///   The categories are:
///   * `correctness` - code that is outright wrong or useless (default).
///   * `suspicious`  - code that is most likely wrong or useless.
///   * `pedantic`    - lints which are rather strict or have occasional false positives.
///   * `style`       - code that should be written in a more idiomatic way.
///   * `nursery`     - new lints that are still under development.
///   * `restriction` - lints which prevent the use of language and library features.
///   * `all`         - all the categories listed above except nursery. Does not enable plugins automatically.
///
/// Arguments:
//  ^ This shows up on the website but not from the cli's `--help`.
#[derive(Debug, Clone, Bpaf)]
pub enum LintFilter {
    Allow(
        /// Allow the rule or category (suppress the lint)
        #[bpaf(short('A'), long("allow"), argument("NAME"))]
        String,
    ),
    Warn(
        /// Deny the rule or category (emit a warning)
        #[bpaf(short('W'), long("warn"), argument("NAME"))]
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
            Self::Warn(s) => (AllowWarnDeny::Warn, s),
            Self::Deny(s) => (AllowWarnDeny::Deny, s),
        }
    }
}

/// Fix Problems
#[derive(Debug, Clone, Bpaf)]
pub struct FixOptions {
    /// Fix as many issues as possible. Only unfixed issues are reported in the output
    #[bpaf(switch, hide_usage)]
    pub fix: bool,
    /// Apply auto-fixable suggestions. May change program behavior.
    #[bpaf(switch, hide_usage)]
    pub fix_suggestions: bool,

    /// Apply dangerous fixes and suggestions.
    #[bpaf(switch, hide_usage)]
    pub fix_dangerously: bool,
}

impl FixOptions {
    pub fn fix_kind(&self) -> FixKind {
        let mut kind = FixKind::None;

        if self.fix {
            kind.set(FixKind::SafeFix, true);
        }

        if self.fix_suggestions {
            kind.set(FixKind::Suggestion, true);
        }

        if self.fix_dangerously {
            if kind.is_none() {
                kind.set(FixKind::Fix, true);
            }
            kind.set(FixKind::Dangerous, true);
        }

        kind
    }

    pub fn is_enabled(&self) -> bool {
        self.fix || self.fix_suggestions || self.fix_dangerously
    }
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

/// Output
#[derive(Debug, Clone, Bpaf)]
pub struct OutputOptions {
    /// Use a specific output format (default, json, unix, checkstyle, github)
    #[bpaf(long, short, fallback(OutputFormat::Default), hide_usage)]
    pub format: OutputFormat,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputFormat {
    Default,
    /// GitHub Check Annotation
    /// <https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions#setting-a-notice-message>
    Github,
    Json,
    Unix,
    Checkstyle,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Self::Json),
            "default" => Ok(Self::Default),
            "unix" => Ok(Self::Unix),
            "checkstyle" => Ok(Self::Checkstyle),
            "github" => Ok(Self::Github),
            _ => Err(format!("'{s}' is not a known format")),
        }
    }
}

/// Enable Plugins
#[allow(clippy::struct_field_names)]
#[derive(Debug, Default, Clone, Bpaf)]
pub struct EnablePlugins {
    /// Disable react plugin, which is turned on by default
    #[bpaf(
        long("disable-react-plugin"),
        flag(OverrideToggle::Disable, OverrideToggle::NotSet),
        hide_usage
    )]
    pub react_plugin: OverrideToggle,

    /// Disable unicorn plugin, which is turned on by default
    #[bpaf(
        long("disable-unicorn-plugin"),
        flag(OverrideToggle::Disable, OverrideToggle::NotSet),
        hide_usage
    )]
    pub unicorn_plugin: OverrideToggle,

    /// Disable oxc unique rules, which is turned on by default
    #[bpaf(
        long("disable-oxc-plugin"),
        flag(OverrideToggle::Disable, OverrideToggle::NotSet),
        hide_usage
    )]
    pub oxc_plugin: OverrideToggle,

    /// Disable TypeScript plugin, which is turned on by default
    #[bpaf(
        long("disable-typescript-plugin"),
        flag(OverrideToggle::Disable, OverrideToggle::NotSet),
        hide_usage
    )]
    pub typescript_plugin: OverrideToggle,

    /// Enable the experimental import plugin and detect ESM problems.
    /// It is recommended to use along side with the `--tsconfig` option.
    #[bpaf(flag(OverrideToggle::Enable, OverrideToggle::NotSet), hide_usage)]
    pub import_plugin: OverrideToggle,

    /// Enable the experimental jsdoc plugin and detect JSDoc problems
    #[bpaf(flag(OverrideToggle::Enable, OverrideToggle::NotSet), hide_usage)]
    pub jsdoc_plugin: OverrideToggle,

    /// Enable the Jest plugin and detect test problems
    #[bpaf(flag(OverrideToggle::Enable, OverrideToggle::NotSet), hide_usage)]
    pub jest_plugin: OverrideToggle,

    /// Enable the Vitest plugin and detect test problems
    #[bpaf(flag(OverrideToggle::Enable, OverrideToggle::NotSet), hide_usage)]
    pub vitest_plugin: OverrideToggle,

    /// Enable the JSX-a11y plugin and detect accessibility problems
    #[bpaf(flag(OverrideToggle::Enable, OverrideToggle::NotSet), hide_usage)]
    pub jsx_a11y_plugin: OverrideToggle,

    /// Enable the Next.js plugin and detect Next.js problems
    #[bpaf(flag(OverrideToggle::Enable, OverrideToggle::NotSet), hide_usage)]
    pub nextjs_plugin: OverrideToggle,

    /// Enable the React performance plugin and detect rendering performance problems
    #[bpaf(flag(OverrideToggle::Enable, OverrideToggle::NotSet), hide_usage)]
    pub react_perf_plugin: OverrideToggle,

    /// Enable the promise plugin and detect promise usage problems
    #[bpaf(flag(OverrideToggle::Enable, OverrideToggle::NotSet), hide_usage)]
    pub promise_plugin: OverrideToggle,

    /// Enable the node plugin and detect node usage problems
    #[bpaf(flag(OverrideToggle::Enable, OverrideToggle::NotSet), hide_usage)]
    pub node_plugin: OverrideToggle,

    /// Enable the security plugin and detect security problems
    #[bpaf(flag(OverrideToggle::Enable, OverrideToggle::NotSet), hide_usage)]
    pub security_plugin: OverrideToggle,
}

/// Enables or disables a boolean option, or leaves it unset.
///
/// We want CLI flags to modify whatever's set in the user's config file, but we don't want them
/// changing default behavior if they're not explicitly passed by the user. This scheme is a bit
/// convoluted, but needed due to architectural constraints imposed by `bpaf`.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum OverrideToggle {
    /// Override the option to enabled
    Enable,
    /// Override the option to disabled
    Disable,
    /// Do not override.
    #[default]
    NotSet,
}

impl From<Option<bool>> for OverrideToggle {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(true) => Self::Enable,
            Some(false) => Self::Disable,
            None => Self::NotSet,
        }
    }
}

impl From<OverrideToggle> for Option<bool> {
    fn from(value: OverrideToggle) -> Self {
        match value {
            OverrideToggle::Enable => Some(true),
            OverrideToggle::Disable => Some(false),
            OverrideToggle::NotSet => None,
        }
    }
}

impl OverrideToggle {
    #[inline]
    pub fn is_enabled(self) -> bool {
        matches!(self, Self::Enable)
    }

    #[inline]
    pub fn is_not_set(self) -> bool {
        matches!(self, Self::NotSet)
    }

    pub fn inspect<F>(self, f: F)
    where
        F: FnOnce(bool),
    {
        if let Some(v) = self.into() {
            f(v);
        }
    }
}

impl EnablePlugins {
    pub fn apply_overrides(&self, plugins: &mut LintPlugins) {
        self.react_plugin.inspect(|yes| plugins.set(LintPlugins::REACT, yes));
        self.unicorn_plugin.inspect(|yes| plugins.set(LintPlugins::UNICORN, yes));
        self.oxc_plugin.inspect(|yes| plugins.set(LintPlugins::OXC, yes));
        self.typescript_plugin.inspect(|yes| plugins.set(LintPlugins::TYPESCRIPT, yes));
        self.import_plugin.inspect(|yes| plugins.set(LintPlugins::IMPORT, yes));
        self.jsdoc_plugin.inspect(|yes| plugins.set(LintPlugins::JSDOC, yes));
        self.jest_plugin.inspect(|yes| plugins.set(LintPlugins::JEST, yes));
        self.vitest_plugin.inspect(|yes| plugins.set(LintPlugins::VITEST, yes));
        self.jsx_a11y_plugin.inspect(|yes| plugins.set(LintPlugins::JSX_A11Y, yes));
        self.nextjs_plugin.inspect(|yes| plugins.set(LintPlugins::NEXTJS, yes));
        self.react_perf_plugin.inspect(|yes| plugins.set(LintPlugins::REACT_PERF, yes));
        self.promise_plugin.inspect(|yes| plugins.set(LintPlugins::PROMISE, yes));
        self.node_plugin.inspect(|yes| plugins.set(LintPlugins::NODE, yes));
        self.security_plugin.inspect(|yes| plugins.set(LintPlugins::SECURITY, yes));

        // Without this, jest plugins adapted to vitest will not be enabled.
        if self.vitest_plugin.is_enabled() && self.jest_plugin.is_not_set() {
            plugins.set(LintPlugins::JEST, true);
        }
    }
}

#[cfg(test)]
mod plugins {
    use super::{EnablePlugins, OverrideToggle};
    use oxc_linter::LintPlugins;

    #[test]
    fn test_override_default() {
        let mut plugins = LintPlugins::default();
        let enable = EnablePlugins::default();

        enable.apply_overrides(&mut plugins);
        assert_eq!(plugins, LintPlugins::default());
    }

    #[test]
    fn test_overrides() {
        let mut plugins = LintPlugins::default();
        let enable = EnablePlugins {
            react_plugin: OverrideToggle::Enable,
            unicorn_plugin: OverrideToggle::Disable,
            ..EnablePlugins::default()
        };
        let expected =
            LintPlugins::default().union(LintPlugins::REACT).difference(LintPlugins::UNICORN);

        enable.apply_overrides(&mut plugins);
        assert_eq!(plugins, expected);
    }

    #[test]
    fn test_override_vitest() {
        let mut plugins = LintPlugins::default();
        let enable =
            EnablePlugins { vitest_plugin: OverrideToggle::Enable, ..EnablePlugins::default() };
        let expected = LintPlugins::default() | LintPlugins::VITEST | LintPlugins::JEST;

        enable.apply_overrides(&mut plugins);
        assert_eq!(plugins, expected);
    }
}

#[cfg(test)]
mod warning_options {
    use super::{lint_command, WarningOptions};

    fn get_warning_options(arg: &str) -> WarningOptions {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        lint_command().run_inner(args.as_slice()).unwrap().warning_options
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
    use std::{fs::File, path::PathBuf};

    use oxc_linter::AllowWarnDeny;

    use super::{lint_command, LintCommand, OutputFormat};

    fn get_lint_options(arg: &str) -> LintCommand {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        lint_command().run_inner(args.as_slice()).unwrap()
    }

    #[test]
    fn default() {
        let options = get_lint_options(".");
        assert_eq!(options.paths, vec![PathBuf::from(".")]);
        assert!(!options.fix_options.fix);
        assert!(!options.list_rules);
        assert_eq!(options.output_options.format, OutputFormat::Default);
    }

    #[test]
    #[allow(clippy::similar_names)]
    fn multiple_paths() {
        let temp_dir = tempfile::tempdir().expect("Could not create a temp dir");
        let file_foo = temp_dir.path().join("foo.js");
        File::create(&file_foo).expect("Could not create foo.js temp file");
        let file_name_foo =
            file_foo.to_str().expect("Could not get path string for foo.js temp file");
        let file_bar = temp_dir.path().join("bar.js");
        File::create(&file_bar).expect("Could not create bar.js temp file");
        let file_name_bar =
            file_bar.to_str().expect("Could not get path string for bar.js temp file");
        let file_baz = temp_dir.path().join("baz");
        File::create(&file_baz).expect("Could not create baz temp file");
        let file_name_baz = file_baz.to_str().expect("Could not get path string for baz temp file");

        let options =
            get_lint_options(format!("{file_name_foo} {file_name_bar} {file_name_baz}").as_str());
        assert_eq!(options.paths, [file_foo, file_bar, file_baz]);
    }

    #[cfg(target_os = "windows")]
    #[test]
    #[allow(clippy::similar_names)]
    fn wildcard_expansion() {
        let temp_dir = tempfile::tempdir().expect("Could not create a temp dir");
        let file_foo = temp_dir.path().join("foo.js");
        File::create(&file_foo).expect("Could not create foo.js temp file");
        let file_bar = temp_dir.path().join("bar.js");
        File::create(&file_bar).expect("Could not create bar.js temp file");
        let file_baz = temp_dir.path().join("baz");
        File::create(&file_baz).expect("Could not create baz temp file");

        let js_files_wildcard = temp_dir.path().join("*.js");
        let options = get_lint_options(
            js_files_wildcard.to_str().expect("could not get js files wildcard path"),
        );
        assert!(options.paths.contains(&file_foo));
        assert!(options.paths.contains(&file_bar));
        assert!(!options.paths.contains(&file_baz));
    }

    #[test]
    fn no_parent_path() {
        match lint_command().run_inner(&["../parent_dir"]) {
            Ok(_) => panic!("Should not allow parent dir"),
            Err(err) => match err {
                bpaf::ParseFailure::Stderr(doc) => {
                    assert_eq!("`../parent_dir`: PATH must not contain \"..\"", format!("{doc}"));
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

    #[test]
    fn format() {
        let options = get_lint_options("-f json");
        assert_eq!(options.output_options.format, OutputFormat::Json);
        assert!(options.paths.is_empty());
    }

    #[test]
    fn format_error() {
        let args = "-f asdf".split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        let result = lint_command().run_inner(args.as_slice());
        assert!(result.is_err_and(
            |err| err.unwrap_stderr() == "couldn't parse `asdf`: 'asdf' is not a known format"
        ));
    }

    #[test]
    fn list_rules() {
        let options = get_lint_options("--rules");
        assert!(options.list_rules);
    }
}
