use std::path::Path;

use serde_json::Value;
use tracing::instrument;

use oxc_allocator::AllocatorPool;
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter::{
    FormatOptions, Formatter, PrinterOptions, UniqueGroupIdBuilder, enable_jsx_source_type,
    get_parse_options, propagate_expand_elements,
};
use oxc_parser::Parser;
use oxc_span::SourceType;

use oxc_formatter::{IndentStyle, IndentWidth, LineEnding};

use super::{FormatFileStrategy, ResolvedOptions};

/// Extract `PrinterOptions` from Prettier-compatible external options JSON.
#[cfg(feature = "napi")]
fn external_options_to_printer_options(options: &Value) -> PrinterOptions {
    let mut printer = PrinterOptions::default();

    if let Some(obj) = options.as_object() {
        if let Some(w) = obj.get("printWidth").and_then(|v| v.as_u64()) {
            #[expect(clippy::cast_possible_truncation)]
            {
                printer.print_width = oxc_formatter::PrintWidth::new(w as u32);
            }
        }
        if let Some(use_tabs) = obj.get("useTabs").and_then(|v| v.as_bool()) {
            printer.indent_style = if use_tabs { IndentStyle::Tab } else { IndentStyle::Space };
        }
        if let Some(tw) = obj.get("tabWidth").and_then(|v| v.as_u64()) {
            #[expect(clippy::cast_possible_truncation)]
            {
                printer.indent_width = IndentWidth::try_from(tw as u8).unwrap_or_default();
            }
        }
        if let Some(eol) = obj.get("endOfLine").and_then(|v| v.as_str()) {
            printer.line_ending = match eol {
                "crlf" => LineEnding::Crlf,
                "cr" => LineEnding::Cr,
                _ => LineEnding::Lf,
            };
        }
    }

    printer
}

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
                    filepath_override,
                    insert_final_newline,
                },
            ) => (
                self.format_by_oxc_formatter(
                    source_text,
                    path,
                    *source_type,
                    *format_options,
                    external_options,
                    filepath_override.as_deref(),
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
                ResolvedOptions::ExternalFormatter { external_options, insert_final_newline },
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

    /// Format JS/TS source code using `oxc_formatter`.
    /// For embedded part and Tailwindcss sorting, `external_options` and `filepath_override` are used.
    #[instrument(level = "debug", name = "oxfmt::format::oxc_formatter", skip_all)]
    fn format_by_oxc_formatter(
        &self,
        source_text: &str,
        path: &Path,
        source_type: SourceType,
        format_options: FormatOptions,
        external_options: Value,
        filepath_override: Option<&Path>,
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
        let external_callbacks = Some(self.build_external_callbacks(
            &format_options,
            external_options,
            path,
            filepath_override,
        ));
        #[cfg(not(feature = "napi"))]
        let external_callbacks = {
            let _ = (path, external_options, filepath_override);
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

    /// Format TOML file using `oxc_toml`.
    #[instrument(level = "debug", name = "oxfmt::format::oxc_toml", skip_all)]
    fn format_by_toml(source_text: &str, options: oxc_toml::Options) -> String {
        oxc_toml::format(source_text, options)
    }
}

// ---

/// NAPI-only methods for `SourceFormatter`.
///
/// These methods handle external formatter (Prettier) integration,
/// which is only available when running through the Node.js NAPI bridge.
#[cfg(feature = "napi")]
impl SourceFormatter {
    #[must_use]
    pub fn with_external_formatter(
        mut self,
        external_formatter: Option<super::ExternalFormatter>,
    ) -> Self {
        self.external_formatter = external_formatter;
        self
    }

    /// Build external callbacks for `oxc_formatter` from the NAPI external formatter.
    ///
    /// Sets `filepath` on options for Prettier plugins that depend on it,
    /// and for the Tailwind sorter to resolve config.
    /// `filepath_override` is `Some` in js-in-xxx flow (via `textToDoc()`),
    /// where `path` is a dummy like `embedded.ts` but callbacks need the parent file path.
    /// See `oxfmtrc::finalize_external_options()` for where this filepath originates.
    fn build_external_callbacks(
        &self,
        format_options: &FormatOptions,
        mut external_options: Value,
        path: &Path,
        filepath_override: Option<&Path>,
    ) -> oxc_formatter::ExternalCallbacks {
        let external_formatter = self
            .external_formatter
            .as_ref()
            .expect("`external_formatter` must exist when `napi` feature is enabled");

        if let Value::Object(ref mut map) = external_options {
            let filepath = filepath_override.unwrap_or(path);
            map.insert(
                "filepath".to_string(),
                Value::String(filepath.to_string_lossy().to_string()),
            );
        }

        external_formatter.to_external_callbacks(format_options, external_options)
    }

    /// Format non-JS/TS file using external formatter (Prettier).
    #[instrument(level = "debug", name = "oxfmt::format::external_formatter", skip_all, fields(parser = %parser_name))]
    fn format_by_external_formatter(
        &self,
        source_text: &str,
        path: &Path,
        parser_name: &str,
        mut external_options: Value,
    ) -> Result<String, OxcDiagnostic> {
        let external_formatter = self
            .external_formatter
            .as_ref()
            .expect("`external_formatter` must exist when `napi` feature is enabled");

        // Experimental: Use Doc path for JSON files
        let use_doc_path = matches!(parser_name, "json" | "json-stringify" | "json5");

        // Set `parser` and `filepath` on options for Prettier.
        // We specify `parser` to skip parser inference for perf,
        // and `filepath` because some plugins depend on it.
        if let Value::Object(ref mut map) = external_options {
            map.insert("parser".to_string(), Value::String(parser_name.to_string()));
            map.insert("filepath".to_string(), Value::String(path.to_string_lossy().to_string()));
            if use_doc_path {
                map.insert("_returnDoc".to_string(), Value::Bool(true));
            }
        }

        // Extract printer options before moving external_options
        let printer_options = if use_doc_path {
            Some(external_options_to_printer_options(&external_options))
        } else {
            None
        };

        let result =
            external_formatter.format_file(external_options, source_text).map_err(|err| {
                let relative = std::env::current_dir()
                    .ok()
                    .and_then(|cwd| path.strip_prefix(cwd).ok().map(Path::to_path_buf));
                let display_path = relative.as_deref().unwrap_or(path).to_string_lossy();
                let message = if let Some((first, rest)) = err.split_once('\n') {
                    format!("{first}\n[{display_path}]\n{rest}")
                } else {
                    format!("{err}\n[{display_path}]")
                };
                OxcDiagnostic::error(message)
            })?;

        if let Some(printer_options) = printer_options {
            self.print_doc_json(&result, &printer_options).map_err(|err| {
                OxcDiagnostic::error(format!("Doc path failed for {}: {err}", path.display()))
            })
        } else {
            Ok(result)
        }
    }

    /// Convert Doc JSON string to formatted code via IR.
    fn print_doc_json(
        &self,
        doc_json_str: &str,
        printer_options: &PrinterOptions,
    ) -> Result<String, String> {
        use crate::prettier_compat::from_prettier_doc;
        use oxc_formatter::Printer;

        let allocator = self.allocator_pool.get();
        let group_id_builder = UniqueGroupIdBuilder::default();

        let doc_json: serde_json::Value = serde_json::from_str(doc_json_str)
            .map_err(|e| format!("Failed to parse Doc JSON: {e}"))?;

        let elements = from_prettier_doc::to_format_elements_for_file(
            &doc_json,
            &allocator,
            &group_id_builder,
        )?;

        propagate_expand_elements(&elements);

        let printed = Printer::new(printer_options.clone(), &[])
            .print(&elements)
            .map_err(|e| format!("Failed to print: {e}"))?;

        Ok(printed.into_code())
    }

    /// Format `package.json`: optionally sort then format by external formatter.
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
        let source_text: std::borrow::Cow<'_, str> = if let Some(options) = sort_options {
            match sort_package_json::sort_package_json_with_options(source_text, options) {
                Ok(sorted) => std::borrow::Cow::Owned(sorted),
                // `sort_package_json` can only handle strictly valid JSON.
                // On the other hand, Prettier's `json-stringify` parser is very permissive.
                // It can format JSON like input even with unquoted keys or trailing commas.
                // Therefore, rather than bailing out due to a sorting failure, we opt to format without sorting.
                Err(_) => std::borrow::Cow::Borrowed(source_text),
            }
        } else {
            std::borrow::Cow::Borrowed(source_text)
        };

        self.format_by_external_formatter(&source_text, path, parser_name, external_options)
    }
}
