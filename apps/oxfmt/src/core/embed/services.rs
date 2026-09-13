//! Root `SessionServices` assembly: [`for_root`] builds the build's default service set,
//! installed by every session-taking root (JS / CSS / Vue-Svelte script; a future Markdown host too).
//! One name, one definition per build; the napi one additionally takes the `ExternalServices` transport.
//!
//! "Which languages may dispatch at all from this host" is the host crate's own gate
//! (e.g. the CSS crate's front matter gate dispatches only `yaml` / `toml`), not a concern here;
//! a root needing a bespoke set would assemble the `SessionServices` struct literally, none does today.

use std::sync::Arc;

use oxc_formatter_core::SessionServices;
#[cfg(not(feature = "napi"))]
use oxc_formatter_core::StringEmbedder;
#[cfg(feature = "napi")]
use oxc_formatter_core::TailwindSorter;
#[cfg(feature = "napi")]
use tracing::debug_span;

#[cfg(feature = "napi")]
use crate::core::ExternalServices;

use super::dispatcher::ResolvedDispatchConfig;

/// The pure build's default `SessionServices` for a root formatter run:
/// the fallback-less registry dispatcher plus the native-fence string embedder.
/// A non-native fence dispatches to the registry's `PreserveOriginal`,
/// which the adapter answers as `Err` (the string channel's "keep verbatim");
/// no Prettier exists in this build, and no Tailwind sorter either.
#[cfg(not(feature = "napi"))]
pub fn for_root(dispatch_config: &Arc<ResolvedDispatchConfig>) -> SessionServices {
    // A fence dispatcher and this root's dispatcher are the same fallback-less registry,
    // so the root's doubles as the fence adapter's.
    let dispatcher = dispatch_config.root_dispatcher(None);
    let string_embedder = dispatcher.as_ref().map(|dispatcher| {
        let fence_dispatcher = Arc::clone(dispatcher);
        let dispatch_config = Arc::clone(dispatch_config);
        Arc::new(move |language: &str, code: &str, print_width: usize| {
            super::jsdoc_fence::format_native_fence(
                language,
                code,
                print_width,
                &fence_dispatcher,
                &dispatch_config,
                None,
            )
        }) as StringEmbedder
    });
    SessionServices { dispatcher, string_embedder, tailwind_sorter: None }
}

/// The napi build's default `SessionServices` for a root formatter run:
/// the registry dispatcher with the Prettier Doc→IR fallback,
/// the string embedder, and the Tailwind sorter.
/// The options (including `filepath`) are captured in the closures and passed to JS on each call;
/// per-language options are mapped lazily at dispatch time (see [`ResolvedDispatchConfig`]).
///
/// This is the assembly point only: closure factories live in `embed::{prettier_string, dispatcher, prettier_doc}`,
/// and the napi callback `Arc`s come from the transport ([`ExternalServices`]).
///
/// NOTE: Tailwind data paths
/// `sort_tailwindcss_classes` is wired into THREE distinct callback slots,
/// because Tailwind class sorting has four legitimate paths depending on where the classes live and how they reach printing:
///
/// 1. Standalone JS/TS (`className` / functions / custom attributes):
///    `oxc_formatter` collects classes into `FormatElement::TailwindClass`.
///    When the entry document is finalized, the printer sorts them in one host batch via the session's `tailwind_sorter`.
/// 2. Standalone CSS / SCSS / LESS (`@apply` at top level):
///    the CSS root's session carries the same sorter (`CssFormatOptions::sort_tailwindcss` only switches collection);
///    classes sort once at finalize, no embedded boundary involved.
/// 3. Embedded CSS (css-in-js + Angular `@Component({ styles })`):
///    `oxc_formatter_css::format_to_ir()` returns pre-sort `@apply` classes in `DispatchPayload::tailwind_classes`.
///    The JS parent re-indexes them into its own collector (`DispatchPayload::into_doc`), so they ride the SAME parent batch as path 1.
///    The standalone CSS sort closure is NOT invoked here.
/// 4. JSDoc fenced CSS (Markdown code fence in JSDoc descriptions):
///    Goes through the string embedder's fence adapter, which returns a formatted string (not parent-integrated IR),
///    so there is no parent index space to remap into:
///    the adapter sorts the returned `DispatchPayload::tailwind_classes` itself before printing.
///    The `prettier_string::build_string_embedder` factory receives the sorter for this case.
///
/// All four paths use the SAME `sort_tailwindcss_classes` napi callback; only the wiring differs.
/// Moving sorting to the wrong layer (e.g. sorting inside embedded CSS instead of remapping) would double-sort or drop classes.
/// `DispatchPayload::into_doc` folds the merge into doc consumption; the printer `debug_assert` backstops it.
#[cfg(feature = "napi")]
pub fn for_root(
    external_services: &ExternalServices,
    dispatch_config: &Arc<ResolvedDispatchConfig>,
) -> SessionServices {
    let needs_embedded = dispatch_config.is_embedded_formatting_enabled();
    // Built only when the off-gate passes;
    // `root_dispatcher` re-checks the same predicate, so an off run allocates neither fallback nor dispatcher.
    let fallback = needs_embedded.then(|| {
        super::prettier_doc::build_prettier_fallback(
            Arc::clone(dispatch_config),
            Arc::clone(&external_services.format_embedded_doc),
        )
    });

    // The ONE pre-bound Tailwind sorter (options JSON applied here, once);
    // the string embedder receives a clone of the same sorter for fence-collected classes.
    let sorter = dispatch_config.is_tailwind_enabled().then(|| {
        let sort = Arc::clone(&external_services.sort_tailwindcss_classes);
        let dispatch_config = Arc::clone(dispatch_config);
        Arc::new(move |classes: Vec<String>| {
            debug_span!("oxfmt::external::sort_tailwind", classes_count = classes.len())
                .in_scope(|| (sort)(dispatch_config.prettier_options(), classes))
        }) as TailwindSorter
    });

    let string_embedder = needs_embedded.then(|| {
        super::prettier_string::build_string_embedder(
            Arc::clone(&external_services.format_embedded),
            sorter.clone(),
            Arc::clone(dispatch_config),
        )
    });

    SessionServices {
        dispatcher: dispatch_config.root_dispatcher(fallback),
        string_embedder,
        tailwind_sorter: sorter,
    }
}
