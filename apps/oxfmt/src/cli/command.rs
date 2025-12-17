use std::path::PathBuf;

use bpaf::{Bpaf, Parser};
#[cfg(feature = "napi")]
use cow_utils::CowUtils;

const VERSION: &str = match option_env!("OXC_VERSION") {
    Some(v) => v,
    None => "dev",
};

#[expect(clippy::ptr_arg)]
fn validate_paths(paths: &Vec<PathBuf>) -> bool {
    if paths.is_empty() {
        true
    } else {
        paths.iter().all(|p| p.components().all(|c| c != std::path::Component::ParentDir))
    }
}

const PATHS_ERROR_MESSAGE: &str = "PATH must not contain \"..\"";

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version(VERSION))]
pub struct FormatCommand {
    #[bpaf(external(mode))]
    pub mode: Mode,
    #[bpaf(external)]
    pub config_options: ConfigOptions,
    #[bpaf(external)]
    pub ignore_options: IgnoreOptions,
    #[bpaf(external)]
    pub runtime_options: RuntimeOptions,
    /// Single file, single path or list of paths.
    /// If not provided, current working directory is used.
    /// Glob is supported only for exclude patterns like `'!**/fixtures/*.js'`.
    // `bpaf(fallback)` seems to have issues with `many` or `positional`,
    // so we implement the fallback behavior in code instead.
    #[bpaf(positional("PATH"), many, guard(validate_paths, PATHS_ERROR_MESSAGE))]
    pub paths: Vec<PathBuf>,
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

fn mode() -> impl bpaf::Parser<Mode> {
    let output_mode_options = output_mode().map(Mode::Cli);

    #[cfg(feature = "napi")]
    {
        let init = bpaf::long("init")
            .help("Initialize `.oxfmtrc.json` with default values")
            .req_flag(Mode::Init)
            .hide_usage();
        let migrate = bpaf::long("migrate")
            .help("Migrate configuration to `.oxfmtrc.json` from specified source\nAvailable sources: prettier")
            .argument::<String>("SOURCE")
            .parse(|s| match s.cow_to_lowercase().as_ref() {
                "prettier" => Ok(Mode::Migrate(MigrateSource::Prettier)),
                _ => Err(format!("Unknown migration source: {s}. Supported: prettier.")),
            })
            .hide_usage();
        let lsp = bpaf::long("lsp")
            .help("Start language server protocol (LSP) server")
            .req_flag(Mode::Lsp)
            .hide_usage();
        let stdin_filepath = bpaf::long("stdin-filepath")
            .help("Specify the file name to use to infer which parser to use")
            .argument::<PathBuf>("PATH")
            .map(Mode::Stdin)
            .hide_usage();
        let mode_options =
            bpaf::construct!([init, migrate, lsp, stdin_filepath]).group_help("Mode Options:");

        bpaf::construct!([mode_options, output_mode_options]).fallback(Mode::Cli(OutputMode::Write))
    }
    #[cfg(not(feature = "napi"))]
    {
        output_mode_options.fallback(Mode::Cli(OutputMode::Write))
    }
}

/// Format output mode
#[derive(Debug, Clone)]
pub enum OutputMode {
    /// Default - when no output option is specified, behaves like `--write` mode in Prettier
    Write,
    /// Check mode - check if files are formatted, also show statistics
    Check,
    /// List mode - list files that would be changed
    ListDifferent,
}

fn output_mode() -> impl bpaf::Parser<OutputMode> {
    let write = bpaf::long("write")
        .help("Format and write files in place (default)")
        .req_flag(OutputMode::Write)
        .hide_usage();
    let check = bpaf::long("check")
        .help("Check if files are formatted, also show statistics")
        .req_flag(OutputMode::Check)
        .hide_usage();
    let list_different = bpaf::long("list-different")
        .help("List files that would be changed")
        .req_flag(OutputMode::ListDifferent)
        .hide_usage();

    bpaf::construct!([write, check, list_different]).group_help("Output Options:")
}

/// Migration Source
#[cfg(feature = "napi")]
#[derive(Debug, Clone)]
pub enum MigrateSource {
    /// Migrate from Prettier configuration
    Prettier,
}

// ---

/// Config Options
#[derive(Debug, Clone, Bpaf)]
pub struct ConfigOptions {
    /// Path to the configuration file
    #[bpaf(short, long, argument("PATH"))]
    pub config: Option<PathBuf>,
}

/// Ignore Options
#[derive(Debug, Clone, Bpaf)]
pub struct IgnoreOptions {
    /// Path to ignore file(s). Can be specified multiple times.
    /// If not specified, .gitignore and .prettierignore in the current directory are used.
    #[bpaf(argument("PATH"), many, hide_usage)]
    pub ignore_path: Vec<PathBuf>,
    /// Format code in node_modules directory (skipped by default)
    #[bpaf(switch, hide_usage)]
    pub with_node_modules: bool,
}

/// Runtime Options
#[derive(Debug, Clone, Bpaf)]
pub struct RuntimeOptions {
    /// Do not exit with error when pattern is unmatched
    #[bpaf(switch, hide_usage)]
    pub no_error_on_unmatched_pattern: bool,
    /// Number of threads to use. Set to 1 for using only 1 CPU core.
    #[bpaf(argument("INT"), hide_usage)]
    pub threads: Option<usize>,
}
