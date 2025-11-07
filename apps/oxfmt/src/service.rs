use std::{fs, path::Path, sync::mpsc, time::Instant};

use cow_utils::CowUtils;
use rayon::prelude::*;

use oxc_allocator::AllocatorPool;
use oxc_diagnostics::{DiagnosticSender, DiagnosticService, OxcDiagnostic};
use oxc_formatter::{FormatOptions, Formatter, enable_jsx_source_type, get_parse_options};
use oxc_parser::Parser;

use crate::{command::OutputOptions, walk::WalkEntry};

pub struct FormatService {
    allocator_pool: AllocatorPool,
    cwd: Box<Path>,
    output_options: OutputOptions,
    format_options: FormatOptions,
    #[cfg(feature = "napi")]
    external_formatter: Option<crate::prettier_plugins::ExternalFormatter>,
}

impl FormatService {
    pub fn new<T>(
        allocator_pool: AllocatorPool,
        cwd: T,
        output_options: OutputOptions,
        format_options: FormatOptions,
    ) -> Self
    where
        T: Into<Box<Path>>,
    {
        Self {
            allocator_pool,
            cwd: cwd.into(),
            output_options,
            format_options,
            #[cfg(feature = "napi")]
            external_formatter: None,
        }
    }

    #[cfg(feature = "napi")]
    #[must_use]
    pub fn with_external_formatter(
        mut self,
        external_formatter: Option<crate::prettier_plugins::ExternalFormatter>,
    ) -> Self {
        self.external_formatter = external_formatter;
        self
    }

    // TODO: Support reading from stdin for formatting, similar to `prettier --stdin-filepath <path>`
    // This would allow formatting code from stdin and optionally specifying the file path for correct
    // syntax detection (e.g., `echo "const x=1" | oxfmt --stdin-filepath file.js`)

    /// Process entries as they are received from the channel
    #[expect(clippy::needless_pass_by_value)]
    pub fn run_streaming(
        &self,
        rx_entry: mpsc::Receiver<WalkEntry>,
        tx_error: &DiagnosticSender,
        // Take ownership to close the channel when done
        tx_count: mpsc::Sender<()>,
    ) {
        rx_entry.into_iter().par_bridge().for_each(|entry| {
            self.process_entry(&entry, tx_error);
            // Signal that we processed one file (ignore send errors if receiver dropped)
            let _ = tx_count.send(());
        });
    }

    /// Process a single entry
    fn process_entry(&self, entry: &WalkEntry, tx_error: &DiagnosticSender) {
        let start_time = Instant::now();

        let path = &entry.path;
        let source_type = enable_jsx_source_type(entry.source_type);

        let allocator = self.allocator_pool.get();
        let source_text = fs::read_to_string(path).expect("Failed to read file");

        let ret = Parser::new(&allocator, &source_text, source_type)
            .with_options(get_parse_options())
            .parse();
        if !ret.errors.is_empty() {
            let diagnostics = DiagnosticService::wrap_diagnostics(
                self.cwd.clone(),
                path,
                &source_text,
                ret.errors,
            );
            tx_error.send(diagnostics).unwrap();
            return;
        }

        let base_formatter = Formatter::new(&allocator, self.format_options.clone());
        let formatter = if self.format_options.embedded_language_formatting.is_off() {
            base_formatter
        } else {
            #[cfg(feature = "napi")]
            {
                let external_formatter = self
                    .external_formatter
                    .as_ref()
                    .map(crate::prettier_plugins::ExternalFormatter::to_embedded_formatter);
                base_formatter.with_embedded_formatter(external_formatter)
            }
            #[cfg(not(feature = "napi"))]
            {
                base_formatter
            }
        };

        let code = formatter.build(&ret.program);

        let elapsed = start_time.elapsed();
        let is_changed = source_text != code;

        // Write back if needed
        if matches!(self.output_options, OutputOptions::DefaultWrite) && is_changed {
            fs::write(path, code)
                .map_err(|_| format!("Failed to write to '{}'", path.to_string_lossy()))
                .unwrap();
        }

        // Notify if needed
        let display_path = path
            // Show path relative to `cwd` for cleaner output
            .strip_prefix(&self.cwd)
            .unwrap_or(path)
            .to_string_lossy()
            // Normalize path separators for consistent output across platforms
            .cow_replace('\\', "/")
            .to_string();
        let elapsed = elapsed.as_millis();
        if let Some(diagnostic) = match (&self.output_options, is_changed) {
            (OutputOptions::Check, true) => {
                Some(OxcDiagnostic::warn(format!("{display_path} ({elapsed}ms)")))
            }
            (OutputOptions::ListDifferent, true) => Some(OxcDiagnostic::warn(display_path)),
            (OutputOptions::DefaultWrite, _) => Some(OxcDiagnostic::warn(format!(
                "{display_path} {elapsed}ms{}",
                if is_changed { "" } else { " (unchanged)" }
            ))),
            _ => None,
        } {
            tx_error.send(vec![diagnostic.into()]).unwrap();
        }
    }
}
