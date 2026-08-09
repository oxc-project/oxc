//! The embedded routing table ([`route`]) and the `FormatDispatcher` assembly shared by every build.
//!
//! Each language maps to a Rust formatter where available;
//! the [`PrettierLanguage`] set goes to the napi-only Prettier Doc→IR channel ([`super::prettier_doc`]) when one is supplied,
//! and is deliberately preserved as-is otherwise (pure Rust build); everything else stays as-is in every build.

use std::sync::{Arc, OnceLock};

use tracing::{debug, debug_span};

use oxc_formatter::CssInJsTemplate;
use oxc_formatter_core::{
    CoreFormatOptions, DispatchRequest, DispatchResponse, EmbeddedIr, FormatDispatcher,
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

/// The native half of the routing table:
/// a request/fence language routed to [`Route::Native`] parses to its Rust formatter branch here.
pub enum NativeLanguage {
    Graphql,
    /// The fence-derived variant;
    /// the css-in-js typed context overrides it to Scss + placeholders at dispatch time (see the css branch).
    Css(CssVariant),
    Yaml,
    Json(JsonVariant),
}

/// Languages Prettier still formats for us (no Rust formatter yet).
///
/// [`PrettierDocFallback`] receives this instead of a raw string,
/// so the fallback can never be handed a language the table did not route to it.
/// The set shrinks as Rust ports land (markdown first), and the type disappears with the last port.
#[derive(Clone, Copy)]
pub enum PrettierLanguage {
    Html,
    Angular,
    Markdown,
}

#[cfg(feature = "napi")]
impl PrettierLanguage {
    /// The Prettier `parser` name injected into the options JSON.
    ///
    /// NOTE: language identifiers happen to overlap with some Prettier parser names,
    /// but `oxc_formatter` treats them as generic language names;
    /// this method is the only place mapping EMBEDDED language identifiers to Prettier parsers
    /// (the whole-file Tier 3/4 path has its own filename-keyed map in `core::support`).
    pub fn parser(self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::Angular => "angular",
            Self::Markdown => "markdown",
        }
    }

    /// Whether the Doc→IR conversion must surface `HtmlEmbedMeta`
    /// (`htmlHasMultipleRootElements`) to the embed site.
    pub fn wants_html_meta(self) -> bool {
        matches!(self, Self::Html | Self::Angular)
    }
}

/// Where a language identifier routes.
pub enum Route {
    /// A Rust formatter branch in [`build_dispatcher`]; never re-routed to Prettier.
    Native(NativeLanguage),
    /// Prettier serves it (napi Doc→IR fallback / string channel);
    /// the pure build preserves it as-is.
    Prettier(PrettierLanguage),
    /// No formatter anywhere: the part deliberately stays as-is in every build.
    Unsupported,
}

/// THE routing table: "which formatter serves this language?" answered in one place.
/// [`build_dispatcher`] and the napi string channel's fence routing both consult it,
/// so their notions of who formats what can never drift,
/// and aliases (`"gql"` / `"yml"` / `"md"`) are resolved here and nowhere else.
pub fn route(language: &str) -> Route {
    match language {
        "graphql" | "gql" => Route::Native(NativeLanguage::Graphql),
        "css" => Route::Native(NativeLanguage::Css(CssVariant::Css)),
        "scss" => Route::Native(NativeLanguage::Css(CssVariant::Scss)),
        "less" => Route::Native(NativeLanguage::Css(CssVariant::Less)),
        "yaml" | "yml" => Route::Native(NativeLanguage::Yaml),
        "json" => Route::Native(NativeLanguage::Json(JsonVariant::Json)),
        "jsonc" => Route::Native(NativeLanguage::Json(JsonVariant::Jsonc)),
        "json5" => Route::Native(NativeLanguage::Json(JsonVariant::Json5)),
        "html" => Route::Prettier(PrettierLanguage::Html),
        "angular" => Route::Prettier(PrettierLanguage::Angular),
        "markdown" | "md" => Route::Prettier(PrettierLanguage::Markdown),
        _ => Route::Unsupported,
    }
}

/// Per-root context shared by every embedded service (dispatcher, string embedder, Tailwind sorter):
/// the host file's resolved config plus lazily-mapped per-language options.
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
    /// The options handed to Prettier; see [`PrettierOptions`].
    #[cfg(feature = "napi")]
    prettier: PrettierOptions,
}

/// The lazily-built options JSON handed to Prettier (+ plugins),
/// consumed by the Doc→IR / string paths and the Tailwind sorter.
/// `path` is an ingredient, not a sibling datum: it becomes the JSON's `filepath` at last
/// (see [`crate::core::options::build_prettier_options`]).
///
/// NOTE: The late merge is load-bearing: the JSON must derive from the RESOLVED per-file config
/// (a pre-built Value loses overrides, #18246), and `path` is the one per-file ingredient,
/// keeping it out of the config is what keeps the config shareable across files.
/// Lazy so an embed-free file never builds the JSON at all.
#[cfg(feature = "napi")]
#[derive(Default)]
struct PrettierOptions {
    path: std::path::PathBuf,
    options: OnceLock<serde_json::Value>,
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
            prettier: PrettierOptions::default(),
        }
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

    /// The single off-predicate: [`Self::root_dispatcher`] and both build's `services::for_root` definitions consult it,
    /// so the off-semantics can never diverge between channels or builds.
    pub fn is_embedded_formatting_enabled(&self) -> bool {
        self.config.is_embedded_formatting_enabled()
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
    /// the fence adapter ([`super::jsdoc_fence`]) derives its per-fence options from these
    /// (width overridden to the fence's effective width).
    pub fn print_options(&self) -> PrinterOptions {
        self.core.as_print_options()
    }
}

/// Napi-only methods: the [`PrettierOptions`] accessors and the Tailwind predicate.
#[cfg(feature = "napi")]
impl ResolvedDispatchConfig {
    /// Sets the host file path for `filepath` injection into [`Self::prettier_options`];
    /// chained by [`Self::for_root`].
    fn with_path(mut self, path: std::path::PathBuf) -> Self {
        self.prettier.path = path;
        self
    }

    /// The single Tailwind predicate, same pattern as
    /// [`Self::is_embedded_formatting_enabled`]: every sorter-assembly site consults it
    /// (the sorter is napi-only; the pure build has no JS-side class order source).
    pub fn is_tailwind_enabled(&self) -> bool {
        self.config.is_tailwind_enabled()
    }

    /// The options JSON handed to Prettier
    /// (see [`crate::core::options::build_prettier_options`]).
    pub fn prettier_options(&self) -> &serde_json::Value {
        self.prettier.options.get_or_init(|| {
            crate::core::options::build_prettier_options(&self.config, &self.prettier.path)
        })
    }
}

/// Fallback invoked for [`Route::Prettier`] languages.
/// Same shape as `FormatDispatcher` minus the request envelope
/// (the Doc path consumes neither `input_kind` nor `parent_context` today;
/// re-examine if it ever serves envelope-bearing inputs).
///
/// Assembled only in napi builds ([`super::prettier_doc`]);
/// the pure Rust build passes `None` and these languages are deliberately preserved as-is.
pub type PrettierDocFallback = Arc<
    dyn for<'a> Fn(
            &FormatSession<'a>,
            PrettierLanguage,
            &str,
        ) -> Result<DispatchResponse<'a>, String>
        + Send
        + Sync,
>;

/// Build the `FormatDispatcher` carried by the root's `FormatSession`:
/// Rust formatters for the [`Route::Native`] branches (never re-routed to Prettier, even on failure),
/// `fallback` for the [`Route::Prettier`] set, deliberate preservation for the rest.
pub fn build_dispatcher(
    dispatch_config: Arc<ResolvedDispatchConfig>,
    fallback: Option<PrettierDocFallback>,
) -> FormatDispatcher {
    Arc::new(move |session: &FormatSession<'_>, request: DispatchRequest<'_>| {
        let text = request.text;
        match route(request.language) {
            Route::Native(NativeLanguage::Graphql) => Ok(format_native("graphql", || {
                oxc_formatter_graphql::format_to_ir(
                    session,
                    text,
                    dispatch_config.graphql_options(),
                )
            })),
            Route::Native(NativeLanguage::Css(variant)) => {
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
            Route::Native(NativeLanguage::Yaml) => Ok(format_native("yaml", || {
                oxc_formatter_yaml::format_to_ir(session, text, dispatch_config.yaml_options())
            })),
            Route::Native(NativeLanguage::Json(variant)) => Ok(format_native("json", || {
                oxc_formatter_json::format_to_ir(
                    session,
                    text,
                    dispatch_config.json_options(variant),
                )
            })),

            // Prettier-served languages: Doc→IR fallback when available (napi),
            // deliberate skip otherwise (pure build).
            Route::Prettier(language) => {
                if let Some(fallback) = &fallback {
                    fallback(session, language, text)
                } else {
                    debug!(
                        "No fallback for Prettier language '{}' in this build, part stays as-is",
                        request.language
                    );
                    Ok(DispatchResponse::PreserveOriginal)
                }
            }

            // A language without a formatter is a deliberate skip, in every build.
            Route::Unsupported => {
                debug!("No formatter for language '{}', part stays as-is", request.language);
                Ok(DispatchResponse::PreserveOriginal)
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
) -> DispatchResponse<'a> {
    debug_span!("oxfmt::embed::format_to_ir", language).in_scope(|| match format_to_ir() {
        Ok(embedded) => DispatchResponse::Formatted(embedded.into()),
        Err(err) => {
            debug!("native '{language}' format_to_ir failed, part stays as-is: {err}");
            DispatchResponse::PreserveOriginal
        }
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use oxc_allocator::Allocator;
    use oxc_formatter_core::{
        CoreFormatOptions, DispatchRequest, DispatchResponse, FormatSession, InputKind,
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

    /// Every fence language the routing table claims as native must format
    /// WITHOUT a fallback installed
    /// (an accidentally dropped [`super::route`] entry would fall through to `PreserveOriginal` and fail here).
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
            let response = session.dispatch(DispatchRequest {
                language,
                text,
                input_kind: InputKind::Fragment,
                parent_context: None,
            });
            assert!(
                matches!(response, Ok(DispatchResponse::Formatted(_))),
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

        let response = session.dispatch(DispatchRequest {
            language: "yaml",
            text: "a:   1",
            input_kind: InputKind::Fragment,
            parent_context: None,
        });
        assert!(matches!(response, Ok(DispatchResponse::Formatted(_))));
    }

    /// Both non-native routes preserve without a fallback installed:
    /// a Prettier-served language (pure-build behavior) and a fully unsupported one.
    #[test]
    fn non_native_language_without_fallback_preserves_original() {
        let allocator = Allocator::default();
        let session = FormatSession::with_services(
            &allocator,
            InputKind::PhysicalFile,
            SessionServices {
                dispatcher: Some(build_dispatcher(dispatch_config(), None)),
                ..SessionServices::default()
            },
        );

        for (language, text) in [("html", "<div></div>"), ("toml", "a = 1")] {
            let response = session.dispatch(DispatchRequest {
                language,
                text,
                input_kind: InputKind::Fragment,
                parent_context: None,
            });
            assert!(
                matches!(response, Ok(DispatchResponse::PreserveOriginal)),
                "language '{language}' should be preserved without a fallback"
            );
        }
    }
}
