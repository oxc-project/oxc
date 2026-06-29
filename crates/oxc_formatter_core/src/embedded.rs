//! Embedded-language formatting infrastructure.
//!
//! All formatters are peers: any formatter may act as a parent (containing
//! embedded code) or as a child (being embedded). Only the entry formatter is
//! called directly by the orchestrator (oxfmt); every further embedded call
//! goes through a [`FormatDispatcher`] that the orchestrator assembles,
//! mapping a language name to a formatter implementation (or a fallback).
//!
//! Core only carries the shared plumbing (arena, group-id space, recursion
//! handle) and the cross-language contract fields ([`DispatchResult`]'s
//! `tailwind_classes` / `placeholder_count`); anything truly language-pair
//! specific crosses as a `dyn Any` passthrough. Core knows nothing about any
//! concrete language.

use std::any::Any;
use std::sync::Arc;

use oxc_allocator::{Allocator, ArenaVec};

use crate::{FormatElement, group_id::UniqueGroupIdBuilder};

/// Shared IR-infrastructure context for formatting embedded code.
///
/// The same context is threaded through recursive dispatcher calls so that
/// nested embeddings (e.g. css-in-html-in-js) share one arena and one
/// `GroupId` space.
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

/// Dispatcher resolving a language name to a formatter implementation.
///
/// Assembled by the orchestrator (oxfmt), which knows all languages;
/// formatter crates only invoke it. Arguments are
/// `(context, language, texts, parent_context)`:
///
/// - `language`: generic language identifier (e.g. `"css"`, `"graphql"`)
/// - `texts`: code to format. Usually a single text; GraphQL sends N quasis
///   and receives N IRs back
/// - `parent_context`: parent→child language-pair specific data, downcast by
///   the implementation (`None` for most pairs)
pub type FormatDispatcher = Arc<
    dyn for<'a, 'g> Fn(
            &EmbeddedContext<'a, 'g>,
            &str,
            &[&str],
            Option<&dyn Any>,
        ) -> Result<DispatchResult<'a>, String>
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
    pub docs: Vec<ArenaVec<'a, FormatElement<'a>>>,
    /// Pre-sort Tailwind classes referenced by the docs'
    /// `FormatElement::TailwindClass` indices (0-based, local to this result).
    /// The receiving parent MUST merge them into its own class space via
    /// [`Self::remap_tailwind_into`] — the printer asserts on dangling indices.
    pub tailwind_classes: Vec<String>,
    /// How many host-substituted placeholder markers (standing in for `${}`
    /// interpolations) survived formatting. The parent compares this against
    /// its expression count to decide whether it can splice them back in.
    /// `None` when the language pair doesn't use placeholders.
    pub placeholder_count: Option<usize>,
    /// Child→parent language-specific metadata; the parent downcasts it
    /// (e.g. HTML's `has_multiple_root_elements`).
    pub meta: Option<Box<dyn Any>>,
}

impl<'a> DispatchResult<'a> {
    /// Extract the IR for single-text dispatches (every language except GraphQL).
    /// Returns `None` when the dispatcher produced no docs.
    pub fn into_single_doc(self) -> Option<ArenaVec<'a, FormatElement<'a>>> {
        self.docs.into_iter().next()
    }

    /// Move the child's pre-sort Tailwind classes into the parent's class
    /// space and shift the docs' `TailwindClass` indices to match.
    ///
    /// Call this once per received dispatch result (a no-op when the child
    /// collected nothing). The entry formatter's document then sorts all
    /// collected classes in one host-supplied batch.
    pub fn remap_tailwind_into(&mut self, collector: &mut dyn TailwindCollector) {
        let mut classes = std::mem::take(&mut self.tailwind_classes).into_iter();
        let Some(first) = classes.next() else {
            return;
        };
        // The collector hands out consecutive indices, so the first one is
        // the base offset for every local index.
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
/// merges them with [`DispatchResult::remap_tailwind_into`].
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
