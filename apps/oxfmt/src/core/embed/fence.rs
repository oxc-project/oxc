//! Native-fence string adapter:
//! a JSDoc fenced code block whose language has a Rust formatter branch
//! formats through the dispatch registry, in EVERY build.
//!
//! This is the build-independent half of the string-out channel:
//! the napi [`super::string_channel`] routes native fences here before its Prettier string paths;
//! the pure Rust build assembles its whole `SessionServices` here
//! (`session_services`; non-native fences stay verbatim).

use std::sync::Arc;

use tracing::debug_span;

use oxc_allocator::Allocator;
use oxc_formatter_core::{
    DispatchOutcome, DispatchRequest, DispatchResult, Document, FormatDispatcher, FormatSession,
    InputKind, SessionServices, TailwindSorter,
};

use super::dispatcher::ResolvedDispatchConfig;

/// The pure build's `SessionServices`: the fallback-less registry dispatcher plus
/// the native-fence string embedder, both behind the shared off-predicate
/// (the napi twin is `ExternalFormatter::session_services`).
/// A non-native fence dispatches to the registry's `PreserveOriginal`,
/// which the adapter answers as `Err` (the string channel's "keep verbatim");
/// no Prettier exists in this build, and no Tailwind sorter either.
#[cfg(not(feature = "napi"))]
pub fn session_services(dispatch_config: &Arc<ResolvedDispatchConfig>) -> SessionServices {
    // A fence dispatcher and this root's dispatcher are the same fallback-less registry,
    // so the root's doubles as the fence adapter's.
    let dispatcher = dispatch_config.root_dispatcher(None);
    let string_embedder = dispatcher.as_ref().map(|dispatcher| {
        let fence_dispatcher = Arc::clone(dispatcher);
        let dispatch_config = Arc::clone(dispatch_config);
        Arc::new(move |language: &str, code: &str| {
            format_native_fence(language, code, &fence_dispatcher, &dispatch_config, None)
        }) as oxc_formatter_core::StringEmbedder
    });
    SessionServices { dispatcher, string_embedder, tailwind_sorter: None }
}

/// Format a JSDoc fenced code block through the native dispatch registry:
/// a string-in/string-out adapter over the IR contract.
///
/// Load-bearing notes:
/// - The fence has no parent index space, so its Tailwind classes are sorted here
///   (element-wise: the sorter reorders classes WITHIN each collected string, never the vector,
///   keeping `TailwindClass(index)` references valid). The pure build passes no sorter.
/// - `Err` keeps the fence verbatim, covering both `PreserveOriginal`
///   (parse failure — never a Prettier fallback for native languages) and operational errors.
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
