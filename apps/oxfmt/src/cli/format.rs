use std::{env, io::BufWriter, path::PathBuf, sync::mpsc, time::Instant};

use oxc_diagnostics::DiagnosticService;

use super::{
    command::{FormatCommand, Mode, OutputMode},
    reporter::DefaultReporter,
    result::CliRunResult,
    service::{FormatService, SuccessResult},
    walk::Walk,
};
#[cfg(feature = "napi")]
use crate::core::JsConfigLoaderCb;
use crate::core::{ConfigResolver, SourceFormatter, resolve_editorconfig_path, utils};

pub struct CliRunner {
    options: FormatCommand,
    cwd: PathBuf,
    #[cfg(feature = "napi")]
    external_formatter: Option<crate::core::ExternalFormatter>,
    #[cfg(feature = "napi")]
    js_config_loader: Option<JsConfigLoaderCb>,
}

impl CliRunner {
    /// Creates a new CliRunner instance.
    ///
    /// # Panics
    /// Panics if the current working directory cannot be determined.
    pub fn new(options: FormatCommand) -> Self {
        Self {
            options,
            cwd: env::current_dir().expect("Failed to get current working directory"),
            #[cfg(feature = "napi")]
            external_formatter: None,
            #[cfg(feature = "napi")]
            js_config_loader: None,
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

    #[cfg(feature = "napi")]
    #[must_use]
    pub fn with_js_config_loader(mut self, js_config_loader: JsConfigLoaderCb) -> Self {
        self.js_config_loader = Some(js_config_loader);
        self
    }

    /// # Panics
    /// Panics if `napi` feature is enabled but external_formatter is not set.
    pub fn run(self) -> CliRunResult {
        // stdio is blocked by `LineWriter`, use a `BufWriter` to reduce syscalls.
        // See https://github.com/rust-lang/rust/issues/60673
        let stdout = &mut BufWriter::new(std::io::stdout());
        let stderr = &mut BufWriter::new(std::io::stderr());

        let start_time = Instant::now();

        let cwd = self.cwd;
        let FormatCommand { paths, mode, config_options, ignore_options, runtime_options } =
            self.options;
        // If `napi` feature is disabled, there is no other mode.
        #[cfg_attr(not(feature = "napi"), expect(irrefutable_let_patterns))]
        let Mode::Cli(format_mode) = mode else {
            unreachable!("`CliRunner` should only be called with Mode::Cli");
        };
        let num_of_threads = rayon::current_num_threads();

        // Find and load config file
        // NOTE: Currently, we only load single config file.
        // - from `--config` if specified
        // - else, search nearest config file from cwd upwards
        let editorconfig_path = resolve_editorconfig_path(&cwd);
        let mut config_resolver = match ConfigResolver::from_config(
            &cwd,
            config_options.config.as_deref(),
            editorconfig_path.as_deref(),
            #[cfg(feature = "napi")]
            self.js_config_loader.as_ref(),
        ) {
            Ok(r) => r,
            Err(err) => {
                utils::print_and_flush(
                    stderr,
                    &format!("Failed to load configuration file.\n{err}\n"),
                );
                return CliRunResult::InvalidOptionConfig;
            }
        };
        let ignore_patterns = match config_resolver.build_and_validate() {
            Ok(patterns) => patterns,
            Err(err) => {
                utils::print_and_flush(stderr, &format!("Failed to parse configuration.\n{err}\n"));
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
                utils::print_and_flush(
                    stderr,
                    &format!("Failed to setup external formatter.\n{err}\n"),
                );
                return CliRunResult::InvalidOptionConfig;
            }
        }

        let walker = match Walk::build(
            &cwd,
            &paths,
            &ignore_options.ignore_path,
            ignore_options.with_node_modules,
            config_resolver.config_dir(),
            &ignore_patterns,
        ) {
            Ok(walker) => walker,
            Err(err) => {
                utils::print_and_flush(
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

        if matches!(format_mode, OutputMode::Check) {
            utils::print_and_flush(stdout, "Checking formatting...\n");
            utils::print_and_flush(stdout, "\n");
        }

        // Create `SourceFormatter` instance
        let source_formatter = SourceFormatter::new(num_of_threads);
        #[cfg(feature = "napi")]
        let source_formatter = source_formatter.with_external_formatter(self.external_formatter);

        let no_config = config_resolver.config_dir().is_none() && editorconfig_path.is_none();

        // Spawn a thread to run formatting service with streaming entries
        rayon::spawn(move || {
            let format_service =
                FormatService::new(cwd, format_mode, source_formatter, config_resolver);
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
            utils::print_and_flush(stdout, &changed_paths.join("\n"));
        }

        // Then, output diagnostics errors to stderr
        // NOTE: This is blocking and print errors
        let diagnostics = diagnostic_service.run(stderr);
        // NOTE: We are not using `DiagnosticService` for warnings
        let error_count = diagnostics.errors_count();

        // Count the processed files
        let total_target_files_count = changed_paths.len() + unchanged_count + error_count;
        let print_stats = |stdout, stderr| {
            utils::print_and_flush(
                stdout,
                &format!(
                    "Finished in {}ms on {total_target_files_count} files using {num_of_threads} threads.\n",
                    start_time.elapsed().as_millis()
                ),
            );
            // Config stats: only show when no config is found
            if no_config {
                #[cfg(feature = "napi")]
                let hint = "No config found, using defaults. Please add a config file or try `oxfmt --init` if needed.\n";
                #[cfg(not(feature = "napi"))]
                let hint =
                    "No config found, using defaults. Please add `.oxfmtrc.json` if needed.\n";
                utils::print_and_flush(stderr, hint);
            }
        };

        // Check if no files were found
        if total_target_files_count == 0 {
            if runtime_options.no_error_on_unmatched_pattern {
                utils::print_and_flush(stderr, "No files found matching the given patterns.\n");
                print_stats(stdout, stderr);
                return CliRunResult::None;
            }

            utils::print_and_flush(
                stderr,
                "Expected at least one target file. All matched files may have been excluded by ignore rules.\n",
            );
            return CliRunResult::NoFilesFound;
        }

        if 0 < error_count {
            // Each error is already printed in reporter
            utils::print_and_flush(
                stderr,
                "Error occurred when checking code style in the above files.\n",
            );
            return CliRunResult::FormatFailed;
        }

        match (&format_mode, changed_paths.len()) {
            // `--list-different` outputs nothing here, mismatched paths are already printed to stdout
            (OutputMode::ListDifferent, 0) => CliRunResult::FormatSucceeded,
            (OutputMode::ListDifferent, _) => CliRunResult::FormatMismatch,
            // `--check` outputs friendly summary
            (OutputMode::Check, 0) => {
                utils::print_and_flush(stdout, "All matched files use the correct format.\n");
                print_stats(stdout, stderr);
                CliRunResult::FormatSucceeded
            }
            (OutputMode::Check, changed_count) => {
                utils::print_and_flush(stdout, "\n\n");
                utils::print_and_flush(
                    stdout,
                    &format!(
                        "Format issues found in above {changed_count} files. Run without `--check` to fix.\n",
                    ),
                );
                print_stats(stdout, stderr);
                CliRunResult::FormatMismatch
            }
            // Default (write) outputs only stats
            (OutputMode::Write, changed_count) => {
                // Each changed file is also NOT printed
                debug_assert_eq!(
                    changed_count, 0,
                    "In write mode, changed_count should not be counted"
                );
                print_stats(stdout, stderr);
                CliRunResult::FormatSucceeded
            }
        }
    }
}
