//! Native dispatch registry: the `FormatDispatcher` assembly shared by every build.
//!
//! Each language maps to a Rust formatter where available;
//! everything else goes to the napi-only Prettier Doc→IR fallback ([`super::prettier_fallback`]) when
//! one is supplied, and is deliberately preserved as-is otherwise (pure Rust build).

use std::sync::{Arc, OnceLock};

use tracing::{debug, debug_span};

use oxc_formatter::CssInJsTemplate;
use oxc_formatter_core::{
    CoreFormatOptions, DispatchOutcome, DispatchRequest, EmbeddedIr, FormatDispatcher,
    FormatSession,
};
use oxc_formatter_core::{FormatOptions, PrinterOptions};
use oxc_formatter_css::{CssFormatOptions, CssVariant};
use oxc_formatter_graphql::GraphqlFormatOptions;
use oxc_formatter_json::{JsonFormatOptions, JsonVariant};
use oxc_formatter_yaml::YamlFormatOptions;

use crate::core::{
    options::{
        to_oxc_formatter_css, to_oxc_formatter_graphql, to_oxc_formatter_json,
        to_oxc_formatter_yaml,
    },
    oxfmtrc::FormatConfig,
};

/// The native formatter registry:
/// a request/fence language parses to its Rust formatter branch here, or it has none.
///
/// [`build_dispatcher`] and the napi string channel's fence routing (`is_native_language`)
/// both consult this single mapping, so their notions of "native" can never drift.
enum NativeLanguage {
    Graphql,
    /// The fence-derived variant;
    /// the css-in-js typed context overrides it to Scss + placeholders at dispatch time (see the css branch).
    Css(CssVariant),
    Yaml,
    Json(JsonVariant),
}

fn native_language(language: &str) -> Option<NativeLanguage> {
    Some(match language {
        "graphql" | "gql" => NativeLanguage::Graphql,
        "css" => NativeLanguage::Css(CssVariant::Css),
        "scss" => NativeLanguage::Css(CssVariant::Scss),
        "less" => NativeLanguage::Css(CssVariant::Less),
        "yaml" | "yml" => NativeLanguage::Yaml,
        "json" => NativeLanguage::Json(JsonVariant::Json),
        "jsonc" => NativeLanguage::Json(JsonVariant::Jsonc),
        "json5" => NativeLanguage::Json(JsonVariant::Json5),
        _ => return None,
    })
}

/// Whether `language` has a native (Rust formatter) branch in the registry.
/// (Consulted by the napi string channel's routing; the dispatcher itself matches
/// [`native_language`] directly, and the pure build's fence adapter just dispatches.)
#[cfg(feature = "napi")]
pub fn is_native_language(language: &str) -> bool {
    native_language(language).is_some()
}

/// Per-run dispatch configuration: the resolved config plus lazily-mapped per-language options.
///
/// Language options are NOT built up front: an embed-free file pays only for empty cells,
/// and a host where every language is embeddable (Markdown-scale) maps exactly the languages that actually appear,
/// once each (`OnceLock` memoizes and is safe under the rayon-parallel format runs).
pub struct ResolvedDispatchConfig {
    /// Resolved config of the HOST file (its overrides / editorconfig applied).
    /// Embedded children inherit it, mirroring Prettier's `textToDoc` (parent-options spread);
    /// never a re-resolution for a virtual path.
    config: Arc<FormatConfig>,
    /// Core options validated once by the config-resolution gate (`options::validate`).
    /// Holding them pre-validated is what lets the per-language mappers be infallible.
    core: CoreFormatOptions,
    graphql: OnceLock<GraphqlFormatOptions>,
    /// One cell per [`CssVariant`]: JSDoc fences dispatch css/scss/less as-is, while css-in-js always uses Scss.
    css: [OnceLock<CssFormatOptions>; 3],
    yaml: OnceLock<YamlFormatOptions>,
    /// One cell per fence-reachable [`JsonVariant`] (json / jsonc / json5; `JsonStringify` is `package.json`-only).
    json: [OnceLock<JsonFormatOptions>; 3],
    /// Host file path, for `filepath` injection into the Prettier options JSON.
    #[cfg(feature = "napi")]
    path: std::path::PathBuf,
    /// Prettier-compatible options JSON for the JS-side consumers (Doc→IR fallback, string channel, Tailwind sorter).
    #[cfg(feature = "napi")]
    external_options: OnceLock<serde_json::Value>,
}

impl ResolvedDispatchConfig {
    /// `core` is the pre-validated bundle carried from the config-resolution gate (`options::validate`);
    /// it never gets re-derived here.
    /// Private so [`Self::for_root`] stays the only construction recipe.
    fn new(config: Arc<FormatConfig>, core: CoreFormatOptions) -> Self {
        Self {
            config,
            core,
            graphql: OnceLock::new(),
            css: [OnceLock::new(), OnceLock::new(), OnceLock::new()],
            yaml: OnceLock::new(),
            json: [OnceLock::new(), OnceLock::new(), OnceLock::new()],
            #[cfg(feature = "napi")]
            path: std::path::PathBuf::new(),
            #[cfg(feature = "napi")]
            external_options: OnceLock::new(),
        }
    }

    /// Sets the host file path for `filepath` injection into [`Self::external_options`];
    /// chained by [`Self::for_root`].
    #[cfg(feature = "napi")]
    fn with_path(mut self, path: std::path::PathBuf) -> Self {
        self.path = path;
        self
    }

    /// The one construction recipe for a root formatter run at `path`:
    /// [`Self::new`] plus the napi-only path recording
    /// (the pure build has no JS-side consumers, so `path` goes unused there).
    pub fn for_root(
        config: &Arc<FormatConfig>,
        core: CoreFormatOptions,
        path: &std::path::Path,
    ) -> Arc<Self> {
        let dispatch_config = Self::new(Arc::clone(config), core);
        #[cfg(feature = "napi")]
        let dispatch_config = dispatch_config.with_path(path.to_path_buf());
        #[cfg(not(feature = "napi"))]
        let _ = path;
        Arc::new(dispatch_config)
    }

    /// Assembles the root's `FormatDispatcher` behind the off-gate:
    /// `None` under `embeddedLanguageFormatting: off`,
    /// so a root cannot install the registry without honoring the off-semantics.
    /// `fallback` is the one build-dependent datum (the napi Prettier Doc→IR path).
    pub fn root_dispatcher(
        self: &Arc<Self>,
        fallback: Option<PrettierDocFallback>,
    ) -> Option<FormatDispatcher> {
        self.is_embedded_formatting_enabled().then(|| build_dispatcher(Arc::clone(self), fallback))
    }

    /// The single off-predicate: [`Self::root_dispatcher`] and the service builders
    /// (`fence::session_services`, `ExternalFormatter::session_services`) all consult it,
    /// so the off-semantics can never diverge between channels or builds.
    pub fn is_embedded_formatting_enabled(&self) -> bool {
        self.config.is_embedded_formatting_enabled()
    }

    /// The single Tailwind predicate, same pattern as
    /// [`Self::is_embedded_formatting_enabled`]: every sorter-assembly site consults it
    /// (the sorter is napi-only; the pure build has no JS-side class order source).
    #[cfg(feature = "napi")]
    pub fn is_tailwind_enabled(&self) -> bool {
        self.config.is_tailwind_enabled()
    }

    pub fn graphql_options(&self) -> GraphqlFormatOptions {
        *self.graphql.get_or_init(|| to_oxc_formatter_graphql(&self.config, self.core))
    }

    pub fn css_options(&self, variant: CssVariant) -> CssFormatOptions {
        let cell = match variant {
            CssVariant::Css => &self.css[0],
            CssVariant::Scss => &self.css[1],
            CssVariant::Less => &self.css[2],
        };
        *cell.get_or_init(|| to_oxc_formatter_css(&self.config, self.core, variant))
    }

    pub fn yaml_options(&self) -> YamlFormatOptions {
        *self.yaml.get_or_init(|| to_oxc_formatter_yaml(&self.config, self.core))
    }

    pub fn json_options(&self, variant: JsonVariant) -> JsonFormatOptions {
        let cell = match variant {
            JsonVariant::Json => &self.json[0],
            JsonVariant::Jsonc => &self.json[1],
            JsonVariant::Json5 => &self.json[2],
            JsonVariant::JsonStringify => {
                unreachable!(
                    "JsonStringify is the package.json pipeline's variant, never dispatched"
                )
            }
        };
        *cell.get_or_init(|| to_oxc_formatter_json(&self.config, self.core, variant))
    }

    /// Printer options from the shared resolved core bundle;
    /// the fence adapter ([`super::fence`]) prints dispatched child IR standalone with these.
    pub fn print_options(&self) -> PrinterOptions {
        self.core.as_print_options()
    }

    /// The Prettier options JSON shared by the JS-side consumers
    /// (see [`crate::core::options::build_external_options`]).
    #[cfg(feature = "napi")]
    pub fn external_options(&self) -> &serde_json::Value {
        self.external_options
            .get_or_init(|| crate::core::options::build_external_options(&self.config, &self.path))
    }
}

/// Fallback invoked for languages without a native branch.
/// Same shape as `FormatDispatcher` minus the request envelope
/// (the Doc path consumes neither `input_kind` nor `parent_context` today;
/// re-examine if it ever serves envelope-bearing inputs).
///
/// Assembled only in napi builds ([`super::prettier_fallback`]);
/// the pure Rust build passes `None` and unsupported languages are deliberately preserved as-is.
pub type PrettierDocFallback = Arc<
    dyn for<'a> Fn(&FormatSession<'a>, &str, &str) -> Result<DispatchOutcome<'a>, String>
        + Send
        + Sync,
>;

/// Build the `FormatDispatcher` carried by the root's `FormatSession`:
/// Rust formatters for the [`NativeLanguage`] registry (no Prettier fallback for them),
/// `fallback` for everything else.
pub fn build_dispatcher(
    dispatch_config: Arc<ResolvedDispatchConfig>,
    fallback: Option<PrettierDocFallback>,
) -> FormatDispatcher {
    Arc::new(move |session: &FormatSession<'_>, request: DispatchRequest<'_>| {
        let text = request.text;
        match native_language(request.language) {
            Some(NativeLanguage::Graphql) => Ok(format_native("graphql", || {
                oxc_formatter_graphql::format_to_ir(
                    session,
                    text,
                    dispatch_config.graphql_options(),
                )
            })),
            Some(NativeLanguage::Css(variant)) => {
                // css-in-js (typed `CssInJsTemplate` context) is always parsed as SCSS with `${}` placeholder markers.
                // Any other caller gets the strict standalone grammar with the fence/request language's variant.
                let (variant, template_placeholders) = if request
                    .parent_context
                    .is_some_and(|c| c.downcast_ref::<CssInJsTemplate>().is_some())
                {
                    (CssVariant::Scss, true)
                } else {
                    (variant, false)
                };
                Ok(format_native("css", || {
                    oxc_formatter_css::format_to_ir(
                        session,
                        text,
                        dispatch_config.css_options(variant),
                        template_placeholders,
                    )
                }))
            }
            Some(NativeLanguage::Yaml) => Ok(format_native("yaml", || {
                oxc_formatter_yaml::format_to_ir(session, text, dispatch_config.yaml_options())
            })),
            Some(NativeLanguage::Json(variant)) => Ok(format_native("json", || {
                oxc_formatter_json::format_to_ir(
                    session,
                    text,
                    dispatch_config.json_options(variant),
                )
            })),
            // Everything else: Prettier fallback (Doc→IR path) when available
            None => {
                if let Some(fallback) = &fallback {
                    fallback(session, request.language, text)
                } else {
                    // A language without a formatter is a deliberate skip.
                    debug!("No formatter for language '{}', part stays as-is", request.language);
                    Ok(DispatchOutcome::PreserveOriginal)
                }
            }
        }
    })
}

/// Runs one native branch: a parse failure is a deliberate skip
/// (the embedded part stays as-is), never an operational error.
/// The `From<EmbeddedIr>` conversion carries the child's Tailwind classes.
fn format_native<'a, E: std::fmt::Display>(
    language: &'static str,
    format_to_ir: impl FnOnce() -> Result<EmbeddedIr<'a>, E>,
) -> DispatchOutcome<'a> {
    debug_span!("oxfmt::embed::format_to_ir", language).in_scope(|| match format_to_ir() {
        Ok(embedded) => DispatchOutcome::Formatted(embedded.into()),
        Err(err) => {
            debug!("native '{language}' format_to_ir failed, part stays as-is: {err}");
            DispatchOutcome::PreserveOriginal
        }
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use oxc_allocator::Allocator;
    use oxc_formatter_core::{
        CoreFormatOptions, DispatchOutcome, DispatchRequest, FormatSession, InputKind,
        SessionServices,
    };

    use super::{ResolvedDispatchConfig, build_dispatcher};
    use crate::core::oxfmtrc::FormatConfig;

    fn dispatch_config() -> Arc<ResolvedDispatchConfig> {
        Arc::new(ResolvedDispatchConfig::new(
            Arc::new(FormatConfig::default()),
            CoreFormatOptions::default(),
        ))
    }

    /// Every fence language the registry claims as native must format
    /// WITHOUT a fallback installed
    /// (an accidentally dropped `native_language` entry would fall through to `PreserveOriginal` and fail here).
    #[test]
    fn every_native_language_dispatches() {
        let allocator = Allocator::default();
        let session = FormatSession::with_services(
            &allocator,
            InputKind::PhysicalFile,
            SessionServices {
                dispatcher: Some(build_dispatcher(dispatch_config(), None)),
                ..SessionServices::default()
            },
        );

        for language in
            ["graphql", "gql", "css", "scss", "less", "yaml", "yml", "json", "jsonc", "json5"]
        {
            let text = match language {
                "graphql" | "gql" => "{ a }",
                "css" | "scss" | "less" => "a { color: red }",
                "yaml" | "yml" => "a: 1",
                "json" | "jsonc" | "json5" => "{ \"a\": 1 }",
                other => panic!("no sample input for native language '{other}'"),
            };
            let outcome = session.dispatch(DispatchRequest {
                language,
                text,
                input_kind: InputKind::Fragment,
                parent_context: None,
            });
            assert!(
                matches!(outcome, Ok(DispatchOutcome::Formatted(_))),
                "language '{language}' did not dispatch natively"
            );
        }
    }

    /// Pure-build criterion: the native registry dispatches YAML with no fallback installed.
    #[test]
    fn native_yaml_dispatch_works_without_fallback() {
        let allocator = Allocator::default();
        let session = FormatSession::with_services(
            &allocator,
            InputKind::PhysicalFile,
            SessionServices {
                dispatcher: Some(build_dispatcher(dispatch_config(), None)),
                ..SessionServices::default()
            },
        );

        let outcome = session.dispatch(DispatchRequest {
            language: "yaml",
            text: "a:   1",
            input_kind: InputKind::Fragment,
            parent_context: None,
        });
        assert!(matches!(outcome, Ok(DispatchOutcome::Formatted(_))));
    }

    #[test]
    fn unsupported_language_without_fallback_preserves_original() {
        let allocator = Allocator::default();
        let session = FormatSession::with_services(
            &allocator,
            InputKind::PhysicalFile,
            SessionServices {
                dispatcher: Some(build_dispatcher(dispatch_config(), None)),
                ..SessionServices::default()
            },
        );

        let outcome = session.dispatch(DispatchRequest {
            language: "html",
            text: "<div></div>",
            input_kind: InputKind::Fragment,
            parent_context: None,
        });
        assert!(matches!(outcome, Ok(DispatchOutcome::PreserveOriginal)));
    }
}
