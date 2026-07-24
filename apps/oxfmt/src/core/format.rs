use std::{path::Path, sync::Arc};

use tracing::instrument;

use oxc_allocator::AllocatorPool;
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter::JsFormatOptions;
use oxc_formatter_css::CssFormatOptions;
#[cfg(feature = "napi")]
use oxc_formatter_css::CssVariant;
use oxc_formatter_graphql::GraphqlFormatOptions;
use oxc_formatter_json::{JsonFormatOptions, JsonVariant};
use oxc_formatter_yaml::YamlFormatOptions;
use oxc_span::SourceType;
use oxc_toml::Options as TomlFormatterOptions;

#[cfg(feature = "napi")]
use super::options::{
    inject_filepath, inject_oxfmt_plugin_payload, inject_parser, inject_svelte_plugin_payload,
    inject_tailwind_plugin_payload, to_prettier,
};
use super::{
    options::{
        to_oxc_formatter, to_oxc_formatter_css, to_oxc_formatter_graphql, to_oxc_formatter_json,
        to_oxc_formatter_yaml, to_oxc_toml, to_sort_package_json,
    },
    oxfmtrc::FormatConfig,
    support::FileKind,
};

/// A resolved formatting target: classification + per-file resolved options.
///
/// Built by [`super::ConfigResolver::resolve`] (or the NAPI variant-specific helpers)
/// from a [`FileKind`] and the resolved [`FormatConfig`] for that file.
/// Each variant carries everything needed to format the file.
///
/// Prettier `Value` options are derived from `config` at the format step, not stored.
#[derive(Debug)]
pub enum FormatStrategy {
    /// For JS/TS files formatted by oxc_formatter.
    /// `config` is retained so embedded callbacks (e.g., xxx-in-js) can lazily
    /// derive Prettier options at callback time.
    OxcFormatter {
        path: Arc<Path>,
        source_type: SourceType,
        format_options: Box<JsFormatOptions>,
        #[cfg(feature = "napi")]
        config: Box<FormatConfig>,
        insert_final_newline: bool,
    },
    /// For JSON (and JSON-like) files formatted by `oxc_formatter_json`.
    OxcFormatterJson {
        path: Arc<Path>,
        format_options: Box<JsonFormatOptions>,
        insert_final_newline: bool,
    },
    /// For `package.json` files: optionally sorted, then formatted by
    /// `oxc_formatter_json` with the `json-stringify` variant.
    OxcFormatterJsonPackageJson {
        path: Arc<Path>,
        format_options: Box<JsonFormatOptions>,
        sort_package_json: Option<sort_package_json::SortOptions>,
        insert_final_newline: bool,
    },
    /// For GraphQL files formatted by `oxc_formatter_graphql`.
    OxcFormatterGraphql {
        path: Arc<Path>,
        format_options: Box<GraphqlFormatOptions>,
        insert_final_newline: bool,
    },
    /// For CSS/SCSS/Less files formatted by `oxc_formatter_css`.
    /// `config` is retained (napi only) to build the Tailwind class sorter options.
    OxcFormatterCss {
        path: Arc<Path>,
        format_options: Box<CssFormatOptions>,
        #[cfg(feature = "napi")]
        config: Box<FormatConfig>,
        insert_final_newline: bool,
    },
    /// For YAML files formatted by `oxc_formatter_yaml`.
    OxcFormatterYaml {
        path: Arc<Path>,
        format_options: Box<YamlFormatOptions>,
        insert_final_newline: bool,
    },
    /// For `.prettierrc` and friends: mirroring Prettier's yaml embed,
    /// formatted by `oxc_formatter_json` when the whole text parses as JSON,
    /// and by `oxc_formatter_yaml` otherwise.
    OxcFormatterYamlRc {
        path: Arc<Path>,
        yaml_format_options: Box<YamlFormatOptions>,
        json_format_options: Box<JsonFormatOptions>,
        insert_final_newline: bool,
    },
    /// For TOML files.
    OxfmtToml { path: Arc<Path>, toml_options: TomlFormatterOptions, insert_final_newline: bool },
    /// For non-JS files formatted by external formatter (Prettier).
    ///
    /// `supports_xxx` are capability flags carried over from [`FileKind::ExternalFormatter`].
    /// The format step injects the corresponding payload (`_useXxxPlugin`) only when
    /// the capability AND the user config both enable the plugin.
    ///
    /// When `supports_oxfmt` is true, `config` doubles as the host Prettier options
    /// source AND the `_oxfmtPluginOptionsJson` payload — single SoT for both.
    #[cfg(feature = "napi")]
    ExternalFormatter {
        path: Arc<Path>,
        parser_name: &'static str,
        config: Box<FormatConfig>,
        supports_tailwind: bool,
        supports_oxfmt: bool,
        supports_svelte: bool,
        insert_final_newline: bool,
    },
}

impl FormatStrategy {
    pub fn path(&self) -> &Arc<Path> {
        match self {
            Self::OxcFormatter { path, .. }
            | Self::OxcFormatterJson { path, .. }
            | Self::OxcFormatterJsonPackageJson { path, .. }
            | Self::OxcFormatterGraphql { path, .. }
            | Self::OxcFormatterCss { path, .. }
            | Self::OxcFormatterYaml { path, .. }
            | Self::OxcFormatterYamlRc { path, .. }
            | Self::OxfmtToml { path, .. } => path,
            #[cfg(feature = "napi")]
            Self::ExternalFormatter { path, .. } => path,
        }
    }

    /// Build a `FormatStrategy` from a typed [`FormatConfig`] and a [`FileKind`].
    ///
    /// `to_oxc_formatter` / `to_oxc_toml` run eagerly: their validating
    /// typed conversion belongs at carving so the format step stays infallible.
    /// The Prettier `Value` for `ExternalFormatter*` is deferred:
    /// `FormatConfig` is the single SoT, no validation needed,
    /// and `Box<FormatConfig>` is materially smaller per file than a fully-built `Value`.
    ///
    /// # Errors
    /// Returns `Err` if the kind needs `JsFormatOptions`/`TomlFormatterOptions`
    /// and the config fails validation.
    // `config` is moved into the napi-only `ExternalFormatter*` variants;
    // when the `napi` feature is off, those branches are cfg-gated out and the
    // value is only borrowed, but we keep the by-value signature for symmetry.
    #[cfg_attr(not(feature = "napi"), expect(clippy::needless_pass_by_value))]
    pub(crate) fn from_format_config(config: FormatConfig, kind: FileKind) -> Result<Self, String> {
        let insert_final_newline = config.insert_final_newline.unwrap_or(true);

        Ok(match kind {
            FileKind::OxcFormatter { path, source_type } => Self::OxcFormatter {
                path,
                source_type,
                format_options: Box::new(to_oxc_formatter(&config)?),
                #[cfg(feature = "napi")]
                config: Box::new(config),
                insert_final_newline,
            },
            FileKind::OxcFormatterJson { path, variant } => Self::OxcFormatterJson {
                path,
                format_options: Box::new(to_oxc_formatter_json(&config, variant)?),
                insert_final_newline,
            },
            FileKind::OxcFormatterJsonPackageJson { path } => Self::OxcFormatterJsonPackageJson {
                path,
                format_options: Box::new(to_oxc_formatter_json(
                    &config,
                    JsonVariant::JsonStringify,
                )?),
                sort_package_json: to_sort_package_json(&config),
                insert_final_newline,
            },
            FileKind::OxcFormatterGraphql { path } => Self::OxcFormatterGraphql {
                path,
                format_options: Box::new(to_oxc_formatter_graphql(&config)?),
                insert_final_newline,
            },
            FileKind::OxcFormatterCss { path, variant } => Self::OxcFormatterCss {
                path,
                format_options: Box::new(to_oxc_formatter_css(&config, variant)?),
                #[cfg(feature = "napi")]
                config: Box::new(config),
                insert_final_newline,
            },
            FileKind::OxcFormatterYaml { path } => Self::OxcFormatterYaml {
                path,
                format_options: Box::new(to_oxc_formatter_yaml(&config)?),
                insert_final_newline,
            },
            FileKind::OxcFormatterYamlRc { path } => Self::OxcFormatterYamlRc {
                path,
                yaml_format_options: Box::new(to_oxc_formatter_yaml(&config)?),
                json_format_options: Box::new(to_oxc_formatter_json(&config, JsonVariant::Json)?),
                insert_final_newline,
            },
            FileKind::OxfmtToml { path } => {
                Self::OxfmtToml { path, toml_options: to_oxc_toml(&config)?, insert_final_newline }
            }
            #[cfg(feature = "napi")]
            FileKind::ExternalFormatter {
                path,
                parser_name,
                supports_tailwind,
                supports_oxfmt,
                supports_svelte,
            } => Self::ExternalFormatter {
                path,
                parser_name,
                config: Box::new(config),
                supports_tailwind,
                supports_oxfmt,
                supports_svelte,
                insert_final_newline,
            },
        })
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
        // Early return for empty files to avoid unnecessary formatting work.
        // > Editors must not insert newlines in empty files when saving those files,
        // > even if insert_final_newline = true.
        // https://spec.editorconfig.org/#supported-pairs
        if source_text.trim().is_empty() {
            return FormatResult::Success {
                is_changed: !source_text.is_empty(),
                code: String::new(),
            };
        }

        let (result, insert_final_newline) = match resolved {
            FormatStrategy::OxcFormatter {
                path,
                source_type,
                format_options,
                #[cfg(feature = "napi")]
                config,
                insert_final_newline,
            } => (
                self.format_by_oxc_formatter(
                    source_text,
                    &path,
                    source_type,
                    *format_options,
                    #[cfg(feature = "napi")]
                    &config,
                ),
                insert_final_newline,
            ),
            FormatStrategy::OxcFormatterJson { path, format_options, insert_final_newline } => (
                self.format_by_oxc_formatter_json(source_text, &path, *format_options),
                insert_final_newline,
            ),
            FormatStrategy::OxcFormatterJsonPackageJson {
                path,
                format_options,
                sort_package_json,
                insert_final_newline,
            } => (
                self.format_by_oxc_formatter_json_package_json(
                    source_text,
                    &path,
                    *format_options,
                    sort_package_json.as_ref(),
                ),
                insert_final_newline,
            ),
            FormatStrategy::OxcFormatterGraphql { path, format_options, insert_final_newline } => (
                self.format_by_oxc_formatter_graphql(source_text, &path, *format_options),
                insert_final_newline,
            ),
            FormatStrategy::OxcFormatterCss {
                path,
                format_options,
                #[cfg(feature = "napi")]
                config,
                insert_final_newline,
            } => (
                self.format_by_oxc_formatter_css(
                    source_text,
                    &path,
                    *format_options,
                    #[cfg(feature = "napi")]
                    &config,
                ),
                insert_final_newline,
            ),
            FormatStrategy::OxcFormatterYaml { path, format_options, insert_final_newline } => (
                self.format_by_oxc_formatter_yaml(source_text, &path, *format_options),
                insert_final_newline,
            ),
            FormatStrategy::OxcFormatterYamlRc {
                path,
                yaml_format_options,
                json_format_options,
                insert_final_newline,
            } => (
                self.format_by_oxc_formatter_yaml_rc(
                    source_text,
                    &path,
                    *yaml_format_options,
                    *json_format_options,
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
                config,
                supports_tailwind,
                supports_oxfmt,
                supports_svelte,
                insert_final_newline,
            } => (
                self.format_by_external_formatter(
                    source_text,
                    &path,
                    parser_name,
                    &config,
                    supports_tailwind,
                    supports_oxfmt,
                    supports_svelte,
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
    /// `config` is needed to derive Prettier options for embedded callbacks (xxx-in-js, Tailwind).
    #[instrument(level = "debug", name = "oxfmt::format::oxc_formatter", skip_all)]
    fn format_by_oxc_formatter(
        &self,
        source_text: &str,
        path: &Path,
        source_type: SourceType,
        format_options: JsFormatOptions,
        #[cfg(feature = "napi")] config: &FormatConfig,
    ) -> Result<String, OxcDiagnostic> {
        let allocator = self.allocator_pool.get();

        #[cfg(feature = "napi")]
        let external_callbacks = Some(self.build_external_callbacks(&format_options, config, path));
        #[cfg(not(feature = "napi"))]
        let external_callbacks = {
            let _ = path;
            None
        };

        let formatted = oxc_formatter::format(
            &allocator,
            source_text,
            source_type,
            format_options,
            external_callbacks,
        )?;

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

    /// Format JSON (and JSON-like) source using `oxc_formatter_json`.
    #[instrument(level = "debug", name = "oxfmt::format::oxc_formatter_json", skip_all)]
    fn format_by_oxc_formatter_json(
        &self,
        source_text: &str,
        path: &Path,
        format_options: JsonFormatOptions,
    ) -> Result<String, OxcDiagnostic> {
        let allocator = self.allocator_pool.get();
        let formatted = oxc_formatter_json::format(&allocator, source_text, format_options)?;
        let printed = formatted.print().map_err(|err| {
            OxcDiagnostic::error(format!(
                "Failed to print formatted JSON: {}\n{err}",
                path.display()
            ))
        })?;
        Ok(printed.into_code())
    }

    /// Format `package.json`: optionally sort, then format using `oxc_formatter_json`.
    #[instrument(
        level = "debug",
        name = "oxfmt::format::oxc_formatter_json_package_json",
        skip_all
    )]
    fn format_by_oxc_formatter_json_package_json(
        &self,
        source_text: &str,
        path: &Path,
        format_options: JsonFormatOptions,
        sort_options: Option<&sort_package_json::SortOptions>,
    ) -> Result<String, OxcDiagnostic> {
        use std::borrow::Cow;
        let source_text: Cow<'_, str> = if let Some(options) = sort_options {
            match sort_package_json::sort_package_json_with_options(source_text, options) {
                Ok(sorted) => Cow::Owned(sorted),
                // `sort_package_json` can only handle strictly valid JSON.
                // On the other hand, the `json-stringify` variant parser is very permissive.
                // It can format JSON like input even with unquoted keys or trailing commas.
                // Therefore, rather than bailing out due to a sorting failure, we opt to format without sorting.
                Err(_) => Cow::Borrowed(source_text),
            }
        } else {
            Cow::Borrowed(source_text)
        };

        self.format_by_oxc_formatter_json(&source_text, path, format_options)
    }

    /// Format GraphQL source using `oxc_formatter_graphql`.
    #[instrument(level = "debug", name = "oxfmt::format::oxc_formatter_graphql", skip_all)]
    fn format_by_oxc_formatter_graphql(
        &self,
        source_text: &str,
        path: &Path,
        format_options: GraphqlFormatOptions,
    ) -> Result<String, OxcDiagnostic> {
        let allocator = self.allocator_pool.get();
        let formatted = oxc_formatter_graphql::format(&allocator, source_text, format_options)?;
        let printed = formatted.print().map_err(|err| {
            OxcDiagnostic::error(format!(
                "Failed to print formatted GraphQL: {}\n{err}",
                path.display()
            ))
        })?;
        Ok(printed.into_code())
    }

    /// Format CSS/SCSS/Less source using `oxc_formatter_css`.
    ///
    /// When the config enables Tailwind class sorting,
    /// the napi build passes a JS-side sorter for the `@apply` classes the formatter collects
    /// (the order itself comes from the Tailwind config, which only the JS side can resolve).
    /// The pure build never collects classes.
    #[instrument(level = "debug", name = "oxfmt::format::oxc_formatter_css", skip_all)]
    fn format_by_oxc_formatter_css(
        &self,
        source_text: &str,
        path: &Path,
        format_options: CssFormatOptions,
        #[cfg(feature = "napi")] config: &FormatConfig,
    ) -> Result<String, OxcDiagnostic> {
        #[cfg(feature = "napi")]
        let sorter = self.tailwind_sorter(config, path);
        #[cfg(not(feature = "napi"))]
        let sorter: Option<fn(Vec<String>) -> Vec<String>> = None;

        let allocator = self.allocator_pool.get();
        let formatted = oxc_formatter_css::format(
            &allocator,
            source_text,
            format_options,
            sorter.as_ref().map(|s| s as &dyn Fn(Vec<String>) -> Vec<String>),
        )?;
        let printed = formatted.print().map_err(|err| {
            OxcDiagnostic::error(format!(
                "Failed to print formatted CSS: {}\n{err}",
                path.display()
            ))
        })?;
        Ok(printed.into_code())
    }

    /// Format YAML source using `oxc_formatter_yaml`.
    #[instrument(level = "debug", name = "oxfmt::format::oxc_formatter_yaml", skip_all)]
    fn format_by_oxc_formatter_yaml(
        &self,
        source_text: &str,
        path: &Path,
        format_options: YamlFormatOptions,
    ) -> Result<String, OxcDiagnostic> {
        let allocator = self.allocator_pool.get();
        let formatted = oxc_formatter_yaml::format(&allocator, source_text, format_options)?;
        let printed = formatted.print().map_err(|err| {
            OxcDiagnostic::error(format!(
                "Failed to print formatted YAML: {}\n{err}",
                path.display()
            ))
        })?;
        Ok(printed.into_code())
    }

    /// Format `.prettierrc` and friends the way Prettier's yaml embed does.
    #[instrument(level = "debug", name = "oxfmt::format::oxc_formatter_yaml_rc", skip_all)]
    fn format_by_oxc_formatter_yaml_rc(
        &self,
        source_text: &str,
        path: &Path,
        yaml_format_options: YamlFormatOptions,
        json_format_options: JsonFormatOptions,
    ) -> Result<String, OxcDiagnostic> {
        if let Ok(printed) =
            self.format_by_oxc_formatter_json(source_text, path, json_format_options)
        {
            return Ok(printed);
        }
        self.format_by_oxc_formatter_yaml(source_text, path, yaml_format_options)
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

    /// Build the Prettier options JSON shared by the embedded callbacks and the Tailwind sorter:
    /// resolved config + `filepath` + the Tailwind plugin payload
    /// (which the JS-side sorter resolves the class order from).
    fn build_external_options(config: &FormatConfig, path: &Path) -> serde_json::Value {
        let mut external_options = to_prettier(config);
        inject_filepath(&mut external_options, path);
        inject_tailwind_plugin_payload(&mut external_options, config);
        external_options
    }

    /// Build the JS-side Tailwind class sorter for `oxc_formatter_css`'s `@apply` collection,
    /// or `None` when the config does not enable it.
    fn tailwind_sorter(
        &self,
        config: &FormatConfig,
        path: &Path,
    ) -> Option<impl Fn(Vec<String>) -> Vec<String>> {
        config.is_tailwind_enabled().then(|| {
            let external_formatter = self
                .external_formatter
                .as_ref()
                .expect("`external_formatter` must exist when `napi` feature is enabled");
            let sort = std::sync::Arc::clone(&external_formatter.sort_tailwindcss_classes);
            let external_options = Self::build_external_options(config, path);
            move |classes: Vec<String>| sort(&external_options, classes)
        })
    }

    /// Build external callbacks for `oxc_formatter` from the NAPI external formatter.
    ///
    /// Tailwind is always considered "capable" here because `oxc_formatter` embeds the sorter internally;
    /// the inject helper itself decides whether to fire based on user config.
    fn build_external_callbacks(
        &self,
        format_options: &JsFormatOptions,
        config: &FormatConfig,
        path: &Path,
    ) -> oxc_formatter::ExternalCallbacks {
        let external_formatter = self
            .external_formatter
            .as_ref()
            .expect("`external_formatter` must exist when `napi` feature is enabled");

        let external_options = Self::build_external_options(config, path);

        // Dual mapping of the same resolved config for the dispatcher's Rust branches.
        // Cannot fail here: building `JsFormatOptions` from this config already succeeded,
        // and both share the same `to_core_options()` validation.
        let graphql_options = to_oxc_formatter_graphql(config)
            .expect("config was already validated when building `JsFormatOptions`");
        // CSS-in-JS is always parsed as SCSS (Prettier's embed uses the
        // `scss` parser for all of css/scss/less template tags).
        let css_options = to_oxc_formatter_css(config, CssVariant::Scss)
            .expect("config was already validated when building `JsFormatOptions`");

        external_formatter.to_external_callbacks(
            format_options,
            external_options,
            graphql_options,
            css_options,
        )
    }

    /// Format non-JS/TS file using external formatter (Prettier).
    ///
    /// Plugin payloads are injected based on capability flags & user config.
    #[instrument(level = "debug", name = "oxfmt::format::external_formatter", skip_all, fields(parser = %parser_name))]
    fn format_by_external_formatter(
        &self,
        source_text: &str,
        path: &Path,
        parser_name: &str,
        config: &FormatConfig,
        supports_tailwind: bool,
        supports_oxfmt: bool,
        supports_svelte: bool,
    ) -> Result<String, OxcDiagnostic> {
        let mut external_options = to_prettier(config);
        inject_parser(&mut external_options, parser_name);
        inject_filepath(&mut external_options, path);

        if supports_tailwind {
            inject_tailwind_plugin_payload(&mut external_options, config);
        }
        if supports_oxfmt {
            inject_oxfmt_plugin_payload(&mut external_options, config, path);
        }
        if supports_svelte {
            inject_svelte_plugin_payload(&mut external_options, config);
        }

        let external_formatter = self
            .external_formatter
            .as_ref()
            .expect("`external_formatter` must exist when `napi` feature is enabled");

        external_formatter.format_file(external_options, source_text).map_err(|err| {
            // NOTE: We are trying to make the error from oxc_formatter(_xxx) and external_formatter (Prettier) look similar.
            // Ideally, we would unify them into `OxcDiagnostic`, which would eliminate the need for relative path conversion.
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
}
