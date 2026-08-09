use std::{path::Path, sync::Arc};

use tracing::instrument;

use oxc_allocator::AllocatorPool;
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter::JsFormatOptions;
use oxc_formatter_core::{CoreFormatOptions, FormatSession, InputKind, SessionServices};
use oxc_formatter_css::CssFormatOptions;
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
    embed::dispatcher::ResolvedDispatchConfig,
    options::{
        ValidatedOptions, to_oxc_formatter, to_oxc_formatter_css, to_oxc_formatter_graphql,
        to_oxc_formatter_json, to_oxc_formatter_yaml, to_oxc_toml, to_sort_package_json,
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
    /// `config` + `core` build the dispatch config for the root's session
    /// (native xxx-in-js in every build, plus the napi embedded callbacks / Tailwind).
    OxcFormatter {
        path: Arc<Path>,
        source_type: SourceType,
        format_options: Box<JsFormatOptions>,
        config: Arc<FormatConfig>,
        /// The validated core bundle, carried from resolution so dispatch-config
        /// construction never re-derives (or re-fails) it.
        core: CoreFormatOptions,
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
    /// `config` + `core` build the dispatch config for the root's session
    /// (front matter YAML, plus the napi Tailwind sorter options).
    OxcFormatterCss {
        path: Arc<Path>,
        format_options: Box<CssFormatOptions>,
        config: Arc<FormatConfig>,
        /// The validated core bundle, carried from resolution so dispatch-config
        /// construction never re-derives (or re-fails) it.
        core: CoreFormatOptions,
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
    /// For non-JS files formatted by delegating to Prettier (Tier 3/4).
    ///
    /// `supports_xxx` are capability flags carried over from [`FileKind::Prettier`].
    /// The format step injects the corresponding payload (`_useXxxPlugin`) only when
    /// the capability AND the user config both enable the plugin.
    ///
    /// When `supports_oxfmt` is true, `config` doubles as the host Prettier options
    /// source AND the `_oxfmtPluginOptionsJson` payload — single SoT for both.
    #[cfg(feature = "napi")]
    Prettier {
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
            Self::Prettier { path, .. } => path,
        }
    }

    /// Build a `FormatStrategy` from a typed [`FormatConfig`], the validation gate's artifacts, and a [`FileKind`].
    ///
    /// Infallible by construction:
    /// every fallible conversion already ran at the gate (`options::validate`),
    /// whose derived values arrive as `validated`; the option mappers below are pure field translation.
    ///
    /// The Prettier `Value` for the `Prettier` kind is deferred to the format step:
    /// `FormatConfig` is the single SoT, no validation needed,
    /// and `Box<FormatConfig>` is materially smaller per file than a fully-built `Value`.
    pub(crate) fn from_format_config(
        config: FormatConfig,
        validated: &ValidatedOptions,
        kind: FileKind,
    ) -> Self {
        let insert_final_newline = config.insert_final_newline.unwrap_or(true);
        // Borrowed so the resolver's fast path can hand out its cached artifacts per file;
        // `sort_imports` (the only non-`Copy` member) is cloned in the one arm that needs it.
        let core = validated.core;

        match kind {
            FileKind::OxcFormatter { path, source_type } => Self::OxcFormatter {
                path,
                source_type,
                format_options: Box::new(to_oxc_formatter(
                    &config,
                    core,
                    validated.sort_imports.clone(),
                )),
                config: Arc::new(config),
                core,
                insert_final_newline,
            },
            FileKind::OxcFormatterJson { path, variant } => Self::OxcFormatterJson {
                path,
                format_options: Box::new(to_oxc_formatter_json(&config, core, variant)),
                insert_final_newline,
            },
            FileKind::OxcFormatterJsonPackageJson { path } => Self::OxcFormatterJsonPackageJson {
                path,
                format_options: Box::new(to_oxc_formatter_json(
                    &config,
                    core,
                    JsonVariant::JsonStringify,
                )),
                sort_package_json: to_sort_package_json(&config),
                insert_final_newline,
            },
            FileKind::OxcFormatterGraphql { path } => Self::OxcFormatterGraphql {
                path,
                format_options: Box::new(to_oxc_formatter_graphql(&config, core)),
                insert_final_newline,
            },
            FileKind::OxcFormatterCss { path, variant } => Self::OxcFormatterCss {
                path,
                format_options: Box::new(to_oxc_formatter_css(&config, core, variant)),
                config: Arc::new(config),
                core,
                insert_final_newline,
            },
            FileKind::OxcFormatterYaml { path } => Self::OxcFormatterYaml {
                path,
                format_options: Box::new(to_oxc_formatter_yaml(&config, core)),
                insert_final_newline,
            },
            FileKind::OxcFormatterYamlRc { path } => Self::OxcFormatterYamlRc {
                path,
                yaml_format_options: Box::new(to_oxc_formatter_yaml(&config, core)),
                json_format_options: Box::new(to_oxc_formatter_json(
                    &config,
                    core,
                    JsonVariant::Json,
                )),
                insert_final_newline,
            },
            FileKind::OxfmtToml { path } => Self::OxfmtToml {
                path,
                toml_options: to_oxc_toml(&config, core),
                insert_final_newline,
            },
            #[cfg(feature = "napi")]
            FileKind::Prettier {
                path,
                parser_name,
                supports_tailwind,
                supports_oxfmt,
                supports_svelte,
            } => Self::Prettier {
                path,
                parser_name,
                config: Box::new(config),
                supports_tailwind,
                supports_oxfmt,
                supports_svelte,
                insert_final_newline,
            },
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
    external_services: Option<super::ExternalServices>,
}

impl SourceFormatter {
    pub fn new(num_of_threads: usize) -> Self {
        Self {
            allocator_pool: AllocatorPool::new(num_of_threads),
            #[cfg(feature = "napi")]
            external_services: None,
        }
    }

    /// The build's default `SessionServices` for a root run (`embed::services::for_root`),
    /// with the napi transport threaded in; the cfg fork lives here once, not per root.
    #[cfg_attr(not(feature = "napi"), expect(clippy::unused_self))]
    fn root_services(&self, dispatch_config: &Arc<ResolvedDispatchConfig>) -> SessionServices {
        #[cfg(feature = "napi")]
        {
            super::embed::services::for_root(self.external_services(), dispatch_config)
        }
        #[cfg(not(feature = "napi"))]
        {
            super::embed::services::for_root(dispatch_config)
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
                config,
                core,
                insert_final_newline,
            } => (
                self.format_by_oxc_formatter(
                    source_text,
                    &path,
                    source_type,
                    *format_options,
                    &config,
                    core,
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
                config,
                core,
                insert_final_newline,
            } => (
                self.format_by_oxc_formatter_css(
                    source_text,
                    &path,
                    *format_options,
                    &config,
                    core,
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
            FormatStrategy::Prettier {
                path,
                parser_name,
                config,
                supports_tailwind,
                supports_oxfmt,
                supports_svelte,
                insert_final_newline,
            } => (
                self.format_by_prettier(
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

    /// Format JS/TS source code using `oxc_formatter` on a `PhysicalFile` session
    /// carrying the registry dispatcher in every build
    /// (fallback-less without napi, so non-native embeds stay verbatim);
    /// the napi build adds the Prettier Doc→IR fallback and the string-channel callbacks.
    #[instrument(level = "debug", name = "oxfmt::format::oxc_formatter", skip_all)]
    fn format_by_oxc_formatter(
        &self,
        source_text: &str,
        path: &Path,
        source_type: SourceType,
        format_options: JsFormatOptions,
        config: &Arc<FormatConfig>,
        core: CoreFormatOptions,
    ) -> Result<String, OxcDiagnostic> {
        let allocator = self.allocator_pool.get();
        let session = {
            let dispatch_config = ResolvedDispatchConfig::for_root(config, core, path);
            let services = self.root_services(&dispatch_config);
            FormatSession::with_services(&allocator, InputKind::PhysicalFile, services)
        };

        let code = {
            let formatted = oxc_formatter::format_with_session(
                &session,
                source_text,
                source_type,
                format_options,
            )?;
            formatted.print().map_err(|err| {
                OxcDiagnostic::error(format!(
                    "Failed to print formatted code: {}\n{err}",
                    path.display()
                ))
            })?
        };

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

    /// Format CSS/SCSS/Less source using `oxc_formatter_css` on a `PhysicalFile` session
    /// carrying the build's default services.
    /// The crate's front matter gate dispatches only `yaml` / `toml`
    /// (yaml formats natively; toml and other languages stay verbatim, matching Prettier).
    /// `embeddedLanguageFormatting: off` installs no dispatcher and the block stays verbatim wholesale.
    /// Tailwind `@apply` classes sort via the napi sorter
    /// (the order comes from the Tailwind config, which only the JS side can resolve);
    /// the pure build never collects classes.
    #[instrument(level = "debug", name = "oxfmt::format::oxc_formatter_css", skip_all)]
    fn format_by_oxc_formatter_css(
        &self,
        source_text: &str,
        path: &Path,
        format_options: CssFormatOptions,
        config: &Arc<FormatConfig>,
        core: CoreFormatOptions,
    ) -> Result<String, OxcDiagnostic> {
        let allocator = self.allocator_pool.get();
        let session = {
            let dispatch_config = ResolvedDispatchConfig::for_root(config, core, path);
            let services = self.root_services(&dispatch_config);
            FormatSession::with_services(&allocator, InputKind::PhysicalFile, services)
        };

        let code = {
            let formatted =
                oxc_formatter_css::format_with_session(&session, source_text, format_options)?;
            formatted.print().map_err(|err| {
                OxcDiagnostic::error(format!(
                    "Failed to print formatted CSS: {}\n{err}",
                    path.display()
                ))
            })?
        };

        Ok(code.into_code())
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
/// These methods handle Prettier delegation,
/// which is only available when running through the Node.js NAPI bridge.
#[cfg(feature = "napi")]
impl SourceFormatter {
    #[must_use]
    pub fn with_external_services(
        mut self,
        external_services: Option<super::ExternalServices>,
    ) -> Self {
        self.external_services = external_services;
        self
    }

    /// The napi transport, installed by [`Self::with_external_services`] before any format run.
    fn external_services(&self) -> &super::ExternalServices {
        self.external_services
            .as_ref()
            .expect("`external_services` must exist when `napi` feature is enabled")
    }

    /// Format non-JS/TS file by delegating to Prettier.
    ///
    /// Plugin payloads are injected based on capability flags & user config.
    #[instrument(level = "debug", name = "oxfmt::format::prettier", skip_all, fields(parser = %parser_name))]
    fn format_by_prettier(
        &self,
        source_text: &str,
        path: &Path,
        parser_name: &str,
        config: &FormatConfig,
        supports_tailwind: bool,
        supports_oxfmt: bool,
        supports_svelte: bool,
    ) -> Result<String, OxcDiagnostic> {
        let mut prettier_options = to_prettier(config);
        inject_parser(&mut prettier_options, parser_name);
        inject_filepath(&mut prettier_options, path);

        if supports_tailwind {
            inject_tailwind_plugin_payload(&mut prettier_options, config);
        }
        if supports_oxfmt {
            inject_oxfmt_plugin_payload(&mut prettier_options, config, path);
        }
        if supports_svelte {
            inject_svelte_plugin_payload(&mut prettier_options, config);
        }

        self.external_services().format_file(prettier_options, source_text).map_err(|err| {
            // NOTE: We are trying to make the error from oxc_formatter(_xxx) and Prettier look similar.
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

/// The JS root's session wiring (registry dispatcher installed / off-gate honored),
/// which the registry-level tests in `embed::dispatcher` cannot see.
/// The napi build runs on `ExternalServices::dummy()` (native branches never call JS).
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        options::validate,
        oxfmtrc::{EmbeddedLanguageFormattingConfig, JsdocUserConfig},
    };

    fn format_ts(config: FormatConfig, source: &str) -> String {
        let validated = validate(&config).expect("default-ish config must validate");
        let kind = FileKind::OxcFormatter {
            path: Arc::from(Path::new("test.ts")),
            source_type: SourceType::ts(),
        };
        let strategy = FormatStrategy::from_format_config(config, &validated, kind);
        let formatter = SourceFormatter::new(1);
        #[cfg(feature = "napi")]
        let formatter =
            formatter.with_external_services(Some(crate::core::ExternalServices::dummy()));
        match formatter.format(source, strategy) {
            FormatResult::Success { code, .. } => code,
            FormatResult::Error(errors) => panic!("format failed: {errors:?}"),
        }
    }

    /// The JS root carries the registry dispatcher in every build,
    /// so native xxx-in-js formats without Prettier.
    #[test]
    fn js_root_installs_the_registry_dispatcher() {
        let code = format_ts(FormatConfig::default(), "const a = css`a{color:  red}`;\n");
        assert!(code.contains("color: red;"), "css-in-js should format natively: {code}");
    }

    /// Pure-build criterion: html has no native branch and no Prettier fallback,
    /// so the embed deliberately stays verbatim (under napi it reaches the fallback instead).
    #[cfg(not(feature = "napi"))]
    #[test]
    fn non_native_embed_stays_verbatim_without_fallback() {
        let source = "const h = html`<div>  <p>x</p></div>`;\n";
        assert_eq!(format_ts(FormatConfig::default(), source), source);
    }

    /// A JSDoc fence whose language is in the native registry formats through
    /// the fence adapter in every build (no Prettier involved).
    #[test]
    fn jsdoc_native_fence_formats_without_prettier() {
        let config =
            FormatConfig { jsdoc: Some(JsdocUserConfig::Bool(true)), ..FormatConfig::default() };
        let source = "/**\n * ```css\n * a{color:  red}\n * ```\n */\n";
        let code = format_ts(config, source);
        assert!(code.contains("color: red;"), "css fence should format natively: {code}");
    }

    /// `embeddedLanguageFormatting: off` installs no dispatcher:
    /// even native embeds stay verbatim.
    #[test]
    fn embedded_off_keeps_native_embeds_verbatim() {
        let config = FormatConfig {
            embedded_language_formatting: Some(EmbeddedLanguageFormattingConfig::Off),
            ..FormatConfig::default()
        };
        let source = "const a = css`a{color:  red}`;\n";
        assert_eq!(format_ts(config, source), source);
    }
}
