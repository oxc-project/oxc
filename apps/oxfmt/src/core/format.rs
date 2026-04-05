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

fn trim_single_trailing_linebreak_len(text: &str) -> Option<usize> {
    text.strip_suffix("\r\n")
        .map(str::len)
        .or_else(|| text.strip_suffix('\n').map(str::len))
        .or_else(|| text.strip_suffix('\r').map(str::len))
}

fn should_preserve_external_missing_final_newline(
    entry: &FormatFileStrategy,
    source_text: &str,
    formatted_text: &str,
) -> bool {
    if !matches!(
        entry,
        FormatFileStrategy::ExternalFormatter { .. }
            | FormatFileStrategy::ExternalFormatterPackageJson { .. }
    ) || source_text.ends_with('\n')
        || source_text.ends_with('\r')
    {
        return false;
    }

    if source_text == formatted_text {
        return true;
    }

    let Some(trimmed_len) = trim_single_trailing_linebreak_len(formatted_text) else {
        return false;
    };

    source_text == &formatted_text[..trimmed_len]
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
                if insert_final_newline
                    && should_preserve_external_missing_final_newline(entry, source_text, &code)
                {
                    preserve_external_missing_final_newline(&mut code);
                } else {
                    apply_final_newline(&mut code, insert_final_newline);
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

        // Set `parser` and `filepath` on options for Prettier.
        // We specify `parser` to skip parser inference for perf,
        // and `filepath` because some plugins depend on it.
        if let Value::Object(ref mut map) = external_options {
            map.insert("parser".to_string(), Value::String(parser_name.to_string()));
            map.insert("filepath".to_string(), Value::String(path.to_string_lossy().to_string()));
        }

        external_formatter.format_file(external_options, source_text).map_err(|err| {
            // NOTE: We are trying to make the error from oxc_formatter and external_formatter (Prettier) look similar.
            // Ideally, we would unify them into `OxcDiagnostic`,
            // which would eliminate the need for relative path conversion.
            // However, doing so would require:
            // - Parsing Prettier's error messages
            // - Converting span information from UTF-16 to UTF-8
            // This is a non-trivial amount of work, so for now, just leave this as a best effort.
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
        })
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
        let source_text: Cow<'_, str> = if let Some(options) = sort_options {
            match sort_package_json::sort_package_json_with_options(source_text, options) {
                Ok(sorted) => Cow::Owned(sorted),
                // `sort_package_json` can only handle strictly valid JSON.
                // On the other hand, Prettier's `json-stringify` parser is very permissive.
                // It can format JSON like input even with unquoted keys or trailing commas.
                // Therefore, rather than bailing out due to a sorting failure, we opt to format without sorting.
                Err(_) => Cow::Borrowed(source_text),
            }
        } else {
            Cow::Borrowed(source_text)
        };

        self.format_by_external_formatter(&source_text, path, parser_name, external_options)
    }
}

fn preserve_external_missing_final_newline(code: &mut String) {
    if let Some(trimmed_len) = trim_single_trailing_linebreak_len(code) {
        code.truncate(trimmed_len);
    }
}

fn apply_final_newline(code: &mut String, insert_final_newline: bool) {
    if insert_final_newline {
        if !code.ends_with('\n') {
            code.push('\n');
        }
        return;
    }

    let trimmed_len = code.trim_end().len();
    code.truncate(trimmed_len);
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, path::PathBuf};

    use crate::core::FormatFileStrategy;

    use super::{
        apply_final_newline, preserve_external_missing_final_newline,
        should_preserve_external_missing_final_newline, trim_single_trailing_linebreak_len,
    };

    #[test]
    fn adds_missing_final_newline_when_enabled() {
        let mut code = "<h1>Hello</h1>".to_string();
        apply_final_newline(&mut code, true);
        assert_eq!(code, "<h1>Hello</h1>\n");
    }

    #[test]
    fn keeps_existing_final_newline_when_enabled() {
        let mut code = "<h1>Hello</h1>\n".to_string();
        apply_final_newline(&mut code, true);
        assert_eq!(code, "<h1>Hello</h1>\n");
    }

    #[test]
    fn trims_trailing_whitespace_when_disabled() {
        let mut code = "<h1>Hello</h1>\n\n".to_string();
        apply_final_newline(&mut code, false);
        assert_eq!(code, "<h1>Hello</h1>");
    }

    #[test]
    fn preserves_external_missing_final_newline_without_panicking_when_formatter_returns_same_text() {
        let mut code = String::new();
        preserve_external_missing_final_newline(&mut code);
        assert_eq!(code, "");

        let mut code = "<h1>Hello</h1>".to_string();
        preserve_external_missing_final_newline(&mut code);
        assert_eq!(code, "<h1>Hello</h1>");
    }

    #[test]
    fn trims_external_missing_final_newline_when_formatter_added_one() {
        let mut code = "<h1>Hello</h1>\r\n".to_string();
        preserve_external_missing_final_newline(&mut code);
        assert_eq!(code, "<h1>Hello</h1>");
    }

    #[test]
    fn preserves_external_formatter_newline_only_diffs() {
        let entry = FormatFileStrategy::ExternalFormatter {
            path: PathBuf::from("AlreadyFormattedNoFinalNewline.svelte"),
            parser_name: Cow::Borrowed("svelte"),
        };

        assert!(should_preserve_external_missing_final_newline(
            &entry,
            "<h1>Hello</h1>",
            "<h1>Hello</h1>",
        ));
        assert!(should_preserve_external_missing_final_newline(
            &entry,
            "",
            "",
        ));
        assert!(should_preserve_external_missing_final_newline(
            &entry,
            "<h1>Hello</h1>",
            "<h1>Hello</h1>\n",
        ));
        assert!(should_preserve_external_missing_final_newline(
            &entry,
            "<h1>Hello</h1>",
            "<h1>Hello</h1>\r\n",
        ));
        assert_eq!(
            trim_single_trailing_linebreak_len("<h1>Hello</h1>\r\n"),
            Some("<h1>Hello</h1>".len()),
        );
    }

    #[test]
    fn does_not_preserve_external_newline_when_other_text_changes() {
        let entry = FormatFileStrategy::ExternalFormatterPackageJson {
            path: PathBuf::from("package.json"),
            parser_name: Cow::Borrowed("json-stringify"),
        };

        assert!(!should_preserve_external_missing_final_newline(
            &entry,
            "{\"a\":1}",
            "{\n  \"a\": 1\n}\n",
        ));
    }

    #[test]
    fn does_not_preserve_for_non_external_entries() {
        let entry = FormatFileStrategy::OxcFormatter {
            path: PathBuf::from("a.ts"),
            source_type: oxc_span::SourceType::default().with_module(true),
        };

        assert!(!should_preserve_external_missing_final_newline(
            &entry,
            "const answer = 42;",
            "const answer = 42;\n",
        ));
    }
}
