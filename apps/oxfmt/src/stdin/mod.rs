use std::{
    env,
    io::{self, BufWriter, Read},
    path::PathBuf,
};

use crate::cli::{CliRunResult, FormatCommand, Mode};
use crate::core::{
    ConfigResolver, ExternalFormatter, FormatFileStrategy, FormatResult, SourceFormatter,
    resolve_editorconfig_path, resolve_oxfmtrc_path, utils,
};

#[derive(Debug)]
pub struct StdinRunner {
    options: FormatCommand,
    cwd: PathBuf,
    external_formatter: ExternalFormatter,
}

impl StdinRunner {
    /// Creates a new StdinRunner instance.
    ///
    /// # Panics
    /// Panics if the current working directory cannot be determined.
    pub fn new(options: FormatCommand, external_formatter: ExternalFormatter) -> Self {
        Self {
            options,
            cwd: env::current_dir().expect("Failed to get current working directory"),
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
        let oxfmtrc_path = resolve_oxfmtrc_path(&cwd, config_options.config.as_deref());
        let editorconfig_path = resolve_editorconfig_path(&cwd);
        let mut config_resolver = match ConfigResolver::from_config_paths(
            &cwd,
            oxfmtrc_path.as_deref(),
            editorconfig_path.as_deref(),
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
        match config_resolver.build_and_validate() {
            Ok(_) => {}
            Err(err) => {
                utils::print_and_flush(stderr, &format!("Failed to parse configuration.\n{err}\n"));
                return CliRunResult::InvalidOptionConfig;
            }
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
        let Ok(strategy) = FormatFileStrategy::try_from(filepath) else {
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
