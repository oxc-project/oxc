#![expect(clippy::print_stdout, clippy::print_stderr)]
//! Port of typescript-go's `internal/execute/tsc.go`.

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use crate::tsoptions::{TypeCheckCommand, parse_command_line, parse_config_file};

/// Run the type checker from the command line.
///
/// Mirrors tsgo's `execute.CommandLine`: parse the arguments, then drive compilation.
/// `bpaf` reads `std::env::args()` and handles `--help`/`--version`/argument errors itself.
pub fn command_line() -> ExitCode {
    let command = parse_command_line();
    tsc_compilation(&command)
}

/// Mirrors tsgo's `tscCompilation`: resolve the project into a config file (or root files)
/// and report the outcome.
///
/// For now this only prints the resolved target; parsing the `tsconfig.json` and type
/// checking are later steps.
fn tsc_compilation(command: &TypeCheckCommand) -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(err) => {
            eprintln!("Unable to determine the current working directory: {err}");
            return ExitCode::FAILURE;
        }
    };

    match resolve_config_file(command, &cwd) {
        // A resolved config file: parse it (resolving `extends`) via `oxc_resolver`. Later
        // steps will expand its file globs and type check the project.
        Ok(Some(config_file)) => match parse_config_file(&config_file) {
            Ok(tsconfig) => {
                println!("project: {}", config_file.display());
                println!("  files:   {:?}", tsconfig.files);
                println!("  include: {:?}", tsconfig.include);
                println!("  exclude: {:?}", tsconfig.exclude);
                ExitCode::SUCCESS
            }
            Err(message) => {
                eprintln!("{message}");
                ExitCode::FAILURE
            }
        },
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
/// the project-resolution block of tsgo's `tscCompilation`.
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

/// Search `dir` and its ancestors for a `tsconfig.json`, mirroring tsgo's `findConfigFile`
/// (`tspath.ForEachAncestorDirectory`).
fn find_config_file(dir: &Path) -> Option<PathBuf> {
    dir.ancestors()
        .map(|ancestor| ancestor.join("tsconfig.json"))
        .find(|candidate| candidate.is_file())
}

/// Turn `path` into an absolute path against `cwd`.
///
// NOTE: this is a lightweight stand-in for tsgo's `tspath.NormalizePath`; a faithful
// lexical normalization (collapsing `.`/`..` segments) is a later step.
fn normalize(path: &Path, cwd: &Path) -> PathBuf {
    let joined = cwd.join(path);
    std::path::absolute(&joined).unwrap_or(joined)
}
