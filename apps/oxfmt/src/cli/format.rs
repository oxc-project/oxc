use std::{
    env,
    io::Write,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Instant,
};

use serde_json::Value;

use oxc_diagnostics::DiagnosticService;
use oxc_formatter::{FormatOptions, OxfmtOptions, Oxfmtrc};

use super::{
    command::{FormatCommand, OutputOptions},
    reporter::DefaultReporter,
    result::CliRunResult,
    service::{FormatService, SuccessResult},
    walk::Walk,
};
use crate::core::{SourceFormatter, utils};

#[derive(Debug)]
pub struct FormatRunner {
    options: FormatCommand,
    cwd: PathBuf,
    #[cfg(feature = "napi")]
    external_formatter: Option<crate::core::ExternalFormatter>,
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
        external_formatter: Option<crate::core::ExternalFormatter>,
    ) -> Self {
        self.external_formatter = external_formatter;
        self
    }

    /// # Panics
    /// Panics if `napi` feature is enabled but external_formatter is not set.
    pub fn run(self, stdout: &mut dyn Write, stderr: &mut dyn Write) -> CliRunResult {
        let start_time = Instant::now();

        let cwd = self.cwd;
        let FormatCommand { paths, output_options, basic_options, ignore_options, misc_options } =
            self.options;
        let num_of_threads = rayon::current_num_threads();

        // Find config file
        // NOTE: Currently, we only load single config file.
        // - from `--config` if specified
        // - else, search nearest for the nearest `.oxfmtrc.json` from cwd upwards
        let config_path = load_config_path(&cwd, basic_options.config.as_deref());
        // Load and parse config file
        // - `format_options`: Parsed formatting options used by `oxc_formatter`
        // - `external_config`: JSON value used by `external_formatter`, populated with `format_options`
        let (format_options, oxfmt_options, external_config) =
            match load_config(config_path.as_deref()) {
                Ok(c) => c,
                Err(err) => {
                    print_and_flush(
                        stderr,
                        &format!("Failed to load configuration file.\n{err}\n"),
                    );
                    return CliRunResult::InvalidOptionConfig;
                }
            };

        // TODO: Plugins support
        // - Parse returned `languages`
        // - Allow its `extensions` and `filenames` in `walk.rs`
        // - Pass `parser` to `SourceFormatter`
        #[cfg(feature = "napi")]
        if let Err(err) = self
            .external_formatter
            .as_ref()
            .expect("External formatter must be set when `napi` feature is enabled")
            .setup_config(&external_config.to_string(), num_of_threads)
        {
            print_and_flush(
                stderr,
                &format!("Failed to setup external formatter config.\n{err}\n"),
            );
            return CliRunResult::InvalidOptionConfig;
        }

        #[cfg(not(feature = "napi"))]
        let _ = (external_config, oxfmt_options.sort_package_json);

        let walker = match Walk::build(
            &cwd,
            &paths,
            &ignore_options.ignore_path,
            ignore_options.with_node_modules,
            &oxfmt_options.ignore_patterns,
        ) {
            Ok(Some(walker)) => walker,
            // All target paths are ignored
            Ok(None) => {
                if misc_options.no_error_on_unmatched_pattern {
                    print_and_flush(stderr, "No files found matching the given patterns.\n");
                    return CliRunResult::None;
                }
                print_and_flush(stderr, "Expected at least one target file\n");
                return CliRunResult::NoFilesFound;
            }
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
        // Collect format results (changed paths or unchanged count)
        let (tx_success, rx_success) = mpsc::channel();
        // Diagnostic from formatting service
        let (mut diagnostic_service, tx_error) =
            DiagnosticService::new(Box::new(DefaultReporter::default()));

        if matches!(output_options, OutputOptions::Check) {
            print_and_flush(stdout, "Checking formatting...\n");
            print_and_flush(stdout, "\n");
        }

        // Create `SourceFormatter` instance
        let source_formatter = SourceFormatter::new(num_of_threads, format_options);
        #[cfg(feature = "napi")]
        let source_formatter = source_formatter
            .with_external_formatter(self.external_formatter, oxfmt_options.sort_package_json);

        let output_options_clone = output_options.clone();

        // Spawn a thread to run formatting service with streaming entries
        rayon::spawn(move || {
            let format_service = FormatService::new(cwd, output_options_clone, source_formatter);
            format_service.run_streaming(rx_entry, &tx_error, &tx_success);
        });

        // Collect results and separate changed paths from unchanged count
        let mut changed_paths: Vec<String> = vec![];
        let mut unchanged_count: usize = 0;
        for result in rx_success {
            match result {
                SuccessResult::Changed(path) => changed_paths.push(path),
                SuccessResult::Unchanged => unchanged_count += 1,
            }
        }

        // Print sorted changed file paths to stdout
        if !changed_paths.is_empty() {
            changed_paths.sort_unstable();
            print_and_flush(stdout, &changed_paths.join("\n"));
        }

        // Then, output diagnostics errors to stderr
        // NOTE: This is blocking and print errors
        let diagnostics = diagnostic_service.run(stderr);
        // NOTE: We are not using `DiagnosticService` for warnings
        let error_count = diagnostics.errors_count();

        // Count the processed files
        let total_target_files_count = changed_paths.len() + unchanged_count + error_count;
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

        if 0 < error_count {
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

// ---

/// Resolve config file path from cwd and optional explicit path.
fn load_config_path(cwd: &Path, config_path: Option<&Path>) -> Option<PathBuf> {
    // If `--config` is explicitly specified, use that path
    if let Some(config_path) = config_path {
        return Some(if config_path.is_absolute() {
            config_path.to_path_buf()
        } else {
            cwd.join(config_path)
        });
    }

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
}

/// # Errors
/// Returns error if:
/// - Config file is specified but not found or invalid
/// - Config file parsing fails
fn load_config(config_path: Option<&Path>) -> Result<(FormatOptions, OxfmtOptions, Value), String> {
    // Read and parse config file, or use empty JSON if not found
    let json_string = match config_path {
        Some(path) => {
            let mut json_string = utils::read_to_string(path)
                // Do not include OS error, it differs between platforms
                .map_err(|_| format!("Failed to read config {}: File not found", path.display()))?;
            // Strip comments (JSONC support)
            json_strip_comments::strip(&mut json_string).map_err(|err| {
                format!("Failed to strip comments from {}: {err}", path.display())
            })?;
            json_string
        }
        None => "{}".to_string(),
    };

    // Parse as raw JSON value (to pass to external formatter)
    let mut raw_config: Value = serde_json::from_str(&json_string)
        .map_err(|err| format!("Failed to parse config: {err}"))?;

    // NOTE: Field validation for `enum` are done here
    let oxfmtrc: Oxfmtrc = serde_json::from_str(&json_string)
        .map_err(|err| format!("Failed to deserialize config: {err}"))?;

    // NOTE: Other validation based on it's field values are done here
    let (format_options, oxfmt_options) =
        oxfmtrc.into_options().map_err(|err| format!("Failed to parse configuration.\n{err}"))?;

    // Populate `raw_config` with resolved options to apply our defaults
    Oxfmtrc::populate_prettier_config(&format_options, &mut raw_config);

    Ok((format_options, oxfmt_options, raw_config))
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
