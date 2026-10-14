//! The `FormatDispatcher` assembly shared by every build, routed by [`route_embedded`].
//!
//! Each language maps to a Rust formatter where available;
//! the [`PrettierLanguage`] set goes to the napi-only Prettier Doc→IR channel ([`super::prettier_doc`]) when one is supplied,
//! and is deliberately preserved as-is otherwise (pure Rust build); everything else stays as-is in every build.

use std::sync::{Arc, OnceLock};

use tracing::{debug, debug_span};

use oxc_formatter::{CssInJsTemplate, JsEmbeddedIn, JsFormatOptions, MarkdownInJsTemplate};
use oxc_formatter_core::{
    DispatchRequest, DispatchResponse, EmbeddedIr, FormatDispatcher, FormatSession,
};
use oxc_formatter_core::{FormatOptions, PrinterOptions};
use oxc_formatter_css::{CssFormatOptions, CssVariant};
use oxc_formatter_graphql::GraphqlFormatOptions;
use oxc_formatter_json::{JsonFormatOptions, JsonVariant};
use oxc_formatter_markdown::{MarkdownFormatOptions, XxxInMarkdownCodeBlock};
use oxc_formatter_toml::TomlFormatOptions;
use oxc_formatter_yaml::YamlFormatOptions;

use crate::core::{
    language::{Route, route_embedded},
    options::{
        ValidatedOptions, to_oxc_formatter, to_oxc_formatter_css, to_oxc_formatter_graphql,
        to_oxc_formatter_json, to_oxc_formatter_markdown, to_oxc_formatter_toml,
        to_oxc_formatter_yaml,
    },
    oxfmtrc::FormatConfig,
    support::{NativeLanguage, PrettierLanguage},
};

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
    /// The config-resolution gate's artifacts (`options::validate`), shared with the resolver's cache.
    /// Holding them pre-validated is what lets the per-language mappers be infallible.
    /// `sort_imports` is for JS children too: a Markdown code block is a whole program,
    /// sorted like the host's own imports (and a Vue `<script>`'s).
    validated: Arc<ValidatedOptions>,
    js: OnceLock<JsFormatOptions>,
    graphql: OnceLock<GraphqlFormatOptions>,
    /// One cell per [`CssVariant`]: JSDoc fences dispatch css/scss/less as-is, while css-in-js always uses Scss.
    css: [OnceLock<CssFormatOptions>; 3],
    yaml: OnceLock<YamlFormatOptions>,
    /// One cell per [`JsonVariant`].
    json: [OnceLock<JsonFormatOptions>; 4],
    markdown: OnceLock<MarkdownFormatOptions>,
    toml: OnceLock<TomlFormatOptions>,
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
    /// Private so [`Self::for_root`] stays the only construction recipe.
    fn new(config: Arc<FormatConfig>, validated: Arc<ValidatedOptions>) -> Self {
        Self {
            config,
            validated,
            js: OnceLock::new(),
            graphql: OnceLock::new(),
            css: [OnceLock::new(), OnceLock::new(), OnceLock::new()],
            yaml: OnceLock::new(),
            json: [OnceLock::new(), OnceLock::new(), OnceLock::new(), OnceLock::new()],
            markdown: OnceLock::new(),
            toml: OnceLock::new(),
            #[cfg(feature = "napi")]
            prettier: PrettierOptions::default(),
        }
    }

    /// The one construction recipe for a root formatter run at `path`:
    /// [`Self::new`] plus the napi-only path recording
    /// (the pure build has no JS-side consumers, so `path` goes unused there).
    /// `validated` is the config-resolution gate's artifacts (`options::validate`),
    /// carried from resolution so they never get re-derived (or re-fail) here.
    pub fn for_root(
        config: Arc<FormatConfig>,
        validated: Arc<ValidatedOptions>,
        path: &std::path::Path,
    ) -> Arc<Self> {
        let dispatch_config = Self::new(config, validated);
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

    pub fn js_options(&self) -> JsFormatOptions {
        self.js
            .get_or_init(|| {
                to_oxc_formatter(
                    &self.config,
                    self.validated.core,
                    self.validated.sort_imports.clone(),
                )
            })
            .clone()
    }

    pub fn graphql_options(&self) -> GraphqlFormatOptions {
        *self.graphql.get_or_init(|| to_oxc_formatter_graphql(&self.config, self.validated.core))
    }

    pub fn css_options(&self, variant: CssVariant) -> CssFormatOptions {
        let cell = match variant {
            CssVariant::Css => &self.css[0],
            CssVariant::Scss => &self.css[1],
            CssVariant::Less => &self.css[2],
        };
        *cell.get_or_init(|| to_oxc_formatter_css(&self.config, self.validated.core, variant))
    }

    pub fn yaml_options(&self) -> YamlFormatOptions {
        *self.yaml.get_or_init(|| to_oxc_formatter_yaml(&self.config, self.validated.core))
    }

    pub fn json_options(&self, variant: JsonVariant) -> JsonFormatOptions {
        let cell = match variant {
            JsonVariant::Json => &self.json[0],
            JsonVariant::Jsonc => &self.json[1],
            JsonVariant::Json5 => &self.json[2],
            JsonVariant::JsonStringify => &self.json[3],
        };
        *cell.get_or_init(|| to_oxc_formatter_json(&self.config, self.validated.core, variant))
    }

    pub fn markdown_options(&self) -> MarkdownFormatOptions {
        *self.markdown.get_or_init(|| to_oxc_formatter_markdown(&self.config, self.validated.core))
    }

    pub fn toml_options(&self) -> TomlFormatOptions {
        *self.toml.get_or_init(|| to_oxc_formatter_toml(&self.config, self.validated.core))
    }

    /// Printer options from the shared resolved core bundle;
    /// the fence adapter ([`super::jsdoc_fence`]) derives its per-fence options from these
    /// (width overridden to the fence's effective width).
    pub fn print_options(&self) -> PrinterOptions {
        self.validated.core.as_print_options()
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

    /// [`Self::prettier_options`] for one `language`: its parser, plus the payload of the plugin it needs.
    /// `None` when that plugin is not enabled (svelte / astro without the config key): the part stays as-is.
    pub fn prettier_options_for(&self, language: PrettierLanguage) -> Option<serde_json::Value> {
        if language.missing_plugin(&self.config).is_some() {
            return None;
        }

        let mut options = self.prettier_options().clone();
        crate::core::options::inject_parser(&mut options, language.parser());
        // e.g. svelte-in-md (a ```svelte code block), svelte-in-mdx-in-md
        crate::core::options::inject_opt_in_plugin_payloads(&mut options, language, &self.config);
        Some(options)
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
        match route_embedded(request.language) {
            Route::Native(language) => {
                Ok(format_native(language.trace_label(), || match language {
                    NativeLanguage::Js(source_type) => {
                        let embedded_in = request
                            .parent_context_as::<XxxInMarkdownCodeBlock>()
                            .map(|_| JsEmbeddedIn::MarkdownCodeBlock);
                        oxc_formatter::format_to_ir(
                            session,
                            text,
                            source_type,
                            dispatch_config.js_options(),
                            embedded_in,
                        )
                    }
                    NativeLanguage::Graphql => oxc_formatter_graphql::format_to_ir(
                        session,
                        text,
                        dispatch_config.graphql_options(),
                    ),
                    NativeLanguage::Css(variant) => {
                        // css-in-js (typed `CssInJsTemplate` context) is always parsed as SCSS with `${}` placeholder markers.
                        // Any other caller gets the strict standalone grammar with the fence/request language's variant.
                        let (variant, template_placeholders) =
                            if request.parent_context_as::<CssInJsTemplate>().is_some() {
                                (CssVariant::Scss, true)
                            } else {
                                (variant, false)
                            };
                        oxc_formatter_css::format_to_ir(
                            session,
                            text,
                            dispatch_config.css_options(variant),
                            template_placeholders,
                        )
                    }
                    NativeLanguage::Yaml => oxc_formatter_yaml::format_to_ir(
                        session,
                        text,
                        dispatch_config.yaml_options(),
                    ),
                    NativeLanguage::Json(variant) => oxc_formatter_json::format_to_ir(
                        session,
                        text,
                        dispatch_config.json_options(variant),
                    ),
                    NativeLanguage::Markdown => {
                        // `~` fences in a JS template, and in Markdown nested in one (md-in-md-in-js)
                        let in_js_template =
                            request.parent_context_as::<MarkdownInJsTemplate>().is_some()
                                || request
                                    .parent_context_as::<XxxInMarkdownCodeBlock>()
                                    .is_some_and(|c| c.in_js_template);
                        oxc_formatter_markdown::format_to_ir(
                            session,
                            text,
                            dispatch_config.markdown_options(),
                            in_js_template,
                        )
                    }
                    NativeLanguage::Toml => oxc_formatter_toml::format_to_ir(
                        session,
                        text,
                        dispatch_config.toml_options(),
                    ),
                }))
            }

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

            // A language without a formatter is a deliberate skip, in every build
            Route::Unsupported => {
                debug!("No formatter for language '{}', part stays as-is", request.language);
                Ok(DispatchResponse::PreserveOriginal)
            }
        }
    })
}

/// Runs one native branch: a parse failure is a deliberate skip
/// (the embedded part stays as-is), never an operational error.
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
    use crate::core::{options::ValidatedOptions, oxfmtrc::FormatConfig};

    fn dispatch_config() -> Arc<ResolvedDispatchConfig> {
        Arc::new(ResolvedDispatchConfig::new(
            Arc::new(FormatConfig::default()),
            Arc::new(ValidatedOptions { core: CoreFormatOptions::default(), sort_imports: None }),
        ))
    }

    /// Every language the routing table claims as native must format
    /// WITHOUT a fallback installed
    /// (an accidentally dropped [`route_embedded`](crate::core::language::route_embedded) entry would fall through to `PreserveOriginal` and fail here).
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

        for language in [
            "js",
            "javascript",
            "jsx",
            "mjs",
            "cjs",
            "ts",
            "typescript",
            "angular-ts",
            "mts",
            "cts",
            "tsx",
            "graphql",
            "gql",
            "css",
            "postcss",
            "scss",
            "less",
            "yaml",
            "yml",
            "json",
            "jsonc",
            "json5",
            "markdown",
            "md",
            "toml",
        ] {
            let text = match language {
                "graphql" | "gql" => "{ a }",
                "css" | "postcss" | "scss" | "less" => "a { color: red }",
                "yaml" | "yml" => "a: 1",
                "json" | "jsonc" | "json5" => "{ \"a\": 1 }",
                "markdown" | "md" => "#  a",
                _ => "a  =  1",
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

        for (language, text) in [("html", "<div></div>"), ("ini", "a = 1")] {
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
