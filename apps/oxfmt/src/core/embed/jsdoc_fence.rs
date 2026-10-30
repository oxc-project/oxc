//! Native-fence string adapter:
//! a JSDoc fenced code block whose language has a Rust formatter branch
//! formats through the dispatch registry, in EVERY build; any other fence stays verbatim.
//!
//! This is the whole string-out channel; [`super::services`] installs it as the session's string embedder.

use std::sync::Arc;

use tracing::debug_span;

use oxc_allocator::Allocator;
use oxc_formatter_core::{
    DispatchRequest, FormatDispatcher, FormatSession, InputKind, PrintWidth, SessionServices,
    StringEmbedder, TailwindSorter,
};

use super::dispatcher::ResolvedDispatchConfig;

/// The session's string embedder: [`format_native_fence`] over `fence_dispatcher`,
/// a fallback-less registry (a native fence never falls back to Prettier).
/// `sort_tailwind` is the root's pre-bound sorter, for the `@apply` classes a CSS fence collects.
pub fn build_string_embedder(
    fence_dispatcher: FormatDispatcher,
    dispatch_config: Arc<ResolvedDispatchConfig>,
    sort_tailwind: Option<TailwindSorter>,
) -> StringEmbedder {
    Arc::new(move |language: &str, code: &str, print_width: usize| {
        format_native_fence(
            language,
            code,
            print_width,
            &fence_dispatcher,
            &dispatch_config,
            sort_tailwind.as_ref(),
        )
    })
}

/// Format a JSDoc fenced code block through the native dispatch registry:
/// a string-in/string-out adapter over `FormatSession::dispatch_to_string`.
///
/// Load-bearing notes:
/// - `print_width` is the fence's effective width at its comment position;
///   it overrides the configured width, the other print knobs come from the resolved config
/// - `Err` keeps the fence verbatim, covering both the deliberate keep
///   (`Ok(None)`; a failed native language never re-routes to Prettier) and operational errors
/// - The session-less `StringEmbedder` contract forces a fresh root session per fence,
///   so `dispatch_depth` resets at this string boundary (inert today: the fence prints to a string, its placeholders never meet the parent's).
///   Threading the parent session through the callback is the eventual fix.
fn format_native_fence(
    language: &str,
    code: &str,
    print_width: usize,
    fence_dispatcher: &FormatDispatcher,
    dispatch_config: &ResolvedDispatchConfig,
    sort_tailwind: Option<&TailwindSorter>,
) -> Result<String, String> {
    debug_span!("oxfmt::embed::format_native_fence", language = language).in_scope(|| {
        let allocator = Allocator::default();
        let session = FormatSession::with_services(
            &allocator,
            InputKind::Fragment,
            SessionServices {
                dispatcher: Some(Arc::clone(fence_dispatcher)),
                tailwind_sorter: sort_tailwind.cloned(),
                ..SessionServices::default()
            },
        );

        let printer_options = dispatch_config
            .print_options()
            .with_print_width(PrintWidth::new(u32::try_from(print_width).unwrap_or(u32::MAX)));
        session
            .dispatch_to_string(
                DispatchRequest {
                    language,
                    text: code,
                    input_kind: InputKind::Fragment,
                    parent_context: None,
                },
                printer_options,
            )?
            .map(|mut code| {
                // The block is re-embedded line-by-line into the comment; no trailing newline
                code.truncate(code.trim_end().len());
                code
            })
            .ok_or_else(|| format!("Native formatter for '{language}' kept the input as-is"))
    })
}
