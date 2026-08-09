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

use oxc_allocator::ArenaVec;

use crate::{FormatElement, FormatSession, InputKind};

/// One embedded-language formatting request, as the host formatter states it.
pub struct DispatchRequest<'r> {
    /// Generic language identifier (e.g. `"css"`, `"graphql"`);
    /// the dispatcher implementation maps it to its own parser/language names.
    pub language: &'r str,
    /// The code to format.
    pub text: &'r str,
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
/// formatter crates only invoke it via [`FormatSession::dispatch`],
/// which owns the recursion limit and the no-dispatcher case.
/// The callback receives the CHILD session, already derived from the caller's
/// (same arena / `GroupId` space / dispatcher, the request's `InputKind`, depth + 1).
pub type FormatDispatcher = Arc<
    dyn for<'a, 'r> Fn(
            &FormatSession<'a>,
            DispatchRequest<'r>,
        ) -> Result<DispatchOutcome<'a>, String>
        + Send
        + Sync,
>;

/// IR built by a language crate's embedded entry point (`format_to_ir`) for one input text.
/// The orchestrator's dispatcher wraps it into a [`DispatchResult`].
///
/// Every language crate's `format_to_ir` returns this shape,
/// so a new child language only has to fill in the fields (no per-crate tuple conventions).
pub struct EmbeddedIr<'a> {
    /// The formatter IR, arena-allocated alongside its elements.
    pub ir: ArenaVec<'a, FormatElement<'a>>,
    /// Pre-sort Tailwind classes referenced by the IR's
    /// `FormatElement::TailwindClass` indices (0-based, local to this IR).
    /// Empty unless the language collects classes (e.g. CSS `@apply`).
    pub tailwind_classes: Vec<String>,
}

/// One [`EmbeddedIr`] becomes a result,
/// carrying the child's Tailwind classes through (hand-rolling the literal invites silently dropping them).
impl<'a> From<EmbeddedIr<'a>> for DispatchResult<'a> {
    fn from(embedded: EmbeddedIr<'a>) -> Self {
        Self { doc: embedded.ir, tailwind_classes: embedded.tailwind_classes, meta: None }
    }
}

/// Result of a [`FormatDispatcher`] call.
///
/// Consume through [`Self::into_doc`] when a parent index space exists
/// (every IR-channel embed site); only a parent-less consumer
/// (the fence adapter, which sorts locally) destructures the fields directly.
pub struct DispatchResult<'a> {
    /// The formatted IR, arena-allocated alongside its elements.
    pub doc: ArenaVec<'a, FormatElement<'a>>,
    /// Pre-sort Tailwind classes referenced by the doc's
    /// `FormatElement::TailwindClass` indices (0-based, local to this result).
    /// The receiving parent MUST merge them into its own class space ([`Self::into_doc`] does);
    /// the printer's `debug_assert` catches a forgotten merge.
    pub tailwind_classes: Vec<String>,
    /// Child→parent language-specific metadata; the parent downcasts it
    /// (e.g. HTML's `has_multiple_root_elements`).
    pub meta: Option<Box<dyn Any>>,
}

impl<'a> DispatchResult<'a> {
    /// Consumes the result:
    /// moves the child's pre-sort Tailwind classes into the parent's class space and hands out the doc.
    /// Folding the merge into the only way to get the doc makes a forgotten merge unrepresentable.
    /// The entry formatter's document then sorts all collected classes in one host-supplied batch.
    ///
    /// A consumer may DISCARD the returned doc afterwards (an all-or-nothing embed site keeping its template verbatim):
    /// the already-merged classes stay as unreferenced collector entries, which are inert.
    /// The sorter reorders classes WITHIN each string, never the vector,
    /// so indices stay stable and unprinted entries never reach the output.
    pub fn into_doc(
        mut self,
        collector: &mut dyn TailwindCollector,
    ) -> ArenaVec<'a, FormatElement<'a>> {
        // The remap below reaches only top-level elements;
        // a `TailwindClass` below an `Interned` / `BestFitting` boundary is unrewritable
        // (`Interned` targets are shared `&[FormatElement]`),
        // its stale index would print the WRONG classes with no assert.
        // If this ever fires, that is the trigger to revisit the session-shared collector
        // (see the NOTE on `TailwindCollector`).
        debug_assert!(
            self.tailwind_classes.is_empty() || !has_nested_tailwind_class(&self.doc),
            "child IR holds a TailwindClass inside an Interned/BestFitting subtree; the flat remap cannot reach it"
        );
        let mut classes = std::mem::take(&mut self.tailwind_classes).into_iter();
        if let Some(first) = classes.next() {
            // The collector hands out consecutive indices,
            // so the first one is the base offset for every local index.
            let base = collector.add_class(first);
            for class in classes {
                collector.add_class(class);
            }
            for element in &mut self.doc {
                if let FormatElement::TailwindClass(index) = element {
                    *index += base;
                }
            }
        }
        self.doc
    }
}

/// Whether a `TailwindClass` sits below an `Interned` / `BestFitting` boundary,
/// where [`DispatchResult::into_doc`]'s flat remap cannot rewrite it
/// (a top-level `TailwindClass` is the remap's normal input, not a hit).
fn has_nested_tailwind_class(elements: &[FormatElement<'_>]) -> bool {
    fn contains(element: &FormatElement<'_>) -> bool {
        matches!(element, FormatElement::TailwindClass(_)) || descends(element)
    }
    fn descends(element: &FormatElement<'_>) -> bool {
        match element {
            FormatElement::Interned(interned) => interned.iter().any(contains),
            FormatElement::BestFitting(best_fitting) => {
                best_fitting.variants().iter().any(|variant| variant.iter().any(contains))
            }
            _ => false,
        }
    }
    elements.iter().any(descends)
}

/// Index-space provider for batched Tailwind class sorting.
///
/// `FormatElement::TailwindClass(usize)` holds pre-sort class strings by index;
/// sorting happens in one host-supplied batch when the entry formatter's document is finalized.
/// A child formatter collects classes locally (0-based) and returns them in [`DispatchResult::tailwind_classes`];
/// the receiving parent implements this trait on its format context and receives them through [`DispatchResult::into_doc`].
///
/// NOTE: an alternative design (threading one shared collector through [`FormatSession`]
/// so children allocate parent indices directly) was considered and deferred:
/// it needs interior mutability plumbing through every format context for no current gain.
/// Revisit if deep embedding nests (e.g. css-in-html-in-js at plan Step 8/9) make per-boundary remapping burdensome.
pub trait TailwindCollector {
    /// Register a class string, returning its index in the collector's space.
    fn add_class(&mut self, class: String) -> usize;
}
