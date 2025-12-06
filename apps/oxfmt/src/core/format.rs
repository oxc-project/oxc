use std::path::Path;

use oxc_allocator::AllocatorPool;
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter::{FormatOptions, Formatter, enable_jsx_source_type, get_parse_options};
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::FormatFileSource;
#[cfg(feature = "napi")]
use super::package_json_sorter::sort_package_json_content;

pub enum FormatResult {
    Success { is_changed: bool, code: String },
    Error(Vec<OxcDiagnostic>),
}

pub struct SourceFormatter {
    allocator_pool: AllocatorPool,
    format_options: FormatOptions,
    #[cfg(feature = "napi")]
    external_formatter: Option<super::ExternalFormatter>,
}

impl SourceFormatter {
    pub fn new(num_of_threads: usize, format_options: FormatOptions) -> Self {
        Self {
            allocator_pool: AllocatorPool::new(num_of_threads),
            format_options,
            #[cfg(feature = "napi")]
            external_formatter: None,
        }
    }

    #[cfg(feature = "napi")]
    #[must_use]
    pub fn with_external_formatter(
        mut self,
        external_formatter: Option<super::ExternalFormatter>,
    ) -> Self {
        self.external_formatter = external_formatter;
        self
    }

    /// Format a file based on its source type.
    pub fn format(&self, entry: &FormatFileSource, source_text: &str) -> FormatResult {
        match entry {
            FormatFileSource::OxcFormatter { path, source_type } => {
                self.format_by_oxc_formatter(source_text, path, *source_type)
            }
            #[cfg(feature = "napi")]
            FormatFileSource::ExternalFormatter { path, parser_name } => {
                self.format_by_external_formatter(source_text, path, parser_name)
            }
            #[cfg(not(feature = "napi"))]
            FormatFileSource::ExternalFormatter { .. } => {
                unreachable!("If `napi` feature is disabled, this should not be passed here")
            }
        }
    }

    /// Format JS/TS source code using oxc_formatter.
    fn format_by_oxc_formatter(
        &self,
        source_text: &str,
        path: &Path,
        source_type: SourceType,
    ) -> FormatResult {
        let source_type = enable_jsx_source_type(source_type);
        let allocator = self.allocator_pool.get();

        let ret = Parser::new(&allocator, source_text, source_type)
            .with_options(get_parse_options())
            .parse();
        if !ret.errors.is_empty() {
            return FormatResult::Error(ret.errors);
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
                return FormatResult::Error(vec![OxcDiagnostic::error(format!(
                    "Failed to print formatted code: {}\n{err}",
                    path.display()
                ))]);
            }
        };

        #[cfg(feature = "detect_code_removal")]
        {
            if let Some(diff) = oxc_formatter::detect_code_removal(source_text, &code, source_type)
            {
                unreachable!("Code removal detected in `{}`:\n{diff}", path.to_string_lossy());
            }
        }

        FormatResult::Success { is_changed: source_text != code, code }
    }

    /// Format non-JS/TS file using external formatter (Prettier).
    /// For package.json files, sorts them first before formatting.
    #[cfg(feature = "napi")]
    fn format_by_external_formatter(
        &self,
        source_text: &str,
        path: &Path,
        parser_name: &str,
    ) -> FormatResult {
        let external_formatter = self
            .external_formatter
            .as_ref()
            .expect("`external_formatter` must exist when `napi` feature is enabled");

        // Special handling for package.json: sort before formatting
        let code_to_format = if parser_name == "json-stringify"
            && path.file_name().and_then(|f| f.to_str()) == Some("package.json")
        {
            // Sort package.json content first
            match sort_package_json_content(source_text) {
                Ok(sorted) => sorted,
                Err(err) => {
                    // If sorting fails, return error - don't fall back to unsorted
                    return FormatResult::Error(vec![err]);
                }
            }
        } else {
            // For all other files, use original source
            source_text.to_string()
        };

        // Format with Prettier
        match external_formatter.format_file(parser_name, &code_to_format) {
            Ok(code) => FormatResult::Success { is_changed: source_text != code, code },
            Err(err) => FormatResult::Error(vec![OxcDiagnostic::error(format!(
                "Failed to format file with external formatter: {}\n{err}",
                path.display()
            ))]),
        }
    }
}
