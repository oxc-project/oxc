use std::{
    env,
    io::{self, BufWriter, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use super::{
    CliRunResult, FormatCommand, Mode,
    command::ConfigOptions,
    resolve::{build_global_ignore_matchers, is_ignored, resolve_ignore_paths},
};
use crate::core::{
    ConfigResolver, ExternalServices, FormatResult, JsConfigLoaderCb, NestedConfigCtx,
    ResolveOutcome, SourceFormatter, classify_file_kind, resolve_editorconfig_path,
    resolve_file_scope_config, utils,
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

        // Resolve filepath to absolute for nested config resolution and ignore check
        let filepath = utils::normalize_relative_path(&cwd, &filepath);

        let ExplicitFileConfig { config_resolver, ignored } = match resolve_explicit_file(
            &cwd,
            &filepath,
            &config_options,
            &ignore_options.ignore_path,
            &self.js_config_loader,
        ) {
            Ok(resolved) => resolved,
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

        if ignored {
            utils::print_and_flush(stdout, &source_text);
            return CliRunResult::FormatSucceeded;
        }

        let Some(kind) = classify_file_kind(Arc::from(filepath)) else {
            utils::print_and_flush(stderr, "Unsupported file type for stdin-filepath\n");
            return CliRunResult::InvalidOptionConfig;
        };
        let strategy = match config_resolver.resolve(kind) {
            Ok(ResolveOutcome::Format(strategy)) => strategy,
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
        match tokio::task::block_in_place(|| source_formatter.format(&source_text, strategy)) {
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

/// Config scope and ignore status resolved for a single explicitly requested file.
pub struct ExplicitFileConfig {
    /// The config scope that applies to the file (root or nearest nested config).
    pub config_resolver: Arc<ConfigResolver>,
    /// Whether formatter-owned ignores (`.prettierignore`, `--ignore-path`, `ignorePatterns`) exclude the file.
    pub ignored: bool,
}

/// Resolve the config scope and ignore status for a single explicitly requested file.
///
/// Shared by `--stdin-filepath` and the NAPI `resolveConfig()` API, so both apply the same rules:
/// - root config discovered upwards from `cwd` (or `config_options.config`)
/// - nested config scope for `filepath`, following the same logic as `walk_runner`
/// - `.editorconfig` nearest to `cwd`
/// - `.gitignore` is deliberately not consulted, since the file is explicitly requested
///
/// `filepath` must be absolute.
///
/// # Errors
/// Returns a message ready to print if config or ignore file loading, parsing, or validation fails.
pub fn resolve_explicit_file(
    cwd: &Path,
    filepath: &Path,
    config_options: &ConfigOptions,
    ignore_paths: &[PathBuf],
    js_config_loader: &JsConfigLoaderCb,
) -> Result<ExplicitFileConfig, String> {
    let editorconfig_path = resolve_editorconfig_path(cwd);
    let mut config_resolver = ConfigResolver::from_config(
        cwd,
        config_options.config.as_deref(),
        editorconfig_path.as_deref(),
        Some(js_config_loader),
    )
    .map_err(|err| format!("Failed to load configuration file.\n{err}"))?;
    config_resolver
        .build_and_validate()
        .map_err(|err| format!("Failed to parse configuration.\n{err}"))?;

    let nested_ctx = config_options.use_nested_configs().then(|| {
        NestedConfigCtx::new(
            editorconfig_path.as_deref().map(Arc::from),
            Some(Arc::clone(js_config_loader)),
        )
    });
    let config_resolver =
        resolve_file_scope_config(filepath, &Arc::new(config_resolver), nested_ctx.as_ref())
            .map_err(|err| format!("Failed to load configuration file.\n{err}"))?;

    let global_matchers = resolve_ignore_paths(cwd, ignore_paths)
        .and_then(|paths| build_global_ignore_matchers(cwd, &[], &paths))?;
    let ignored = is_ignored(&global_matchers, filepath, false, true)
        || config_resolver.is_path_ignored(filepath, false);

    Ok(ExplicitFileConfig { config_resolver, ignored })
}
