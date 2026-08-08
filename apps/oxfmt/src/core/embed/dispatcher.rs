//! Native dispatch registry: the `FormatDispatcher` assembly shared by every build.
//!
//! Each language maps to a Rust formatter where available;
//! everything else goes to the napi-only Prettier Doc→IR fallback ([`super::prettier_fallback`]) when
//! one is supplied, and is deliberately preserved as-is otherwise (pure Rust build).

use std::sync::{Arc, OnceLock};

use tracing::{debug, debug_span};

use oxc_formatter::CssInJsTemplate;
use oxc_formatter_core::{
    CoreFormatOptions, DispatchOutcome, DispatchRequest, DispatchResult, EmbeddedIr,
    FormatDispatcher, FormatSession,
};
use oxc_formatter_css::{CssFormatOptions, CssVariant};
use oxc_formatter_graphql::GraphqlFormatOptions;
use oxc_formatter_yaml::YamlFormatOptions;

use crate::core::{
    options::{to_oxc_formatter_css, to_oxc_formatter_graphql, to_oxc_formatter_yaml},
    oxfmtrc::FormatConfig,
};

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
    pub fn new(config: Arc<FormatConfig>, core: CoreFormatOptions) -> Self {
        Self {
            config,
            core,
            graphql: OnceLock::new(),
            css: [OnceLock::new(), OnceLock::new(), OnceLock::new()],
            yaml: OnceLock::new(),
            #[cfg(feature = "napi")]
            path: std::path::PathBuf::new(),
            #[cfg(feature = "napi")]
            external_options: OnceLock::new(),
        }
    }

    /// Sets the host file path for `filepath` injection into [`Self::external_options`];
    /// both napi construction sites chain this.
    #[cfg(feature = "napi")]
    #[must_use]
    pub fn with_path(mut self, path: std::path::PathBuf) -> Self {
        self.path = path;
        self
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
    dyn for<'a> Fn(&FormatSession<'a>, &str, &[&str]) -> Result<DispatchOutcome<'a>, String>
        + Send
        + Sync,
>;

/// Build the `FormatDispatcher` carried by `ExternalCallbacks` (and, once hosts are session-aware, by `FormatSession`):
/// Rust formatters for graphql / css / yaml (no Prettier fallback for them),
/// `fallback` for everything else.
pub fn build_dispatcher(
    dispatch_config: Arc<ResolvedDispatchConfig>,
    fallback: Option<PrettierDocFallback>,
) -> FormatDispatcher {
    Arc::new(move |session: &FormatSession<'_>, request: DispatchRequest<'_>| {
        // Rust implementations replace branches one by one;
        match request.language {
            "graphql" | "gql" => Ok(formatted_or_preserved(
                format_graphql_to_irs(session, request.texts, dispatch_config.graphql_options()),
                "format_graphql_to_irs",
            )),
            "css" | "scss" | "less" => {
                // A wrong text count is a host-contract violation, not a parse failure:
                // unlike GraphQL's one-IR-per-quasi, the CSS embed joins quasis with
                // placeholders into a single text before dispatching.
                let [text] = request.texts else {
                    return Err(format!(
                        "CSS dispatch expects exactly one text, got {}",
                        request.texts.len()
                    ));
                };
                // css-in-js (typed `CssInJsTemplate` context) is always parsed as SCSS
                // with `${}` placeholder markers.
                // Any other caller gets the strict standalone grammar with the variant
                // taken from the fence/request language.
                let (variant, template_placeholders) = if request
                    .parent_context
                    .is_some_and(|c| c.downcast_ref::<CssInJsTemplate>().is_some())
                {
                    (CssVariant::Scss, true)
                } else {
                    (css_variant_for(request.language), false)
                };
                Ok(formatted_or_preserved(
                    format_css_to_ir(
                        session,
                        text,
                        dispatch_config.css_options(variant),
                        template_placeholders,
                    ),
                    "format_css_to_ir",
                ))
            }
            "yaml" | "yml" => Ok(formatted_or_preserved(
                format_yaml_to_irs(session, request.texts, dispatch_config.yaml_options()),
                "format_yaml_to_irs",
            )),
            // Everything else: Prettier fallback (Doc→IR path) when available
            _ => {
                if let Some(fallback) = &fallback {
                    fallback(session, request.language, request.texts)
                } else {
                    // A language without a formatter is a deliberate skip.
                    debug!("No formatter for language '{}', part stays as-is", request.language);
                    Ok(DispatchOutcome::PreserveOriginal)
                }
            }
        }
    })
}

/// Map a fence/request language to its standalone [`CssVariant`]
/// (css-in-js is Scss regardless of the tag; see the dispatcher's css arm).
pub fn css_variant_for(language: &str) -> CssVariant {
    match language {
        "scss" => CssVariant::Scss,
        "less" => CssVariant::Less,
        _ => CssVariant::Css,
    }
}

/// Maps a native branch result: a parse failure is a deliberate skip
/// (the embedded part stays as-is), never an operational error.
fn formatted_or_preserved<'a>(
    result: Result<DispatchResult<'a>, String>,
    debug_label: &str,
) -> DispatchOutcome<'a> {
    match result {
        Ok(result) => DispatchOutcome::Formatted(result),
        Err(err) => {
            debug!("`{debug_label}` failed, part stays as-is: {err}");
            DispatchOutcome::PreserveOriginal
        }
    }
}

/// Format each text as a standalone GraphQL document via `oxc_formatter_graphql`,
/// returning one IR per text (the IR-channel contract for GraphQL).
///
/// Any parse error fails the whole batch (an embedded template is all-or-nothing).
fn format_graphql_to_irs<'a>(
    session: &FormatSession<'a>,
    texts: &[&str],
    options: GraphqlFormatOptions,
) -> Result<DispatchResult<'a>, String> {
    let docs = texts
        .iter()
        .map(|text| {
            debug_span!("oxfmt::external::format_graphql_to_ir").in_scope(|| {
                let embedded = oxc_formatter_graphql::format_to_ir(session, text, options)
                    .map_err(|err| err.to_string())?;
                Ok(embedded.ir)
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(DispatchResult { docs, tailwind_classes: Vec::new(), meta: None })
}

/// Format the single joined CSS text (placeholders included) via `oxc_formatter_css`,
/// returning one IR per call (the IR-channel contract for CSS).
fn format_css_to_ir<'a>(
    session: &FormatSession<'a>,
    text: &str,
    options: CssFormatOptions,
    template_placeholders: bool,
) -> Result<DispatchResult<'a>, String> {
    debug_span!("oxfmt::external::format_css_to_ir").in_scope(|| {
        let EmbeddedIr { ir, tailwind_classes } =
            oxc_formatter_css::format_to_ir(session, text, options, template_placeholders)
                .map_err(|err| err.to_string())?;
        Ok(DispatchResult { docs: vec![ir], tailwind_classes, meta: None })
    })
}

/// Format each text as a standalone YAML document via `oxc_formatter_yaml`,
/// returning one IR per text (front matter bodies and future fenced blocks).
///
/// Any parse error fails the whole batch (an embedded template is all-or-nothing).
fn format_yaml_to_irs<'a>(
    session: &FormatSession<'a>,
    texts: &[&str],
    options: YamlFormatOptions,
) -> Result<DispatchResult<'a>, String> {
    let docs = texts
        .iter()
        .map(|text| {
            debug_span!("oxfmt::external::format_yaml_to_ir").in_scope(|| {
                let embedded = oxc_formatter_yaml::format_to_ir(session, text, options)
                    .map_err(|err| err.to_string())?;
                Ok(embedded.ir)
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(DispatchResult { docs, tailwind_classes: Vec::new(), meta: None })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use oxc_allocator::Allocator;
    use oxc_formatter_core::{
        CoreFormatOptions, DispatchOutcome, DispatchRequest, FormatSession, InputKind,
    };

    use super::{ResolvedDispatchConfig, build_dispatcher};
    use crate::core::oxfmtrc::FormatConfig;

    fn dispatch_config() -> Arc<ResolvedDispatchConfig> {
        Arc::new(ResolvedDispatchConfig::new(
            Arc::new(FormatConfig::default()),
            CoreFormatOptions::default(),
        ))
    }

    /// Pure-build criterion: the native registry dispatches YAML with no fallback installed.
    #[test]
    fn native_yaml_dispatch_works_without_fallback() {
        let allocator = Allocator::default();
        let session = FormatSession::new(
            &allocator,
            InputKind::PhysicalFile,
            Some(build_dispatcher(dispatch_config(), None)),
        );

        let outcome = session.dispatch(DispatchRequest {
            language: "yaml",
            texts: &["a:   1"],
            input_kind: InputKind::Fragment,
            parent_context: None,
        });
        assert!(
            matches!(outcome, Ok(DispatchOutcome::Formatted(ref result)) if result.docs.len() == 1)
        );
    }

    #[test]
    fn unsupported_language_without_fallback_preserves_original() {
        let allocator = Allocator::default();
        let session = FormatSession::new(
            &allocator,
            InputKind::PhysicalFile,
            Some(build_dispatcher(dispatch_config(), None)),
        );

        let outcome = session.dispatch(DispatchRequest {
            language: "html",
            texts: &["<div></div>"],
            input_kind: InputKind::Fragment,
            parent_context: None,
        });
        assert!(matches!(outcome, Ok(DispatchOutcome::PreserveOriginal)));
    }
}
