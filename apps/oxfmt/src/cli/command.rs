// `usage_rs::Args` generates public partial structs with underscore-prefixed fields.
#![allow(clippy::allow_attributes, clippy::pub_underscore_fields)]

use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

use usage_rs as usage;
use usage_rs::{Args, Cli, ValidationError};

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
    usage = "oxfmt [-c=PATH] [PATH]...",
    help_template = "{{about}}\n\n{{usage}}\n\n{{commands}}\n\n{{grouped_flags}}\n\n{{ungrouped_args}}\n\n{{ungrouped_flags}}\n\n{{after_help}}",
    example("oxfmt --check .", header = "Check formatting"),
    example("oxfmt --write src", header = "Format files"),
    exit_code(0, "files are formatted or were formatted successfully"),
    exit_code(1, "invalid configuration or formatting differences were found"),
    exit_code(2, "no files were found or formatting failed"),
    try_into = FormatCommand
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
    #[usage(name = "PATH", value_hint = usage::ValueHint::AnyPath)]
    paths: Vec<PathBuf>,
}

impl FormatCommand {
    pub fn parse() -> Self {
        FormatCli::parse_into()
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
        FormatCli::parse_into_from(&refs)
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

    pub fn embedded_outcome(args: &[OsString]) -> usage::embedded::Outcome<Self> {
        let refs = args.iter().map(AsRef::as_ref).collect::<Vec<&OsStr>>();
        if let Some(text) = FormatCli::spec_request(&refs) {
            return usage::embedded::Outcome::Exit(usage::embedded::Exit {
                text,
                stderr: false,
                code: 0,
            });
        }
        if let Some(text) = FormatCli::completion_request(args) {
            return usage::embedded::Outcome::Exit(usage::embedded::Exit {
                text,
                stderr: false,
                code: 0,
            });
        }
        usage::embedded::outcome(
            FormatCli::spec(),
            FormatCli::command(),
            &refs,
            FormatCli::parse_into_from,
        )
    }
}

impl TryFrom<FormatCli> for FormatCommand {
    type Error = ValidationError;

    fn try_from(cli: FormatCli) -> Result<Self, Self::Error> {
        if let Some(path) = invalid_path(&cli.paths) {
            return Err(ValidationError::field("PATH")
                .value(path.display().to_string())
                .reason(PATHS_ERROR_MESSAGE));
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
    #[usage(
        long,
        value_name = "SOURCE",
        value_enum,
        group = "mode",
        help_heading = "Mode Options"
    )]
    migrate: Option<MigrateSource>,

    /// Start language server protocol (LSP) server
    #[cfg(feature = "napi")]
    #[usage(long, group = "mode", help_heading = "Mode Options")]
    lsp: bool,

    /// Specify the file name to use to infer which parser to use
    #[cfg(feature = "napi")]
    #[usage(
        long,
        value_name = "PATH",
        value_hint = usage::ValueHint::FilePath,
        group = "mode",
        help_heading = "Mode Options"
    )]
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
#[derive(Debug, Clone, usage::ValueEnum)]
#[usage(ignore_case)]
pub enum MigrateSource {
    /// Migrate from Prettier configuration
    Prettier,
    /// Migrate from Biome configuration
    Biome,
}

// ---

/// Config Options
#[derive(Debug, Clone, Args)]
pub struct ConfigOptions {
    /// Path to the configuration file (.json, .jsonc, .ts, .mts, .cts, .js, .mjs, .cjs)
    #[usage(short, long, value_name = "PATH", value_hint = usage::ValueHint::FilePath)]
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
    #[usage(long, value_name = "PATH", value_hint = usage::ValueHint::FilePath)]
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

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use usage_rs as usage;

    use super::FormatCommand;

    #[test]
    fn typed_finalization_reports_invalid_paths() {
        let error = FormatCommand::parse_from(&["../src"]).unwrap_err();
        let usage::Error::InvalidValue(error) = error else {
            panic!("expected invalid path value");
        };
        assert_eq!(error.name, "PATH");
        assert_eq!(error.value, "../src");
    }

    #[test]
    fn embedded_help_preserves_section_order() {
        let args = [OsString::from("--help")];
        let outcome = FormatCommand::embedded_outcome(&args);
        let exit = outcome.exit().expect("help should return an embedded exit");
        assert_eq!(exit.code, 0);
        assert!(!exit.stderr);

        let output_options = exit.text.find("Output Options:").expect("output options heading");
        let arguments = exit.text.find("Arguments:").expect("arguments heading");
        let flags = exit.text.find("Flags:").expect("flags heading");
        assert!(output_options < arguments && arguments < flags);
    }

    #[cfg(feature = "napi")]
    #[test]
    fn migration_source_remains_case_insensitive() {
        assert!(FormatCommand::parse_from(&["--migrate", "PRETTIER"]).is_ok());
    }
}
