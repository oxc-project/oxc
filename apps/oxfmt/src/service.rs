use std::{fs, io, path::Path, sync::mpsc, time::Instant};

use cow_utils::CowUtils;
use rayon::prelude::*;

use oxc_allocator::AllocatorPool;
use oxc_diagnostics::{DiagnosticSender, DiagnosticService, Error, OxcDiagnostic};
use oxc_formatter::{FormatOptions, Formatter, enable_jsx_source_type, get_parse_options};
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::{command::OutputOptions, support::FormatFileSource};

enum FormatFileResult {
    Success { is_changed: bool, code: String },
    Error(Vec<Error>),
}

pub enum SuccessResult {
    Changed(String),
    Unchanged,
}

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
    pub fn run_streaming(
        &self,
        rx_entry: mpsc::Receiver<FormatFileSource>,
        tx_error: &DiagnosticSender,
        tx_success: &mpsc::Sender<SuccessResult>,
    ) {
        rx_entry.into_iter().par_bridge().for_each(|entry| {
            let start_time = Instant::now();

            let (code, is_changed) = match self.format_file(&entry) {
                FormatFileResult::Success { is_changed, code } => (code, is_changed),
                FormatFileResult::Error(diagnostics) => {
                    tx_error.send(diagnostics).unwrap();
                    return;
                }
            };

            let elapsed = start_time.elapsed();
            let path = &entry.path();

            // Write back if needed
            if matches!(self.output_options, OutputOptions::Write) && is_changed {
                fs::write(path, code)
                    .map_err(|_| format!("Failed to write to '{}'", path.to_string_lossy()))
                    .unwrap();
            }

            // Report result
            let result = match (&self.output_options, is_changed) {
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
                        SuccessResult::Changed(format!("{display_path} ({elapsed}ms)"))
                    } else {
                        SuccessResult::Changed(display_path)
                    }
                }
                _ => SuccessResult::Unchanged,
            };
            tx_success.send(result).unwrap();
        });
    }

    // NOTE: This may be `pub` for LSP usage in the future
    fn format_file(&self, entry: &FormatFileSource) -> FormatFileResult {
        let path = entry.path();

        let Ok(source_text) = read_to_string(path) else {
            // This happens if binary file is attempted to be formatted
            // e.g. `.ts` for MPEG-TS video file
            let diagnostics = DiagnosticService::wrap_diagnostics(
                self.cwd.clone(),
                path,
                "",
                vec![
                    OxcDiagnostic::error(format!("Failed to read file: {}", path.display()))
                        .with_help("This may be due to the file being a binary or inaccessible."),
                ],
            );
            return FormatFileResult::Error(diagnostics);
        };

        match entry {
            FormatFileSource::OxcFormatter { path, source_type } => {
                self.format_by_oxc_formatter(&source_text, path, *source_type)
            }
            #[cfg(feature = "napi")]
            FormatFileSource::ExternalFormatter { path, parser_name } => {
                self.format_by_external_formatter(&source_text, path, parser_name)
            }
            #[cfg(not(feature = "napi"))]
            FormatFileSource::ExternalFormatter { .. } => {
                unreachable!("If `napi` feature is disabled, this should not be passed here",)
            }
        }
    }

    fn format_by_oxc_formatter(
        &self,
        source_text: &str,
        path: &Path,
        source_type: SourceType,
    ) -> FormatFileResult {
        let source_type = enable_jsx_source_type(source_type);

        let allocator = self.allocator_pool.get();
        let ret = Parser::new(&allocator, source_text, source_type)
            .with_options(get_parse_options())
            .parse();
        if !ret.errors.is_empty() {
            let diagnostics = DiagnosticService::wrap_diagnostics(
                self.cwd.clone(),
                path,
                source_text,
                ret.errors,
            );
            return FormatFileResult::Error(diagnostics);
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
                    source_text,
                    vec![OxcDiagnostic::error(format!(
                        "Failed to print formatted code: {}\n{err}",
                        path.display()
                    ))],
                );
                return FormatFileResult::Error(diagnostics);
            }
        };

        #[cfg(feature = "detect_code_removal")]
        {
            if let Some(diff) = oxc_formatter::detect_code_removal(source_text, &code, source_type)
            {
                unreachable!("Code removal detected in `{}`:\n{diff}", path.to_string_lossy());
            }
        }

        FormatFileResult::Success { is_changed: source_text != code, code }
    }

    #[cfg(feature = "napi")]
    fn format_by_external_formatter(
        &self,
        source_text: &str,
        path: &Path,
        parser_name: &str,
    ) -> FormatFileResult {
        let code = match self
            .external_formatter
            .as_ref()
            .expect("`external_formatter` must exist when `napi` feature is enabled")
            .format_file(parser_name, source_text)
        {
            Ok(code) => code,
            Err(err) => {
                let diagnostics = DiagnosticService::wrap_diagnostics(
                    self.cwd.clone(),
                    path,
                    source_text,
                    vec![OxcDiagnostic::error(format!(
                        "Failed to format file with external formatter: {}\n{err}",
                        path.display()
                    ))],
                );
                return FormatFileResult::Error(diagnostics);
            }
        };

        FormatFileResult::Success { is_changed: source_text != code, code }
    }
}

// ---

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
