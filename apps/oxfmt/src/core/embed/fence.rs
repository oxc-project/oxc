//! Native-fence string adapter: a JSDoc fenced code block whose language has a
//! Rust formatter branch formats through the dispatch registry, in EVERY build.
//!
//! This is the build-independent half of the string-out channel:
//! the napi [`super::string_channel`] routes native fences here before its
//! Prettier string paths; the pure Rust build installs `build_external_callbacks`
//! directly (non-native fences stay verbatim).

use std::sync::Arc;

use tracing::debug_span;

use oxc_allocator::Allocator;
#[cfg(not(feature = "napi"))]
use oxc_formatter::ExternalCallbacks;
use oxc_formatter::TailwindCallback;
use oxc_formatter_core::{
    DispatchOutcome, DispatchRequest, DispatchResult, Document, FormatDispatcher, FormatSession,
    InputKind,
};

use super::dispatcher::{self, ResolvedDispatchConfig};

/// Build the dispatcher a fence callback holds for its lifetime.
/// Fence dispatchers never take the Prettier fallback (native fences never fall
/// back), so one is invariant across the callback's lifetime: build it once, not per fence.
pub fn build_fence_dispatcher(dispatch_config: &Arc<ResolvedDispatchConfig>) -> FormatDispatcher {
    dispatcher::build_dispatcher(Arc::clone(dispatch_config), None)
}

/// The pure build's `ExternalCallbacks`: just the native-fence adapter, gated by
/// the shared off-predicate (the napi twin is `ExternalFormatter::to_external_callbacks`).
/// Non-native fences answer `Err` — the string channel's "keep verbatim" — since
/// no Prettier exists in this build.
#[cfg(not(feature = "napi"))]
pub fn build_external_callbacks(
    dispatch_config: &Arc<ResolvedDispatchConfig>,
) -> ExternalCallbacks {
    let embedded_callback = dispatch_config.is_embedded_formatting_enabled().then(|| {
        let fence_dispatcher = build_fence_dispatcher(dispatch_config);
        let dispatch_config = Arc::clone(dispatch_config);
        Arc::new(move |language: &str, code: &str| {
            if !dispatcher::is_native_language(language) {
                return Err(format!("Unsupported language: {language}"));
            }
            format_native_fence(language, code, &fence_dispatcher, &dispatch_config, None)
        }) as oxc_formatter::EmbeddedFormatterCallback
    });
    ExternalCallbacks::new().with_embedded_formatter(embedded_callback)
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
/// - The session-less `EmbeddedFormatterCallback` contract forces a fresh root session per fence,
///   so `dispatch_depth` resets at this string boundary (inert today: no native fence language re-dispatches).
///   Threading the parent session through the callback is the eventual fix.
pub fn format_native_fence(
    language: &str,
    code: &str,
    fence_dispatcher: &FormatDispatcher,
    dispatch_config: &ResolvedDispatchConfig,
    sort_tailwind: Option<&TailwindCallback>,
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
            Some(sort) if !tailwind_classes.is_empty() => sort(tailwind_classes),
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
