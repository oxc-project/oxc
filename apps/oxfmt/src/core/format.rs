use std::{path::Path, sync::Arc};

use tracing::instrument;

use oxc_allocator::Allocator;
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::{FormatContext, FormatSession, Formatted, InputKind, Printed};
use oxc_formatter_json::{JsonFormatOptions, JsonVariant};
use oxc_formatter_yaml::YamlFormatOptions;
use oxc_parser::Source;

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
    support::{FormatStrategy, NativeLanguage},
};

/// A file's path and [`FormatStrategy`] with its resolved config, ready to format once given the source text.
///
/// Built by [`super::ConfigResolver::resolve`] or `resolve_for_api`,
/// where every fallible conversion already ran (`options::validate`).
/// So the per-formatter options are mapped from `config` + `validated` at the format step.
#[derive(Debug)]
pub struct FormatPlan {
    pub(crate) path: Arc<Path>,
    pub(crate) strategy: FormatStrategy,
    pub(crate) config: Arc<FormatConfig>,
    pub(crate) validated: Arc<ValidatedOptions>,
}

// ---

pub enum FormatResult {
    Success { is_changed: bool, code: String },
    Error(Vec<OxcDiagnostic>),
}

/// Return value of [`SourceFormatter::format`].
pub struct FormatReturn<'a> {
    /// Source text which was formatted.
    ///
    /// Formatting consumes the [`Source`], so this is how the caller gets the source text back,
    /// e.g. to render errors against.
    ///
    /// This is an empty string if the source is not valid UTF-8, or is too long to be parsed.
    pub source_text: &'a str,
    /// Formatted code, or errors.
    pub result: FormatResult,
}

impl<'a> FormatReturn<'a> {
    /// Create a [`FormatReturn`] from the source text, and the formatted code or an error.
    fn new(
        source_text: &'a str,
        result: Result<String, OxcDiagnostic>,
        config: &FormatConfig,
    ) -> Self {
        let result = match result {
            Ok(mut code) => {
                // Every formatter always ends with a newline (no option to disable it),
                // so trimming is enough without allocating a new string.
                if !config.insert_final_newline.unwrap_or(true) {
                    code.truncate(code.trim_end().len());
                }
                FormatResult::Success { is_changed: source_text != code, code }
            }
            Err(err) => FormatResult::Error(vec![err]),
        };
        Self { source_text, result }
    }
}

#[derive(Default)]
pub struct SourceFormatter {
    #[cfg(feature = "napi")]
    external_services: Option<super::ExternalServices>,
}

impl SourceFormatter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Format a file following its plan.
    ///
    /// `source` holds the file's source text. Its `SourceType` is not used.
    /// The plan says what language the file is, and for JS/TS, its `SourceType`.
    ///
    /// Everything is allocated in `source`'s allocator.
    ///
    /// Formatting consumes `source`. The source text is returned in the [`FormatReturn`].
    #[instrument(level = "debug", name = "oxfmt::format", skip_all, fields(path = %plan.path.display(), strategy = %plan.strategy.trace_label()))]
    pub fn format<'a>(&self, mut source: Source<'a>, plan: FormatPlan) -> FormatReturn<'a> {
        let allocator = source.allocator();
        let FormatPlan { path, strategy, config, validated } = plan;

        // JS/TS is parsed directly from the `Source`, without copying the source text.
        // Parsing consumes the `Source`, and hands the source text back.
        //
        // JS/TS source which is blank is not parsed. It's handled below, like blank source
        // in any other language. So is JS/TS source which is not valid UTF-8.
        if let FormatStrategy::Native(NativeLanguage::Js(source_type)) = strategy
            && source.as_str().is_ok_and(|source_text| !is_blank(source_text))
        {
            source.set_source_type(source_type);
            let (source_text, result) = self.format_js(source, &path, &config, validated);
            return FormatReturn::new(source_text, result, &config);
        }

        // All other formatters take the source text
        let Ok(source_text) = source.into_str() else {
            let error = OxcDiagnostic::error("Source is not valid UTF-8");
            return FormatReturn::new("", Err(error), &config);
        };

        // > Editors must not insert newlines in empty files when saving those files,
        // > even if insert_final_newline = true.
        // https://spec.editorconfig.org/#supported-pairs
        if is_blank(source_text) {
            return FormatReturn::new(source_text, Ok(String::new()), &config);
        }

        let core = validated.core;
        // Roots with a session take options from their dispatch config,
        // others map them directly without building one.
        let result = match strategy {
            FormatStrategy::Native(NativeLanguage::Js(_)) => {
                unreachable!("JS/TS is formatted above, unless it's blank, which returned above")
            }
            FormatStrategy::Native(NativeLanguage::Json(variant)) => {
                let options = to_oxc_formatter_json(&config, core, variant);
                format_json(source_text, &path, options, allocator)
            }
            FormatStrategy::PackageJson => {
                // `sort_package_json` only accepts strictly valid JSON,
                // but the `json-stringify` parser also accepts unquoted keys, trailing commas, etc.
                // So format without sorting rather than bailing out.
                let sorted = to_sort_package_json(&config).and_then(|options| {
                    sort_package_json::sort_package_json_with_options(source_text, &options).ok()
                });
                let options = to_oxc_formatter_json(&config, core, JsonVariant::JsonStringify);
                format_json(sorted.as_deref().unwrap_or(source_text), &path, options, allocator)
            }
            FormatStrategy::Native(NativeLanguage::Graphql) => {
                let options = to_oxc_formatter_graphql(&config, core);
                oxc_formatter_graphql::format(allocator, source_text, options)
                    .and_then(|formatted| print(formatted, &path))
            }
            FormatStrategy::Native(NativeLanguage::Css(variant)) => {
                let dispatch_config =
                    ResolvedDispatchConfig::for_root(Arc::clone(&config), validated, &path);
                let session = self.root_session(allocator, &dispatch_config);
                let options = dispatch_config.css_options(variant);
                oxc_formatter_css::format_with_session(&session, source_text, options)
                    .and_then(|formatted| print(formatted, &path))
            }
            FormatStrategy::Native(NativeLanguage::Yaml) => {
                let options = to_oxc_formatter_yaml(&config, core);
                format_yaml(source_text, &path, options, allocator)
            }
            // Mirroring Prettier's yaml embed: JSON if the whole text parses as JSON, YAML otherwise
            FormatStrategy::YamlRc => {
                let options = to_oxc_formatter_json(&config, core, JsonVariant::Json);
                format_json(source_text, &path, options, allocator).or_else(|_| {
                    let options = to_oxc_formatter_yaml(&config, core);
                    format_yaml(source_text, &path, options, allocator)
                })
            }
            FormatStrategy::Native(NativeLanguage::Markdown) => {
                let dispatch_config =
                    ResolvedDispatchConfig::for_root(Arc::clone(&config), validated, &path);
                let session = self.root_session(allocator, &dispatch_config);
                let options = dispatch_config.markdown_options();
                oxc_formatter_markdown::format_with_session(&session, source_text, options)
                    .and_then(|formatted| print(formatted, &path))
            }
            FormatStrategy::Native(NativeLanguage::Html(variant)) => {
                let dispatch_config =
                    ResolvedDispatchConfig::for_root(Arc::clone(&config), validated, &path);
                let session = self.root_session(allocator, &dispatch_config);
                let options = dispatch_config.htmlish_options(variant);
                oxc_formatter_htmlish::format_with_session(&session, source_text, options)
                    .and_then(|formatted| print(formatted, &path))
            }
            FormatStrategy::Native(NativeLanguage::Toml) => {
                let options = to_oxc_formatter_toml(&config, core);
                oxc_formatter_toml::format(allocator, source_text, options)
                    .and_then(|formatted| print(formatted, &path))
            }
            #[cfg(feature = "napi")]
            FormatStrategy::Prettier(language) => {
                self.format_by_prettier(source_text, &path, language, &config)
            }
        };

        FormatReturn::new(source_text, result, &config)
    }

    /// Format JS/TS source.
    ///
    /// `source` is parsed in place. Returns the source text, and the formatted code or an error.
    fn format_js<'a>(
        &self,
        source: Source<'a>,
        path: &Path,
        config: &Arc<FormatConfig>,
        validated: Arc<ValidatedOptions>,
    ) -> (&'a str, Result<String, OxcDiagnostic>) {
        #[cfg(feature = "detect_code_removal")]
        let source_type = source.source_type();

        let dispatch_config = ResolvedDispatchConfig::for_root(Arc::clone(config), validated, path);
        let session = self.root_session(source.allocator(), &dispatch_config);
        let options = dispatch_config.js_options();
        let ret = oxc_formatter::format_source_with_session(&session, source, options);

        let source_text = ret.source_text;
        let result = ret.result.and_then(|formatted| print(formatted, path));

        #[cfg(feature = "detect_code_removal")]
        if let Ok(code) = &result
            && let Some(diff) = oxc_formatter::detect_code_removal(source_text, code, source_type)
        {
            unreachable!("Code removal detected in `{}`:\n{diff}", path.display());
        }

        (source_text, result)
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
}

/// Returns `true` if `source_text` contains only whitespace.
fn is_blank(source_text: &str) -> bool {
    source_text.trim().is_empty()
}

/// Format `source_text` as JSON.
fn format_json(
    source_text: &str,
    path: &Path,
    options: JsonFormatOptions,
    allocator: &Allocator,
) -> Result<String, OxcDiagnostic> {
    oxc_formatter_json::format(allocator, source_text, options)
        .and_then(|formatted| print(formatted, path))
}

/// Format `source_text` as YAML.
fn format_yaml<'a>(
    source_text: &'a str,
    path: &Path,
    options: YamlFormatOptions,
    allocator: &'a Allocator,
) -> Result<String, OxcDiagnostic> {
    oxc_formatter_yaml::format(allocator, source_text, options)
        .and_then(|formatted| print(formatted, path))
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
        use PrettierLanguage::{Astro, Glimmer, Svelte};

        let mut prettier_options = to_prettier(config);
        inject_parser(&mut prettier_options, language.parser());
        inject_filepath(&mut prettier_options, path);

        // CSS/SCSS/Less also benefit, but they are formatted by `oxc_formatter_css`
        if matches!(language, Glimmer | Svelte | Astro) {
            inject_tailwind_plugin_payload(&mut prettier_options, config);
        }
        // Languages that embed JS/TS code.
        if matches!(language, Svelte | Astro) {
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
