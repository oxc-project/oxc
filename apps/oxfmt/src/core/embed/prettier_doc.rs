//! The Prettier Doc→IR channel: the one path for `Route::Prettier` languages (napi only).
//!
//! Sends the text to JS `printToDoc()`, then converts the returned Doc JSON into formatter IR
//! that integrates into the parent's arena / `GroupId` space.
//! Fills the dispatcher's optional [`PrettierDocFallback`] slot;
//! the string twin is [`super::prettier_string`].

use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, debug_span};

use oxc_formatter::HtmlEmbedMeta;
use oxc_formatter_core::{DispatchPayload, DispatchResponse, FormatSession};

use crate::{
    core::{
        embed::{
            FormatEmbeddedDocWithConfigCallback,
            dispatcher::{PrettierDocFallback, PrettierLanguage, ResolvedDispatchConfig},
        },
        options::inject_parser,
    },
    prettier_compat::from_prettier_doc,
};

/// Build the Prettier Doc→IR fallback installed on the dispatcher's `Route::Prettier` arm.
/// The routing table already narrowed the language, so there is nothing left to reject here.
pub fn build_prettier_fallback(
    dispatch_config: Arc<ResolvedDispatchConfig>,
    format_embedded_doc: FormatEmbeddedDocWithConfigCallback,
) -> PrettierDocFallback {
    Arc::new(move |session: &FormatSession<'_>, language: PrettierLanguage, text: &str| {
        let parser_name = language.parser();
        debug_span!("oxfmt::external::format_embedded_doc", parser = parser_name)
            .in_scope(|| {
                let mut options = dispatch_config.prettier_options().clone();
                inject_parser(&mut options, parser_name);
                let doc_json_str = (format_embedded_doc)(options, text).map_err(|err| {
                    format!("Failed to get Doc for embedded code (parser '{parser_name}'): {err}")
                })?;
                // Prettier's Doc can produce deeply nested arrays
                // (e.g., md-in-js with `proseWrap: preserve`, which nests each word in `[[[prev, " "], word], " "]`).
                // The default recursion limit of 128 is not enough for long paragraphs.
                // This only affects this deserialization call;
                // other `serde_json` usage in the codebase keeps the default limit.
                let mut de = serde_json::Deserializer::from_str(&doc_json_str);
                de.disable_recursion_limit();
                let doc_json = serde_json::Value::deserialize(&mut de)
                    .map_err(|e| format!("Failed to parse Doc JSON: {e}"))?;

                let allocator = session.allocator();
                let (mut ir, metadata) = from_prettier_doc::convert_envelope(
                    doc_json,
                    allocator,
                    session.group_id_builder(),
                )?;
                from_prettier_doc::postprocess(&mut ir, allocator);
                // HTML/Angular additionally surface `HtmlEmbedMeta` to the embed site.
                let child_context = language.wants_html_meta().then(|| {
                    Box::new(HtmlEmbedMeta {
                        has_multiple_root_elements: metadata
                            .get("htmlHasMultipleRootElements")
                            .and_then(Value::as_bool),
                    }) as Box<dyn std::any::Any>
                });
                Ok(DispatchResponse::Formatted(DispatchPayload {
                    doc: ir,
                    tailwind_classes: Vec::new(),
                    child_context,
                }))
            })
            .inspect_err(|err| {
                debug!("Failed to format embedded doc for parser '{parser_name}': {err}");
            })
    })
}
