use std::{fs, io, path::Path, sync::mpsc, time::Instant};

use cow_utils::CowUtils;
use rayon::prelude::*;

use oxc_allocator::AllocatorPool;
use oxc_diagnostics::{DiagnosticSender, DiagnosticService, OxcDiagnostic};
use oxc_formatter::{FormatOptions, Formatter, enable_jsx_source_type, get_parse_options};
use oxc_parser::Parser;

use crate::{command::OutputOptions, walk::WalkEntry};

type PathSender = mpsc::Sender<String>;

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

    /// Process entries as they are received from the channel
    #[expect(clippy::needless_pass_by_value)]
    pub fn run_streaming(
        &self,
        rx_entry: mpsc::Receiver<WalkEntry>,
        tx_error: &DiagnosticSender,
        tx_path: &PathSender,
        // Take ownership to close the channel when done
        tx_count: mpsc::Sender<()>,
    ) {
        rx_entry.into_iter().par_bridge().for_each(|entry| {
            self.process_entry(&entry, tx_error, tx_path);
            // Signal that we processed one file (ignore send errors if receiver dropped)
            let _ = tx_count.send(());
        });
    }

    /// Process a single entry
    fn process_entry(&self, entry: &WalkEntry, tx_error: &DiagnosticSender, tx_path: &PathSender) {
        let start_time = Instant::now();

        let path = &entry.path;
        let source_type = enable_jsx_source_type(entry.source_type);

        let allocator = self.allocator_pool.get();
        let Ok(source_text) = read_to_string(path) else {
            // This happens if `.ts` for MPEG-TS binary is attempted to be formatted
            let diagnostics = DiagnosticService::wrap_diagnostics(
                self.cwd.clone(),
                path,
                "",
                vec![
                    OxcDiagnostic::error(format!("Failed to read file: {}", path.display()))
                        .with_help("This may be due to the file being a binary or inaccessible."),
                ],
            );
            tx_error.send(diagnostics).unwrap();
            return;
        };

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

        #[cfg(feature = "napi")]
        let formatted = {
            if self.format_options.embedded_language_formatting.is_off() {
                base_formatter.format(&ret.program)
            } else {
                let embedded_formatter = self
                    .external_formatter
                    .as_ref()
                    .expect("`external_formatter` must exist when `napi` feature is enabled")
                    .to_embedded_formatter();
                base_formatter.format_with_embedded(&ret.program, embedded_formatter)
            }
        };
        #[cfg(not(feature = "napi"))]
        let formatted = base_formatter.format(&ret.program);

        let code = match formatted.print() {
            Ok(printed) => printed.into_code(),
            Err(err) => {
                let diagnostics = DiagnosticService::wrap_diagnostics(
                    self.cwd.clone(),
                    path,
                    &source_text,
                    vec![OxcDiagnostic::error(format!(
                        "Failed to print formatted code: {}\n{err}",
                        path.display()
                    ))],
                );
                tx_error.send(diagnostics).unwrap();
                return;
            }
        };

        #[cfg(feature = "detect_code_removal")]
        {
            if let Some(diff) = oxc_formatter::detect_code_removal(&source_text, &code, source_type)
            {
                unreachable!("Code removal detected in `{}`:\n{diff}", path.to_string_lossy());
            }
        }

        let elapsed = start_time.elapsed();
        let is_changed = source_text != code;

        // Write back if needed
        if matches!(self.output_options, OutputOptions::Write) && is_changed {
            fs::write(path, code)
                .map_err(|_| format!("Failed to write to '{}'", path.to_string_lossy()))
                .unwrap();
        }

        // Notify if needed
        if let Some(output) = match (&self.output_options, is_changed) {
            (OutputOptions::Check | OutputOptions::ListDifferent, true) => {
                let display_path = path
                    // Show path relative to `cwd` for cleaner output
                    .strip_prefix(&self.cwd)
                    .unwrap_or(path)
                    .to_string_lossy()
                    // Normalize path separators for consistent output across platforms
                    .cow_replace('\\', "/")
                    .to_string();
                let elapsed = elapsed.as_millis();

                if matches!(self.output_options, OutputOptions::Check) {
                    Some(format!("{display_path} ({elapsed}ms)"))
                } else {
                    Some(display_path)
                }
            }
            _ => None,
        } {
            tx_path.send(output).unwrap();
        }
    }
}

fn read_to_string(path: &Path) -> io::Result<String> {
    // `simdutf8` is faster than `std::str::from_utf8` which `fs::read_to_string` uses internally
    let bytes = fs::read(path)?;
    if simdutf8::basic::from_utf8(&bytes).is_err() {
        // Same error as `fs::read_to_string` produces (using `io::ErrorKind::InvalidData`)
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "stream did not contain valid UTF-8",
        ));
    }
    // SAFETY: `simdutf8` has ensured it's a valid UTF-8 string
    Ok(unsafe { String::from_utf8_unchecked(bytes) })
}
