#[cfg(feature = "napi")]
use std::borrow::Cow;
use std::path::Path;

use serde_json::Value;
use tracing::instrument;

use oxc_allocator::AllocatorPool;
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter::{FormatOptions, Formatter, enable_jsx_source_type, get_parse_options};
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::{FormatFileStrategy, ResolvedOptions};

pub enum FormatResult {
    Success { is_changed: bool, code: String },
    Error(Vec<OxcDiagnostic>),
}

pub struct SourceFormatter {
    allocator_pool: AllocatorPool,
    #[cfg(feature = "napi")]
    external_formatter: Option<super::ExternalFormatter>,
}

impl SourceFormatter {
    pub fn new(num_of_threads: usize) -> Self {
        Self {
            allocator_pool: AllocatorPool::new(num_of_threads),
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

    /// Format a file based on its entry type and resolved options.
    #[instrument(level = "debug", name = "oxfmt::format", skip_all, fields(path = %entry.path().display()))]
    pub fn format(
        &self,
        entry: &FormatFileStrategy,
        source_text: &str,
        resolved_options: ResolvedOptions,
    ) -> FormatResult {
        let (result, insert_final_newline) = match (entry, resolved_options) {
            (
                FormatFileStrategy::OxcFormatter { path, source_type },
                ResolvedOptions::OxcFormatter {
                    format_options,
                    external_options,
                    insert_final_newline,
                },
            ) => (
                self.format_by_oxc_formatter(
                    source_text,
                    path,
                    *source_type,
                    *format_options,
                    external_options,
                ),
                insert_final_newline,
            ),
            (
                FormatFileStrategy::OxfmtToml { .. },
                ResolvedOptions::OxfmtToml { toml_options, insert_final_newline },
            ) => (Ok(Self::format_by_toml(source_text, toml_options)), insert_final_newline),
            #[cfg(feature = "napi")]
            (
                FormatFileStrategy::ExternalFormatter { path, parser_name },
                ResolvedOptions::ExternalFormatter {
                    format_options: _,
                    external_options,
                    insert_final_newline,
                },
            ) => (
                self.format_by_external_formatter(source_text, path, parser_name, external_options),
                insert_final_newline,
            ),
            #[cfg(feature = "napi")]
            (
                FormatFileStrategy::ExternalFormatterPackageJson { path, parser_name },
                ResolvedOptions::ExternalFormatterPackageJson {
                    external_options,
                    sort_package_json,
                    insert_final_newline,
                },
            ) => (
                self.format_by_external_formatter_package_json(
                    source_text,
                    path,
                    parser_name,
                    external_options,
                    sort_package_json.as_ref(),
                ),
                insert_final_newline,
            ),
            _ => unreachable!("FormatFileStrategy and ResolvedOptions variant mismatch"),
        };

        match result {
            Ok(mut code) => {
                // NOTE: `insert_final_newline` relies on the fact that:
                // - each formatter already ensures there is traliling newline
                // - each formatter does not have an option to disable trailing newline
                // So we can trim it here without allocating new string.
                if !insert_final_newline {
                    let trimmed_len = code.trim_end().len();
                    code.truncate(trimmed_len);
                }

                FormatResult::Success { is_changed: source_text != code, code }
            }
            Err(err) => FormatResult::Error(vec![err]),
        }
    }

    /// Format JS/TS source code using oxc_formatter.
    #[instrument(level = "debug", name = "oxfmt::format::oxc_formatter", skip_all)]
    fn format_by_oxc_formatter(
        &self,
        source_text: &str,
        path: &Path,
        source_type: SourceType,
        format_options: FormatOptions,
        external_options: Value,
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

        #[cfg(feature = "napi")]
        let external_callbacks = {
            let external_formatter = self
                .external_formatter
                .as_ref()
                .expect("`external_formatter` must exist when `napi` feature is enabled");

            Some(external_formatter.to_external_callbacks(path, &format_options, external_options))
        };

        #[cfg(not(feature = "napi"))]
        let external_callbacks = {
            let _ = (path, external_options);
            None
        };

        let base_formatter = Formatter::new(&allocator, format_options);
        let formatted =
            base_formatter.format_with_external_callbacks(&ret.program, external_callbacks);

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

    /// Format TOML file using `toml`.
    #[instrument(level = "debug", name = "oxfmt::format::oxc_toml", skip_all)]
    fn format_by_toml(source_text: &str, options: oxc_toml::Options) -> String {
        oxc_toml::format(source_text, options)
    }

    /// Format non-JS/TS file using external formatter (Prettier).
    #[cfg(feature = "napi")]
    #[expect(clippy::needless_pass_by_value)]
    #[instrument(level = "debug", name = "oxfmt::format::external_formatter", skip_all, fields(parser = %parser_name))]
    fn format_by_external_formatter(
        &self,
        source_text: &str,
        path: &Path,
        parser_name: &str,
        external_options: Value,
    ) -> Result<String, OxcDiagnostic> {
        let external_formatter = self
            .external_formatter
            .as_ref()
            .expect("`external_formatter` must exist when `napi` feature is enabled");

        // NOTE: To call Prettier, we need to either:
        // - let Prettier infer the parser from `filepath`
        // - or specify the `parser`
        //
        // We are specifying the `parser` for perf, so `filepath` is not actually necessary,
        // but since some plugins might depend on `filepath`, we pass the actual file name as well.
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        external_formatter
            .format_file(&external_options, parser_name, file_name, source_text)
            .map_err(|err| {
                OxcDiagnostic::error(format!(
                    "Failed to format file with external formatter: {}\n{err}",
                    path.display()
                ))
            })
    }

    /// Format `package.json`: optionally sort then format by external formatter.
    #[cfg(feature = "napi")]
    #[instrument(
        level = "debug",
        name = "oxfmt::format::external_formatter_package_json",
        skip_all
    )]
    fn format_by_external_formatter_package_json(
        &self,
        source_text: &str,
        path: &Path,
        parser_name: &str,
        external_options: Value,
        sort_options: Option<&sort_package_json::SortOptions>,
    ) -> Result<String, OxcDiagnostic> {
        let source_text: Cow<'_, str> = if let Some(options) = sort_options {
            Cow::Owned(
                sort_package_json::sort_package_json_with_options(source_text, options).map_err(
                    |err| {
                        OxcDiagnostic::error(format!(
                            "Failed to sort package.json: {}\n{err}",
                            path.display()
                        ))
                    },
                )?,
            )
        } else {
            Cow::Borrowed(source_text)
        };

        self.format_by_external_formatter(&source_text, path, parser_name, external_options)
    }
}
