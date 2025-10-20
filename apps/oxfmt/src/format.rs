use std::{
    env,
    io::Write,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Instant,
};

use ignore::overrides::OverrideBuilder;

use oxc_diagnostics::DiagnosticService;
use oxc_formatter::{FormatOptions, Oxfmtrc};

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
        let FormatCommand { paths, output_options, basic_options, ignore_options, misc_options } =
            self.options;

        // Find and load config
        // NOTE: Currently, we only load single config file.
        // - from `--config` if specified
        // - else, search nearest for the nearest `.oxfmtrc.json` from cwd upwards
        let format_options = match load_config(&cwd, basic_options.config.as_ref()) {
            Ok(options) => options,
            Err(err) => {
                print_and_flush_stdout(
                    stdout,
                    &format!("Failed to load configuration file.\n{err}\n"),
                );
                return CliRunResult::InvalidOptionConfig;
            }
        };

        // Normalize user input paths
        let (target_paths, exclude_patterns) = normalize_paths(&cwd, &paths);

        // Build exclude patterns if any exist
        let override_builder = (!exclude_patterns.is_empty())
            .then(|| {
                let mut builder = OverrideBuilder::new(&cwd);
                for pattern_str in exclude_patterns {
                    builder.add(&pattern_str).ok()?;
                }
                builder.build().ok()
            })
            .flatten();

        // TODO: Support ignoring files
        let walker = Walk::new(&target_paths, override_builder, ignore_options.with_node_modules);

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
            let format_service = FormatService::new(cwd, output_options_clone, format_options);
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

const DEFAULT_OXFMTRC: &str = ".oxfmtrc.json";

/// # Errors
///
/// Returns error if:
/// - Config file is specified but not found or invalid
/// - Config file parsing fails
fn load_config(cwd: &Path, config: Option<&PathBuf>) -> Result<FormatOptions, String> {
    // If `--config` is explicitly specified, use that path directly
    if let Some(config_path) = config {
        let full_path = if config_path.is_absolute() {
            PathBuf::from(config_path)
        } else {
            cwd.join(config_path)
        };

        // This will error if the file does not exist or is invalid
        let oxfmtrc = Oxfmtrc::from_file(&full_path)?;
        return oxfmtrc.into_format_options();
    }

    // If `--config` is not specified, search the nearest config file from cwd upwards
    for dir in cwd.ancestors() {
        let config_path = dir.join(DEFAULT_OXFMTRC);
        if config_path.exists() {
            let oxfmtrc = Oxfmtrc::from_file(&config_path)?;
            return oxfmtrc.into_format_options();
        }
    }

    // No config file found, use defaults
    Ok(FormatOptions::default())
}

/// Normalize user input paths into `target_paths` and `exclude_patterns`.
/// - `target_paths`: Absolute paths to format
/// - `exclude_patterns`: Pattern strings to exclude (with `!` prefix)
fn normalize_paths(cwd: &Path, input_paths: &[PathBuf]) -> (Vec<PathBuf>, Vec<String>) {
    let mut target_paths = vec![];
    let mut exclude_patterns = vec![];

    for path in input_paths {
        let path_str = path.to_string_lossy();

        // Instead of `oxlint`'s `--ignore-pattern=PAT`,
        // `oxfmt` supports `!` prefix in paths like Prettier.
        if path_str.starts_with('!') {
            exclude_patterns.push(path_str.to_string());
            continue;
        }

        // Otherwise, treat as target path

        if path.is_absolute() {
            target_paths.push(path.clone());
            continue;
        }

        // NOTE: `.` and cwd behaves differently, need to normalize
        let path = if path_str == "." {
            cwd.to_path_buf()
        } else if let Some(stripped) = path_str.strip_prefix("./") {
            cwd.join(stripped)
        } else {
            cwd.join(path)
        };
        target_paths.push(path);
    }

    // Default to cwd if no `target_paths` are provided
    if target_paths.is_empty() {
        target_paths.push(cwd.into());
    }

    (target_paths, exclude_patterns)
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
