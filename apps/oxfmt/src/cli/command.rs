// `usage_rs::Args` generates public partial structs with underscore-prefixed fields.
#![allow(clippy::allow_attributes, clippy::pub_underscore_fields)]

#[cfg(feature = "napi")]
use std::str::FromStr;
use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

use usage_rs as usage;
use usage_rs::{Args, Cli};

fn invalid_path(paths: &[PathBuf]) -> Option<&PathBuf> {
    paths.iter().find(|path| {
        path.components().any(|component| component == std::path::Component::ParentDir)
    })
}

const PATHS_ERROR_MESSAGE: &str = "PATH must not contain \"..\"";

#[derive(Debug, Clone)]
pub struct FormatCommand {
    pub mode: Mode,
    pub config_options: ConfigOptions,
    pub ignore_options: IgnoreOptions,
    pub runtime_options: RuntimeOptions,
    /// Single file, path or list of paths.
    /// Glob patterns are also supported.
    /// (Be sure to quote them, otherwise your shell may expand them before passing.)
    /// Exclude patterns with `!` prefix like `'!**/fixtures/*.js'` are also supported.
    /// If not provided, current working directory is used.
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Cli)]
#[usage(
    bin = "oxfmt",
    version,
    completion,
    unknown_flags = "error",
    args_override_self = false,
    usage = "oxfmt [-c=PATH] [PATH]..."
)]
struct FormatCli {
    #[usage(flatten)]
    mode_options: ModeOptions,
    #[usage(flatten, next_help_heading = "Config Options")]
    config_options: ConfigOptions,
    #[usage(flatten, next_help_heading = "Ignore Options")]
    ignore_options: IgnoreOptions,
    #[usage(flatten, next_help_heading = "Runtime Options")]
    runtime_options: RuntimeOptions,
    /// Single file, path or list of paths.
    /// Glob patterns are also supported.
    /// (Be sure to quote them, otherwise your shell may expand them before passing.)
    /// Exclude patterns with `!` prefix like `'!**/fixtures/*.js'` are also supported.
    /// If not provided, current working directory is used.
    #[usage(name = "PATH")]
    paths: Vec<PathBuf>,
}

impl FormatCommand {
    #[expect(clippy::print_stderr)]
    pub fn parse() -> Self {
        let cli = FormatCli::parse();
        match Self::from_cli(cli) {
            Ok(command) => command,
            Err(error) => {
                let args = std::env::args_os().skip(1).collect::<Vec<_>>();
                let refs = args.iter().map(AsRef::as_ref).collect::<Vec<&OsStr>>();
                eprint!("{}", FormatCli::render_failure(&refs, &error));
                usage::__usage_process_exit(2);
            }
        }
    }

    /// Parses formatter options from an argument slice.
    ///
    /// # Errors
    ///
    /// Returns an error when an argument is invalid or conflicts with another option.
    pub fn parse_from<'v, T>(args: &'v [T]) -> Result<Self, usage::Error<'static, 'v>>
    where
        T: AsRef<OsStr> + 'v,
    {
        let refs = args.iter().map(AsRef::as_ref).collect::<Vec<_>>();
        let cli = FormatCli::parse_from(&refs)?;
        Self::from_cli(cli)
    }

    pub fn command() -> &'static usage::Command<'static> {
        FormatCli::command()
    }

    pub fn spec() -> &'static usage::spec::Spec<'static> {
        FormatCli::spec()
    }

    pub fn render_help(cmd: &usage::Command<'_>, long: bool) -> Option<String> {
        FormatCli::render_help(cmd, long)
    }

    pub fn render_failure<'v, T>(args: &'v [T], error: &usage::Error<'static, 'v>) -> String
    where
        T: AsRef<OsStr> + 'v,
    {
        let refs = args.iter().map(AsRef::as_ref).collect::<Vec<_>>();
        FormatCli::render_failure(&refs, error)
    }

    pub fn to_kdl() -> String {
        FormatCli::to_kdl()
    }

    pub fn spec_request<T: AsRef<OsStr>>(args: &[T]) -> Option<String> {
        let refs = args.iter().map(AsRef::as_ref).collect::<Vec<_>>();
        FormatCli::spec_request(&refs)
    }

    pub fn completion_request(args: &[OsString]) -> Option<String> {
        FormatCli::completion_request(args)
    }

    fn from_cli<'v>(cli: FormatCli) -> Result<Self, usage::Error<'static, 'v>> {
        if let Some(path) = invalid_path(&cli.paths) {
            return Err(usage::Error::InvalidValue(Box::new(usage::InvalidValue {
                name: "PATH",
                value: path.display().to_string(),
                reason: PATHS_ERROR_MESSAGE.to_string(),
            })));
        }

        Ok(Self {
            mode: cli.mode_options.into(),
            config_options: cli.config_options,
            ignore_options: cli.ignore_options,
            runtime_options: cli.runtime_options,
            paths: cli.paths,
        })
    }
}

// ---

/// Operation Mode
#[derive(Debug, Clone)]
pub enum Mode {
    /// Default CLI mode run against files and directories
    Cli(OutputMode),
    /// Stdin mode - read from stdin and write to stdout
    #[cfg(feature = "napi")]
    Stdin(PathBuf),
    /// Start language server protocol (LSP) server
    #[cfg(feature = "napi")]
    Lsp,
    /// Initialize `.oxfmtrc.json` with default values
    // NOTE: Actual logic is handled by JS side.
    #[cfg(feature = "napi")]
    Init,
    /// Migrate Prettier configuration to `.oxfmtrc.json`
    // NOTE: Actual logic is handled by JS side.
    #[cfg(feature = "napi")]
    Migrate(MigrateSource),
}

#[derive(Debug, Clone, Args)]
#[usage(group("mode"))]
struct ModeOptions {
    /// Initialize `.oxfmtrc.json` with default values
    #[cfg(feature = "napi")]
    #[usage(long, group = "mode", help_heading = "Mode Options")]
    init: bool,

    /// Migrate configuration to `.oxfmtrc.json` from specified source.
    /// Available sources: prettier, biome.
    #[cfg(feature = "napi")]
    #[usage(long, value_name = "SOURCE", group = "mode", help_heading = "Mode Options")]
    migrate: Option<MigrateSource>,

    /// Start language server protocol (LSP) server
    #[cfg(feature = "napi")]
    #[usage(long, group = "mode", help_heading = "Mode Options")]
    lsp: bool,

    /// Specify the file name to use to infer which parser to use
    #[cfg(feature = "napi")]
    #[usage(long, value_name = "PATH", group = "mode", help_heading = "Mode Options")]
    stdin_filepath: Option<PathBuf>,

    /// Format and write files in place (default)
    #[usage(long, group = "mode", help_heading = "Output Options")]
    write: bool,

    /// Check if files are formatted, also show statistics
    #[usage(long, group = "mode", help_heading = "Output Options")]
    check: bool,

    /// List files that would be changed
    #[usage(long, group = "mode", help_heading = "Output Options")]
    list_different: bool,
}

impl From<ModeOptions> for Mode {
    fn from(options: ModeOptions) -> Self {
        #[cfg(feature = "napi")]
        if options.init {
            return Self::Init;
        }
        #[cfg(feature = "napi")]
        if let Some(source) = options.migrate {
            return Self::Migrate(source);
        }
        #[cfg(feature = "napi")]
        if options.lsp {
            return Self::Lsp;
        }
        #[cfg(feature = "napi")]
        if let Some(path) = options.stdin_filepath {
            return Self::Stdin(path);
        }
        if options.check {
            Self::Cli(OutputMode::Check)
        } else if options.list_different {
            Self::Cli(OutputMode::ListDifferent)
        } else {
            let _ = options.write;
            Self::Cli(OutputMode::Write)
        }
    }
}

/// Format output mode
#[derive(Debug, Clone, Copy)]
pub enum OutputMode {
    /// Default - when no output option is specified, behaves like `--write` mode in Prettier
    Write,
    /// Check mode - check if files are formatted, also show statistics
    Check,
    /// List mode - list files that would be changed
    ListDifferent,
}

/// Migration Source
#[cfg(feature = "napi")]
#[derive(Debug, Clone)]
pub enum MigrateSource {
    /// Migrate from Prettier configuration
    Prettier,
    /// Migrate from Biome configuration
    Biome,
}

#[cfg(feature = "napi")]
impl FromStr for MigrateSource {
    type Err = String;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        if source.eq_ignore_ascii_case("prettier") {
            Ok(Self::Prettier)
        } else if source.eq_ignore_ascii_case("biome") {
            Ok(Self::Biome)
        } else {
            Err(format!("Unknown migration source: {source}. Supported: prettier, biome."))
        }
    }
}

// ---

/// Config Options
#[derive(Debug, Clone, Args)]
pub struct ConfigOptions {
    /// Path to the configuration file (.json, .jsonc, .ts, .mts, .cts, .js, .mjs, .cjs)
    #[usage(short, long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    /// Do not search for configuration files in subdirectories
    #[usage(long)]
    pub disable_nested_config: bool,
}

/// Ignore Options
#[derive(Debug, Clone, Args)]
pub struct IgnoreOptions {
    /// Path to ignore file(s). Can be specified multiple times.
    /// If not specified, .gitignore and .prettierignore in the current directory are used.
    #[usage(long, value_name = "PATH")]
    pub ignore_path: Vec<PathBuf>,
    /// Format code in node_modules directory (skipped by default)
    #[usage(long)]
    pub with_node_modules: bool,
}

/// Runtime Options
#[derive(Debug, Clone, Args)]
pub struct RuntimeOptions {
    /// Do not exit with error when pattern is unmatched
    #[usage(long)]
    pub no_error_on_unmatched_pattern: bool,
    /// Number of threads to use. Set to 1 for using only 1 CPU core.
    #[usage(long, value_name = "INT")]
    pub threads: Option<usize>,
}
