//! Embedded-language formatting infrastructure.
//!
//! All formatters are peers:
//! any formatter may act as a parent (containing embedded code) or as a child (being embedded).
//!
//! Only the entry formatter is called directly by the orchestrator (oxfmt);
//! every further embedded call goes through a [`FormatDispatcher`] that the orchestrator assembles,
//! mapping a language name to a formatter implementation (or a fallback).
//!
//! Core only carries the shared plumbing (arena, group-id space, recursion handle)
//! and the cross-language contract field ([`DispatchResult::tailwind_classes`]);
//! anything truly language-pair specific crosses as a `dyn Any` passthrough.
//! Core knows nothing about any concrete language.

use std::{any::Any, sync::Arc};

use oxc_allocator::{Allocator, ArenaVec};

use crate::{FormatElement, InputKind, group_id::UniqueGroupIdBuilder};

/// Shared IR-infrastructure context for formatting embedded code.
///
/// The same context is threaded through recursive dispatcher calls so that
/// nested embeddings (e.g. css-in-html-in-js) share one arena and one
/// `GroupId` space.
///
/// TODO: Migration adapter: [`crate::FormatSession`] supersedes this type
/// and will replace it once every entry point is session-aware.
pub struct EmbeddedContext<'a, 'g> {
    /// Arena shared between parent and child formatters;
    /// strings allocated by the child live as long as the parent's IR.
    pub allocator: &'a Allocator,
    /// `GroupId` builder shared to avoid id collisions across formatters.
    pub group_id_builder: &'g UniqueGroupIdBuilder,
    /// Dispatcher for the child formatter to format its own embedded languages.
    /// `None` when recursion is not available (e.g. plain standalone formatting).
    pub dispatcher: Option<FormatDispatcher>,
}

/// One embedded-language formatting request, as the host formatter states it.
pub struct DispatchRequest<'r> {
    /// Generic language identifier (e.g. `"css"`, `"graphql"`);
    /// the dispatcher implementation maps it to its own parser/language names.
    pub language: &'r str,
    /// Code to format. Usually a single text;
    /// GraphQL sends N quasis and receives N IRs back (the batch is atomic: all format or none do).
    pub texts: &'r [&'r str],
    /// Envelope semantics of the child input, as declared by the host.
    pub input_kind: InputKind,
    /// Parent→child language-pair specific data,
    /// downcast by the implementation (`None` for most pairs).
    pub parent_context: Option<&'r dyn Any>,
}

/// What a dispatch produced.
///
/// [`Self::PreserveOriginal`] is the DELIBERATE "do not format" answer
/// (unsupported language, child parse failure, an envelope the child refuses,
/// embedded formatting turned off): the caller keeps the original source as-is.
/// `Result::Err` around this enum is reserved for operational failures (transport / internal errors);
/// optional-embed callers degrade the same way for both,
/// but the two must never be conflated at the source.
pub enum DispatchOutcome<'a> {
    /// The child formatted the request; consume [`DispatchResult`].
    Formatted(DispatchResult<'a>),
    /// Deliberately not formatted; keep the original source untouched.
    PreserveOriginal,
}

/// Dispatcher resolving a language name to a formatter implementation.
///
/// Assembled by the orchestrator (oxfmt), which knows all languages;
/// formatter crates only invoke it.
/// The intended entry is [`crate::FormatSession::dispatch`]
/// (which owns the recursion limit and the no-dispatcher case),
/// but until every entry point is session-aware,
/// the JS host's `dispatch_embedded` adapter still invokes it directly WITHOUT the depth guard.
pub type FormatDispatcher = Arc<
    dyn for<'a, 'g, 'r> Fn(
            &EmbeddedContext<'a, 'g>,
            DispatchRequest<'r>,
        ) -> Result<DispatchOutcome<'a>, String>
        + Send
        + Sync,
>;

/// IR built by a language crate's embedded entry point (`format_to_ir`) for
/// ONE input text. The orchestrator's dispatcher assembles one or more of
/// these into a [`DispatchResult`].
///
/// Every language crate's `format_to_ir` returns this shape, so a new child
/// language only has to fill in the fields (no per-crate tuple conventions).
pub struct EmbeddedIr<'a> {
    /// The formatter IR, arena-allocated alongside its elements.
    pub ir: ArenaVec<'a, FormatElement<'a>>,
    /// Pre-sort Tailwind classes referenced by the IR's
    /// `FormatElement::TailwindClass` indices (0-based, local to this IR).
    /// Empty unless the language collects classes (e.g. CSS `@apply`).
    pub tailwind_classes: Vec<String>,
}

/// Result of a [`FormatDispatcher`] call.
pub struct DispatchResult<'a> {
    /// One IR per input text (usually one; GraphQL returns one per quasi).
    /// Each IR is arena-allocated alongside its elements.
    /// Single-doc consumers extract the IR via `docs.into_iter().next()` after calling
    /// [`Self::remap_tailwind_into`]; multi-doc consumers (GraphQL) walk `docs`.
    pub docs: Vec<ArenaVec<'a, FormatElement<'a>>>,
    /// Pre-sort Tailwind classes referenced by the docs'
    /// `FormatElement::TailwindClass` indices (0-based, local to this result).
    /// The receiving parent MUST merge them into its own class space via [`Self::remap_tailwind_into`] before printing,
    /// the printer's `debug_assert` catches a forgotten merge.
    pub tailwind_classes: Vec<String>,
    /// Child→parent language-specific metadata; the parent downcasts it
    /// (e.g. HTML's `has_multiple_root_elements`).
    pub meta: Option<Box<dyn Any>>,
}

impl DispatchResult<'_> {
    /// Move the child's pre-sort Tailwind classes into the parent's class space
    /// and shift the docs' `TailwindClass` indices to match. Call once per
    /// received result before consuming `docs` (a no-op when the child collected nothing).
    /// The entry formatter's document then sorts all collected classes in one host-supplied batch.
    pub fn remap_tailwind_into(&mut self, collector: &mut dyn TailwindCollector) {
        let mut classes = std::mem::take(&mut self.tailwind_classes).into_iter();
        let Some(first) = classes.next() else {
            return;
        };
        // The collector hands out consecutive indices,
        // so the first one is the base offset for every local index.
        let base = collector.add_class(first);
        for class in classes {
            collector.add_class(class);
        }
        for doc in &mut self.docs {
            for element in doc.iter_mut() {
                if let FormatElement::TailwindClass(index) = element {
                    *index += base;
                }
            }
        }
    }
}

/// Index-space provider for batched Tailwind class sorting.
///
/// `FormatElement::TailwindClass(usize)` holds pre-sort class strings by
/// index; sorting happens in one host-supplied batch when the entry
/// formatter's document is finalized. A child formatter collects classes
/// locally (0-based) and returns them in [`DispatchResult::tailwind_classes`];
/// the receiving parent implements this trait on its format context and
/// calls [`DispatchResult::remap_tailwind_into`] before consuming `docs`.
///
/// NOTE: an alternative design — threading one shared collector through
/// [`EmbeddedContext`] so children allocate parent indices directly — was
/// considered and deferred: it needs interior mutability plumbing through
/// every format context for no current gain. Revisit if deep embedding nests
/// (e.g. css-in-html-in-js at plan Step 8/9) make per-boundary remapping
/// burdensome.
pub trait TailwindCollector {
    /// Register a class string, returning its index in the collector's space.
    fn add_class(&mut self, class: String) -> usize;
}
