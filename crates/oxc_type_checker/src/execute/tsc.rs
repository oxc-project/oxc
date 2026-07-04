#![expect(clippy::print_stdout, clippy::print_stderr)]
//! Port of typescript-go's `internal/execute/tsc.go`.

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use crate::{
    compiler::{CompilerHost, Program},
    tsoptions::{TypeCheckCommand, get_file_names, parse_command_line, parse_config_file},
    tspath::to_path,
};

/// Run the type checker from the command line.
///
/// Mirrors tsgo's `execute.CommandLine`: parse the arguments, then drive compilation.
/// `bpaf` reads `std::env::args()` and handles `--help`/`--version`/argument errors itself.
pub fn command_line() -> ExitCode {
    let command = parse_command_line();
    tsc_compilation(&command)
}

/// Mirrors tsgo's `tscCompilation`: resolve the project into a config file (or root files),
/// parse and bind those files into a [`Program`], and report the collected files.
///
/// Type checking the program is a later step.
fn tsc_compilation(command: &TypeCheckCommand) -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(err) => {
            eprintln!("Unable to determine the current working directory: {err}");
            return ExitCode::FAILURE;
        }
    };

    let root_files = match resolve_config_file(command, &cwd) {
        // A resolved config file: parse it (resolving `extends`) via `oxc_resolver`, then expand
        // its file globs into the root file list.
        Ok(Some(config_file)) => match parse_config_file(&config_file) {
            Ok(tsconfig) => {
                println!("project: {}", config_file.display());
                get_file_names(&tsconfig)
            }
            Err(message) => {
                eprintln!("{message}");
                return ExitCode::FAILURE;
            }
        },
        // Source files given without a config file: use them directly as roots.
        Ok(None) => command.files.iter().map(|file| to_path(&cwd, file)).collect(),
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };

    // Parse and bind every root file into the program's file store.
    let program = Program::new(CompilerHost::new(cwd), &root_files);
    for source_file in program.source_files() {
        println!("  {}", source_file.file_name().display());
    }
    println!("({} files)", program.len());
    ExitCode::SUCCESS
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

        let file_or_directory = to_path(cwd, project);
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
