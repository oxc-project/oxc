use std::{path::Path, sync::Arc};

use tracing::instrument;

use oxc_allocator::{Allocator, AllocatorPool};
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::{FormatContext, FormatSession, Formatted, InputKind, Printed};
use oxc_formatter_json::{JsonFormatOptions, JsonVariant};
use oxc_formatter_yaml::YamlFormatOptions;

#[cfg(feature = "napi")]
use super::options::{
    inject_filepath, inject_opt_in_plugin_payloads, inject_oxfmt_plugin_payload, inject_parser,
    inject_tailwind_plugin_payload, to_prettier,
};
#[cfg(feature = "napi")]
use super::support::PrettierLanguage;
use super::{
    embed::dispatcher::ResolvedDispatchConfig,
    options::{
        ValidatedOptions, to_oxc_formatter_graphql, to_oxc_formatter_json, to_oxc_formatter_toml,
        to_oxc_formatter_yaml, to_sort_package_json,
    },
    oxfmtrc::FormatConfig,
    support::{FileKind, NativeLanguage},
};

/// A classified file with its resolved config.
///
/// Built by [`super::ConfigResolver::resolve`] or `resolve_for_api`,
/// where every fallible conversion already ran (`options::validate`).
/// So the per-formatter options are mapped from `config` + `validated` at the format step.
#[derive(Debug)]
pub struct FormatStrategy {
    pub(crate) kind: FileKind,
    pub(crate) config: Arc<FormatConfig>,
    pub(crate) validated: Arc<ValidatedOptions>,
}

impl FormatStrategy {
    pub fn path(&self) -> &Arc<Path> {
        self.kind.path()
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

    /// Format a file based on its resolved strategy.
    #[instrument(level = "debug", name = "oxfmt::format", skip_all, fields(path = %strategy.path().display(), kind = %strategy.kind.trace_label()))]
    pub fn format(&self, source_text: &str, strategy: FormatStrategy) -> FormatResult {
        // > Editors must not insert newlines in empty files when saving those files,
        // > even if insert_final_newline = true.
        // https://spec.editorconfig.org/#supported-pairs
        if source_text.trim().is_empty() {
            return FormatResult::Success {
                is_changed: !source_text.is_empty(),
                code: String::new(),
            };
        }

        let FormatStrategy { kind, config, validated } = strategy;
        let core = validated.core;
        // Roots with a session take options from their dispatch config,
        // others map them directly without building one.
        let result = match kind {
            FileKind::Native { path, language: NativeLanguage::Js(source_type) } => {
                let allocator = self.allocator_pool.get();
                let dispatch_config =
                    ResolvedDispatchConfig::for_root(Arc::clone(&config), validated, &path);
                let session = self.root_session(&allocator, &dispatch_config);
                let options = dispatch_config.js_options();
                let result =
                    oxc_formatter::format_with_session(&session, source_text, source_type, options)
                        .and_then(|formatted| print(formatted, &path));
                #[cfg(feature = "detect_code_removal")]
                if let Ok(code) = &result
                    && let Some(diff) =
                        oxc_formatter::detect_code_removal(source_text, code, source_type)
                {
                    unreachable!("Code removal detected in `{}`:\n{diff}", path.display());
                }
                result
            }
            FileKind::Native { path, language: NativeLanguage::Json(variant) } => {
                self.format_json(source_text, &path, to_oxc_formatter_json(&config, core, variant))
            }
            FileKind::PackageJson { path } => {
                // `sort_package_json` only accepts strictly valid JSON,
                // but the `json-stringify` parser also accepts unquoted keys, trailing commas, etc.
                // So format without sorting rather than bailing out.
                let sorted = to_sort_package_json(&config).and_then(|options| {
                    sort_package_json::sort_package_json_with_options(source_text, &options).ok()
                });
                self.format_json(
                    sorted.as_deref().unwrap_or(source_text),
                    &path,
                    to_oxc_formatter_json(&config, core, JsonVariant::JsonStringify),
                )
            }
            FileKind::Native { path, language: NativeLanguage::Graphql } => {
                let options = to_oxc_formatter_graphql(&config, core);
                let allocator = self.allocator_pool.get();
                oxc_formatter_graphql::format(&allocator, source_text, options)
                    .and_then(|formatted| print(formatted, &path))
            }
            FileKind::Native { path, language: NativeLanguage::Css(variant) } => {
                let allocator = self.allocator_pool.get();
                let dispatch_config =
                    ResolvedDispatchConfig::for_root(Arc::clone(&config), validated, &path);
                let session = self.root_session(&allocator, &dispatch_config);
                let options = dispatch_config.css_options(variant);
                oxc_formatter_css::format_with_session(&session, source_text, options)
                    .and_then(|formatted| print(formatted, &path))
            }
            FileKind::Native { path, language: NativeLanguage::Yaml } => {
                self.format_yaml(source_text, &path, to_oxc_formatter_yaml(&config, core))
            }
            // Mirroring Prettier's yaml embed: JSON if the whole text parses as JSON, YAML otherwise
            FileKind::YamlRc { path } => self
                .format_json(
                    source_text,
                    &path,
                    to_oxc_formatter_json(&config, core, JsonVariant::Json),
                )
                .or_else(|_| {
                    self.format_yaml(source_text, &path, to_oxc_formatter_yaml(&config, core))
                }),
            FileKind::Native { path, language: NativeLanguage::Markdown } => {
                let allocator = self.allocator_pool.get();
                let dispatch_config =
                    ResolvedDispatchConfig::for_root(Arc::clone(&config), validated, &path);
                let session = self.root_session(&allocator, &dispatch_config);
                let options = dispatch_config.markdown_options();
                oxc_formatter_markdown::format_with_session(&session, source_text, options)
                    .and_then(|formatted| print(formatted, &path))
            }
            FileKind::Native { language: NativeLanguage::Toml, .. } => {
                oxc_formatter_toml::format(source_text, to_oxc_formatter_toml(&config, core))
            }
            #[cfg(feature = "napi")]
            FileKind::Prettier { path, language } => {
                self.format_by_prettier(source_text, &path, language, &config)
            }
        };

        match result {
            Ok(mut code) => {
                // Every formatter always ends with a newline (no option to disable it),
                // so trimming is enough without allocating a new string.
                if !config.insert_final_newline.unwrap_or(true) {
                    code.truncate(code.trim_end().len());
                }
                FormatResult::Success { is_changed: source_text != code, code }
            }
            Err(err) => FormatResult::Error(vec![err]),
        }
    }

    /// A `PhysicalFile` root session carrying the build's default services
    /// (`embed::services::for_root`).
    #[cfg_attr(not(feature = "napi"), expect(clippy::unused_self))]
    fn root_session<'a>(
        &self,
        allocator: &'a Allocator,
        dispatch_config: &Arc<ResolvedDispatchConfig>,
    ) -> FormatSession<'a> {
        #[cfg(feature = "napi")]
        let services = super::embed::services::for_root(self.external_services(), dispatch_config);
        #[cfg(not(feature = "napi"))]
        let services = super::embed::services::for_root(dispatch_config);
        FormatSession::with_services(allocator, InputKind::PhysicalFile, services)
    }

    fn format_json(
        &self,
        source_text: &str,
        path: &Path,
        options: JsonFormatOptions,
    ) -> Result<String, OxcDiagnostic> {
        let allocator = self.allocator_pool.get();
        oxc_formatter_json::format(&allocator, source_text, options)
            .and_then(|formatted| print(formatted, path))
    }

    fn format_yaml(
        &self,
        source_text: &str,
        path: &Path,
        options: YamlFormatOptions,
    ) -> Result<String, OxcDiagnostic> {
        let allocator = self.allocator_pool.get();
        oxc_formatter_yaml::format(&allocator, source_text, options)
            .and_then(|formatted| print(formatted, path))
    }
}

fn print<C: FormatContext>(
    formatted: Formatted<'_, C>,
    path: &Path,
) -> Result<String, OxcDiagnostic> {
    formatted.print().map(Printed::into_code).map_err(|err| {
        OxcDiagnostic::error(format!("Failed to print formatted code: {}\n{err}", path.display()))
    })
}

// ---

/// NAPI-only methods for `SourceFormatter`, handling Prettier delegation.
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

    /// Format a file by delegating to Prettier,
    /// with plugin payloads injected when the parser can use them.
    #[instrument(level = "debug", name = "oxfmt::external::format_file", skip_all, fields(parser = %language.parser()))]
    fn format_by_prettier(
        &self,
        source_text: &str,
        path: &Path,
        language: PrettierLanguage,
        config: &FormatConfig,
    ) -> Result<String, OxcDiagnostic> {
        use PrettierLanguage::{Angular, Astro, Handlebars, Html, Svelte, Vue};

        let mut prettier_options = to_prettier(config);
        inject_parser(&mut prettier_options, language.parser());
        inject_filepath(&mut prettier_options, path);

        // CSS/SCSS/Less also benefit, but they are formatted by `oxc_formatter_css`
        if matches!(language, Html | Vue | Angular | Handlebars | Svelte | Astro) {
            inject_tailwind_plugin_payload(&mut prettier_options, config);
        }
        // Languages that embed JS/TS code.
        // Expressions like `__vue_expression` and `__ng_directive` are not supported yet.
        if matches!(language, Vue | Svelte | Astro) {
            inject_oxfmt_plugin_payload(&mut prettier_options, config, path);
        }
        inject_opt_in_plugin_payloads(&mut prettier_options, language, config);

        self.external_services().format_file(prettier_options, source_text).map_err(|err| {
            // NOTE: We are trying to make the error from oxc_formatter(_xxx) and Prettier look similar.
            // Ideally, we would unify them into `OxcDiagnostic`, which would eliminate the need for relative path conversion.
            // However, doing so would require:
            // - Parsing Prettier's error messages
            // - Converting span information from UTF-16 to UTF-8
            // This is a non-trivial amount of work, so for now, just leave this as a best effort.
            //
            // This is the only place in the formatting pipeline that depends on `cwd`.
            // It goes away together with this Prettier path, as `oxc_formatter_*` cover more languages.
            // (Their errors carry labels, and entry points render paths with their own `cwd`.)
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
