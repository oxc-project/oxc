use std::{
    env,
    io::{self, BufWriter, Read},
    path::PathBuf,
};

use super::{
    CliRunResult, FormatCommand, Mode,
    resolve::{
        build_global_ignore_matchers, is_ignored, resolve_file_scope_config, resolve_ignore_paths,
    },
};
use crate::core::{
    ConfigResolver, ExternalFormatter, FormatFileStrategy, FormatResult, JsConfigLoaderCb,
    SourceFormatter, resolve_editorconfig_path, utils,
};

pub struct StdinRunner {
    options: FormatCommand,
    cwd: PathBuf,
    js_config_loader: JsConfigLoaderCb,
    external_formatter: ExternalFormatter,
}

impl StdinRunner {
    /// Creates a new StdinRunner instance.
    ///
    /// # Panics
    /// Panics if the current working directory cannot be determined.
    pub fn new(
        options: FormatCommand,
        js_config_loader: JsConfigLoaderCb,
        external_formatter: ExternalFormatter,
    ) -> Self {
        Self {
            options,
            cwd: env::current_dir().expect("Failed to get current working directory"),
            js_config_loader,
            external_formatter,
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
        let editorconfig_path = resolve_editorconfig_path(&cwd);
        let mut config_resolver = match ConfigResolver::from_config(
            &cwd,
            config_options.config.as_deref(),
            editorconfig_path.as_deref(),
            Some(&self.js_config_loader),
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
        if let Err(err) = config_resolver.build_and_validate() {
            utils::print_and_flush(stderr, &format!("Failed to parse configuration.\n{err}\n"));
            return CliRunResult::InvalidOptionConfig;
        }

        // Use `block_in_place()` to avoid nested async runtime access
        match tokio::task::block_in_place(|| self.external_formatter.init(num_of_threads)) {
            // TODO: Plugins support
            Ok(_) => {}
            Err(err) => {
                utils::print_and_flush(
                    stderr,
                    &format!("Failed to setup external formatter.\n{err}\n"),
                );
                return CliRunResult::InvalidOptionConfig;
            }
        }

        // Determine format strategy from filepath
        let Ok(strategy) =
            FormatFileStrategy::try_from(filepath).map(|s| s.resolve_relative_path(&cwd))
        else {
            utils::print_and_flush(stderr, "Unsupported file type for stdin-filepath\n");
            return CliRunResult::InvalidOptionConfig;
        };

        // Resolve nested config based on filepath's parent directory
        // (same as CLI direct file path behavior)
        let detect_nested =
            config_options.config.is_none() && !config_options.disable_nested_config;
        if detect_nested {
            match resolve_file_scope_config(
                strategy.path(),
                config_resolver.config_dir(),
                editorconfig_path.as_deref(),
                Some(&self.js_config_loader),
            ) {
                Ok(Some(nested)) => config_resolver = nested,
                Ok(None) => {} // No nested config or same as root — use root
                Err(err) => {
                    utils::print_and_flush(
                        stderr,
                        &format!("Failed to load configuration file.\n{err}\n"),
                    );
                    return CliRunResult::InvalidOptionConfig;
                }
            }
        }

        // Check if the file is ignored by global ignores or config's `ignorePatterns`
        let global_matchers = match resolve_ignore_paths(&cwd, &ignore_options.ignore_path)
            .and_then(|paths| build_global_ignore_matchers(&cwd, &[], &paths))
        {
            Ok(matchers) => matchers,
            Err(err) => {
                utils::print_and_flush(stderr, &format!("{err}\n"));
                return CliRunResult::InvalidOptionConfig;
            }
        };
        if is_ignored(&global_matchers, strategy.path(), false, true)
            || config_resolver.is_path_ignored(strategy.path(), false)
        {
            utils::print_and_flush(stdout, &source_text);
            return CliRunResult::FormatSucceeded;
        }

        // Resolve options for the stdin file entry
        let resolved_options = match config_resolver.resolve(&strategy) {
            Ok(options) => options,
            Err(err) => {
                utils::print_and_flush(stderr, &format!("{err}\n"));
                return CliRunResult::InvalidOptionConfig;
            }
        };

        // Create formatter and format
        let source_formatter = SourceFormatter::new(num_of_threads)
            .with_external_formatter(Some(self.external_formatter));

        // Use `block_in_place()` to avoid nested async runtime access
        match tokio::task::block_in_place(|| {
            source_formatter.format(&strategy, &source_text, resolved_options)
        }) {
            FormatResult::Success { code, .. } => {
                utils::print_and_flush(stdout, &code);
                CliRunResult::FormatSucceeded
            }
            FormatResult::Error(errors) => {
                for err in errors {
                    utils::print_and_flush(stderr, &format!("{err}\n"));
                }
                CliRunResult::FormatFailed
            }
        }
    }
}
