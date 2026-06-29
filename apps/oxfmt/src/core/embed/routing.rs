//! Language-aware routing for embedded IRs that come from the Prettier Doc path.
//!
//! [`from_prettier_doc::convert_envelope`] is a pure primitive (envelope unwrap +
//! Doc→IR). What postprocessing to apply, what placeholder format to count, and
//! what metadata to surface differ per language — that orchestration lives here.
//!
//! Today only HTML/Angular and Markdown reach the Doc path; CSS and GraphQL are
//! handled by Rust formatter crates and never enter this routing.

use serde_json::Value;

use oxc_allocator::Allocator;
use oxc_formatter::HtmlEmbedMeta;
use oxc_formatter_core::{DispatchResult, UniqueGroupIdBuilder};

use crate::{
    core::embed::postprocess::{TemplateEscape, postprocess},
    prettier_compat::from_prettier_doc,
};

/// Converts parsed Prettier Doc JSON values into a [`DispatchResult`].
///
/// Handles language-specific processing:
/// - HTML/Angular: postprocess with full template-char escaping + count
///   `PRETTIER_HTML_PLACEHOLDER_N_M_IN_JS` markers; surfaces
///   [`HtmlEmbedMeta`] (`htmlHasMultipleRootElements`).
/// - Markdown: postprocess with raw-backtick escaping (uses `.raw` quasi values).
pub fn to_format_elements_for_template<'a>(
    language: &str,
    doc_jsons: Vec<Value>,
    allocator: &'a Allocator,
    group_id_builder: &UniqueGroupIdBuilder,
) -> Result<DispatchResult<'a>, String> {
    match language {
        // NOTE: no "css" / "graphql" arms — those languages never reach the
        // Prettier Doc path (their dispatcher branches are Rust-only).
        "html" | "angular" => {
            let (mut ir, metadata) = from_prettier_doc::convert_envelope(
                doc_jsons.into_iter().next().expect("Doc JSON for HTML"),
                allocator,
                group_id_builder,
            )?;
            let html_has_multiple_root_elements =
                metadata.get("htmlHasMultipleRootElements").and_then(Value::as_bool);
            let placeholder_count = postprocess(
                &mut ir,
                allocator,
                // HTML/Angular use `.cooked` values, so template chars need escaping
                TemplateEscape::Full,
                Some(("PRETTIER_HTML_PLACEHOLDER_", "_IN_JS")),
            );
            Ok(DispatchResult {
                docs: vec![ir],
                tailwind_classes: Vec::new(),
                placeholder_count: Some(placeholder_count),
                meta: Some(Box::new(HtmlEmbedMeta {
                    has_multiple_root_elements: html_has_multiple_root_elements,
                })),
            })
        }
        "markdown" => {
            let (mut ir, _) = from_prettier_doc::convert_envelope(
                doc_jsons.into_iter().next().expect("Doc JSON for Markdown"),
                allocator,
                group_id_builder,
            )?;
            postprocess(
                &mut ir,
                allocator,
                // Markdown uses `.raw` values with backtick unescaping on Rust side
                TemplateEscape::RawBacktick,
                None,
            );
            Ok(DispatchResult {
                docs: vec![ir],
                tailwind_classes: Vec::new(),
                placeholder_count: None,
                meta: None,
            })
        }
        _ => unreachable!("Unsupported embedded_doc language: {language}"),
    }
}
