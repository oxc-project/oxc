use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, instrument};

use oxc_allocator::Allocator;
use oxc_formatter::FragmentContext;
use oxc_formatter_core::{FormatSession, InputKind};
use oxc_span::SourceType;

use crate::{
    core::{
        EmbeddedCallbackResolved, ExternalServices, JsFormatEmbeddedCb, JsFormatEmbeddedDocCb,
        JsFormatFileCb, JsSortTailwindClassesCb,
        embed::{self, dispatcher::ResolvedDispatchConfig},
        oxfmtrc::FormatConfig,
        resolve_for_embedded_js,
    },
    prettier_compat::to_prettier_doc,
};

/// Fragment kind for embedded JS/TS contexts.
#[expect(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug)]
enum FragmentKind {
    /// `v-for` left-hand side: `(item, index) in items` → formats `item, index` part.
    VueForBindingLeft,
    /// `v-slot` / slot binding: `{ item }` → formats the destructured parameters.
    VueBindings,
    /// `<script generic="T extends Foo">` → formats type parameters without angle brackets.
    VueScriptGeneric,
}

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
    fields(source_ext = %source_ext, parent_context = %parent_context)
)]
pub fn run(
    source_ext: &str,
    source_text: &str,
    oxfmt_plugin_options_json: &str,
    parent_context: &str,
    format_file_cb: JsFormatFileCb,
    format_embedded_cb: JsFormatEmbeddedCb,
    format_embedded_doc_cb: JsFormatEmbeddedDocCb,
    sort_tailwind_classes_cb: JsSortTailwindClassesCb,
) -> Option<String> {
    // Embedded text belongs to the host file (`.vue`, `.md`, ...),
    // so the `SourceType` carries no file extension of its own.
    // `source_ext` selects the parse grammar only,
    // and extension-keyed formatter rules (e.g. the `.mts`/`.cts` trailing comma reservation) must not fire from it.
    //
    // The JS side owns the grammar resolution (including the `lang="tsx"` scan for Vue, see `hasTsxScriptBlock` in `apis.ts`),
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

    let fragment_kind = match parent_context {
        "vue-for-binding-left" => Some(FragmentKind::VueForBindingLeft),
        "vue-bindings" => Some(FragmentKind::VueBindings),
        "vue-script-generic" => Some(FragmentKind::VueScriptGeneric),
        // "vue-script" | "svelte-script"
        _ => None,
    };

    let doc_json = if let Some(kind) = fragment_kind {
        run_fragment(source_type, source_text, oxfmt_plugin_options_json, kind)?
    } else {
        run_full(
            source_type,
            source_text,
            oxfmt_plugin_options_json,
            format_file_cb,
            format_embedded_cb,
            format_embedded_doc_cb,
            sort_tailwind_classes_cb,
        )?
    };

    Some(serde_json::to_string(&doc_json).expect("Doc JSON serialization should not fail"))
}

// ---

/// Full mode:
/// - Format entire source as IR
/// - Convert IR to Prettier Doc
///
/// NOTE: Why we need to convert IR to Doc instead of just splitting by lines:
/// A simple line-splitting approach might seem sufficient and can cover most cases,
/// but it fails to handle newlines that appear within string, such as `TemplateLiteral`.
///
/// This is critical for `vueIndentScriptAndStyle: true`, (Prettier wraps the `<script>` content with `indent()`)
/// `literalline` (used for template literal content) is not affected by `indent()`,
/// while `hardline` (used for normal code) is.
#[instrument(level = "debug", name = "oxfmt::text_to_doc::full", skip_all, fields(?source_type))]
fn run_full(
    source_type: SourceType,
    source_text: &str,
    oxfmt_plugin_options_json: &str,
    format_file_cb: JsFormatFileCb,
    format_embedded_cb: JsFormatEmbeddedCb,
    format_embedded_doc_cb: JsFormatEmbeddedDocCb,
    sort_tailwind_classes_cb: JsSortTailwindClassesCb,
) -> Option<Value> {
    // Tailwind paths in the payload are already absolute (resolved by the host before serialization),
    // so no `cwd` is threaded through here.
    let (config, parent_filepath) = parse_payload(oxfmt_plugin_options_json);

    let external_services = ExternalServices::new(
        format_file_cb,
        format_embedded_cb,
        format_embedded_doc_cb,
        sort_tailwind_classes_cb,
    );

    let EmbeddedCallbackResolved { format_options, config, core, parent_filepath } =
        resolve_for_embedded_js(config, parent_filepath)
            .expect("`_oxfmtPluginOptionsJson` should contain valid config");

    // Per-language options (and the Prettier options JSON with the Tailwind payload)
    // are mapped lazily at dispatch time; `core` was validated during resolution.
    let dispatch_config = ResolvedDispatchConfig::for_root(&config, core, &parent_filepath);

    let services = embed::services::for_root(&external_services, &dispatch_config);

    let allocator = Allocator::default();
    let session = FormatSession::with_services(
        &allocator,
        // A Vue/Svelte `<script>` block is a complete document the host passes as embedded input,
        // never the owner of file envelopes (BOM / front matter).
        InputKind::VirtualDocument,
        services,
    );
    let formatted = match tokio::task::block_in_place(|| {
        oxc_formatter::format_with_session(&session, source_text, source_type, *format_options)
    }) {
        Ok(formatted) => formatted,
        Err(err) => {
            debug!("`oxc_formatter::format()` failed for {source_type:?}: {err:?}");
            external_services.cleanup();
            return None;
        }
    };

    let (elements, sorted_tailwind_classes) =
        formatted.into_final_document().into_elements_and_tailwind_classes();

    external_services.cleanup();
    Some(
        to_prettier_doc::format_elements_to_prettier_doc(elements, &sorted_tailwind_classes)
            .expect("Formatter IR to Prettier Doc conversion should not fail"),
    )
}

// ---

/// Fragment mode:
/// - Parse pre-wrapped source
///   - Prettier already wraps the fragment text before calling `textToDoc()`
///     - v-for / v-slot: `function _(PARAMS) {}`
///     - generic: `type T<PARAMS> = any`
/// - Extract target node
/// - Format as IR
/// - Convert to Prettier Doc JSON
#[instrument(level = "debug", name = "oxfmt::text_to_doc::fragment", skip_all, fields(?source_type, ?kind))]
fn run_fragment(
    source_type: SourceType,
    source_text: &str,
    oxfmt_plugin_options_json: &str,
    kind: FragmentKind,
) -> Option<Value> {
    let (config, parent_filepath) = parse_payload(oxfmt_plugin_options_json);
    // Reuses the same config resolver as `run_full()`, but only `format_options` is needed here,
    // since `run_fragment()` does not dispatch external services callbacks.
    let resolved = resolve_for_embedded_js(config, parent_filepath)
        .expect("`_oxfmtPluginOptionsJson` should contain valid config");
    let format_options = resolved.format_options;

    // Map the Prettier-side fragment kind to the formatter's usage context.
    // The parens-vs-no-parens / quote-style decisions live inside `format_fragment`.
    let context = match kind {
        FragmentKind::VueForBindingLeft => FragmentContext::FunctionParamsAsBindingLhs,
        FragmentKind::VueBindings => FragmentContext::FunctionParamsAsBinding,
        FragmentKind::VueScriptGeneric => FragmentContext::TypeParameters,
    };

    let allocator = Allocator::default();
    let formatted = match oxc_formatter::format_fragment(
        &allocator,
        source_text,
        source_type,
        *format_options,
        context,
    ) {
        Ok(formatted) => formatted,
        Err(err) => {
            debug!("`oxc_formatter::format_fragment()` failed: {err:?}");
            return None;
        }
    };

    let (elements, sorted_tailwind_classes) =
        formatted.into_final_document().into_elements_and_tailwind_classes();
    Some(
        to_prettier_doc::format_elements_to_prettier_doc(elements, &sorted_tailwind_classes)
            .expect("Formatter IR to Prettier Doc conversion should not fail"),
    )
}

// ---

/// Deserialize `_oxfmtPluginOptionsJson` into the typed config + parent filepath.
fn parse_payload(oxfmt_plugin_options_json: &str) -> (FormatConfig, PathBuf) {
    #[derive(Deserialize)]
    struct Payload {
        config: FormatConfig,
        filepath: String,
    }
    let payload: Payload = serde_json::from_str(oxfmt_plugin_options_json)
        .expect("`_oxfmtPluginOptionsJson` should deserialize");
    (payload.config, PathBuf::from(payload.filepath))
}
