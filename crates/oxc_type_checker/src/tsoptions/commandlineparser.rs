//! Port of typescript-go's `internal/tsoptions/commandlineparser.go`.

use std::path::PathBuf;

use usage_rs::Cli;

/// oxc type checker (experimental)
#[derive(Debug, Clone, Cli)]
#[usage(bin = "oxcheck", version, completion, unknown_flags = "error", args_override_self = false)]
pub struct TypeCheckCommand {
    /// Compile the project given the path to its configuration file, or to a
    /// folder with a 'tsconfig.json'.
    #[usage(short = 'p', long, value_name = "FILE OR DIRECTORY")]
    pub project: Option<PathBuf>,

    /// Source files to type-check.
    #[usage(name = "FILE")]
    pub files: Vec<PathBuf>,
}

/// Parse `std::env::args()` into a [`TypeCheckCommand`], mirroring tsgo's
/// `ParseCommandLine`.
///
/// `usage` handles `--help`, `--version`, and argument errors, exiting the process itself.
pub fn parse_command_line() -> TypeCheckCommand {
    TypeCheckCommand::parse()
}
