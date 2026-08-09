//! String-in/string-out embedded channel.
//!
//! Two consumers reach this channel through the same `EmbeddedFormatterCallback` contract on `ExternalCallbacks`:
//! - JSDoc fenced code blocks (` ```css `, ` ```yaml `, …)
//! - html-in-js fallback (`format_js_in_html_as_fallback`):
//!   the IR channel's HTML route returned Prettier Doc that the IR converter can't represent,
//!   so the parent re-requests the same HTML via this string channel and substitutes placeholders back to `${expr}`.
//!
//! Fence routing follows ONE rule: a language in the native registry ([`dispatcher::is_native_language`]) formats
//! through the dispatcher via [`format_native_fence`];
//! md/html/angular stay on the Prettier string path
//! (their Doc→IR conversion has unrepresentable cases,
//! so forcing them through the dispatcher would regress to verbatim; the wall falls with the HTML Rust port).
//!
//! Unlike the [`super::dispatcher`], errors here keep the input verbatim
//! (no Prettier fallback for native languages, no diagnostics for JSDoc since it's inside a comment).

use std::sync::Arc;

use tracing::{debug, debug_span};

use oxc_allocator::Allocator;
use oxc_formatter::EmbeddedFormatterCallback;
use oxc_formatter_core::{
    DispatchOutcome, DispatchRequest, DispatchResult, Document, FormatDispatcher, FormatSession,
    InputKind,
};

use crate::core::{
    embed::{
        FormatEmbeddedWithConfigCallback, TailwindWithConfigCallback, dispatcher,
        language_to_prettier_parser,
    },
    options::inject_parser,
};

/// Build the `embedded_formatter` callback installed on `ExternalCallbacks`.
///
/// Dispatches by language identifier: the native registry when available,
/// otherwise Prettier via `format_embedded`.
/// The JSDoc fenced consumer reaches every language;
/// the html-in-js fallback only ever passes `"html"` and therefore always lands on the Prettier branch.
///
/// `dispatch_config.external_options()` already carries the Tailwind plugin payload,
/// so the JS-side sorter can resolve class order when a CSS fence collects `@apply` classes.
pub fn build_embedded_callback(
    format_embedded: FormatEmbeddedWithConfigCallback,
    sort_tailwind: Option<TailwindWithConfigCallback>,
    dispatch_config: Arc<dispatcher::ResolvedDispatchConfig>,
) -> EmbeddedFormatterCallback {
    let fence_dispatcher = dispatcher::build_dispatcher(Arc::clone(&dispatch_config), None);
    Arc::new(move |language: &str, code: &str| {
        // Native registry first (JSDoc fenced code blocks).
        if dispatcher::is_native_language(language) {
            // Native fences never fall back to Prettier,
            // so the dispatcher is invariant across the callback's lifetime: build it once, not per fence.
            return format_native_fence(
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

/// Format a JSDoc fenced code block through the native dispatch registry:
/// a string-in/string-out adapter over the IR contract.
///
/// Load-bearing notes:
/// - The fence has no parent index space, so its Tailwind classes are sorted here
///   (element-wise: the sorter reorders classes WITHIN each collected string, never the vector,
///   keeping `TailwindClass(index)` references valid).
/// - `Err` keeps the fence verbatim, covering both `PreserveOriginal`
///   (parse failure — never a Prettier fallback for native languages) and operational errors.
/// - The session-less `EmbeddedFormatterCallback` contract forces a fresh root session per fence,
///   so `dispatch_depth` resets at this string boundary (inert today: no native fence language re-dispatches).
///   Threading the parent session through the callback is the eventual fix.
fn format_native_fence(
    language: &str,
    code: &str,
    fence_dispatcher: &FormatDispatcher,
    dispatch_config: &Arc<dispatcher::ResolvedDispatchConfig>,
    sort_tailwind: Option<&TailwindWithConfigCallback>,
) -> Result<String, String> {
    debug_span!("oxfmt::external::format_native_fence", language = language).in_scope(|| {
        let allocator = Allocator::default();
        let session =
            FormatSession::new(&allocator, InputKind::Fragment, Some(Arc::clone(fence_dispatcher)));
        let outcome = session.dispatch(DispatchRequest {
            language,
            texts: &[code],
            input_kind: InputKind::Fragment,
            parent_context: None,
        })?;

        let DispatchOutcome::Formatted(result) = outcome else {
            return Err(format!("Native formatter for '{language}' kept the input as-is"));
        };
        let DispatchResult { mut docs, tailwind_classes, .. } = result;
        if docs.len() != 1 {
            return Err(format!("Expected exactly one IR, got {}", docs.len()));
        }
        let ir = docs.pop().unwrap();

        let tailwind_classes = match sort_tailwind {
            Some(sort) if !tailwind_classes.is_empty() => {
                sort(dispatch_config.external_options(), tailwind_classes)
            }
            _ => tailwind_classes,
        };

        let mut code = Document::new(ir, tailwind_classes)
            .print(code.len(), dispatch_config.print_options())
            .map_err(|err| err.to_string())?
            .into_code();
        // The block is re-embedded line-by-line into the comment; no trailing newline
        code.truncate(code.trim_end().len());

        Ok(code)
    })
}
