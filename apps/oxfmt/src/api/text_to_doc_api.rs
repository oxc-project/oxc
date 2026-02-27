use std::path::PathBuf;

use serde_json::Value;
use tracing::{debug, instrument};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_formatter::{
    AstNode, AstNodes, FormatOptions, FormatVueBindingParams, FormatVueScriptGeneric, Formatter,
    get_parse_options,
};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;

use crate::{
    core::{
        ExternalFormatter, FormatFileStrategy, FormatResult, JsFormatEmbeddedCb,
        JsFormatEmbeddedDocCb, JsFormatFileCb, JsInitExternalFormatterCb, JsSortTailwindClassesCb,
        ResolvedOptions, SourceFormatter, resolve_options_from_value,
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
    init_external_formatter_cb: JsInitExternalFormatterCb,
    format_embedded_cb: JsFormatEmbeddedCb,
    format_embedded_doc_cb: JsFormatEmbeddedDocCb,
    format_file_cb: JsFormatFileCb,
    sort_tailwind_classes_cb: JsSortTailwindClassesCb,
) -> Option<String> {
    let fragment_kind = match parent_context {
        "vue-for-binding-left" => Some(FragmentKind::VueForBindingLeft),
        "vue-bindings" => Some(FragmentKind::VueBindings),
        "vue-script-generic" => Some(FragmentKind::VueScriptGeneric),
        // "vue-script"
        _ => None,
    };

    let doc_json = if let Some(kind) = fragment_kind {
        run_fragment(source_ext, source_text, oxfmt_plugin_options_json, kind)?
    } else {
        run_full(
            source_ext,
            source_text,
            oxfmt_plugin_options_json,
            init_external_formatter_cb,
            format_embedded_cb,
            format_embedded_doc_cb,
            format_file_cb,
            sort_tailwind_classes_cb,
        )?
    };

    Some(serde_json::to_string(&doc_json).expect("Doc JSON serialization should not fail"))
}

// ---

/// Full mode:
/// - Format entire source as text
/// - Return hardline-joined Doc string
#[instrument(level = "debug", name = "oxfmt::text_to_doc::full", skip_all, fields(%source_ext))]
fn run_full(
    source_ext: &str,
    source_text: &str,
    oxfmt_plugin_options_json: &str,
    init_external_formatter_cb: JsInitExternalFormatterCb,
    format_embedded_cb: JsFormatEmbeddedCb,
    format_embedded_doc_cb: JsFormatEmbeddedDocCb,
    format_file_cb: JsFormatFileCb,
    sort_tailwind_classes_cb: JsSortTailwindClassesCb,
) -> Option<Value> {
    let num_of_threads = 1;

    // Options and paths in `_oxfmtPluginOptionsJson` are already resolved to absolute paths
    // by `finalize_external_options()` when processing the parent file (e.g., App.vue).
    // No further relative path resolution is needed here.
    let (options, parent_filepath) = parse_options_and_filepath(oxfmt_plugin_options_json);

    let external_formatter = ExternalFormatter::new(
        init_external_formatter_cb,
        format_embedded_cb,
        format_embedded_doc_cb,
        format_file_cb,
        sort_tailwind_classes_cb,
    );

    // Use `block_in_place()` to avoid nested async runtime access
    match tokio::task::block_in_place(|| external_formatter.init(num_of_threads)) {
        // TODO: Plugins support
        Ok(_) => {}
        Err(err) => {
            debug!("`external_formatter.init()` failed: {err}");
            external_formatter.cleanup();
            return None;
        }
    }

    let strategy = FormatFileStrategy::OxcFormatter {
        path: format!("embedded.{source_ext}").into(),
        source_type: SourceType::from_extension(source_ext)
            .expect("source_ext should be a valid JS/TS extension"),
    };
    let mut resolved_options = resolve_options_from_value(options, &strategy, None)
        .expect("`_oxfmtPluginOptionsJson` should contain valid config");
    // Override filepath so external callbacks (e.g., Tailwind sorter) receive the parent
    // file path (e.g., `App.vue`) instead of the dummy `embedded.ts` path.
    resolved_options.set_filepath_override(parent_filepath);

    let formatter = SourceFormatter::new(num_of_threads)
        .with_external_formatter(Some(external_formatter.clone()));

    let code = match tokio::task::block_in_place(|| {
        formatter.format(&strategy, source_text, resolved_options)
    }) {
        FormatResult::Success { code, .. } => code,
        FormatResult::Error(diagnostics) => {
            debug!("`formatter.format()` failed: {diagnostics:?}");
            external_formatter.cleanup();
            return None;
        }
    };

    external_formatter.cleanup();
    Some(to_prettier_doc::printed_string_to_hardline_doc(&code))
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

    // NOTE: Options and paths are already resolved (see `run_full()` comment).
    // And `run_fragment()` does not support external formatting, so no need to use `parent_filepath`.
    let (options, _parent_filepath) = parse_options_and_filepath(oxfmt_plugin_options_json);

    let strategy = FormatFileStrategy::OxcFormatter {
        path: format!("embedded.{source_ext}").into(),
        source_type,
    };

    let resolved_options = resolve_options_from_value(options, &strategy, None)
        .expect("`_oxfmtPluginOptionsJson` should contain valid config");
    let ResolvedOptions::OxcFormatter { format_options, .. } = resolved_options else {
        unreachable!("OxcFormatter strategy should always resolve to OxcFormatter options");
    };

    let allocator = Allocator::default();
    let ParserReturn { program, errors, .. } =
        Parser::new(&allocator, source_text, source_type).with_options(get_parse_options()).parse();
    if !errors.is_empty() {
        debug!("`Parser::new().parse()` failed: {errors:?}");
        return None;
    }

    let formatter = Formatter::new(
        &allocator,
        FormatOptions {
            // TODO: Fragments inside of Vue attributes should always use single quotes,
            // since double quotes are used for the attribute value itself.
            //
            // Prettier also replaces double quotes with `&quot;` in this case,
            // but to reduce the diff, we set the quote style to single, regardless of the user config.
            //
            // However, this option is just a preference, so `singleQuote: true` is not enough.
            // But it works for most cases, so leave it for now...
            quote_style: oxc_formatter::QuoteStyle::Single,
            ..*format_options
        },
    );

    let formatted = match kind {
        FragmentKind::VueForBindingLeft | FragmentKind::VueBindings => {
            let params = {
                let Some(Statement::FunctionDeclaration(func)) = program.body.first() else {
                    unreachable!("Prettier wraps v-for/v-slot as `function _(...) {{}}`");
                };
                &*func.params
            };
            let node = AstNode::new(params, AstNodes::Dummy(), &allocator);
            let content = FormatVueBindingParams::new(
                &node,
                matches!(kind, FragmentKind::VueForBindingLeft)
                    && (1 < params.items.len() || params.rest.is_some()),
            );

            formatter.format_node(
                &content,
                program.source_text,
                source_type,
                &program.comments,
                None,
            )
        }
        FragmentKind::VueScriptGeneric => {
            let type_params = {
                let Some(Statement::TSTypeAliasDeclaration(decl)) = program.body.first() else {
                    unreachable!("Prettier wraps script-generic as `type T<...> = any`");
                };
                let Some(type_params) = decl.type_parameters.as_deref() else {
                    unreachable!("Prettier wraps script-generic as `type T<...> = any`");
                };
                type_params
            };
            let node = AstNode::new(type_params, AstNodes::Dummy(), &allocator);
            let content = FormatVueScriptGeneric::new(&node);

            formatter.format_node(
                &content,
                program.source_text,
                source_type,
                &program.comments,
                None,
            )
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

/// Parses the plugin options JSON and extracts parent file path for external callbacks.
///
/// The `filepath` field (set by `core::oxfmtrc::finalize_external_options`) stores
/// the parent file path (e.g. `App.vue`) for js-in-xxx formatting.
fn parse_options_and_filepath(oxfmt_plugin_options_json: &str) -> (Value, PathBuf) {
    let Ok(Value::Object(mut obj)) = serde_json::from_str(oxfmt_plugin_options_json) else {
        unreachable!("`_oxfmtPluginOptionsJson` should be a valid JSON object");
    };
    let Some(Value::String(s)) = obj.remove("filepath") else {
        unreachable!("Expected `filepath` in `_oxfmtPluginOptionsJson`");
    };
    (Value::Object(obj), PathBuf::from(s))
}
