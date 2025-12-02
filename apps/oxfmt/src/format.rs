use std::{
    env,
    io::Write,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Instant,
};

use oxc_allocator::AllocatorPool;
use oxc_diagnostics::DiagnosticService;
use oxc_formatter::Oxfmtrc;

use crate::{
    command::{FormatCommand, OutputOptions},
    reporter::DefaultReporter,
    result::CliRunResult,
    service::FormatService,
    walk::Walk,
};

#[derive(Debug)]
pub struct FormatRunner {
    options: FormatCommand,
    cwd: PathBuf,
    #[cfg(feature = "napi")]
    external_formatter: Option<crate::prettier_plugins::ExternalFormatter>,
}

impl FormatRunner {
    /// Creates a new FormatRunner instance.
    ///
    /// # Panics
    /// Panics if the current working directory cannot be determined.
    pub fn new(options: FormatCommand) -> Self {
        Self {
            options,
            cwd: env::current_dir().expect("Failed to get current working directory"),
            #[cfg(feature = "napi")]
            external_formatter: None,
        }
    }

    #[cfg(feature = "napi")]
    #[must_use]
    pub fn with_external_formatter(
        mut self,
        external_formatter: Option<crate::prettier_plugins::ExternalFormatter>,
    ) -> Self {
        self.external_formatter = external_formatter;
        self
    }

    pub fn run(self, stdout: &mut dyn Write, stderr: &mut dyn Write) -> CliRunResult {
        let start_time = Instant::now();

        let cwd = self.cwd;
        let FormatCommand { paths, output_options, basic_options, ignore_options, misc_options } =
            self.options;

        // Find and load config
        // NOTE: Currently, we only load single config file.
        // - from `--config` if specified
        // - else, search nearest for the nearest `.oxfmtrc.json` from cwd upwards
        let config = match load_config(&cwd, basic_options.config.as_ref()) {
            Ok(config) => config,
            Err(err) => {
                print_and_flush(stderr, &format!("Failed to load configuration file.\n{err}\n"));
                return CliRunResult::InvalidOptionConfig;
            }
        };

        let ignore_patterns = config.ignore_patterns.clone().unwrap_or_default();
        let format_options = match config.into_format_options() {
            Ok(options) => options,
            Err(err) => {
                print_and_flush(stderr, &format!("Failed to parse configuration.\n{err}\n"));
                return CliRunResult::InvalidOptionConfig;
            }
        };

        let walker = match Walk::build(
            &cwd,
            &paths,
            &ignore_options.ignore_path,
            ignore_options.with_node_modules,
            &ignore_patterns,
        ) {
            Ok(walker) => walker,
            Err(err) => {
                print_and_flush(
                    stderr,
                    &format!("Failed to parse target paths or ignore settings.\n{err}\n"),
                );
                return CliRunResult::InvalidOptionConfig;
            }
        };

        // Get the receiver for streaming entries
        let rx_entry = walker.stream_entries();
        // Count all files for stats
        let (tx_count, rx_count) = mpsc::channel::<()>();
        // Collect file paths that were updated
        let (tx_path, rx_path) = mpsc::channel();
        // Diagnostic from formatting service
        let (mut diagnostic_service, tx_error) =
            DiagnosticService::new(Box::new(DefaultReporter::default()));

        if matches!(output_options, OutputOptions::Check) {
            print_and_flush(stdout, "Checking formatting...\n");
            print_and_flush(stdout, "\n");
        }

        let num_of_threads = rayon::current_num_threads();
        // Create allocator pool for reuse across parallel formatting tasks
        let allocator_pool = AllocatorPool::new(num_of_threads);

        let output_options_clone = output_options.clone();
        #[cfg(feature = "napi")]
        let external_formatter_clone = self.external_formatter;

        // Spawn a thread to run formatting service with streaming entries
        rayon::spawn(move || {
            let format_service =
                FormatService::new(allocator_pool, cwd, output_options_clone, format_options);
            #[cfg(feature = "napi")]
            let format_service = format_service.with_external_formatter(external_formatter_clone);

            format_service.run_streaming(rx_entry, &tx_error, &tx_path, tx_count);
        });

        // First, collect and print sorted file paths to stdout
        let mut changed_paths: Vec<String> = rx_path.iter().collect();
        if !changed_paths.is_empty() {
            changed_paths.sort_unstable();
            print_and_flush(stdout, &changed_paths.join("\n"));
        }

        // Then, output diagnostics errors to stderr
        // NOTE: This is blocking and print errors
        let diagnostics = diagnostic_service.run(stderr);

        // Count the processed files
        let total_target_files_count = rx_count.iter().count();
        let print_stats = |stdout| {
            let elapsed_ms = start_time.elapsed().as_millis();
            print_and_flush(
                stdout,
                &format!(
                    "Finished in {elapsed_ms}ms on {total_target_files_count} files using {num_of_threads} threads.\n",
                ),
            );
        };

        // Check if no files were found
        if total_target_files_count == 0 {
            if misc_options.no_error_on_unmatched_pattern {
                print_and_flush(stderr, "No files found matching the given patterns.\n");
                print_stats(stdout);
                return CliRunResult::None;
            }

            print_and_flush(stderr, "Expected at least one target file\n");
            return CliRunResult::NoFilesFound;
        }

        if 0 < diagnostics.errors_count() {
            // Each error is already printed in reporter
            print_and_flush(
                stderr,
                "Error occurred when checking code style in the above files.\n",
            );
            return CliRunResult::FormatFailed;
        }

        match (&output_options, changed_paths.len()) {
            // `--list-different` outputs nothing here, mismatched paths are already printed to stdout
            (OutputOptions::ListDifferent, 0) => CliRunResult::FormatSucceeded,
            (OutputOptions::ListDifferent, _) => CliRunResult::FormatMismatch,
            // `--check` outputs friendly summary
            (OutputOptions::Check, 0) => {
                print_and_flush(stdout, "All matched files use the correct format.\n");
                print_stats(stdout);
                CliRunResult::FormatSucceeded
            }
            (OutputOptions::Check, changed_count) => {
                print_and_flush(stdout, "\n\n");
                print_and_flush(
                    stdout,
                    &format!(
                        "Format issues found in above {changed_count} files. Run without `--check` to fix.\n",
                    ),
                );
                print_stats(stdout);
                CliRunResult::FormatMismatch
            }
            // Default (write) does not output anything
            (OutputOptions::Write, changed_count) => {
                // Each changed file is also NOT printed
                debug_assert_eq!(
                    changed_count, 0,
                    "In write mode, changed_count should not be counted"
                );
                CliRunResult::FormatSucceeded
            }
        }
    }
}

/// # Errors
///
/// Returns error if:
/// - Config file is specified but not found or invalid
/// - Config file parsing fails
fn load_config(cwd: &Path, config_path: Option<&PathBuf>) -> Result<Oxfmtrc, String> {
    let config_path = if let Some(config_path) = config_path {
        // If `--config` is explicitly specified, use that path
        Some(if config_path.is_absolute() {
            PathBuf::from(config_path)
        } else {
            cwd.join(config_path)
        })
    } else {
        // If `--config` is not specified, search the nearest config file from cwd upwards
        // Support both `.json` and `.jsonc`, but prefer `.json` if both exist
        cwd.ancestors().find_map(|dir| {
            for filename in [".oxfmtrc.json", ".oxfmtrc.jsonc"] {
                let config_path = dir.join(filename);
                if config_path.exists() {
                    return Some(config_path);
                }
            }
            None
        })
    };

    match config_path {
        Some(ref path) => Oxfmtrc::from_file(path),
        // Default if not specified and not found
        None => Ok(Oxfmtrc::default()),
    }
}

fn print_and_flush(writer: &mut dyn Write, message: &str) {
    use std::io::{Error, ErrorKind};
    fn check_for_writer_error(error: Error) -> Result<(), Error> {
        // Do not panic when the process is killed (e.g. piping into `less`).
        if matches!(error.kind(), ErrorKind::Interrupted | ErrorKind::BrokenPipe) {
            Ok(())
        } else {
            Err(error)
        }
    }

    writer.write_all(message.as_bytes()).or_else(check_for_writer_error).unwrap();
    writer.flush().unwrap();
}
