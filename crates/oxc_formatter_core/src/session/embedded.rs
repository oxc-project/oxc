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

use rustc_hash::FxHashMap;

use oxc_allocator::{Allocator, ArenaVec};

use crate::{
    FormatContext, FormatElement, FormatSession, Formatter, IndentWidth, InputKind, LineMode,
    format_element::{BestFittingElement, Interned, TextWidth},
    write::formatter::intern_exact,
};

/// One embedded-language formatting request, as the host formatter states it.
pub struct DispatchRequest<'r> {
    /// Generic language identifier (e.g. `"css"`, `"graphql"`), or a code fence's name as written;
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

impl DispatchRequest<'_> {
    /// [`Self::parent_context`] as a `T`, when the parent passed one.
    pub fn parent_context_as<T: Any>(&self) -> Option<&T> {
        self.parent_context.and_then(|c| c.downcast_ref::<T>())
    }
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

/// Dispatches one embedded fragment and hands out its doc:
/// [`dispatch_ir`] with `InputKind::Fragment`.
pub fn dispatch_fragment_ir<'a, C: FormatContext>(
    f: &Formatter<'_, 'a, C>,
    language: &str,
    text: &str,
    parent_context: Option<&dyn Any>,
) -> Option<ArenaVec<'a, FormatElement<'a>>> {
    dispatch_ir(
        f,
        DispatchRequest { language, text, input_kind: InputKind::Fragment, parent_context },
    )
}

/// Dispatches one embedded request and hands out its doc.
///
/// `None` covers [`DispatchResponse::PreserveOriginal`] and operational errors alike;
/// the caller keeps its original source.
/// Embed sites that must inspect [`DispatchPayload::child_context`] first stay manual.
pub fn dispatch_ir<'a, C: FormatContext>(
    f: &Formatter<'_, 'a, C>,
    request: DispatchRequest<'_>,
) -> Option<ArenaVec<'a, FormatElement<'a>>> {
    let Ok(DispatchResponse::Formatted(result)) = f.session().dispatch(request) else {
        return None;
    };
    Some(result.doc)
}

/// Rebuild an embedded IR, descending into BestFitting variants and interned content.
///
/// `map_text` receives each `Text` and `Token` run (a child may print any character as a token, e.g. JS's `` ` ``);
/// it either pushes replacement elements into the output and returns `true`, or returns `false` to keep the element unchanged.
/// A shared `Interned` subtree is rebuilt once and the rebuilt element re-shared.
#[expect(clippy::mutable_key_type)] // `Interned` hashes by pointer identity
pub fn map_text_in_ir<'a, F>(
    ir: &[FormatElement<'a>],
    allocator: &'a Allocator,
    map_text: &mut F,
) -> ArenaVec<'a, FormatElement<'a>>
where
    F: FnMut(&'a str, &mut ArenaVec<'a, FormatElement<'a>>) -> bool,
{
    let mut interned_cache = FxHashMap::default();
    map_text_in_ir_impl(ir, allocator, map_text, &mut interned_cache)
}

#[expect(clippy::mutable_key_type)] // `Interned` hashes by pointer identity
fn map_text_in_ir_impl<'a, F>(
    ir: &[FormatElement<'a>],
    allocator: &'a Allocator,
    map_text: &mut F,
    interned_cache: &mut FxHashMap<Interned<'a>, Option<FormatElement<'a>>>,
) -> ArenaVec<'a, FormatElement<'a>>
where
    F: FnMut(&'a str, &mut ArenaVec<'a, FormatElement<'a>>) -> bool,
{
    let mut out = ArenaVec::with_capacity_in(ir.len(), &allocator);
    for element in ir {
        match element {
            FormatElement::Text { text, .. } | FormatElement::Token { text } => {
                if !map_text(text, &mut out) {
                    out.push(element.clone());
                }
            }
            FormatElement::BestFitting(best_fitting) => {
                let mut variants =
                    ArenaVec::with_capacity_in(best_fitting.variants().len(), &allocator);
                for variant in best_fitting.variants() {
                    let mapped = map_text_in_ir_impl(variant, allocator, map_text, interned_cache);
                    variants.push(mapped.into_arena_slice());
                }
                // SAFETY: This rebuild preserves the original BestFitting's variant count.
                out.push(FormatElement::BestFitting(unsafe {
                    BestFittingElement::from_vec_unchecked(variants)
                }));
            }
            FormatElement::Interned(interned) => {
                let mapped = if let Some(mapped) = interned_cache.get(interned) {
                    mapped.clone()
                } else {
                    let mapped = map_text_in_ir_impl(interned, allocator, map_text, interned_cache);
                    let mapped = intern_exact(allocator, mapped.into_iter());
                    interned_cache.insert(interned.clone(), mapped.clone());
                    mapped
                };
                out.extend(mapped);
            }
            _ => out.push(element.clone()),
        }
    }
    out
}

/// Pushes `text` with each newline as a literal line (Prettier's `replaceEndOfLine()`).
///
/// For a [`map_text_in_ir`] callback:
/// a literal line adds no indentation of its own, it resumes at the enclosing root (`mark_as_root`).
///
/// TODO: An embedding boundary rewrites the child IR with this, because a newline inside a `Text` has two readings:
/// - the printer prints it as is (column 0), as Prettier prints a string
/// - the IR to Prettier Doc conversion (oxfmt's `to_prettier_doc`) always makes it a `literalline`
///
/// A boundary scope tag (a `mark_as_root` variant whose `Text` newlines resume at the root) would replace the rewrite.
/// Revisit once the Vue and HTML ports remove the Doc conversion path, which leaves the printer as the only reader.
pub fn push_text_with_literal_lines<'a>(
    out: &mut ArenaVec<'a, FormatElement<'a>>,
    text: &'a str,
    indent_width: IndentWidth,
) {
    // Splitting on `\n` is safe because the IR only contains normalized linebreaks
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push(FormatElement::Line(LineMode::Literal));
        }
        if !line.is_empty() {
            out.push(FormatElement::Text {
                text: line,
                width: TextWidth::from_text(line, indent_width),
            });
        }
    }
}
