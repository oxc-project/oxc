//! Prettier Doc→IR fallback for embedded languages without a Rust formatter.
//!
//! Sends the text to JS `printToDoc()`, then converts the returned Doc JSON into
//! formatter IR that integrates into the parent's arena / `GroupId` space.
//! napi-only: the pure Rust build runs the registry without this fallback.

use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, debug_span};

use oxc_allocator::Allocator;
use oxc_formatter::HtmlEmbedMeta;
use oxc_formatter_core::{DispatchOutcome, DispatchResult, FormatSession, UniqueGroupIdBuilder};

use crate::{
    core::{
        embed::{
            FormatEmbeddedDocWithConfigCallback,
            dispatcher::{PrettierDocFallback, ResolvedDispatchConfig},
            language_to_prettier_parser,
        },
        options::inject_parser,
    },
    prettier_compat::from_prettier_doc,
};

/// Build the Prettier Doc→IR fallback installed on the dispatcher's default arm.
pub fn build_prettier_fallback(
    dispatch_config: Arc<ResolvedDispatchConfig>,
    format_embedded_doc: FormatEmbeddedDocWithConfigCallback,
) -> PrettierDocFallback {
    Arc::new(move |session: &FormatSession<'_>, language: &str, text: &str| {
        let Some(parser_name) = language_to_prettier_parser(language) else {
            // An unsupported language is a deliberate skip, not an operational error.
            debug!("No Prettier parser for language '{language}', part stays as-is");
            return Ok(DispatchOutcome::PreserveOriginal);
        };
        debug_span!("oxfmt::external::format_embedded_doc", parser = parser_name)
            .in_scope(|| {
                let mut options = dispatch_config.external_options().clone();
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

                to_format_elements_for_template(
                    language,
                    doc_json,
                    session.allocator(),
                    session.group_id_builder(),
                )
                .map(DispatchOutcome::Formatted)
            })
            .inspect_err(|err| {
                debug!("Failed to format embedded doc for parser '{parser_name}': {err}");
            })
    })
}

/// Converts parsed Prettier Doc JSON values into a [`DispatchResult`].
///
/// Per-language work:
/// - HTML/Angular: structural postprocess; surfaces [`HtmlEmbedMeta`] (`htmlHasMultipleRootElements`).
/// - Markdown: structural postprocess.
fn to_format_elements_for_template<'a>(
    language: &str,
    doc_json: Value,
    allocator: &'a Allocator,
    group_id_builder: &UniqueGroupIdBuilder,
) -> Result<DispatchResult<'a>, String> {
    // The language gate runs BEFORE the conversion work.
    // NOTE: Other languages never reach the Prettier Doc path (their dispatcher branches are Rust-only).
    let needs_html_meta = match language {
        "html" | "angular" => true,
        "markdown" => false,
        _ => unreachable!("Unsupported embedded_doc language: {language}"),
    };

    let (mut ir, metadata) =
        from_prettier_doc::convert_envelope(doc_json, allocator, group_id_builder)?;
    from_prettier_doc::postprocess(&mut ir, allocator);
    let meta = needs_html_meta.then(|| {
        Box::new(HtmlEmbedMeta {
            has_multiple_root_elements: metadata
                .get("htmlHasMultipleRootElements")
                .and_then(Value::as_bool),
        }) as Box<dyn std::any::Any>
    });
    Ok(DispatchResult { doc: ir, tailwind_classes: Vec::new(), meta })
}
