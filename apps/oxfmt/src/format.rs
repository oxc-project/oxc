use std::{env, io::Write, path::PathBuf, sync::mpsc, time::Instant};

use ignore::overrides::OverrideBuilder;

use oxc_diagnostics::DiagnosticService;

use crate::{
    cli::{CliRunResult, FormatCommand},
    command::OutputOptions,
    reporter::DefaultReporter,
    service::FormatService,
    walk::Walk,
};

#[derive(Debug)]
pub struct FormatRunner {
    options: FormatCommand,
    cwd: PathBuf,
}

impl FormatRunner {
    /// Creates a new FormatRunner instance.
    ///
    /// # Panics
    /// Panics if the current working directory cannot be determined.
    pub fn new(options: FormatCommand) -> Self {
        Self { options, cwd: env::current_dir().expect("Failed to get current working directory") }
    }

    #[must_use]
    pub fn with_cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd = cwd;
        self
    }

    pub fn run(self, stdout: &mut dyn Write) -> CliRunResult {
        let start_time = Instant::now();

        let cwd = self.cwd;
        let FormatCommand { paths, output_options, misc_options } = self.options;

        // Default to current working directory if no paths are provided
        let paths = if paths.is_empty() { vec![cwd.clone()] } else { paths };

        // Instead of `--ignore-pattern=PAT`, we support `!` prefix in paths
        let (exclude_patterns, target_paths): (Vec<_>, Vec<_>) =
            paths.into_iter().partition(|p| p.to_string_lossy().starts_with('!'));

        // Resolve relative paths against the current working directory
        let target_paths: Vec<PathBuf> = target_paths
            .into_iter()
            .map(|path| if path.is_relative() { cwd.join(path) } else { path })
            .collect();

        // Build exclude patterns if any exist
        let override_builder = (!exclude_patterns.is_empty())
            .then(|| {
                let mut builder = OverrideBuilder::new(&cwd);
                for pattern in exclude_patterns {
                    builder.add(&pattern.to_string_lossy()).ok()?;
                }
                builder.build().ok()
            })
            .flatten();

        // TODO: Support ignoring files
        let walker = Walk::new(&target_paths, override_builder);

        // Get the receiver for streaming entries
        let rx_entry = walker.stream_entries();

        // Count files for stats
        let (tx_count, rx_count) = mpsc::channel::<()>();

        let (mut diagnostic_service, tx_error) =
            DiagnosticService::new(Box::new(DefaultReporter::default()));

        if matches!(output_options, OutputOptions::Check) {
            print_and_flush_stdout(stdout, "Checking formatting...\n");
        }

        let output_options_clone = output_options.clone();
        // Spawn a thread to run formatting service with streaming entries
        rayon::spawn(move || {
            let format_service = FormatService::new(cwd, output_options_clone);
            format_service.run_streaming(rx_entry, &tx_error, tx_count);
        });

        // NOTE: This is blocking - waits for all diagnostics
        let res = diagnostic_service.run(stdout);

        // Count the processed files
        let target_files_count = rx_count.iter().count();
        let print_stats = |stdout| {
            print_and_flush_stdout(
                stdout,
                &format!(
                    "Finished in {}ms on {target_files_count} files using {} threads.\n",
                    start_time.elapsed().as_millis(),
                    rayon::current_num_threads()
                ),
            );
        };

        // Add a new line between diagnostics and summary
        print_and_flush_stdout(stdout, "\n");

        // Check if no files were found
        if target_files_count == 0 {
            if misc_options.no_error_on_unmatched_pattern {
                print_and_flush_stdout(stdout, "No files found matching the given patterns.\n");
                print_stats(stdout);
                return CliRunResult::None;
            }

            print_and_flush_stdout(stdout, "Expected at least one target file\n");
            return CliRunResult::NoFilesFound;
        }

        if 0 < res.errors_count() {
            // Each error is already printed in reporter
            print_and_flush_stdout(
                stdout,
                "Error occurred when checking code style in the above files.\n",
            );
            return CliRunResult::FormatFailed;
        }

        match (&output_options, res.warnings_count()) {
            // `--list-different` outputs nothing here, mismatched paths are already printed in reporter
            (OutputOptions::ListDifferent, 0) => CliRunResult::FormatSucceeded,
            (OutputOptions::ListDifferent, _) => CliRunResult::FormatMismatch,
            // `--check` outputs friendly summary
            (OutputOptions::Check, 0) => {
                print_and_flush_stdout(stdout, "All matched files use the correct format.\n");
                print_stats(stdout);
                CliRunResult::FormatSucceeded
            }
            (OutputOptions::Check, mismatched_count) => {
                print_and_flush_stdout(
                    stdout,
                    &format!(
                        "Format issues found in above {mismatched_count} files. Run without `--check` to fix.\n",
                    ),
                );
                print_stats(stdout);
                CliRunResult::FormatMismatch
            }
            // Default (write) also outputs friendly summary
            (OutputOptions::DefaultWrite, formatted_count) => {
                print_and_flush_stdout(stdout, &format!("Formatted {formatted_count} files.\n"));
                print_stats(stdout);
                CliRunResult::FormatSucceeded
            }
        }
    }
}

fn print_and_flush_stdout(stdout: &mut dyn Write, message: &str) {
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
