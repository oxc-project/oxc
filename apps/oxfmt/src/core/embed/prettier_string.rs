//! The napi string embedder: fence routing plus the Prettier string paths of the string-out channel.
//!
//! Two consumers reach this channel through the session's `StringEmbedder` contract:
//! - JSDoc fenced code blocks (` ```css `, ` ```yaml `, …)
//! - html-in-js string recovery (`format_js_in_html_as_fallback`):
//!   the IR channel's HTML route returned Prettier Doc that the IR converter can't represent,
//!   so the parent re-requests the same HTML via this string channel and substitutes placeholders back to `${expr}`.
//!
//! Fence routing follows ONE rule, the shared routing table ([`dispatcher::route`]):
//! a `Route::Native` language formats through the dispatcher via [`super::fence::format_native_fence`] (the build-independent adapter);
//! the `Route::Prettier` set (md/html/angular) stays on the Prettier string path
//! (their Doc→IR conversion has unrepresentable cases,
//! so forcing them through the dispatcher would regress to verbatim; the wall falls with the HTML Rust port).
//!
//! Unlike the [`super::dispatcher`] contract (`PreserveOriginal` vs `Err`),
//! this channel has one failure meaning: `Err` keeps the input verbatim.
//! A failed native fence is never re-routed to Prettier, and a JSDoc fence surfaces no diagnostics (it lives inside a comment).

use std::sync::Arc;

use tracing::{debug, debug_span};

use oxc_formatter_core::{StringEmbedder, TailwindSorter};

use crate::core::{
    embed::{
        FormatEmbeddedWithConfigCallback,
        dispatcher::{self, Route},
        fence,
    },
    options::inject_parser,
};

/// Build the napi build's string embedder installed on the session.
///
/// Dispatches by language identifier: the native registry when available,
/// otherwise Prettier via `format_embedded`.
/// The JSDoc fenced consumer reaches every language;
/// the html-in-js string recovery only ever passes `"html"` and therefore always lands on the Prettier branch.
///
/// `sort_tailwind` is the SAME pre-bound sorter as the session's Tailwind service
/// (options JSON already applied by `services::for_root`),
/// for the `@apply` classes a CSS fence collects.
pub fn build_string_embedder(
    format_embedded: FormatEmbeddedWithConfigCallback,
    sort_tailwind: Option<TailwindSorter>,
    dispatch_config: Arc<dispatcher::ResolvedDispatchConfig>,
) -> StringEmbedder {
    // Fence dispatchers never take the Prettier fallback (native fences never fall back),
    // so one is invariant across the callback's lifetime: build it once, not per fence.
    let fence_dispatcher = dispatcher::build_dispatcher(Arc::clone(&dispatch_config), None);
    Arc::new(move |language: &str, code: &str| {
        let parser_name = match dispatcher::route(language) {
            // Native branch (JSDoc fenced code blocks): through the dispatcher, never Prettier.
            Route::Native(_) => {
                return fence::format_native_fence(
                    language,
                    code,
                    &fence_dispatcher,
                    &dispatch_config,
                    sort_tailwind.as_ref(),
                );
            }
            Route::Prettier(prettier_language) => prettier_language.parser(),
            // NOTE: Do not return `Ok(original)` here.
            // We need to keep unsupported content as-is.
            Route::Unsupported => return Err(format!("Unsupported language: {language}")),
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
