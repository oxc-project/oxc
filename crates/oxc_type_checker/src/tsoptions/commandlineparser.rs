//! Port of typescript-go's `internal/tsoptions/commandlineparser.go`.

use std::path::PathBuf;

use bpaf::Bpaf;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// oxc type checker (experimental)
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version(VERSION))]
pub struct TypeCheckCommand {
    /// Compile the project given the path to its configuration file, or to a
    /// folder with a 'tsconfig.json'.
    #[bpaf(short('p'), long("project"), argument("FILE OR DIRECTORY"))]
    pub project: Option<PathBuf>,

    /// Source files to type-check.
    #[bpaf(positional("FILE"), many)]
    pub files: Vec<PathBuf>,
}

/// Parse `std::env::args()` into a [`TypeCheckCommand`], mirroring tsgo's
/// `ParseCommandLine`.
///
/// `bpaf` handles `--help`, `--version`, and argument errors, exiting the process itself.
pub fn parse_command_line() -> TypeCheckCommand {
    type_check_command().run()
}
