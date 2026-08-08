//! String-in/string-out embedded channel.
//!
//! Two consumers reach this channel through the session's `StringEmbedder` contract:
//! - JSDoc fenced code blocks (` ```css `, ` ```yaml `, …)
//! - html-in-js fallback (`format_js_in_html_as_fallback`):
//!   the IR channel's HTML route returned Prettier Doc that the IR converter can't represent,
//!   so the parent re-requests the same HTML via this string channel and substitutes placeholders back to `${expr}`.
//!
//! Fence routing follows ONE rule: a language in the native registry ([`dispatcher::is_native_language`]) formats
//! through the dispatcher via [`super::fence::format_native_fence`] (the build-independent adapter);
//! md/html/angular stay on the Prettier string path
//! (their Doc→IR conversion has unrepresentable cases,
//! so forcing them through the dispatcher would regress to verbatim; the wall falls with the HTML Rust port).
//!
//! Unlike the [`super::dispatcher`], errors here keep the input verbatim
//! (no Prettier fallback for native languages, no diagnostics for JSDoc since it's inside a comment).

use std::sync::Arc;

use tracing::{debug, debug_span};

use oxc_formatter_core::{StringEmbedder, TailwindSorter};

use crate::core::{
    embed::{FormatEmbeddedWithConfigCallback, dispatcher, fence, language_to_prettier_parser},
    options::inject_parser,
};

/// Build the napi build's string embedder installed on the session.
///
/// Dispatches by language identifier: the native registry when available,
/// otherwise Prettier via `format_embedded`.
/// The JSDoc fenced consumer reaches every language;
/// the html-in-js fallback only ever passes `"html"` and therefore always lands on the Prettier branch.
///
/// `sort_tailwind` is the SAME pre-bound sorter as the session's Tailwind service
/// (options JSON already applied by `session_services`),
/// for the `@apply` classes a CSS fence collects.
pub fn build_string_embedder(
    format_embedded: FormatEmbeddedWithConfigCallback,
    sort_tailwind: Option<TailwindSorter>,
    dispatch_config: Arc<dispatcher::ResolvedDispatchConfig>,
) -> StringEmbedder {
    let fence_dispatcher = fence::build_fence_dispatcher(&dispatch_config);
    Arc::new(move |language: &str, code: &str| {
        // Native registry first (JSDoc fenced code blocks).
        if dispatcher::is_native_language(language) {
            return fence::format_native_fence(
                language,
                code,
                &fence_dispatcher,
                &dispatch_config,
                sort_tailwind.as_ref(),
            );
        }
        let Some(parser_name) = language_to_prettier_parser(language) else {
            // NOTE: Do not return `Ok(original)` here.
            // We need to keep unsupported content as-is.
            return Err(format!("Unsupported language: {language}"));
        };
        debug_span!("oxfmt::external::format_embedded", parser = parser_name).in_scope(|| {
            // `clone()` is unavoidable here,
            // because there may be multiple embedded sections in one JS/TS file.
            let mut options = dispatch_config.external_options().clone();
            inject_parser(&mut options, parser_name);
            (format_embedded)(options, code)
                .map(|mut code| {
                    // Remove trailing newline added by Prettier without allocation.
                    // For embedded code, we never want trailing newlines, regardless of options.
                    let trimmed_len = code.trim_end().len();
                    code.truncate(trimmed_len);
                    code
                })
                .inspect_err(|err| {
                    debug!("Failed to format embedded code for parser '{parser_name}': {err}");
                })
        })
    })
}
