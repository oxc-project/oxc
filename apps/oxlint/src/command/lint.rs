// `usage_rs::Args` generates public partial structs with underscore-prefixed fields.
#![allow(clippy::allow_attributes, clippy::pub_underscore_fields)]

use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
    str::FromStr,
    sync::OnceLock,
};

use oxc_linter::{AllowWarnDeny, FixKind, LintPlugins};
use usage_rs as usage;
use usage_rs::{Args, Cli, Event, Parser, ValidationError};

use crate::output_formatter::OutputFormat;

use super::{MiscOptions, PATHS_ERROR_MESSAGE, ignore::IgnoreOptions, invalid_path};

const LINT_FILTERS_HELP: &str = "Lint filters accumulate from left to right on the command line.\n\
For example: `-D correctness -A no-debugger` or `-A all -D no-debugger`.\n\
Categories:\n\
\n\
* `correctness` - Code that is outright wrong or useless (default)\n\
* `suspicious` - Code that is most likely wrong or useless\n\
* `pedantic` - Lints which are rather strict or have occasional false positives\n\
* `perf` - Code that could be written in a more performant way\n\
* `style` - Code that should be written in a more idiomatic way\n\
* `restriction` - Lints which prevent the use of language and library features\n\
* `nursery` - New lints that are still under development\n\
* `all` - All categories listed above except `nursery`. Does not enable plugins automatically.";

#[derive(Debug, Clone)]
pub struct LintCommand {
    pub basic_options: BasicOptions,
    pub filter: Vec<(AllowWarnDeny, String)>,
    pub enable_plugins: EnablePlugins,
    pub fix_options: FixOptions,
    pub ignore_options: IgnoreOptions,
    pub warning_options: WarningOptions,
    pub output_options: OutputOptions,
    /// List all the rules that are currently registered
    pub list_rules: bool,
    /// Start the language server
    pub lsp: bool,
    pub misc_options: MiscOptions,
    /// Disable the automatic loading of nested configuration files
    pub disable_nested_config: bool,
    /// Enable rules that require type information
    pub type_aware: bool,
    /// Enable experimental type checking (includes TypeScript compiler diagnostics)
    pub type_check: bool,
    /// Run only TypeScript type checking diagnostics without regular lint diagnostics
    pub type_check_only: bool,
    pub inline_config_options: InlineConfigOptions,
    pub suppression_options: SuppressionOptions,
    /// Single file, single path or list of paths
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Cli)]
#[usage(
    bin = "oxlint",
    version,
    completion,
    unknown_flags = "error",
    args_override_self = false,
    next_line_help,
    long_about = LINT_FILTERS_HELP,
    usage = "oxlint [-c=./.oxlintrc.json] [PATH]...",
    help_template = "{{about}}\n\n{{usage}}\n\n{{commands}}\n\n{{grouped_flags}}\n\n{{ungrouped_args}}\n\n{{ungrouped_flags}}\n\n{{after_help}}",
    example("oxlint -D correctness src", header = "Deny a category"),
    example("oxlint --format github .", header = "GitHub Actions"),
    validate_with = validate_lint_cli
)]
struct LintCli {
    #[usage(flatten, next_help_heading = "Basic Configuration")]
    basic_options: BasicOptions,
    #[usage(flatten, next_help_heading = "Allowing / Denying Multiple Lints")]
    filter_options: LintFilterOptions,
    #[usage(flatten, next_help_heading = "Enable/Disable Plugins")]
    enable_plugins: EnablePluginsCli,
    #[usage(flatten, next_help_heading = "Fix Problems")]
    fix_options: FixOptions,
    #[usage(flatten, next_help_heading = "Ignore Files")]
    ignore_options: IgnoreOptions,
    #[usage(flatten, next_help_heading = "Handle Warnings")]
    warning_options: WarningOptions,
    #[usage(flatten, next_help_heading = "Output")]
    output_options: OutputOptions,

    /// List all the rules that are currently registered
    #[usage(long = "rules")]
    list_rules: bool,

    /// Start the language server
    #[usage(long = "lsp")]
    lsp: bool,

    #[usage(flatten, next_help_heading = "Miscellaneous")]
    misc_options: MiscOptions,

    /// Disable the automatic loading of nested configuration files
    #[usage(long)]
    disable_nested_config: bool,

    /// Enable rules that require type information
    #[usage(long)]
    type_aware: bool,

    /// Enable experimental type checking (includes TypeScript compiler diagnostics)
    #[usage(long)]
    type_check: bool,

    /// Run only TypeScript type checking diagnostics without regular lint diagnostics
    #[usage(long, hide)]
    type_check_only: bool,

    #[usage(flatten, next_help_heading = "Inline Configuration Comments")]
    inline_config_options: InlineConfigOptionsCli,

    #[usage(flatten)]
    suppression_options: SuppressionOptions,

    /// Single file, single path or list of paths
    #[usage(name = "PATH", value_hint = usage::ValueHint::AnyPath)]
    paths: Vec<PathBuf>,
}

fn validate_lint_cli(cli: &LintCli) -> Result<(), ValidationError> {
    if let Some(path) = invalid_path(&cli.paths) {
        return Err(ValidationError::field("PATH")
            .value(path.display().to_string())
            .reason(PATHS_ERROR_MESSAGE));
    }
    Ok(())
}

#[derive(Debug, Clone, Args)]
pub struct SuppressionOptions {
    /// Generate suppressions for all current violations
    #[usage(long, hide)]
    pub suppress_all: bool,

    /// Remove entries for violations that no longer exist
    #[usage(long, hide)]
    pub prune_suppressions: bool,
}

impl LintCommand {
    pub fn parse() -> Self {
        let cli = LintCli::parse();
        let args = std::env::args_os().skip(1).collect::<Vec<_>>();
        let refs = args.iter().map(AsRef::as_ref).collect::<Vec<&OsStr>>();
        Self::from_cli(cli, &refs)
    }

    /// Parses linter options from an argument slice.
    ///
    /// # Errors
    ///
    /// Returns an error when an argument is invalid or conflicts with another option.
    pub fn parse_from<'v, T>(args: &'v [T]) -> Result<Self, usage::Error<'static, 'v>>
    where
        T: AsRef<OsStr> + 'v,
    {
        let refs = args.iter().map(AsRef::as_ref).collect::<Vec<_>>();
        let cli = LintCli::parse_from(&refs)?;
        Ok(Self::from_cli(cli, &refs))
    }

    fn from_cli(cli: LintCli, args: &[&OsStr]) -> Self {
        let filter = lint_filters(args);
        let _ = cli.filter_options;
        Self {
            basic_options: cli.basic_options,
            filter,
            enable_plugins: cli.enable_plugins.into(),
            fix_options: cli.fix_options,
            ignore_options: cli.ignore_options,
            warning_options: cli.warning_options,
            output_options: cli.output_options,
            list_rules: cli.list_rules,
            lsp: cli.lsp,
            misc_options: cli.misc_options,
            disable_nested_config: cli.disable_nested_config,
            type_aware: cli.type_aware,
            type_check: cli.type_check,
            type_check_only: cli.type_check_only,
            inline_config_options: cli.inline_config_options.into(),
            suppression_options: cli.suppression_options,
            paths: cli.paths,
        }
    }

    pub fn command() -> &'static usage::Command<'static> {
        LintCli::command()
    }

    pub fn spec() -> &'static usage::spec::Spec<'static> {
        LintCli::spec()
    }

    pub fn render_help(cmd: &usage::Command<'_>, long: bool) -> Option<String> {
        LintCli::render_help(cmd, long)
    }

    pub fn render_failure<'v, T>(args: &'v [T], error: &usage::Error<'static, 'v>) -> String
    where
        T: AsRef<OsStr> + 'v,
    {
        let refs = args.iter().map(AsRef::as_ref).collect::<Vec<_>>();
        LintCli::render_failure(&refs, error)
    }

    pub fn to_kdl() -> String {
        LintCli::to_kdl()
    }

    pub fn embedded_outcome(args: &[OsString]) -> usage::embedded::Outcome<Self> {
        let refs = args.iter().map(AsRef::as_ref).collect::<Vec<&OsStr>>();
        match LintCli::embedded_outcome(args) {
            usage::embedded::Outcome::Parsed(cli) => {
                usage::embedded::Outcome::Parsed(Self::from_cli(cli, &refs))
            }
            usage::embedded::Outcome::Exit(exit) => usage::embedded::Outcome::Exit(exit),
        }
    }
}

fn lint_filters(args: &[&OsStr]) -> Vec<(AllowWarnDeny, String)> {
    let mut filters = Vec::new();
    let mut parser = Parser::new(LintCli::command(), args);
    while let Some(event) = parser.next_event() {
        let event = event.expect("LintCli already parsed the same argv successfully");
        let Event::Flag { flag, value: Some(value), .. } = event else {
            continue;
        };
        let severity = match flag.name {
            "allow" => AllowWarnDeny::Allow,
            "warn" => AllowWarnDeny::Warn,
            "deny" => AllowWarnDeny::Deny,
            _ => continue,
        };
        let value = usage::as_str(value).expect("String fields reject non-UTF-8 values");
        filters.push((severity, value.to_string()));
    }
    filters
}

impl LintCommand {
    pub fn handle_threads(&self) {
        Self::init_rayon_thread_pool(self.misc_options.threads);
    }

    /// Initialize Rayon global thread pool with specified number of threads.
    ///
    /// If `--threads` option is not used, or `--threads 0` is given,
    /// default to the number of available CPU cores.
    ///
    /// Idempotent: rayon's global pool can only be initialized once per
    /// process. The `OnceLock` guarantees we only call `build_global` once,
    /// so the napi `lint()` entry point can be invoked more than once in
    /// the same Node process. The thread count from the first call wins;
    /// subsequent calls keep that pool.
    #[expect(clippy::print_stderr)]
    fn init_rayon_thread_pool(threads: Option<usize>) {
        static RAYON_INIT: OnceLock<()> = OnceLock::new();

        // Always initialize thread pool, even if using default thread count,
        // to ensure thread pool's thread count is locked after this point.
        // `rayon::current_num_threads()` will always return the same number after this point.
        //
        // If you don't initialize the global thread pool explicitly, or don't specify `num_threads`,
        // Rayon will initialize the thread pool when it's first used, with a thread count of
        // `std::thread::available_parallelism()`, and that thread count won't change thereafter.
        // So we don't *need* to initialize the thread pool here if we just want the default thread count.
        //
        // However, Rayon's docs state that:
        // > In the future, the default behavior may change to dynamically add or remove threads as needed.
        // https://docs.rs/rayon/1.11.0/rayon/struct.ThreadPoolBuilder.html#method.num_threads
        //
        // To ensure we continue to have a "locked" thread count, even after future Rayon upgrades,
        // we always initialize the thread pool and explicitly specify thread count here.

        let thread_count = if let Some(thread_count) = threads
            && thread_count > 0
        {
            thread_count
        } else if let Ok(thread_count) = std::thread::available_parallelism() {
            thread_count.get()
        } else {
            eprintln!(
                "Unable to determine available thread count. Defaulting to 1.\nConsider specifying the number of threads explicitly with `--threads` option."
            );
            1
        };

        RAYON_INIT.get_or_init(|| {
            rayon::ThreadPoolBuilder::new().num_threads(thread_count).build_global().unwrap();
        });
    }
}

/// Basic Configuration
#[derive(Debug, Clone, Args)]
pub struct BasicOptions {
    /// Oxlint configuration file
    ///  * `.json` and `.jsonc` config files are supported in all runtimes
    ///  * JavaScript/TypeScript config files are experimental and require running via Node.js
    ///  * you can use comments in configuration files.
    ///  * tries to be compatible with ESLint v8's format
    ///
    /// If not provided, Oxlint will look for a `.oxlintrc.json`, `.oxlintrc.jsonc`, `oxlint.config.ts`, or `oxlint.config.mts` file in the current working directory.
    #[usage(
        long,
        short,
        value_name = "./.oxlintrc.json",
        value_hint = usage::ValueHint::FilePath
    )]
    pub config: Option<PathBuf>,

    /// Override the TypeScript config used for import resolution.
    /// Oxlint automatically discovers the relevant `tsconfig.json` for each file.
    /// Use this only when your project uses a non-standard tsconfig name or location.
    ///
    /// **Warning:** Avoid using this option. It can cause differences between import resolution,
    /// and type-aware linting. Type aware linting **does not** respect this option,
    /// and will always discover the appropriate `tsconfig.json` for each file automatically.
    #[usage(
        long,
        value_name = "./tsconfig.json",
        value_hint = usage::ValueHint::FilePath
    )]
    pub tsconfig: Option<PathBuf>,

    /// Initialize oxlint configuration with default values
    #[usage(long)]
    pub init: bool,
}

#[derive(Debug, Clone, Args)]
struct LintFilterOptions {
    /// Allow the rule or category (suppress the lint)
    #[usage(short = 'A', long, value_name = "NAME")]
    allow: Vec<String>,

    /// Warn on the rule or category (emit a warning)
    #[usage(short = 'W', long, value_name = "NAME")]
    warn: Vec<String>,

    /// Deny the rule or category (emit an error)
    #[usage(short = 'D', long, value_name = "NAME")]
    deny: Vec<String>,
}

/// Fix Problems
#[derive(Debug, Clone, Args)]
pub struct FixOptions {
    /// Fix as many issues as possible. Only unfixed issues are reported in the output.
    #[usage(long)]
    pub fix: bool,

    /// Apply auto-fixable suggestions. May change program behavior.
    #[usage(long)]
    pub fix_suggestions: bool,

    /// Apply dangerous fixes and suggestions
    #[usage(long)]
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
            kind.set(FixKind::DangerousFixOrSuggestion, true);
        }

        kind
    }

    pub fn is_enabled(&self) -> bool {
        self.fix || self.fix_suggestions || self.fix_dangerously
    }
}

/// Handle Warnings
#[derive(Debug, Clone, Args)]
pub struct WarningOptions {
    /// Disable reporting on warnings, only errors are reported
    #[usage(long)]
    pub quiet: bool,

    /// Ensure warnings produce a non-zero exit code
    #[usage(long)]
    pub deny_warnings: bool,

    /// Specify a warning threshold,
    /// which can be used to force exit with an error status if there are too many warning-level rule violations in your project
    #[usage(long, value_name = "INT")]
    pub max_warnings: Option<usize>,
}

/// Output
#[derive(Debug, Clone, Args)]
#[allow(clippy::duplicated_attributes)]
#[usage(
    output("default", default, help = "Human-readable diagnostics"),
    output("agent", help = "Diagnostics optimized for coding agents"),
    output("checkstyle", help = "Checkstyle XML diagnostics"),
    output("github", help = "GitHub workflow annotations"),
    output("gitlab", help = "GitLab Code Quality diagnostics"),
    output("json", framing = "json", help = "JSON diagnostics"),
    output("junit", help = "JUnit XML diagnostics"),
    output("sarif", framing = "json", help = "SARIF diagnostics"),
    output("stylish", help = "Stylish text diagnostics"),
    output("unix", help = "Unix-style text diagnostics"),
    exit_code(0, "lint completed without errors"),
    exit_code(1, "lint errors or invalid options were found")
)]
pub struct OutputOptions {
    /// Use a specific output format.
    #[usage(
        long,
        short,
        value_name = "FORMAT",
        value_enum,
        select,
        default_fn = default_output_format,
        default_note = "auto-detected from the runtime environment"
    )]
    pub format: OutputFormat,

    /// Enable debug output options. Options are comma-separated.
    ///
    ///  * `files` - Print the list of files that will be linted, then exit.
    ///  * `timings` - Enable per-rule timing information.
    #[usage(
        long,
        value_name = "OPTIONS",
        choices("files", "timings"),
        default_fn = DebugOptions::default
    )]
    pub debug: DebugOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugOption {
    /// Print the list of files that will be linted
    Files,

    /// Enable per-rule timing information
    Timings,
}

impl DebugOption {
    const FILES_NAME: &str = "files";
    const TIMINGS_NAME: &str = "timings";
}

impl FromStr for DebugOption {
    type Err = String;

    fn from_str(option: &str) -> Result<Self, Self::Err> {
        match option {
            Self::FILES_NAME => Ok(Self::Files),
            Self::TIMINGS_NAME => Ok(Self::Timings),
            _ => Err(format!("'{option}' is not a known debug option")),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DebugOptions {
    options: Vec<DebugOption>,
}

impl DebugOptions {
    pub fn contains(&self, option: DebugOption) -> bool {
        self.options.contains(&option)
    }
}

impl FromStr for DebugOptions {
    type Err = String;

    fn from_str(options: &str) -> Result<Self, Self::Err> {
        let options = options
            .split(',')
            .filter(|option| !option.is_empty())
            .map(DebugOption::from_str)
            .collect::<Result<Vec<_>, _>>()?;

        if options.contains(&DebugOption::Files)
            && options.iter().any(|option| *option != DebugOption::Files)
        {
            return Err("debug option 'files' cannot be combined with other debug options".into());
        }

        Ok(Self { options })
    }
}

impl std::fmt::Display for DebugOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, option) in self.options.iter().enumerate() {
            if index > 0 {
                f.write_str(",")?;
            }
            f.write_str(match option {
                DebugOption::Files => DebugOption::FILES_NAME,
                DebugOption::Timings => DebugOption::TIMINGS_NAME,
            })?;
        }
        Ok(())
    }
}

fn default_output_format() -> OutputFormat {
    if cfg!(debug_assertions) {
        OutputFormat::Default
    } else if !cfg!(test) && crate::agent_detection::is_agent() {
        OutputFormat::Agent
    } else if std::env::var("GITHUB_ACTIONS").is_ok_and(|value| value == "true") {
        OutputFormat::Github
    } else {
        OutputFormat::Default
    }
}

/// Enable/Disable Plugins
#[derive(Debug, Clone, Args)]
struct EnablePluginsCli {
    /// Disable unicorn plugin, which is turned on by default
    #[usage(long = "disable-unicorn-plugin")]
    unicorn_plugin: bool,

    /// Disable oxc unique rules, which is turned on by default
    #[usage(long = "disable-oxc-plugin")]
    oxc_plugin: bool,

    /// Disable TypeScript plugin, which is turned on by default
    #[usage(long = "disable-typescript-plugin")]
    typescript_plugin: bool,

    /// Enable import plugin and detect ESM problems.
    #[usage(long)]
    import_plugin: bool,

    /// Enable react plugin, which is turned off by default
    #[usage(long)]
    react_plugin: bool,

    /// Enable jsdoc plugin and detect JSDoc problems
    #[usage(long)]
    jsdoc_plugin: bool,

    /// Enable the Jest plugin and detect test problems
    #[usage(long)]
    jest_plugin: bool,

    /// Enable the Vitest plugin and detect test problems
    #[usage(long)]
    vitest_plugin: bool,

    /// Enable the JSX-a11y plugin and detect accessibility problems
    #[usage(long)]
    jsx_a11y_plugin: bool,

    /// Enable the Next.js plugin and detect Next.js problems
    #[usage(long)]
    nextjs_plugin: bool,

    /// Enable the React performance plugin and detect rendering performance problems
    #[usage(long)]
    react_perf_plugin: bool,

    /// Enable the promise plugin and detect promise usage problems
    #[usage(long)]
    promise_plugin: bool,

    /// Enable the node plugin and detect node usage problems
    #[usage(long)]
    node_plugin: bool,

    /// Enable the vue plugin and detect vue usage problems
    #[usage(long)]
    vue_plugin: bool,
}

#[expect(clippy::struct_field_names)]
#[derive(Debug, Default, Clone)]
pub struct EnablePlugins {
    pub unicorn_plugin: OverrideToggle,
    pub oxc_plugin: OverrideToggle,
    pub typescript_plugin: OverrideToggle,
    pub import_plugin: OverrideToggle,
    pub react_plugin: OverrideToggle,
    pub jsdoc_plugin: OverrideToggle,
    pub jest_plugin: OverrideToggle,
    pub vitest_plugin: OverrideToggle,
    pub jsx_a11y_plugin: OverrideToggle,
    pub nextjs_plugin: OverrideToggle,
    pub react_perf_plugin: OverrideToggle,
    pub promise_plugin: OverrideToggle,
    pub node_plugin: OverrideToggle,
    pub vue_plugin: OverrideToggle,
}

impl From<EnablePluginsCli> for EnablePlugins {
    fn from(options: EnablePluginsCli) -> Self {
        Self {
            unicorn_plugin: options.unicorn_plugin.then_some(false).into(),
            oxc_plugin: options.oxc_plugin.then_some(false).into(),
            typescript_plugin: options.typescript_plugin.then_some(false).into(),
            import_plugin: options.import_plugin.then_some(true).into(),
            react_plugin: options.react_plugin.then_some(true).into(),
            jsdoc_plugin: options.jsdoc_plugin.then_some(true).into(),
            jest_plugin: options.jest_plugin.then_some(true).into(),
            vitest_plugin: options.vitest_plugin.then_some(true).into(),
            jsx_a11y_plugin: options.jsx_a11y_plugin.then_some(true).into(),
            nextjs_plugin: options.nextjs_plugin.then_some(true).into(),
            react_perf_plugin: options.react_perf_plugin.then_some(true).into(),
            promise_plugin: options.promise_plugin.then_some(true).into(),
            node_plugin: options.node_plugin.then_some(true).into(),
            vue_plugin: options.vue_plugin.then_some(true).into(),
        }
    }
}

/// Enables or disables a boolean option, or leaves it unset.
///
/// We want CLI flags to modify whatever's set in the user's config file, but we don't want them
/// changing default behavior if they're not explicitly passed by the user.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
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
        self.vue_plugin.inspect(|yes| plugins.set(LintPlugins::VUE, yes));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportUnusedDirectives {
    WithoutSeverity(bool),
    WithSeverity(Option<AllowWarnDeny>),
}

/// Inline Configuration Comments
#[derive(Debug, Clone)]
pub struct InlineConfigOptions {
    pub report_unused_directives: ReportUnusedDirectives,
}

#[derive(Debug, Clone, Args)]
#[usage(group("unused-directives"))]
struct InlineConfigOptionsCli {
    /// Report directive comments like `// oxlint-disable-line`, when no errors would have been reported on that line anyway
    // More information at <https://eslint.org/docs/latest/use/command-line-interface#--report-unused-disable-directives>
    #[usage(long, group = "unused-directives")]
    report_unused_disable_directives: bool,

    /// Same as `--report-unused-disable-directives`, but allows you to specify the severity level of the reported errors.
    /// Only one of these two options can be used at a time.
    #[usage(
        long,
        value_name = "SEVERITY",
        choices("allow", "off", "deny", "error", "warn"),
        group = "unused-directives"
    )]
    report_unused_disable_directives_severity: Option<String>,
}

impl From<InlineConfigOptionsCli> for InlineConfigOptions {
    fn from(options: InlineConfigOptionsCli) -> Self {
        let report_unused_directives =
            if let Some(severity) = options.report_unused_disable_directives_severity {
                ReportUnusedDirectives::WithSeverity(Some(
                    AllowWarnDeny::try_from(severity.as_str()).unwrap(),
                ))
            } else {
                ReportUnusedDirectives::WithoutSeverity(options.report_unused_disable_directives)
            };
        Self { report_unused_directives }
    }
}

#[cfg(test)]
mod plugins {
    use oxc_linter::LintPlugins;

    use super::{EnablePlugins, OverrideToggle};

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
        let expected = LintPlugins::default() | LintPlugins::VITEST;

        enable.apply_overrides(&mut plugins);
        assert_eq!(plugins, expected);
    }
}

#[cfg(test)]
mod warning_options {
    use super::{LintCommand, WarningOptions};

    fn get_warning_options(arg: &str) -> WarningOptions {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        LintCommand::parse_from(args.as_slice()).unwrap().warning_options
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
    use usage_rs as usage;

    use super::{DebugOption, DebugOptions, LintCommand, OutputFormat};

    fn get_lint_options(arg: &str) -> LintCommand {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        LintCommand::parse_from(args.as_slice()).unwrap()
    }

    #[test]
    fn default() {
        let options = get_lint_options(".");
        assert_eq!(options.paths, vec![PathBuf::from(".")]);
        assert!(!options.fix_options.fix);
        assert!(!options.list_rules);
        assert_eq!(options.output_options.format, OutputFormat::Default);
        assert_eq!(options.output_options.debug, DebugOptions::default());
    }

    #[test]
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

    #[test]
    fn no_parent_path() {
        match LintCommand::parse_from(&["../parent_dir"]) {
            Ok(_) => panic!("Should not allow parent dir"),
            Err(usage::Error::InvalidValue(error)) => {
                assert_eq!(error.value, "../parent_dir");
                assert_eq!(error.reason, "PATH must not contain \"..\"");
            }
            Err(_) => unreachable!(),
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

        let options = get_lint_options("-f agent");
        assert_eq!(options.output_options.format, OutputFormat::Agent);
    }

    #[test]
    fn debug() {
        let options = get_lint_options("--debug timings src");
        assert!(options.output_options.debug.contains(DebugOption::Timings));
        assert_eq!(options.paths, vec![PathBuf::from("src")]);

        let options = get_lint_options("--debug files src");
        assert!(options.output_options.debug.contains(DebugOption::Files));
        assert_eq!(options.paths, vec![PathBuf::from("src")]);
    }

    #[test]
    fn debug_files_is_exclusive() {
        let args = "--debug files,timings"
            .split(' ')
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();
        let result = LintCommand::parse_from(args.as_slice());
        assert!(matches!(result, Err(usage::Error::InvalidValue(_))));
    }

    #[test]
    fn debug_error() {
        let args =
            "--debug foo".split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        let error = LintCommand::parse_from(args.as_slice()).unwrap_err();
        let usage::Error::InvalidValue(error) = error else {
            panic!("expected invalid debug value");
        };
        assert_eq!(error.value, "foo");
        assert_eq!(error.reason, "'foo' is not a known debug option");
    }

    #[test]
    fn format_error() {
        let args = "-f asdf".split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        let error = LintCommand::parse_from(args.as_slice()).unwrap_err();
        let usage::Error::InvalidChoice { name, choices } = error else {
            panic!("expected invalid output format choice");
        };
        assert_eq!(name, "format");
        assert_eq!(
            choices,
            [
                "default",
                "github",
                "gitlab",
                "json",
                "unix",
                "agent",
                "checkstyle",
                "stylish",
                "junit",
                "sarif",
            ]
        );
    }

    #[test]
    fn list_rules() {
        let options = get_lint_options("--rules");
        assert!(options.list_rules);
    }

    #[test]
    fn disable_nested_config() {
        let options = get_lint_options("--disable-nested-config");
        assert!(options.disable_nested_config);
        let options = get_lint_options(".");
        assert!(!options.disable_nested_config);
    }

    #[test]
    fn type_aware() {
        let options = get_lint_options("--type-aware");
        assert!(options.type_aware);
        let options = get_lint_options(".");
        assert!(!options.type_aware);
    }

    #[test]
    fn type_check() {
        let options = get_lint_options("--type-check");
        assert!(options.type_check);
        let options = get_lint_options(".");
        assert!(!options.type_check);
    }

    #[test]
    fn type_check_only() {
        let options = get_lint_options("--type-check-only");
        assert!(options.type_check_only);
        let options = get_lint_options(".");
        assert!(!options.type_check_only);
    }

    #[test]
    fn suppress_rules() {
        let options = get_lint_options("--suppress-all");
        assert!(options.suppression_options.suppress_all);
        assert!(!options.suppression_options.prune_suppressions);
    }

    #[test]
    fn prune_suppressions() {
        let options = get_lint_options("--prune-suppressions");
        assert!(options.suppression_options.prune_suppressions);
        assert!(!options.suppression_options.suppress_all);
    }

    #[test]
    fn suppress_and_prune() {
        let options = get_lint_options("--suppress-all --prune-suppressions");
        assert!(options.suppression_options.prune_suppressions);
        assert!(options.suppression_options.suppress_all);
    }
}

#[cfg(test)]
mod inline_config_options {
    use oxc_linter::AllowWarnDeny;

    use super::{LintCommand, ReportUnusedDirectives};

    fn get_lint_options(arg: &str) -> LintCommand {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        LintCommand::parse_from(args.as_slice()).unwrap()
    }

    #[test]
    fn default() {
        let options = get_lint_options(".");
        assert_eq!(
            options.inline_config_options.report_unused_directives,
            ReportUnusedDirectives::WithoutSeverity(false)
        );
    }

    #[test]
    fn without_severity() {
        let options = get_lint_options("--report-unused-disable-directives");
        assert_eq!(
            options.inline_config_options.report_unused_directives,
            ReportUnusedDirectives::WithoutSeverity(true)
        );
    }

    #[test]
    fn with_severity_warn() {
        let options = get_lint_options("--report-unused-disable-directives-severity=warn");
        assert_eq!(
            options.inline_config_options.report_unused_directives,
            ReportUnusedDirectives::WithSeverity(Some(AllowWarnDeny::Warn))
        );
    }

    #[test]
    fn with_severity_error() {
        let options = get_lint_options("--report-unused-disable-directives-severity error");
        assert_eq!(
            options.inline_config_options.report_unused_directives,
            ReportUnusedDirectives::WithSeverity(Some(AllowWarnDeny::Deny))
        );
    }
}

#[cfg(test)]
mod usage_integration {
    use std::ffi::OsString;

    use super::LintCommand;

    #[test]
    fn embedded_help_preserves_sections_and_renders_markdown() {
        let args = [OsString::from("--help")];
        let outcome = LintCommand::embedded_outcome(&args);
        let exit = outcome.exit().expect("help should return an embedded exit");
        assert_eq!(exit.code, 0);
        assert!(!exit.stderr);

        let basic = exit.text.find("Basic Configuration:").expect("basic options heading");
        let arguments = exit.text.find("Arguments:").expect("arguments heading");
        let flags = exit.text.find("Flags:").expect("flags heading");
        assert!(basic < arguments && arguments < flags);
        assert!(exit.text.contains("**Warning:**"));
        assert!(!exit.text.contains("::: warning"));
        assert!(exit.text.contains("[possible values: default, github"));
        assert!(exit.text.contains("Examples:"));
    }

    #[test]
    fn spec_exposes_output_and_completion_contracts() {
        let spec = LintCommand::to_kdl();
        assert!(spec.contains("output json framing=json"));
        assert!(spec.contains("output sarif framing=json"));
        assert!(spec.contains("select \"--format\""));
        assert!(spec.contains("complete path type=path"));
    }

    #[test]
    fn embedded_dispatch_handles_control_requests() {
        let spec = LintCommand::embedded_outcome(&[OsString::from(usage_rs::SPEC_REQUEST)]);
        let exit = spec.exit().expect("spec request should return an embedded exit");
        assert_eq!(exit.code, 0);
        assert!(!exit.stderr);
        assert!(exit.text.contains("name oxlint"));

        let args =
            ["__complete_word__", "--shell", "bash", "--line", "oxlint --fo"].map(OsString::from);
        let completions = LintCommand::embedded_outcome(&args);
        let exit = completions.exit().expect("completion request should return an embedded exit");
        assert_eq!(exit.code, 0);
        assert!(!exit.stderr);
        assert!(exit.text.contains("--format"));
    }
}
