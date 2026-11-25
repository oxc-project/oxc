use std::{path::PathBuf, sync::Arc};

use serde::Deserialize;
use tracing::{debug, instrument};

use oxc_allocator::Allocator;
use oxc_formatter_core::{FormatSession, InputKind};
use oxc_span::SourceType;

use crate::{
    core::{
        ExternalServices, JsFormatEmbeddedDocCb, JsFormatFileCb, JsSortTailwindClassesCb,
        embed::{self, dispatcher::ResolvedDispatchConfig},
        options::{ValidatedOptions, validate},
        oxfmtrc::FormatConfig,
    },
    prettier_compat::to_prettier_doc,
};

/// `js_text_to_doc()` implementation for NAPI API.
///
/// Returns `None` on failure.
/// Prettier's `multiparser.js` silently swallows errors from `textToDoc()` in production,
/// so detailed error reporting is unnecessary.
/// Errors are logged via `tracing::debug!` for observability with `OXC_LOG=debug`.
#[instrument(
    level = "debug",
    name = "oxfmt::text_to_doc",
    skip_all,
    fields(source_ext = %source_ext)
)]
pub fn run(
    source_ext: &str,
    source_text: &str,
    oxfmt_plugin_options_json: &str,
    format_file_cb: JsFormatFileCb,
    format_embedded_doc_cb: JsFormatEmbeddedDocCb,
    sort_tailwind_classes_cb: JsSortTailwindClassesCb,
) -> Option<String> {
    // Embedded text belongs to the host file (`.svelte`, `.astro`),
    // so the `SourceType` carries no file extension of its own.
    // `source_ext` selects the parse grammar only,
    // and extension-keyed formatter rules (e.g. the `.mts`/`.cts` trailing comma reservation) must not fire from it.
    //
    // The JS side owns the grammar resolution,
    // so there is no parse retry here: a block
    // that fails to parse under its declared grammar is left unformatted
    // (`textToDoc()` error → Prettier keeps the original text).
    let source_type = match source_ext {
        "jsx" => SourceType::unambiguous().with_jsx(true),
        "ts" => SourceType::ts(),
        "tsx" => SourceType::tsx(),
        _ => {
            unreachable!("text-to-doc.ts should pass `source_ext` as one of 'jsx', 'ts', or 'tsx'")
        }
    };

    // Tailwind paths in the payload are already absolute (resolved by the host before serialization),
    // so no `cwd` is threaded through here.
    let (config, validated, parent_filepath) = parse_payload(oxfmt_plugin_options_json);

    let external_services =
        ExternalServices::new(format_file_cb, format_embedded_doc_cb, sort_tailwind_classes_cb);
    let _cleanup = external_services.cleanup_guard();

    // Per-language options (and the Prettier options JSON with the Tailwind payload)
    // are mapped lazily at dispatch time.
    let dispatch_config =
        ResolvedDispatchConfig::for_root(Arc::new(config), Arc::new(validated), &parent_filepath);
    let format_options = dispatch_config.js_options();

    let services = embed::services::for_root(&external_services, &dispatch_config);

    let allocator = Allocator::default();
    let session = FormatSession::with_services(
        &allocator,
        // A Svelte/Astro `<script>` (or Astro frontmatter) block is a complete document the host passes as embedded input,
        // never the owner of file envelopes (BOM / front matter).
        InputKind::VirtualDocument,
        services,
    );
    let formatted = match tokio::task::block_in_place(|| {
        oxc_formatter::format_with_session(&session, source_text, source_type, format_options)
    }) {
        Ok(formatted) => formatted,
        Err(err) => {
            debug!("`oxc_formatter::format()` failed for {source_type:?}: {err:?}");
            return None;
        }
    };

    // NOTE: Why the IR is converted to a Doc instead of split into lines:
    // a newline in a string (a `TemplateLiteral`) must stay a `literalline`, which an `indent()` around it does not affect,
    // while a `hardline` (normal code) does: Prettier indents a `<script>` with `svelteIndentScriptAndStyle`.
    let (elements, sorted_tailwind_classes) =
        formatted.into_final_document().into_elements_and_tailwind_classes();
    let doc_json =
        to_prettier_doc::format_elements_to_prettier_doc(elements, &sorted_tailwind_classes)
            .expect("Formatter IR to Prettier Doc conversion should not fail");

    Some(serde_json::to_string(&doc_json).expect("Doc JSON serialization should not fail"))
}

// ---

/// Deserialize `_oxfmtPluginOptionsJson` into the validated config + parent filepath.
fn parse_payload(oxfmt_plugin_options_json: &str) -> (FormatConfig, ValidatedOptions, PathBuf) {
    #[derive(Deserialize)]
    struct Payload {
        config: FormatConfig,
        filepath: String,
    }
    let payload: Payload = serde_json::from_str(oxfmt_plugin_options_json)
        .expect("`_oxfmtPluginOptionsJson` should deserialize");
    let validated =
        validate(&payload.config).expect("`_oxfmtPluginOptionsJson` should contain valid config");
    (payload.config, validated, PathBuf::from(payload.filepath))
}
