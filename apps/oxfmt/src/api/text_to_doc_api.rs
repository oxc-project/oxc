use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, instrument};

use oxc_allocator::Allocator;
use oxc_formatter::FragmentContext;
use oxc_formatter_css::CssVariant;
use oxc_span::SourceType;

use crate::{
    core::{
        ExternalFormatter, JsFormatEmbeddedCb, JsFormatEmbeddedDocCb, JsFormatFileCb,
        JsSortTailwindClassesCb,
        options::{
            inject_filepath, inject_tailwind_plugin_payload, to_oxc_formatter_css,
            to_oxc_formatter_graphql, to_prettier,
        },
        oxfmtrc::FormatConfig,
        resolve_for_embedded_js, utils,
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
    let fragment_kind = match parent_context {
        "vue-for-binding-left" => Some(FragmentKind::VueForBindingLeft),
        "vue-bindings" => Some(FragmentKind::VueBindings),
        "vue-script-generic" => Some(FragmentKind::VueScriptGeneric),
        // "vue-script", "svelte-script"
        _ => None,
    };

    let doc_json = if let Some(kind) = fragment_kind {
        run_fragment(source_ext, source_text, oxfmt_plugin_options_json, kind)?
    } else {
        run_full(
            source_ext,
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
#[instrument(level = "debug", name = "oxfmt::text_to_doc::full", skip_all, fields(%source_ext))]
fn run_full(
    source_ext: &str,
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

    let external_formatter = ExternalFormatter::new(
        format_file_cb,
        format_embedded_cb,
        format_embedded_doc_cb,
        sort_tailwind_classes_cb,
    );

    let source_type = SourceType::from_extension(source_ext)
        .expect("source_ext should be a valid JS/TS extension");

    let resolved = resolve_for_embedded_js(config, parent_filepath)
        .expect("`_oxfmtPluginOptionsJson` should contain valid config");

    // Prettier options for callbacks that `oxc_formatter` may dispatch (e.g., CSS-in-JS).
    // The embedded JS context is treated as always Tailwind-capable, so the inject is unconditional.
    // The helper no-ops when user config has Tailwind disabled.
    let mut external_options = to_prettier(&resolved.config);
    inject_filepath(&mut external_options, &resolved.parent_filepath);
    inject_tailwind_plugin_payload(&mut external_options, &resolved.config);

    // Dual mapping of the same resolved config for the dispatcher's Rust branches.
    // Cannot fail here: `resolve_for_embedded_js()` already built `JsFormatOptions`
    // from this config, and both share the same `to_core_options()` validation.
    let graphql_options = to_oxc_formatter_graphql(&resolved.config)
        .expect("config was already validated by `resolve_for_embedded_js()`");
    // CSS-in-JS is always parsed as SCSS, mirroring Prettier's embed.
    let css_options = to_oxc_formatter_css(&resolved.config, CssVariant::Scss)
        .expect("config was already validated by `resolve_for_embedded_js()`");

    let external_callbacks = external_formatter.to_external_callbacks(
        &resolved.format_options,
        external_options,
        graphql_options,
        css_options,
    );
    let format_options = resolved.format_options;

    let allocator = Allocator::default();
    let formatted = match utils::run_blocking(|| {
        oxc_formatter::format(
            &allocator,
            source_text,
            source_type,
            *format_options,
            Some(external_callbacks),
        )
    }) {
        Ok(formatted) => formatted,
        Err(err) => {
            debug!("`oxc_formatter::format()` failed: {err:?}");
            external_formatter.cleanup();
            return None;
        }
    };

    let (elements, sorted_tailwind_classes) =
        formatted.into_document().into_elements_and_tailwind_classes();

    external_formatter.cleanup();
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
#[instrument(level = "debug", name = "oxfmt::text_to_doc::fragment", skip_all, fields(%source_ext, ?kind))]
fn run_fragment(
    source_ext: &str,
    source_text: &str,
    oxfmt_plugin_options_json: &str,
    kind: FragmentKind,
) -> Option<Value> {
    let source_type = SourceType::from_extension(source_ext)
        .expect("source_ext should be a valid JS/TS extension");

    let (config, parent_filepath) = parse_payload(oxfmt_plugin_options_json);
    // Reuses the same config resolver as `run_full()`, but only `format_options` is needed here,
    // since `run_fragment()` does not dispatch external formatter callbacks.
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
        formatted.into_document().into_elements_and_tailwind_classes();
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
