use std::{
    env,
    io::{self, BufWriter, Read},
    path::PathBuf,
};

use crate::cli::{CliRunResult, FormatCommand, Mode};
use crate::core::{
    ExternalFormatter, FormatFileStrategy, FormatResult, SourceFormatter, load_config,
    resolve_config_path, utils,
};

#[derive(Debug)]
pub struct StdinRunner {
    options: FormatCommand,
    cwd: PathBuf,
    external_formatter: Option<ExternalFormatter>,
}

impl StdinRunner {
    /// Creates a new StdinRunner instance.
    ///
    /// # Panics
    /// Panics if the current working directory cannot be determined.
    pub fn new(options: FormatCommand) -> Self {
        Self {
            options,
            cwd: env::current_dir().expect("Failed to get current working directory"),
            external_formatter: None,
        }
    }

    #[must_use]
    pub fn with_external_formatter(
        mut self,
        external_formatter: Option<ExternalFormatter>,
    ) -> Self {
        self.external_formatter = external_formatter;
        self
    }

    pub fn run(self) -> CliRunResult {
        let stdout = &mut BufWriter::new(io::stdout());
        let stderr = &mut BufWriter::new(io::stderr());

        let cwd = self.cwd;
        let FormatCommand { mode, config_options, .. } = self.options;

        let Mode::Stdin(filepath) = mode else {
            unreachable!("`StdinRunner::run()` called with non-Stdin mode");
        };
        let Some(external_formatter) = self.external_formatter else {
            unreachable!("`StdinRunner::run()` called without `external_formatter`");
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
        let config_path = resolve_config_path(&cwd, config_options.config.as_deref());
        let (format_options, oxfmt_options, external_config) =
            match load_config(config_path.as_deref()) {
                Ok(c) => c,
                Err(err) => {
                    utils::print_and_flush(
                        stderr,
                        &format!("Failed to load configuration file.\n{err}\n"),
                    );
                    return CliRunResult::InvalidOptionConfig;
                }
            };

        // TODO: Plugins support
        if let Err(err) =
            external_formatter.setup_config(&external_config.to_string(), num_of_threads)
        {
            utils::print_and_flush(
                stderr,
                &format!("Failed to setup external formatter config.\n{err}\n"),
            );
            return CliRunResult::InvalidOptionConfig;
        }

        // Determine format strategy from filepath
        let Ok(strategy) = FormatFileStrategy::try_from(filepath) else {
            utils::print_and_flush(stderr, "Unsupported file type for stdin-filepath\n");
            return CliRunResult::InvalidOptionConfig;
        };

        // Create formatter and format
        let source_formatter = SourceFormatter::new(num_of_threads, format_options)
            .with_external_formatter(Some(external_formatter), oxfmt_options.sort_package_json);

        // Run formatting in a blocking task within tokio runtime
        // This is needed because external formatter uses `tokio::runtime::Handle::current()`
        match tokio::task::block_in_place(|| source_formatter.format(&strategy, &source_text)) {
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
