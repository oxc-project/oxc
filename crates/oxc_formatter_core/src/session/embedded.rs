//! Embedded-language formatting infrastructure.
//!
//! All formatters are peers:
//! any formatter may act as a parent (containing embedded code) or as a child (being embedded).
//!
//! Only the entry formatter is called directly by the orchestrator (oxfmt);
//! every further embedded call goes through a [`FormatDispatcher`] that the orchestrator assembles,
//! mapping a language name to a formatter implementation (or a fallback).
//!
//! Core only carries the shared plumbing (arena, group-id space, Tailwind class scope, recursion handle);
//! anything truly language-pair specific crosses as a `dyn Any` passthrough.
//! Core knows nothing about any concrete language.

use std::{any::Any, sync::Arc};

use oxc_allocator::ArenaVec;

use crate::{FormatContext, FormatElement, FormatSession, Formatter, InputKind};

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
    /// downcast by the implementation (`None` for most pairs; e.g. JS's `CssInJsTemplate`).
    /// The borrowed counterpart of [`DispatchPayload::child_context`]
    /// (borrowed because the parent outlives the dispatch call).
    pub parent_context: Option<&'r dyn Any>,
}

/// The dispatcher's answer to a [`DispatchRequest`].
///
/// [`Self::PreserveOriginal`] is the DELIBERATE "do not format" answer
/// (unsupported language, child parse failure, an envelope the child refuses,
/// embedded formatting turned off): the caller keeps the original source as-is.
/// `Result::Err` around this enum is reserved for operational failures (transport / internal errors);
/// optional-embed callers degrade the same way for both,
/// but the two must never be conflated at the source.
pub enum DispatchResponse<'a> {
    /// The child formatted the request; consume [`DispatchPayload`].
    Formatted(DispatchPayload<'a>),
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
        ) -> Result<DispatchResponse<'a>, String>
        + Send
        + Sync,
>;

/// IR built by a language crate's embedded entry point (`format_to_ir`) for one input text.
/// The orchestrator's dispatcher wraps it into a [`DispatchPayload`].
///
/// Every language crate's `format_to_ir` returns this shape,
/// so a new child language only has to fill in the fields (no per-crate tuple conventions).
pub struct EmbeddedIr<'a> {
    /// The formatter IR, arena-allocated alongside its elements.
    /// Its `FormatElement::TailwindClass` indices point into the session's class scope.
    pub ir: ArenaVec<'a, FormatElement<'a>>,
}

impl<'a> From<EmbeddedIr<'a>> for DispatchPayload<'a> {
    fn from(embedded: EmbeddedIr<'a>) -> Self {
        Self { doc: embedded.ir, child_context: None }
    }
}

/// The child's formatted product, carried by [`DispatchResponse::Formatted`].
pub struct DispatchPayload<'a> {
    /// The formatted IR, arena-allocated alongside its elements.
    /// Its `FormatElement::TailwindClass` indices already point into the parent's class scope
    /// (the child session shares it), so the parent writes it as-is.
    pub doc: ArenaVec<'a, FormatElement<'a>>,
    /// Child→parent language-pair specific data,
    /// downcast by the parent (`None` for most pairs; e.g. HTML's `has_multiple_root_elements`).
    /// The owned counterpart of [`DispatchRequest::parent_context`]
    /// (owned because it outlives the child's stack frame).
    pub child_context: Option<Box<dyn Any>>,
}

/// Dispatches one embedded fragment (`InputKind::Fragment`) and hands out its doc.
///
/// `None` covers [`DispatchResponse::PreserveOriginal`] and operational errors alike;
/// the caller keeps its original source.
/// Embed sites that must inspect [`DispatchPayload::child_context`] first stay manual.
pub fn dispatch_fragment_ir<'a, C: FormatContext>(
    f: &Formatter<'_, 'a, C>,
    language: &str,
    text: &str,
    parent_context: Option<&dyn Any>,
) -> Option<ArenaVec<'a, FormatElement<'a>>> {
    let Ok(DispatchResponse::Formatted(result)) = f.session().dispatch(DispatchRequest {
        language,
        text,
        input_kind: InputKind::Fragment,
        parent_context,
    }) else {
        return None;
    };
    Some(result.doc)
}
