use std::{
    env,
    io::{Read, Write},
    path::PathBuf,
};

use crate::cli::CliRunResult;
use crate::core::{
    FormatFileStrategy, FormatResult, SourceFormatter, load_config, resolve_config_path, utils,
};

pub struct StdinRunner {
    cwd: PathBuf,
    stdin_file_path: PathBuf,
    config_path: Option<PathBuf>,
    #[cfg(feature = "napi")]
    external_formatter: Option<crate::core::ExternalFormatter>,
}

impl StdinRunner {
    /// Creates a new `StdinRunner` instance.
    ///
    /// # Panics
    /// Panics if the current working directory cannot be determined.
    pub fn new(stdin_file_path: PathBuf, config_path: Option<PathBuf>) -> Self {
        Self {
            cwd: env::current_dir().expect("Failed to get current working directory"),
            stdin_file_path,
            config_path,
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
    pub fn run(
        self,
        stdin: &mut dyn Read,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
    ) -> CliRunResult {
        // Load config
        let config_path = resolve_config_path(&self.cwd, self.config_path.as_deref());
        let (format_options, oxfmt_options, external_config) =
            match load_config(config_path.as_deref()) {
                Ok(config) => config,
                Err(err) => {
                    utils::print_and_flush(
                        stderr,
                        &format!("Failed to load configuration file.\n{err}\n"),
                    );
                    return CliRunResult::InvalidOptionConfig;
                }
            };
        #[cfg(not(feature = "napi"))]
        let _ = (external_config, oxfmt_options.sort_package_json);

        // TODO: Plugins support
        #[cfg(feature = "napi")]
        if let Err(err) = self
            .external_formatter
            .as_ref()
            .expect("External formatter must be set when `napi` feature is enabled")
            .setup_config(&external_config.to_string(), 1)
        {
            utils::print_and_flush(
                stderr,
                &format!("Failed to setup external formatter config.\n{err}\n"),
            );
            return CliRunResult::InvalidOptionConfig;
        }

        // Formatter (single-threaded)
        let source_formatter = SourceFormatter::new(1, format_options);
        #[cfg(feature = "napi")]
        let source_formatter = source_formatter
            .with_external_formatter(self.external_formatter, oxfmt_options.sort_package_json);

        let entry = match FormatFileStrategy::try_from(self.stdin_file_path.clone()) {
            Ok(entry @ FormatFileStrategy::OxcFormatter { .. }) => entry,
            #[cfg(feature = "napi")]
            Ok(entry) => entry,
            _ => {
                utils::print_and_flush(
                    stderr,
                    &format!("Unsupported file type: {}\n", self.stdin_file_path.display()),
                );
                return CliRunResult::InvalidOptionConfig;
            }
        };

        // Read source from stdin
        let mut source_text = String::new();
        if let Err(err) = stdin.read_to_string(&mut source_text) {
            utils::print_and_flush(stderr, &format!("Failed to read from stdin: {err}\n"));
            return CliRunResult::FormatFailed;
        }

        match source_formatter.format(&entry, &source_text) {
            FormatResult::Success { code, .. } => {
                let _ = stdout.write_all(code.as_bytes());
                CliRunResult::FormatSucceeded
            }
            FormatResult::Error(diagnostics) => {
                for diagnostic in diagnostics {
                    utils::print_and_flush(stderr, &format!("{diagnostic}\n"));
                }
                CliRunResult::FormatFailed
            }
        }
    }
}
