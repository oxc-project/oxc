#![expect(clippy::print_stdout, clippy::print_stderr)]
//! # `oxcheck`
//!
//! Experimental command-line entry point for [`oxc_type_checker`].
//!
//! This is a small first step towards porting `tsc` / `typescript-go`: it accepts a
//! project the same way `tsc -p` does and resolves it to a `tsconfig.json` path. It does
//! not parse that config or type check anything yet — those are later steps.

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use bpaf::Bpaf;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// oxc type checker (experimental)
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version(VERSION))]
struct TypeCheckCommand {
    /// Compile the project given the path to its configuration file, or to a
    /// folder with a 'tsconfig.json'.
    #[bpaf(short('p'), long("project"), argument("FILE OR DIRECTORY"))]
    project: Option<PathBuf>,

    /// Source files to type-check.
    #[bpaf(positional("FILE"), many)]
    files: Vec<PathBuf>,
}

fn main() -> ExitCode {
    // bpaf handles `--help`, `--version`, and argument errors, exiting the process itself.
    let command = type_check_command().run();

    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(err) => {
            eprintln!("Unable to determine the current working directory: {err}");
            return ExitCode::FAILURE;
        }
    };

    match resolve_config_file(&command, &cwd) {
        // A resolved config file. Later steps will parse it and type check the project.
        Ok(Some(config_file)) => {
            println!("project: {}", config_file.display());
            ExitCode::SUCCESS
        }
        // Source files were given without a config file. Later steps will type check them
        // directly as root files.
        Ok(None) => {
            println!("files: {:?}", command.files);
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

/// Resolve the command line into the `tsconfig.json` (or config file) to load, mirroring
/// `typescript-go`'s `tscCompilation` (`internal/execute/tsc.go`).
///
/// Returns:
/// - `Ok(Some(path))` — a resolved config file to load,
/// - `Ok(None)` — source files were given with no config file (compile them directly),
/// - `Err(message)` — one of `tsc`'s command-line errors, ready to print.
fn resolve_config_file(command: &TypeCheckCommand, cwd: &Path) -> Result<Option<PathBuf>, String> {
    if let Some(project) = &command.project {
        // TS5042
        if !command.files.is_empty() {
            return Err(
                "Option 'project' cannot be mixed with source files on a command line.".to_string()
            );
        }

        let file_or_directory = normalize(project, cwd);
        if file_or_directory.is_dir() {
            // A directory: look for `tsconfig.json` inside it.
            let config_file = file_or_directory.join("tsconfig.json");
            if config_file.is_file() {
                Ok(Some(config_file))
            } else {
                // TS5081
                Err(format!(
                    "Cannot find a tsconfig.json file at the current directory: {}.",
                    config_file.display()
                ))
            }
        } else if file_or_directory.exists() {
            // An explicit config file (need not be named `tsconfig.json`).
            Ok(Some(file_or_directory))
        } else {
            // TS5058
            Err(format!("The specified path does not exist: '{}'.", file_or_directory.display()))
        }
    } else if let Some(config_file) = find_config_file(cwd) {
        if command.files.is_empty() {
            Ok(Some(config_file))
        } else {
            // TS5112: a tsconfig.json is present but source files were also specified.
            Err("tsconfig.json is present but will not be loaded if files are specified on \
                 commandline. Use '--ignoreConfig' to skip this error."
                .to_string())
        }
    } else if command.files.is_empty() {
        // TS5081 — `tsc` prints version + help here; we report the missing config instead.
        Err(format!(
            "Cannot find a tsconfig.json file at the current directory: {}.",
            cwd.display()
        ))
    } else {
        // Source files with no config file anywhere: compile them directly.
        Ok(None)
    }
}

/// Search `dir` and its ancestors for a `tsconfig.json`, mirroring `typescript-go`'s
/// `findConfigFile` (`tspath.ForEachAncestorDirectory`).
fn find_config_file(dir: &Path) -> Option<PathBuf> {
    dir.ancestors()
        .map(|ancestor| ancestor.join("tsconfig.json"))
        .find(|candidate| candidate.is_file())
}

/// Turn `path` into an absolute path against `cwd`.
///
// NOTE: this is a lightweight stand-in for `typescript-go`'s `tspath.NormalizePath`; a
// faithful lexical normalization (collapsing `.`/`..` segments) is a later step.
fn normalize(path: &Path, cwd: &Path) -> PathBuf {
    let joined = cwd.join(path);
    std::path::absolute(&joined).unwrap_or(joined)
}
