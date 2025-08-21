use std::{env, ffi::OsStr, io::Write, path::PathBuf, sync::Arc};

use ignore::overrides::OverrideBuilder;

use crate::{
    cli::{CliRunResult, FormatCommand},
    walk::Walk,
};

#[derive(Debug)]
pub struct FormatRunner {
    options: FormatCommand,
    cwd: PathBuf,
}

impl FormatRunner {
    pub(crate) fn new(options: FormatCommand) -> Self {
        Self { options, cwd: env::current_dir().expect("Failed to get current working directory") }
    }

    pub(crate) fn run(self, stdout: &mut dyn Write) -> CliRunResult {
        let _cwd = self.cwd;
        let FormatCommand { paths, basic_options: _, misc_options: _ } = self.options;

        // Separate override patterns (starting with !) from regular paths
        let (override_patterns, regular_paths): (Vec<_>, Vec<_>) =
            paths.iter().partition(|path| path.to_string_lossy().starts_with('!'));

        println!("PATHS: {regular_paths:#?}");
        println!("OVERRIDE_PATTERNS: {override_patterns:#?}");

        // Need at least one regular path
        if regular_paths.is_empty() {
            print_and_flush_stdout(
                stdout,
                "Expected at least one target file/dir (non-override pattern)\n",
            );
            return CliRunResult::FormatNoFilesFound;
        }

        // Build override patterns if any exist
        let override_builder = if override_patterns.is_empty() {
            None
        } else {
            let first_regular_path = &regular_paths[0];
            let mut builder = OverrideBuilder::new(first_regular_path);
            for pattern in &override_patterns {
                let pattern = pattern.to_string_lossy();
                builder.add(&pattern).unwrap();
            }
            builder.build().ok()
        };

        let regular_paths: Vec<PathBuf> = regular_paths.into_iter().cloned().collect();
        let walker = Walk::new(&regular_paths, override_builder);
        let paths = walker.paths();

        let files_to_format = paths
            .into_iter()
            // .filter(|path| !config_store.should_ignore(Path::new(path)))
            .collect::<Vec<Arc<OsStr>>>();
        println!("TO_FMT: {files_to_format:#?}");

        // Spawn linting in another thread so diagnostics can be printed immediately from diagnostic_service.run.
        rayon::spawn(move || {
            // let mut lint_service = LintService::new(linter, options);
            // lint_service.with_paths(files_to_lint);

            // lint_service.run(&tx_error);
        });

        CliRunResult::FormatSucceeded
    }
}

pub fn print_and_flush_stdout(stdout: &mut dyn Write, message: &str) {
    use std::io::{Error, ErrorKind};
    fn check_for_writer_error(error: Error) -> Result<(), Error> {
        // Do not panic when the process is killed (e.g. piping into `less`).
        if matches!(error.kind(), ErrorKind::Interrupted | ErrorKind::BrokenPipe) {
            Ok(())
        } else {
            Err(error)
        }
    }

    stdout.write_all(message.as_bytes()).or_else(check_for_writer_error).unwrap();
    stdout.flush().unwrap();
}
