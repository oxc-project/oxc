//! Embedded-language formatting infrastructure.
//!
//! All formatters are peers: any formatter may act as a parent (containing
//! embedded code) or as a child (being embedded). Only the entry formatter is
//! called directly by the orchestrator (oxfmt); every further embedded call
//! goes through a [`FormatDispatcher`] that the orchestrator assembles,
//! mapping a language name to a formatter implementation (or a fallback).
//!
//! Core only carries the shared plumbing (arena, group-id space, recursion
//! handle) plus `dyn Any` passthroughs for language-pair specific data;
//! it knows nothing about any concrete language.

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

/// Result of a [`FormatDispatcher`] call.
pub struct DispatchResult<'a> {
    /// One IR per input text (usually one; GraphQL returns one per quasi).
    /// Each IR is arena-allocated alongside its elements.
    pub docs: Vec<ArenaVec<'a, FormatElement<'a>>>,
    /// Child→parent language-specific metadata; the parent downcasts it
    /// (e.g. placeholder survival counts for CSS/HTML).
    pub meta: Option<Box<dyn Any>>,
}

impl<'a> DispatchResult<'a> {
    /// Extract the IR for single-text dispatches (every language except GraphQL).
    /// Returns `None` when the dispatcher produced no docs.
    pub fn into_single_doc(self) -> Option<ArenaVec<'a, FormatElement<'a>>> {
        self.docs.into_iter().next()
    }
}

/// Collector sharing one Tailwind class index space across embedded boundaries.
///
/// `FormatElement::TailwindClass(usize)` holds pre-sort class strings by index;
/// sorting happens in one batch after the entry formatter completes. Parent and
/// child must allocate indices from the same collector to avoid collisions.
pub trait TailwindCollector {
    /// Register a class string, returning its index in the shared space.
    fn add_class(&mut self, class: String) -> usize;
}

/// No-op collector for languages without Tailwind support (JSON, GraphQL, …).
impl TailwindCollector for () {
    fn add_class(&mut self, _class: String) -> usize {
        0
    }
}
