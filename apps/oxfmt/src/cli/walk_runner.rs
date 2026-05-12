use std::{env, io::BufWriter, path::PathBuf, sync::mpsc, time::Instant};

use oxc_diagnostics::DiagnosticService;

use super::{
    command::{FormatCommand, Mode, OutputMode},
    reporter::DefaultReporter,
    resolve::resolve_ignore_paths,
    result::CliRunResult,
    service::{FormatService, SuccessResult},
    walk::ScopedWalker,
};
#[cfg(feature = "napi")]
use crate::core::JsConfigLoaderCb;
use crate::core::{
    ConfigResolver, FormatStrategy, SourceFormatter, resolve_editorconfig_path, utils,
};

pub struct WalkRunner {
    options: FormatCommand,
    cwd: PathBuf,
    #[cfg(feature = "napi")]
    external_formatter: Option<crate::core::ExternalFormatter>,
    #[cfg(feature = "napi")]
    js_config_loader: Option<JsConfigLoaderCb>,
}

impl WalkRunner {
    /// Creates a new WalkRunner instance.
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
            unreachable!("`WalkRunner` should only be called with Mode::Cli");
        };
        let num_of_threads = rayon::current_num_threads();

        // Find and load root config file
        let editorconfig_path = resolve_editorconfig_path(&cwd);
        let mut root_config_resolver = match ConfigResolver::from_config(
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
        if let Err(err) = root_config_resolver.build_and_validate() {
            utils::print_and_flush(stderr, &format!("Failed to parse configuration.\n{err}\n"));
            return CliRunResult::InvalidOptionConfig;
        }

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

        // Resolve ignore paths early to validate before walk starts
        let resolved_ignore_paths = match resolve_ignore_paths(&cwd, &ignore_options.ignore_path) {
            Ok(paths) => paths,
            Err(err) => {
                utils::print_and_flush(
                    stderr,
                    &format!("Failed to parse target paths or ignore settings.\n{err}\n"),
                );
                return CliRunResult::InvalidOptionConfig;
            }
        };

        // Shared channel for format entries from all scopes
        let (tx_entry, rx_entry) = mpsc::channel::<FormatStrategy>();
        // Collect format results (changed paths or unchanged count)
        let (tx_success, rx_success) = mpsc::channel();
        // Diagnostic from formatting service
        let (mut diagnostic_service, tx_error) =
            DiagnosticService::new(Box::new(DefaultReporter::default()));

        if matches!(format_mode, OutputMode::Check) {
            utils::print_and_flush(stdout, "Checking formatting...\n");
            utils::print_and_flush(stdout, "\n");
        }

        let scoped_walker = ScopedWalker::new(cwd.clone(), &paths);

        // Create `SourceFormatter` instance
        let source_formatter = SourceFormatter::new(num_of_threads);
        #[cfg(feature = "napi")]
        let source_formatter = source_formatter.with_external_formatter(self.external_formatter);

        // Clone `tx_error` so both the walk threads and the format service can report errors
        let tx_error_for_format = tx_error.clone();

        // Spawn formatting service on a dedicated thread so it doesn't occupy the rayon pool.
        // It just blocks on `rx_entry` waiting for entries; `par_bridge()` inside still uses rayon.
        std::thread::spawn(move || {
            let format_service = FormatService::new(cwd, format_mode, source_formatter);
            format_service.run_streaming(rx_entry, &tx_error_for_format, &tx_success);
        });

        // Run scoped walks (root + nested) sends entries to `tx_entry` and errors to `tx_error`.
        // Manually drop after the walk to signal the formatting service that no more entries will be sent.
        let any_config_found = match scoped_walker.run(
            root_config_resolver,
            &resolved_ignore_paths,
            ignore_options.with_node_modules,
            // Nested config detection is disabled when `--config` is explicitly specified
            // or when `--disable-nested-config` is set
            config_options.config.is_none() && !config_options.disable_nested_config,
            editorconfig_path.as_deref(),
            #[cfg(feature = "napi")]
            self.js_config_loader.as_ref(),
            &tx_entry,
            &tx_error,
        ) {
            Ok(found) => {
                drop(tx_entry);
                drop(tx_error);
                found
            }
            Err(err) => {
                drop(tx_entry);
                drop(tx_error);
                utils::print_and_flush(stderr, &format!("Failed to parse configuration.\n{err}\n"));
                return CliRunResult::InvalidOptionConfig;
            }
        };

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
            if !any_config_found && editorconfig_path.is_none() {
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
