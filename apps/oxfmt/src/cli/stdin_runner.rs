use std::{
    env,
    io::{self, BufWriter, Read},
    path::PathBuf,
};

use oxc_diagnostics::{DiagnosticService, GraphicalReportHandler};

use super::{CliRunResult, FormatCommand, Mode};
use crate::core::{
    ConfigScopes, ExternalServices, FormatResult, JsConfigLoaderCb, ResolveOutcome,
    SourceFormatter, build_global_ignore_matchers, classify_file, is_ignored, resolve_ignore_paths,
    utils,
};

pub struct StdinRunner {
    options: FormatCommand,
    cwd: PathBuf,
    js_config_loader: JsConfigLoaderCb,
    external_services: ExternalServices,
}

impl StdinRunner {
    /// Creates a new StdinRunner instance.
    ///
    /// # Panics
    /// Panics if the current working directory cannot be determined.
    pub fn new(
        options: FormatCommand,
        js_config_loader: JsConfigLoaderCb,
        external_services: ExternalServices,
    ) -> Self {
        Self {
            options,
            cwd: env::current_dir().expect("Failed to get current working directory"),
            js_config_loader,
            external_services,
        }
    }

    pub fn run(self) -> CliRunResult {
        let stdout = &mut BufWriter::new(io::stdout());
        let stderr = &mut BufWriter::new(io::stderr());

        let cwd = self.cwd;
        let FormatCommand { mode, config_options, ignore_options, .. } = self.options;

        let Mode::Stdin(filepath) = mode else {
            unreachable!("`StdinRunner::run()` called with non-Stdin mode");
        };
        // Single threaded for stdin formatting
        let num_of_threads = 1;

        // Read source code from stdin
        let mut source_text = String::new();
        if let Err(err) = io::stdin().read_to_string(&mut source_text) {
            utils::print_and_flush(stderr, &format!("Failed to read from stdin: {err}\n"));
            return CliRunResult::InvalidOptionConfig;
        }

        // Load config
        let config_scopes = match ConfigScopes::load(
            &cwd,
            config_options.config.as_deref(),
            config_options.use_nested_configs(),
            Some(&self.js_config_loader),
        ) {
            Ok(scopes) => scopes,
            Err(err) => {
                utils::print_and_flush(stderr, &format!("{err}\n"));
                return CliRunResult::InvalidOptionConfig;
            }
        };

        // Use `block_in_place()` to avoid nested async runtime access
        if let Err(err) =
            tokio::task::block_in_place(|| self.external_services.init(num_of_threads))
        {
            utils::print_and_flush(stderr, &format!("Failed to setup external services.\n{err}\n"));
            return CliRunResult::InvalidOptionConfig;
        }

        // Resolve filepath to absolute for nested config resolution and ignore check
        let filepath = utils::normalize_relative_path(&cwd, &filepath);

        // Check if the file is ignored by tool-ignores.
        // `.gitignore` is deliberately not consulted, stdin is an explicitly requested document.
        // Checked before resolving the scope, so configs under ignored dirs are never loaded.
        let global_matchers = match resolve_ignore_paths(&cwd, &ignore_options.ignore_path)
            .and_then(|paths| build_global_ignore_matchers(&cwd, &[], &paths))
        {
            Ok(matchers) => matchers,
            Err(err) => {
                utils::print_and_flush(stderr, &format!("{err}\n"));
                return CliRunResult::InvalidOptionConfig;
            }
        };
        if is_ignored(&global_matchers, &filepath, false, true) {
            utils::print_and_flush(stdout, &source_text);
            return CliRunResult::FormatSucceeded;
        }

        let config_resolver = match config_scopes.resolve(&filepath) {
            Ok(resolved) => resolved,
            Err(err) => {
                utils::print_and_flush(
                    stderr,
                    &format!("Failed to load configuration file.\n{err}\n"),
                );
                return CliRunResult::InvalidOptionConfig;
            }
        };
        // Then config's `ignorePatterns`
        if config_resolver.is_path_ignored(&filepath, false) {
            utils::print_and_flush(stdout, &source_text);
            return CliRunResult::FormatSucceeded;
        }

        let Some(strategy) = classify_file(&filepath) else {
            utils::print_and_flush(stderr, "Unsupported file type for stdin-filepath\n");
            return CliRunResult::InvalidOptionConfig;
        };
        let plan = match config_resolver.resolve(&filepath, strategy) {
            Ok(ResolveOutcome::Format(plan)) => plan,
            Ok(ResolveOutcome::MissingPlugin(_)) => {
                utils::print_and_flush(stdout, &source_text);
                return CliRunResult::FormatSucceeded;
            }
            Err(err) => {
                utils::print_and_flush(stderr, &format!("{err}\n"));
                return CliRunResult::InvalidOptionConfig;
            }
        };

        // Create formatter and format
        let source_formatter = SourceFormatter::new(num_of_threads)
            .with_external_services(Some(self.external_services));

        // Use `block_in_place()` to avoid nested async runtime access
        match tokio::task::block_in_place(|| source_formatter.format(&source_text, plan)) {
            FormatResult::Success { code, .. } => {
                utils::print_and_flush(stdout, &code);
                CliRunResult::FormatSucceeded
            }
            FormatResult::Error(diagnostics) => {
                let handler = GraphicalReportHandler::new();
                let mut output = String::new();
                for error in
                    DiagnosticService::wrap_diagnostics(&cwd, &filepath, &source_text, diagnostics)
                {
                    // Writing to `String` never fails
                    let _ = handler.render_report(&mut output, error.as_ref());
                }
                utils::print_and_flush(stderr, &output);
                CliRunResult::FormatFailed
            }
        }
    }
}
