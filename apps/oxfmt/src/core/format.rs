use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde_json::Value;
use tracing::instrument;

use oxc_allocator::AllocatorPool;
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter::{FormatOptions, Formatter, enable_jsx_source_type, get_parse_options};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_toml::Options as TomlFormatterOptions;

use super::{
    oxfmtrc::{OxfmtOptions, finalize_external_options},
    support::FileKind,
};

/// A resolved formatting target: classification + per-file resolved options.
///
/// Built by [`super::ConfigResolver::resolve`] (or the NAPI variant-specific helpers)
/// from a [`FileKind`] and the resolved configuration for that file.
/// Each variant carries everything needed to format the file.
#[derive(Debug)]
pub enum FormatStrategy {
    /// For JS/TS files formatted by oxc_formatter.
    OxcFormatter {
        path: Arc<Path>,
        source_type: SourceType,
        format_options: Box<FormatOptions>,
        /// For embedded language (xxx-in-js) formatting
        external_options: Value,
        /// Optional filepath override for external callbacks (e.g., Tailwind sorter).
        /// When set, this path is used instead of `path` as the `options.filepath`
        /// passed to external callbacks.
        /// Needed for js-in-xxx where the path is a dummy, but callbacks need the
        /// parent file path to resolve their config.
        filepath_override: Option<PathBuf>,
        insert_final_newline: bool,
    },
    /// For TOML files.
    OxfmtToml { path: Arc<Path>, toml_options: TomlFormatterOptions, insert_final_newline: bool },
    /// For non-JS files formatted by external formatter (Prettier).
    #[cfg(feature = "napi")]
    ExternalFormatter {
        path: Arc<Path>,
        parser_name: &'static str,
        external_options: Value,
        insert_final_newline: bool,
    },
    /// For `package.json` files: optionally sorted then formatted.
    #[cfg(feature = "napi")]
    ExternalFormatterPackageJson {
        path: Arc<Path>,
        parser_name: &'static str,
        external_options: Value,
        sort_package_json: Option<sort_package_json::SortOptions>,
        insert_final_newline: bool,
    },
}

impl FormatStrategy {
    pub fn path(&self) -> &Arc<Path> {
        match self {
            Self::OxcFormatter { path, .. } | Self::OxfmtToml { path, .. } => path,
            #[cfg(feature = "napi")]
            Self::ExternalFormatter { path, .. }
            | Self::ExternalFormatterPackageJson { path, .. } => path,
        }
    }

    /// Build `FormatStrategy` from a [`FileKind`], `OxfmtOptions`, and external options.
    ///
    /// Also applies plugin-specific options (Tailwind, oxfmt plugin flags) based on file kind.
    pub(crate) fn from_oxfmt_options(
        oxfmt_options: OxfmtOptions,
        mut external_options: Value,
        kind: FileKind,
    ) -> Self {
        finalize_external_options(&mut external_options, &kind);

        #[cfg(feature = "napi")]
        let OxfmtOptions { format_options, toml_options, sort_package_json, insert_final_newline } =
            oxfmt_options;
        #[cfg(not(feature = "napi"))]
        let OxfmtOptions { format_options, toml_options, insert_final_newline, .. } = oxfmt_options;

        match kind {
            FileKind::OxcFormatter { path, source_type } => Self::OxcFormatter {
                path,
                source_type,
                format_options: Box::new(format_options),
                external_options,
                filepath_override: None,
                insert_final_newline,
            },
            FileKind::OxfmtToml { path } => {
                Self::OxfmtToml { path, toml_options, insert_final_newline }
            }
            #[cfg(feature = "napi")]
            FileKind::ExternalFormatter { path, parser_name } => Self::ExternalFormatter {
                path,
                parser_name,
                external_options,
                insert_final_newline,
            },
            #[cfg(feature = "napi")]
            FileKind::ExternalFormatterPackageJson { path, parser_name } => {
                Self::ExternalFormatterPackageJson {
                    path,
                    parser_name,
                    external_options,
                    sort_package_json,
                    insert_final_newline,
                }
            }
        }
    }
}

// ---

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

    /// Format a file based on its resolved strategy.
    #[instrument(level = "debug", name = "oxfmt::format", skip_all, fields(path = %resolved.path().display()))]
    pub fn format(&self, source_text: &str, resolved: FormatStrategy) -> FormatResult {
        let (result, insert_final_newline) = match resolved {
            FormatStrategy::OxcFormatter {
                path,
                source_type,
                format_options,
                external_options,
                filepath_override,
                insert_final_newline,
            } => (
                self.format_by_oxc_formatter(
                    source_text,
                    &path,
                    source_type,
                    *format_options,
                    external_options,
                    filepath_override.as_deref(),
                ),
                insert_final_newline,
            ),
            FormatStrategy::OxfmtToml { toml_options, insert_final_newline, .. } => {
                (Ok(Self::format_by_toml(source_text, toml_options)), insert_final_newline)
            }
            #[cfg(feature = "napi")]
            FormatStrategy::ExternalFormatter {
                path,
                parser_name,
                external_options,
                insert_final_newline,
            } => (
                self.format_by_external_formatter(
                    source_text,
                    &path,
                    parser_name,
                    external_options,
                ),
                insert_final_newline,
            ),
            #[cfg(feature = "napi")]
            FormatStrategy::ExternalFormatterPackageJson {
                path,
                parser_name,
                external_options,
                sort_package_json,
                insert_final_newline,
            } => (
                self.format_by_external_formatter_package_json(
                    source_text,
                    &path,
                    parser_name,
                    external_options,
                    sort_package_json.as_ref(),
                ),
                insert_final_newline,
            ),
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
        use std::borrow::Cow;
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
