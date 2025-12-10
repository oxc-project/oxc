#[cfg(feature = "napi")]
use std::borrow::Cow;
use std::path::Path;

use oxc_allocator::AllocatorPool;
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter::{FormatOptions, Formatter, enable_jsx_source_type, get_parse_options};
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::FormatFileStrategy;

pub enum FormatResult {
    Success { is_changed: bool, code: String },
    Error(Vec<OxcDiagnostic>),
}

pub struct SourceFormatter {
    allocator_pool: AllocatorPool,
    format_options: FormatOptions,
    #[cfg(feature = "napi")]
    pub is_sort_package_json: bool,
    #[cfg(feature = "napi")]
    external_formatter: Option<super::ExternalFormatter>,
}

impl SourceFormatter {
    pub fn new(num_of_threads: usize, format_options: FormatOptions) -> Self {
        Self {
            allocator_pool: AllocatorPool::new(num_of_threads),
            format_options,
            #[cfg(feature = "napi")]
            is_sort_package_json: false,
            #[cfg(feature = "napi")]
            external_formatter: None,
        }
    }

    #[cfg(feature = "napi")]
    #[must_use]
    pub fn with_external_formatter(
        mut self,
        external_formatter: Option<super::ExternalFormatter>,
        sort_package_json: bool,
    ) -> Self {
        self.external_formatter = external_formatter;
        self.is_sort_package_json = sort_package_json;
        self
    }

    /// Format a file based on its source type.
    pub fn format(&self, entry: &FormatFileStrategy, source_text: &str) -> FormatResult {
        let result = match entry {
            FormatFileStrategy::OxcFormatter { path, source_type } => {
                self.format_by_oxc_formatter(source_text, path, *source_type)
            }
            #[cfg(feature = "napi")]
            FormatFileStrategy::ExternalFormatter { path, parser_name } => {
                self.format_by_external_formatter(source_text, path, parser_name)
            }
            #[cfg(feature = "napi")]
            FormatFileStrategy::ExternalFormatterPackageJson { path, parser_name } => {
                self.format_by_external_formatter_package_json(source_text, path, parser_name)
            }
            #[cfg(not(feature = "napi"))]
            FormatFileStrategy::ExternalFormatter { .. }
            | FormatFileStrategy::ExternalFormatterPackageJson { .. } => {
                unreachable!("If `napi` feature is disabled, this should not be passed here")
            }
        };

        match result {
            Ok(code) => FormatResult::Success { is_changed: source_text != code, code },
            Err(err) => FormatResult::Error(vec![err]),
        }
    }

    /// Format JS/TS source code using oxc_formatter.
    fn format_by_oxc_formatter(
        &self,
        source_text: &str,
        path: &Path,
        source_type: SourceType,
    ) -> Result<String, OxcDiagnostic> {
        let source_type = enable_jsx_source_type(source_type);
        let allocator = self.allocator_pool.get();

        let ret = Parser::new(&allocator, source_text, source_type)
            .with_options(get_parse_options())
            .parse();
        if !ret.errors.is_empty() {
            // Return the first error for simplicity
            return Err(ret.errors.into_iter().next().unwrap());
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

        let code = formatted.print().map_err(|err| {
            OxcDiagnostic::error(format!(
                "Failed to print formatted code: {}\n{err}",
                path.display()
            ))
        })?;

        #[cfg(feature = "detect_code_removal")]
        {
            if let Some(diff) =
                oxc_formatter::detect_code_removal(source_text, code.as_code(), source_type)
            {
                unreachable!("Code removal detected in `{}`:\n{diff}", path.to_string_lossy());
            }
        }

        Ok(code.into_code())
    }

    /// Format non-JS/TS file using external formatter (Prettier).
    #[cfg(feature = "napi")]
    fn format_by_external_formatter(
        &self,
        source_text: &str,
        path: &Path,
        parser_name: &str,
    ) -> Result<String, OxcDiagnostic> {
        let external_formatter = self
            .external_formatter
            .as_ref()
            .expect("`external_formatter` must exist when `napi` feature is enabled");

        // NOTE: To call Prettier, we need to either infer the parser from `filepath` or specify the `parser`.
        //
        // We are specifying the `parser`, so `filepath` is not actually necessary,
        // but since some plugins might depend on `filepath`, we pass the actual file name as well.
        //
        // In that sense, it might be OK to just pass `filepath` without specifying `parser`,
        // but considering cases like treating `tsconfig.json` as `jsonc`, we need to specify `parser` as well.
        // (without supporting `overrides` in config file)
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        external_formatter.format_file(parser_name, file_name, source_text).map_err(|err| {
            OxcDiagnostic::error(format!(
                "Failed to format file with external formatter: {}\n{err}",
                path.display()
            ))
        })
    }

    /// Format `package.json`: optionally sort then format by external formatter.
    #[cfg(feature = "napi")]
    fn format_by_external_formatter_package_json(
        &self,
        source_text: &str,
        path: &Path,
        parser_name: &str,
    ) -> Result<String, OxcDiagnostic> {
        let source_text: Cow<'_, str> = if self.is_sort_package_json {
            Cow::Owned(sort_package_json::sort_package_json(source_text).map_err(|err| {
                OxcDiagnostic::error(format!(
                    "Failed to sort package.json: {}\n{err}",
                    path.display()
                ))
            })?)
        } else {
            Cow::Borrowed(source_text)
        };

        self.format_by_external_formatter(&source_text, path, parser_name)
    }
}
