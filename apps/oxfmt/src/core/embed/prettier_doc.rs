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

use oxc_allocator::ArenaVec;
use oxc_formatter::HtmlInJsTemplate;
use oxc_formatter_core::{
    DispatchRequest, DispatchResponse, EmbeddedIr, FormatElement, FormatSession, Tag,
};

use crate::{
    core::{
        embed::{
            FormatEmbeddedDocWithConfigCallback,
            dispatcher::{PrettierDocFallback, ResolvedDispatchConfig},
        },
        support::PrettierLanguage,
    },
    prettier_compat::from_prettier_doc,
};

/// Build the Prettier Doc→IR fallback installed on the dispatcher's `Route::Prettier` arm.
/// The routing table already narrowed the language; only a language whose plugin is off is rejected (as-is).
pub fn build_prettier_fallback(
    dispatch_config: Arc<ResolvedDispatchConfig>,
    format_embedded_doc: FormatEmbeddedDocWithConfigCallback,
) -> PrettierDocFallback {
    Arc::new(
        move |session: &FormatSession<'_>,
              language: PrettierLanguage,
              request: &DispatchRequest<'_>| {
            let text = request.text;
            let parser_name = language.parser();
            let Some(options) = dispatch_config.prettier_options_for(language) else {
                debug!("The plugin for parser '{parser_name}' is not enabled, part stays as-is");
                return Ok(DispatchResponse::PreserveOriginal);
            };
            debug_span!("oxfmt::external::format_embedded_doc", parser = parser_name)
                .in_scope(|| {
                    let doc_json_str = (format_embedded_doc)(options, text).map_err(|err| {
                        format!(
                            "Failed to get Doc for embedded code (parser '{parser_name}'): {err}"
                        )
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
                    // html(angular)-in-js: indent the content when more than one node prints at the root
                    let indent = request
                        .parent_context_as::<HtmlInJsTemplate>()
                        .is_some_and(|c| c.indent_multiple_roots)
                        && metadata
                            .get("htmlHasMultipleRootElements")
                            .and_then(Value::as_bool)
                            .unwrap_or(true);
                    if indent {
                        let mut indented = ArenaVec::with_capacity_in(ir.len() + 2, &allocator);
                        indented.push(FormatElement::Tag(Tag::StartIndent));
                        indented.extend(ir);
                        indented.push(FormatElement::Tag(Tag::EndIndent));
                        ir = indented;
                    }
                    Ok(DispatchResponse::Formatted(EmbeddedIr { ir }))
                })
                .inspect_err(|err| {
                    debug!("Failed to format embedded doc for parser '{parser_name}': {err}");
                })
        },
    )
}
