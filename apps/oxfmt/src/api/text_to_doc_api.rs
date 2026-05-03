use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, instrument};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_formatter::{
    AstNode, AstNodes, FormatOptions, FormatVueBindingParams, FormatVueScriptGeneric, Formatter,
    enable_jsx_source_type, get_parse_options,
};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;

use crate::{
    core::{
        ExternalFormatter, JsFormatEmbeddedCb, JsFormatEmbeddedDocCb, JsFormatFileCb,
        JsInitExternalFormatterCb, JsSortTailwindClassesCb,
        options::{inject_filepath, inject_tailwind_plugin_payload, to_prettier},
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
    init_external_formatter_cb: JsInitExternalFormatterCb,
    format_file_cb: JsFormatFileCb,
    format_embedded_cb: JsFormatEmbeddedCb,
    format_embedded_doc_cb: JsFormatEmbeddedDocCb,
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
    init_external_formatter_cb: JsInitExternalFormatterCb,
    format_file_cb: JsFormatFileCb,
    format_embedded_cb: JsFormatEmbeddedCb,
    format_embedded_doc_cb: JsFormatEmbeddedDocCb,
    sort_tailwind_classes_cb: JsSortTailwindClassesCb,
) -> Option<Value> {
    let num_of_threads = 1;

    // Tailwind paths in the payload are already absolute (resolved by the host before serialization),
    // so no `cwd` is threaded through here.
    let (config, parent_filepath) = parse_payload(oxfmt_plugin_options_json);

    let external_formatter = ExternalFormatter::new(
        init_external_formatter_cb,
        format_file_cb,
        format_embedded_cb,
        format_embedded_doc_cb,
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

    let source_type = enable_jsx_source_type(
        SourceType::from_extension(source_ext)
            .expect("source_ext should be a valid JS/TS extension"),
    );

    let resolved = resolve_for_embedded_js(config, parent_filepath)
        .expect("`_oxfmtPluginOptionsJson` should contain valid config");

    // Prettier options for callbacks that `oxc_formatter` may dispatch (e.g., CSS-in-JS).
    // The embedded JS context is treated as always Tailwind-capable, so the inject is unconditional.
    // The helper no-ops when user config has Tailwind disabled.
    let mut external_options = to_prettier(&resolved.config);
    inject_filepath(&mut external_options, &resolved.parent_filepath);
    inject_tailwind_plugin_payload(&mut external_options, &resolved.config);

    let external_callbacks =
        external_formatter.to_external_callbacks(&resolved.format_options, external_options);
    let format_options = resolved.format_options;

    let allocator = Allocator::default();
    let ret =
        Parser::new(&allocator, source_text, source_type).with_options(get_parse_options()).parse();
    if !ret.errors.is_empty() {
        debug!("`Parser::new().parse()` failed: {:?}", ret.errors);
        external_formatter.cleanup();
        return None;
    }

    let base_formatter = Formatter::new(&allocator, *format_options);
    let formatted = tokio::task::block_in_place(|| {
        base_formatter.format_with_external_callbacks(&ret.program, Some(external_callbacks))
    });

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
