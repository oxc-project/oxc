//! IR-in/IR-out embedded channel.
//!
//! Builds the `FormatDispatcher` carried by `ExternalCallbacks`:
//! each language is mapped to a Rust formatter where available,
//! otherwise to the Prettier Doc→IR fallback.
//! Results integrate into the parent's arena / `GroupId` space via [`EmbeddedContext`].

use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, debug_span};

use oxc_allocator::{Allocator, ArenaVec};
use oxc_formatter::HtmlEmbedMeta;
use oxc_formatter_core::{
    DispatchResult, EmbeddedContext, EmbeddedIr, FormatDispatcher, FormatElement, IndentWidth,
    LineMode, UniqueGroupIdBuilder,
};
use oxc_formatter_css::CssFormatOptions;
use oxc_formatter_graphql::GraphqlFormatOptions;

use crate::{
    core::{
        embed::{FormatEmbeddedDocWithConfigCallback, language_to_prettier_parser},
        options::inject_parser,
    },
    prettier_compat::from_prettier_doc,
};

/// Build the `FormatDispatcher` installed on `ExternalCallbacks`:
/// Rust formatters for graphql/css/scss/less (no Prettier fallback),
/// Prettier Doc→IR fallback for everything else.
pub fn build_dispatcher(
    format_embedded_doc: FormatEmbeddedDocWithConfigCallback,
    options: Value,
    graphql_options: GraphqlFormatOptions,
    css_options: CssFormatOptions,
) -> FormatDispatcher {
    let prettier_fallback = build_prettier_fallback(format_embedded_doc, options);
    Arc::new(
        move |ctx: &EmbeddedContext<'_, '_>, language: &str, texts: &[&str], _parent_context| {
            // Rust implementations replace branches one by one;
            match language {
                // This is always `graphql` for now
                "graphql" | "gql" => format_graphql_to_irs(ctx, texts, graphql_options)
                    .inspect_err(|err| {
                        debug!(
                            "`format_graphql_to_irs` failed, gql-in-xxx part stays as-is: {err}"
                        );
                    }),
                // This is always `css` with `CssVariant:Scss` for now
                "css" | "scss" | "less" => {
                    format_css_to_irs(ctx, texts, css_options).inspect_err(|err| {
                        debug!("`format_css_to_irs` failed, css-in-xxx part stays as-is: {err}");
                    })
                }
                // Everything else: Prettier fallback (Doc→IR path)
                _ => prettier_fallback(ctx, language, texts),
            }
        },
    )
}

/// Format each text as a standalone GraphQL document via `oxc_formatter_graphql`,
/// returning one IR per text (the IR-channel contract for GraphQL).
///
/// Any parse error fails the whole batch (an embedded template is all-or-nothing):
/// `Err` makes the parent print the template as-is.
fn format_graphql_to_irs<'a>(
    ctx: &EmbeddedContext<'a, '_>,
    texts: &[&str],
    options: GraphqlFormatOptions,
) -> Result<DispatchResult<'a>, String> {
    let docs = texts
        .iter()
        .map(|text| {
            debug_span!("oxfmt::external::format_graphql_to_ir").in_scope(|| {
                let embedded = oxc_formatter_graphql::format_to_ir(ctx, text, options)
                    .map_err(|err| err.to_string())?;
                Ok(embedded.ir)
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(DispatchResult { docs, tailwind_classes: Vec::new(), meta: None })
}

/// Format the single joined CSS text (placeholders included) via `oxc_formatter_css`,
/// returning one IR per call (the IR-channel contract for CSS).
fn format_css_to_irs<'a>(
    ctx: &EmbeddedContext<'a, '_>,
    texts: &[&str],
    options: CssFormatOptions,
) -> Result<DispatchResult<'a>, String> {
    // Unlike GraphQL's one-IR-per-quasi,
    // the CSS embed joins quasis with placeholders into a single text before dispatching.
    let [text] = texts else {
        return Err(format!("CSS dispatch expects exactly one text, got {}", texts.len()));
    };
    debug_span!("oxfmt::external::format_css_to_ir").in_scope(|| {
        let EmbeddedIr { ir, tailwind_classes } =
            oxc_formatter_css::format_to_ir(ctx, text, options).map_err(|err| err.to_string())?;
        Ok(DispatchResult { docs: vec![ir], tailwind_classes, meta: None })
    })
}

/// Type of the Prettier Doc→IR fallback used inside [`build_dispatcher`].
/// Same shape as `FormatDispatcher` minus the `parent_context` passthrough.
type PrettierDocFallback = Arc<
    dyn for<'a, 'g> Fn(
            &EmbeddedContext<'a, 'g>,
            &str,
            &[&str],
        ) -> Result<DispatchResult<'a>, String>
        + Send
        + Sync,
>;

/// Build the Prettier Doc→IR fallback for embedded languages:
/// sends texts to JS `printToDoc()`, then converts the returned Doc JSON into formatter IR.
fn build_prettier_fallback(
    format_embedded_doc: FormatEmbeddedDocWithConfigCallback,
    options: Value,
) -> PrettierDocFallback {
    Arc::new(move |ctx: &EmbeddedContext<'_, '_>, language: &str, texts: &[&str]| {
        let Some(parser_name) = language_to_prettier_parser(language) else {
            return Err(format!("Unsupported language: {language}"));
        };
        debug_span!("oxfmt::external::format_embedded_doc", parser = parser_name)
            .in_scope(|| {
                let mut options = options.clone();
                inject_parser(&mut options, parser_name);
                let doc_json_strs = (format_embedded_doc)(options, texts).map_err(|err| {
                    format!("Failed to get Doc for embedded code (parser '{parser_name}'): {err}")
                })?;
                let doc_jsons = doc_json_strs
                    .into_iter()
                    .map(|s| {
                        // Prettier's Doc can produce deeply nested arrays
                        // (e.g., md-in-js with `proseWrap: preserve`,
                        // which nests each word in `[[[prev, " "], word], " "]`).
                        // The default recursion limit of 128 is not enough for long paragraphs.
                        // This only affects this deserialization call;
                        // other `serde_json` usage in the codebase keeps the default limit.
                        let mut de = serde_json::Deserializer::from_str(&s);
                        de.disable_recursion_limit();
                        serde_json::Value::deserialize(&mut de)
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("Failed to parse Doc JSON: {e}"))?;

                to_format_elements_for_template(
                    language,
                    doc_jsons,
                    ctx.allocator,
                    ctx.group_id_builder,
                )
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
    doc_jsons: Vec<Value>,
    allocator: &'a Allocator,
    group_id_builder: &UniqueGroupIdBuilder,
) -> Result<DispatchResult<'a>, String> {
    match language {
        "html" | "angular" => {
            let (mut ir, metadata) = from_prettier_doc::convert_envelope(
                doc_jsons.into_iter().next().expect("Doc JSON for HTML"),
                allocator,
                group_id_builder,
            )?;
            let html_has_multiple_root_elements =
                metadata.get("htmlHasMultipleRootElements").and_then(Value::as_bool);
            postprocess(&mut ir, allocator);
            Ok(DispatchResult {
                docs: vec![ir],
                tailwind_classes: Vec::new(),
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
            postprocess(&mut ir, allocator);
            Ok(DispatchResult { docs: vec![ir], tailwind_classes: Vec::new(), meta: None })
        }
        // NOTE: no "css" / "graphql" arms
        // Those languages never reach the Prettier Doc path (their dispatcher branches are Rust-only).
        _ => unreachable!("Unsupported embedded_doc language: {language}"),
    }
}

/// Post-process FormatElements in a single compaction pass:
/// - strip trailing hardline (useless for embedded parts)
/// - collapse double-hardlines `[Hard, ExpandParent, Hard, ExpandParent]` → `[Empty, ExpandParent]`
/// - merge consecutive Text nodes (the Prettier Doc path can emit adjacent `Text`s)
fn postprocess<'a>(ir: &mut ArenaVec<'a, FormatElement<'a>>, allocator: &'a Allocator) {
    // Strip trailing hardline
    if ir.len() >= 2
        && matches!(ir[ir.len() - 1], FormatElement::ExpandParent)
        && matches!(ir[ir.len() - 2], FormatElement::Line(LineMode::Hard))
    {
        let new_len = ir.len() - 2;
        ir.truncate(new_len);
    }

    let mut write = 0;
    let mut read = 0;
    while read < ir.len() {
        // Collapse double-hardline → empty line
        if read + 3 < ir.len()
            && matches!(ir[read], FormatElement::Line(LineMode::Hard))
            && matches!(ir[read + 1], FormatElement::ExpandParent)
            && matches!(ir[read + 2], FormatElement::Line(LineMode::Hard))
            && matches!(ir[read + 3], FormatElement::ExpandParent)
        {
            ir[write] = FormatElement::Line(LineMode::Empty);
            ir[write + 1] = FormatElement::ExpandParent;
            write += 2;
            read += 4;
        } else if matches!(ir[read], FormatElement::ArenaText(_)) {
            // Merge consecutive Text nodes
            let run_start = read;
            read += 1;
            while read < ir.len() && matches!(ir[read], FormatElement::ArenaText(_)) {
                read += 1;
            }

            if read - run_start == 1 {
                if write != run_start {
                    ir[write] = ir[run_start].clone();
                }
            } else {
                // Heap staging: the merged text lands in the arena exactly-sized below.
                let mut sb = String::new();
                for element in &ir[run_start..read] {
                    if let FormatElement::ArenaText(text) = element {
                        sb.push_str(text.text());
                    }
                }
                ir[write] =
                    FormatElement::arena_text_measured(&sb, IndentWidth::default(), allocator);
            }
            write += 1;
        } else {
            if write != read {
                ir[write] = ir[read].clone();
            }
            write += 1;
            read += 1;
        }
    }
    ir.truncate(write);
}
