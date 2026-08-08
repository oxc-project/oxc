//! Native-fence string adapter:
//! a JSDoc fenced code block whose language has a Rust formatter branch
//! formats through the dispatch registry, in EVERY build.
//!
//! This is the build-independent half of the string-out channel;
//! the profiles in [`super::services`] wire it into the session's string embedder
//! (the napi one via [`super::prettier_string`], which routes native fences here
//! before its Prettier string paths; the pure one directly, non-native fences stay verbatim).

use std::sync::Arc;

use tracing::debug_span;

use oxc_allocator::Allocator;
use oxc_formatter_core::{
    DispatchOutcome, DispatchRequest, DispatchResult, Document, FormatDispatcher, FormatSession,
    InputKind, SessionServices, TailwindSorter,
};

use super::dispatcher::ResolvedDispatchConfig;

/// Format a JSDoc fenced code block through the native dispatch registry:
/// a string-in/string-out adapter over the IR contract.
///
/// Load-bearing notes:
/// - The fence has no parent index space, so its Tailwind classes are sorted here
///   (element-wise: the sorter reorders classes WITHIN each collected string, never the vector,
///   keeping `TailwindClass(index)` references valid). The pure build passes no sorter.
/// - `Err` keeps the fence verbatim, covering both `PreserveOriginal`
///   (parse failure; a failed native language never re-routes to Prettier) and operational errors.
/// - The session-less `StringEmbedder` contract forces a fresh root session per fence,
///   so `dispatch_depth` resets at this string boundary (inert today: no native fence language re-dispatches).
///   Threading the parent session through the callback is the eventual fix.
pub fn format_native_fence(
    language: &str,
    code: &str,
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
        let outcome = session.dispatch(DispatchRequest {
            language,
            text: code,
            input_kind: InputKind::Fragment,
            parent_context: None,
        })?;

        let DispatchOutcome::Formatted(result) = outcome else {
            return Err(format!("Native formatter for '{language}' kept the input as-is"));
        };
        // The parent-less consumer: no index space to remap into,
        // so the classes sort locally instead of going through `into_doc`.
        let DispatchResult { doc: ir, tailwind_classes, .. } = result;

        let tailwind_classes = session.sort_tailwind_classes(tailwind_classes);

        let mut code = Document::new(ir, tailwind_classes)
            .print(code.len(), dispatch_config.print_options())
            .map_err(|err| err.to_string())?
            .into_code();
        // The block is re-embedded line-by-line into the comment; no trailing newline
        code.truncate(code.trim_end().len());

        Ok(code)
    })
}
