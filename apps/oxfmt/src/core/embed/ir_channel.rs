//! IR-in/IR-out embedded channel.
//!
//! Builds the `FormatDispatcher` carried by `ExternalCallbacks`: each language
//! is mapped to a Rust formatter where available (graphql/css), otherwise to
//! the Prettier Doc→IR fallback. Results integrate into the parent's arena /
//! `GroupId` space via [`EmbeddedContext`].
//!
//! Unlike [`super::string_channel`], errors here make the parent print the
//! template literal as-is — the same behavior as Prettier's embed when it
//! throws. No Prettier fallback for the Rust paths (graphql/css): the forks
//! cover what Prettier accepts, so what still errors is genuinely broken
//! input that Prettier's embed can't format either.

use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, debug_span};

use oxc_formatter_core::{
    DispatchResult, EmbeddedContext, EmbeddedIr, FormatDispatcher, FormatElement,
};
use oxc_formatter_css::CssFormatOptions;
use oxc_formatter_graphql::GraphqlFormatOptions;

use crate::core::{
    embed::{
        FormatEmbeddedDocWithConfigCallback, language_to_prettier_parser,
        postprocess::escape_template_characters_in_ir, routing,
    },
    options::inject_parser,
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
                "graphql" | "gql" => format_graphql_to_irs(ctx, texts, graphql_options)
                    .inspect_err(|err| {
                        debug!(
                            "`format_graphql_to_irs` failed, gql-in-xxx part stays as-is: {err}"
                        );
                    }),
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
/// Template-literal characters in the output are re-escaped because the parent
/// re-inserts the IR into a JS template literal built from `.cooked` values.
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
                let mut embedded = oxc_formatter_graphql::format_to_ir(ctx, text, options)
                    .map_err(|err| err.to_string())?;
                escape_template_characters_in_ir(
                    &mut embedded.ir,
                    ctx.allocator,
                    options.indent_width,
                );
                Ok(embedded.ir)
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(DispatchResult { docs, tailwind_classes: Vec::new(), placeholder_count: None, meta: None })
}

/// Format the single joined CSS text (placeholders included) via
/// `oxc_formatter_css`, returning one IR plus the surviving placeholder count
/// (the IR-channel contract for CSS, mirroring the Prettier Doc path).
///
/// css-in-js uses `.raw` template values, so unlike GraphQL,
/// no template-char re-escaping is needed.
/// `oxc_formatter_css` emits each placeholder as a typed [`FormatElement::EmbedPlaceholder`],
/// so the surviving count is just a structural tally (no string scan);
/// the parent splices `${expr}` per marker.
fn format_css_to_irs<'a>(
    ctx: &EmbeddedContext<'a, '_>,
    texts: &[&str],
    options: CssFormatOptions,
) -> Result<DispatchResult<'a>, String> {
    // Unlike GraphQL's one-IR-per-quasi, the CSS embed joins quasis with
    // placeholders into a single text before dispatching.
    let [text] = texts else {
        return Err(format!("CSS dispatch expects exactly one text, got {}", texts.len()));
    };
    debug_span!("oxfmt::external::format_css_to_ir").in_scope(|| {
        let EmbeddedIr { ir, tailwind_classes } =
            oxc_formatter_css::format_to_ir(ctx, text, options).map_err(|err| err.to_string())?;
        // Surviving placeholders:
        // - typed `EmbedPlaceholder` markers (the main path)
        // - and sentinels that stayed embedded in verbatim `Text`
        //   - because a lexical context (string/`url()`) doesn't tokenize them
        // Counting both keeps the host's count == expressions check passing
        // so it doesn't fall back to plain template formatting (the host substitutes both kinds).
        let placeholder_count = ir
            .iter()
            .map(|el| match el {
                FormatElement::EmbedPlaceholder(_) => 1,
                FormatElement::Text { text, .. } => count_text_sentinels(text),
                _ => 0,
            })
            .sum();
        Ok(DispatchResult {
            docs: vec![ir],
            tailwind_classes,
            placeholder_count: Some(placeholder_count),
            meta: None,
        })
    })
}

/// Counts `` `PLACEHOLDER-N` `` sentinels left inside a verbatim `Text`
/// run (placeholders that landed in a string / `url()` the CSS lexer keeps opaque).
/// Non-overlapping matches of `prefix <digits> suffix`.
///
/// NOTE: Same sentinel grammar is scanned in two sibling sites with different actions
/// (host `oxc_formatter` splits to substitute, `oxc_formatter_css` emits);
/// kept duplicated on purpose for now, but may revisit later.
fn count_text_sentinels(text: &str) -> usize {
    use oxc_formatter_css::{
        TEMPLATE_PLACEHOLDER_PREFIX as PREFIX, TEMPLATE_PLACEHOLDER_SUFFIX as SUFFIX,
    };
    let mut count = 0;
    let mut rest = text;
    while let Some(start) = rest.find(PREFIX) {
        let after = &rest[start + PREFIX.len()..];
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 && after[digits..].starts_with(SUFFIX) {
            count += 1;
            rest = &after[digits + SUFFIX.len()..];
        } else {
            rest = after;
        }
    }
    count
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

                routing::to_format_elements_for_template(
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
