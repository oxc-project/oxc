use std::{
    env,
    io::{self, BufWriter, Read},
    path::{Path, PathBuf},
};

use ignore::gitignore::GitignoreBuilder;

use crate::cli::{CliRunResult, FormatCommand, Mode};
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
        let FormatCommand { mode, config_options, .. } = self.options;

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
        let ignore_patterns = match config_resolver.build_and_validate() {
            Ok(patterns) => patterns,
            Err(err) => {
                utils::print_and_flush(stderr, &format!("Failed to parse configuration.\n{err}\n"));
                return CliRunResult::InvalidOptionConfig;
            }
        };

        // Check if the stdin filepath matches any ignore pattern.
        // If ignored, output the source text unchanged (like Prettier).
        match is_ignored_by_patterns(&ignore_patterns, config_resolver.config_dir(), &filepath, &cwd) {
            Ok(true) => {
                utils::print_and_flush(stdout, &source_text);
                return CliRunResult::FormatSucceeded;
            }
            Err(err) => {
                utils::print_and_flush(stderr, &format!("{err}\n"));
                return CliRunResult::InvalidOptionConfig;
            }
            Ok(false) => {}
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

        // Resolve options for the stdin file entry
        let resolved_options = config_resolver.resolve(&strategy);

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

/// Check if a filepath should be ignored based on `ignorePatterns` from the config.
fn is_ignored_by_patterns(
    ignore_patterns: &[String],
    config_dir: Option<&Path>,
    filepath: &Path,
    cwd: &Path,
) -> Result<bool, String> {
    if ignore_patterns.is_empty() {
        return Ok(false);
    }
    let Some(config_dir) = config_dir else {
        return Ok(false);
    };

    let mut builder = GitignoreBuilder::new(config_dir);
    for pattern in ignore_patterns {
        builder.add_line(None, pattern).map_err(|_| {
            format!("Failed to add ignore pattern `{pattern}` from `.ignorePatterns`")
        })?;
    }
    let gitignore = builder.build().map_err(|_| "Failed to build ignores".to_string())?;

    // Resolve filepath relative to cwd for matching
    let full_path =
        if filepath.is_absolute() { filepath.to_path_buf() } else { cwd.join(filepath) };

    let matched = gitignore.matched_path_or_any_parents(&full_path, false);
    Ok(matched.is_ignore() && !matched.is_whitelist())
}
