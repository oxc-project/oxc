use std::{env, path::PathBuf, sync::mpsc, time::Instant};

use oxc_diagnostics::DiagnosticService;

use super::{
    command::{FormatCommand, Mode, OutputMode},
    reporter::DefaultReporter,
    result::CliRunResult,
    service::{FormatService, SuccessResult},
    walk::Walk,
};
use crate::core::{
    ConfigResolver, SourceFormatter, resolve_editorconfig_path, resolve_oxfmtrc_path, utils,
};

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
    pub async fn run(self) -> CliRunResult {
        let start_time = Instant::now();

        let cwd = self.cwd;
        let FormatCommand { paths, mode, config_options, ignore_options, runtime_options } =
            self.options;
        // If `napi` feature is disabled, there is no other mode.
        #[cfg_attr(not(feature = "napi"), expect(irrefutable_let_patterns))]
        let Mode::Cli(format_mode) = mode else {
            unreachable!("`FormatRunner` should only be called with Mode::Cli");
        };
        let num_of_threads = rayon::current_num_threads();

        // Find and load config file
        // NOTE: Currently, we only load single config file.
        // - from `--config` if specified
        // - else, search nearest for the nearest `.oxfmtrc.json` from cwd upwards
        let oxfmtrc_path = resolve_oxfmtrc_path(&cwd, config_options.config.as_deref());
        let editorconfig_path = resolve_editorconfig_path(&cwd);
        let mut config_resolver = match ConfigResolver::from_config_paths(
            &cwd,
            oxfmtrc_path.as_deref(),
            editorconfig_path.as_deref(),
        ) {
            Ok(r) => r,
            Err(err) => {
                utils::print_stderr(&format!("Failed to load configuration file.\n{err}\n")).await;
                return CliRunResult::InvalidOptionConfig;
            }
        };
        let ignore_patterns = match config_resolver.build_and_validate() {
            Ok(patterns) => patterns,
            Err(err) => {
                utils::print_stderr(&format!("Failed to parse configuration.\n{err}\n")).await;
                return CliRunResult::InvalidOptionConfig;
            }
        };

        // Use `block_in_place()` to avoid nested async runtime access
        #[cfg(feature = "napi")]
        match tokio::task::block_in_place(|| {
            self.external_formatter
                .as_ref()
                .expect("External formatter must be set when `napi` feature is enabled")
                .init(num_of_threads)
        }) {
            // TODO: Plugins support
            // - Parse returned `languages`
            // - Allow its `extensions` and `filenames` in `walk.rs`
            // - Pass `parser` to `SourceFormatter`
            Ok(_) => {}
            Err(err) => {
                utils::print_stderr(&format!("Failed to setup external formatter.\n{err}\n")).await;
                return CliRunResult::InvalidOptionConfig;
            }
        }

        let walker = match Walk::build(
            &cwd,
            &paths,
            &ignore_options.ignore_path,
            ignore_options.with_node_modules,
            oxfmtrc_path.as_deref(),
            &ignore_patterns,
        ) {
            Ok(Some(walker)) => walker,
            // All target paths are ignored
            Ok(None) => {
                if runtime_options.no_error_on_unmatched_pattern {
                    utils::print_stderr("No files found matching the given patterns.\n").await;
                    return CliRunResult::None;
                }
                utils::print_stderr("Expected at least one target file\n").await;
                return CliRunResult::NoFilesFound;
            }
            Err(err) => {
                utils::print_stderr(&format!(
                    "Failed to parse target paths or ignore settings.\n{err}\n"
                ))
                .await;
                return CliRunResult::InvalidOptionConfig;
            }
        };

        if matches!(format_mode, OutputMode::Check) {
            utils::print_stdout("Checking formatting...\n\n").await;
        }

        // Run formatting in a sync block to avoid holding non-Send DiagnosticService across await points
        #[cfg(feature = "napi")]
        let external_formatter = self.external_formatter;

        let (changed_paths, unchanged_count, diagnostic_output, error_count) = {
            // Get the receiver for streaming entries
            let rx_entry = walker.stream_entries();
            // Collect format results (changed paths or unchanged count)
            let (tx_success, rx_success) = mpsc::channel();
            // Diagnostic from formatting service
            let (mut diagnostic_service, tx_error) =
                DiagnosticService::new(Box::new(DefaultReporter::default()));

            // Create `SourceFormatter` instance
            let source_formatter = SourceFormatter::new(num_of_threads);
            #[cfg(feature = "napi")]
            let source_formatter = source_formatter.with_external_formatter(external_formatter);

            let format_mode_clone = format_mode.clone();

            // Spawn a thread to run formatting service with streaming entries
            rayon::spawn(move || {
                let format_service =
                    FormatService::new(cwd, format_mode_clone, source_formatter, config_resolver);
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

            // Sort changed paths for deterministic output
            if !changed_paths.is_empty() {
                changed_paths.sort_unstable();
            }

            // Run diagnostics service and collect output (sync)
            let mut diagnostic_output = Vec::new();
            let diagnostics = diagnostic_service.run(&mut diagnostic_output);
            // NOTE: We are not using `DiagnosticService` for warnings
            let error_count = diagnostics.errors_count();

            (changed_paths, unchanged_count, diagnostic_output, error_count)
        };

        // Print sorted changed file paths to stdout
        if !changed_paths.is_empty() {
            utils::print_stdout(&changed_paths.join("\n")).await;
        }

        // Then, output diagnostics errors to stderr
        if !diagnostic_output.is_empty() {
            utils::write_stderr(&diagnostic_output).await;
        }

        // Count the processed files
        let total_target_files_count = changed_paths.len() + unchanged_count + error_count;
        let print_stats = || async {
            let elapsed_ms = start_time.elapsed().as_millis();
            utils::print_stdout(&format!(
                "Finished in {elapsed_ms}ms on {total_target_files_count} files using {num_of_threads} threads.\n",
            )).await;
        };

        // Check if no files were found
        if total_target_files_count == 0 {
            if runtime_options.no_error_on_unmatched_pattern {
                utils::print_stderr("No files found matching the given patterns.\n").await;
                print_stats().await;
                return CliRunResult::None;
            }

            utils::print_stderr("Expected at least one target file\n").await;
            return CliRunResult::NoFilesFound;
        }

        if 0 < error_count {
            // Each error is already printed in reporter
            utils::print_stderr("Error occurred when checking code style in the above files.\n")
                .await;
            return CliRunResult::FormatFailed;
        }

        match (&format_mode, changed_paths.len()) {
            // `--list-different` outputs nothing here, mismatched paths are already printed to stdout
            (OutputMode::ListDifferent, 0) => CliRunResult::FormatSucceeded,
            (OutputMode::ListDifferent, _) => CliRunResult::FormatMismatch,
            // `--check` outputs friendly summary
            (OutputMode::Check, 0) => {
                utils::print_stdout("All matched files use the correct format.\n").await;
                print_stats().await;
                CliRunResult::FormatSucceeded
            }
            (OutputMode::Check, changed_count) => {
                utils::print_stdout(&format!("\n\nFormat issues found in above {changed_count} files. Run without `--check` to fix.\n")).await;
                print_stats().await;
                CliRunResult::FormatMismatch
            }
            // Default (write) does not output anything
            (OutputMode::Write, changed_count) => {
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
