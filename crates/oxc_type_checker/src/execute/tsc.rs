#![expect(clippy::print_stdout, clippy::print_stderr)]
//! Port of typescript-go's `internal/execute/tsc.go`.

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result, bail};

use crate::{
    compiler::Program,
    tsoptions::{TypeCheckCommand, get_file_names, parse_command_line, parse_config_file},
    tspath::to_path,
};

/// Run the type checker from the command line.
///
/// Mirrors tsgo's `execute.CommandLine`: parse the arguments, then drive compilation.
/// `bpaf` reads `std::env::args()` and handles `--help`/`--version`/argument errors itself.
pub fn command_line() -> ExitCode {
    let command = parse_command_line();
    match tsc_compilation(&command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // `{:#}` prints the whole error chain (the message plus any source), matching tsc's
            // single-line error output.
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}

/// Mirrors tsgo's `tscCompilation`: resolve the project into a config file (or root files),
/// collect those files into a [`Program`], and report them.
///
/// Parsing and type checking the program are later steps.
fn tsc_compilation(command: &TypeCheckCommand) -> Result<()> {
    let cwd =
        std::env::current_dir().context("Unable to determine the current working directory")?;

    let root_files = match resolve_config_file(command, &cwd)? {
        // A resolved config file: parse it (resolving `extends`) via `oxc_resolver`, then expand
        // its file globs into the root file list.
        Some(config_file) => {
            let tsconfig = parse_config_file(&config_file)?;
            println!("project: {}", config_file.display());
            get_file_names(&tsconfig)
        }
        // Source files given without a config file: use them directly as roots.
        None => command.files.clone(),
    };

    // Collect the root files (normalized + deduplicated) into the program's file list.
    let program = Program::new(&cwd, &root_files);
    for file in program.files() {
        println!("  {}", file.display());
    }
    println!("({} files)", program.len());
    Ok(())
}

/// Resolve the command line into the `tsconfig.json` (or config file) to load, mirroring
/// the project-resolution block of tsgo's `tscCompilation`.
///
/// Returns:
/// - `Ok(Some(path))` — a resolved config file to load,
/// - `Ok(None)` — source files were given with no config file (compile them directly),
/// - `Err(_)` — one of `tsc`'s command-line errors, ready to print.
fn resolve_config_file(command: &TypeCheckCommand, cwd: &Path) -> Result<Option<PathBuf>> {
    if let Some(project) = &command.project {
        // TS5042
        if !command.files.is_empty() {
            bail!("Option 'project' cannot be mixed with source files on a command line.");
        }

        let file_or_directory = to_path(cwd, project);
        if file_or_directory.is_dir() {
            // A directory: look for `tsconfig.json` inside it.
            let config_file = file_or_directory.join("tsconfig.json");
            if config_file.is_file() {
                Ok(Some(config_file))
            } else {
                // TS5081
                bail!(
                    "Cannot find a tsconfig.json file at the current directory: {}.",
                    config_file.display()
                );
            }
        } else if file_or_directory.exists() {
            // An explicit config file (need not be named `tsconfig.json`).
            Ok(Some(file_or_directory))
        } else {
            // TS5058
            bail!("The specified path does not exist: '{}'.", file_or_directory.display());
        }
    } else if let Some(config_file) = find_config_file(cwd) {
        if command.files.is_empty() {
            Ok(Some(config_file))
        } else {
            // TS5112: a tsconfig.json is present but source files were also specified.
            bail!(
                "tsconfig.json is present but will not be loaded if files are specified on \
                 commandline. Use '--ignoreConfig' to skip this error."
            );
        }
    } else if command.files.is_empty() {
        // TS5081 — `tsc` prints version + help here; we report the missing config instead.
        bail!("Cannot find a tsconfig.json file at the current directory: {}.", cwd.display());
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
