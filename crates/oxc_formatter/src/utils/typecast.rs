//! JSDoc type casts: `/** @type {T} */ (expr)`.
//!
//! The cast comment types the parenthesized expression right after it,
//! so the parens must survive formatting and the comment must stay in front of them.
//! The AST has no paren node (`preserve_parens: false`); the cast is recovered from the source (see `CastCommentGap`):
//! a cast comment, then only `(`s and trivia, then the node, then `)` makes the node a [`TypeCast::Target`].
//!
//! # Printing protocol
//!
//! The generated `fmt` of every node with `NeedsParentheses` first offers the node to [`format_type_cast_comment_node`]:
//! 1. [`classify_type_cast`] finds the node's binding cast comment among its UNPRINTED leading comments
//!    (the first one: nested casts `/** U */ (/** T */ (x))` are peeled from the outside, one per pass);
//! 2. [`format_type_cast_comment_node`] prints them, marks the node, and re-enters its `fmt` inside `(`...`)`;
//! 3. the re-entered `fmt` offers the node again;
//!    the cast comment just printed counts as handled (`Comments::is_handled_type_cast_comment`),
//!    so only a further unprinted cast can bind now (the inner one of a nest, which repeats step 2);
//!    with none, the offer is declined and the node prints itself.
//!
//! A cast whose paren closes inside the node ([`TypeCast::BindsInner`]) is kept adjacent to its target
//! by [`format_leading_comments_and_open_paren`].
//!
//! # Layout decisions
//!
//! Shape-based layout rules see a cast target as Prettier's kept `ParenthesizedExpression` and ask [`is_cast_target`].
//! `classify_type_cast` and the mark are for printing only.

use oxc_ast::Comment;
use oxc_span::{GetSpan, Span};

use crate::{
    Buffer, Format, format_args,
    formatter::{
        JsFormatter,
        prelude::*,
        trivia::{FormatLeadingComments, format_leading_comments},
    },
    print::write_comments_before_closing_paren,
    utils::suppressed::FormatSuppressedNode,
    write,
};

/// How a JSDoc type cast comment relates to a node.
pub enum TypeCast<'a> {
    /// The node itself is the cast target
    /// (the parenthesis right after the comment closes right after the node):
    /// ```js
    /// /** @type {string} */ (value).length
    ///                        ^^^^^ Target
    /// /** @type {Document} */ (root.head ?? fallback)
    ///                          ^^^^^^^^^^^^^^^^^^^^^ Target
    /// ```
    /// `pending` holds the comments still to be printed;
    /// it is empty when the cast comment was already printed
    /// (re-entry from [`format_type_cast_comment_node`], or printed by an ancestor).
    /// `cast_comment` is the cast comment either way.
    Target { pending: &'a [Comment], cast_comment: &'a Comment },
    /// The node's unprinted leading comments end with a cast comment that binds
    /// to an inner expression (its parenthesis closes before the node ends):
    /// ```js
    /// /** @type {Number} */ (bar).zoo
    ///                       ^^^^^^^^^ BindsInner (the member expression)
    ///                       ^^^^^ the cast target is inside it
    /// x ? y : /** @type {D} */ (a).b ?? c
    ///                          ^^^^^^^^^^ BindsInner (the `??` expression)
    /// ```
    /// Formatter-added parentheses around the node must not separate the
    /// comment from its target (see [`format_leading_comments_and_open_paren`]).
    /// The slice holds the node's leading comments, ending with the cast comment.
    BindsInner(&'a [Comment]),
    /// No cast is involved with this node:
    /// no adjacent cast comment, a cast-shaped comment without following `(`
    /// (`/** @type {N} */ value`), or an already-printed cast settled at an
    /// inner level (`.zoo` in the `BindsInner` example above, once the comment is printed).
    None,
}

/// Classifies how a type cast comment relates to the node at `span`, for printing:
/// which comments are still pending is a cursor question,
/// so this reads the printed state (the first unprinted cast comment, or the last printed one).
/// Both [`format_type_cast_comment_node`] and [`format_leading_comments_and_open_paren`] consume it.
pub fn classify_type_cast<'a>(span: Span, f: &JsFormatter<'_, 'a>) -> TypeCast<'a> {
    let comments = f.context().comments();

    // The cast comment may already be printed:
    // by the re-entry from `format_type_cast_comment_node`,
    // or as the leading comment of an ancestor starting at the same position.
    if !comments.is_handled_type_cast_comment()
        && let Some(last_printed_comment) = comments.printed_comments().last()
        && last_printed_comment.span.end <= span.start
        && comments.is_type_cast_comment_followed_by_paren(last_printed_comment)
    {
        match classify_cast_comment_gap(last_printed_comment.span.end, span.start, f) {
            CastCommentGap::ParensAndTrivia if comments.is_followed_by_closing_paren(span.end) => {
                return TypeCast::Target { pending: &[], cast_comment: last_printed_comment };
            }
            // The printed comment is unrelated to this node;
            // fall through to look for the node's own unprinted cast comment.
            CastCommentGap::Code => {}
            // `Trivia` here means the cast binds to an inner node;
            // since the comment is already printed, there is nothing left to keep adjacent at this level.
            _ => return TypeCast::None,
        }
    }

    if let Some(type_cast_comment_index) = comments.get_type_cast_comment_index(span) {
        let unprinted_comments = comments.unprinted_comments();
        let type_cast_comment = &unprinted_comments[type_cast_comment_index];

        return match classify_cast_comment_gap(type_cast_comment.span.end, span.start, f) {
            CastCommentGap::Trivia => {
                TypeCast::BindsInner(&unprinted_comments[..=type_cast_comment_index])
            }
            CastCommentGap::ParensAndTrivia if comments.is_followed_by_closing_paren(span.end) => {
                TypeCast::Target {
                    pending: &unprinted_comments[..=type_cast_comment_index],
                    cast_comment: type_cast_comment,
                }
            }
            _ => TypeCast::None,
        };
    }

    TypeCast::None
}

/// Whether the node at `span` is a cast target (see [`TypeCast::Target`]), for layout decisions.
/// Position-based (source only, no printed state), so every frame agrees, the target mid-format included.
/// Every cast target closes with `)`: the byte peek rejects the common case before any comment lookup.
pub fn is_cast_target(span: Span, f: &JsFormatter<'_, '_>) -> bool {
    let source = f.source_text();
    // A target is preceded by its `(` or by the `/` closing a comment:
    // one byte peek rejects most nodes before the `)` peek (true for every last argument and parenthesized test)
    // and the comment lookup.
    if !matches!(
        source.bytes_to(span.start).find(|byte| !byte.is_ascii_whitespace()),
        Some(b'(' | b'/')
    ) {
        return false;
    }
    let comments = f.context().comments();
    if !comments.is_followed_by_closing_paren(span.end) {
        return false;
    }
    // Walk back over the comments before the node, one gap segment at a time, stopping at the first code byte.
    // A cast comment with no `(` between it and the node binds to an inner node, and the walk continues:
    // an earlier cast may still wrap this one (`/** @type {U} */ (/** @type {T} */ (a).b)`).
    let mut cursor = span.start;
    let mut has_paren = false;
    for comment in comments.all_comments_before(span.start).iter().rev() {
        let Some(segment_has_paren) =
            classify_gap_segment(source.bytes_range(comment.span.end, cursor))
        else {
            return false;
        };
        has_paren |= segment_has_paren;
        if has_paren && comments.is_type_cast_comment_followed_by_paren(comment) {
            return true;
        }
        cursor = comment.span.start;
    }
    false
}

/// A cast target's pending comments and the span of its cast parentheses
/// (from the first `(` after the cast comment to after the matching `)`), `None` for any other node.
fn cast_target_parens<'a>(span: Span, f: &JsFormatter<'_, 'a>) -> Option<(&'a [Comment], Span)> {
    // Fast path: every target closes with `)`
    if !f.context().comments().is_followed_by_closing_paren(span.end) {
        return None;
    }
    let TypeCast::Target { pending, cast_comment } = classify_type_cast(span, f) else {
        return None;
    };
    cast_parens_span(cast_comment.span.end, span, f).map(|parens| (pending, parens))
}

/// The position after the cast parentheses of a cast target, `None` for any other node.
///
/// A site that hides a child's trailing comments must hide from here, not from the child's span end:
/// the target prints its parens and the comments inside them,
/// and hiding those blinds its trailing-comments pass, so the comment escapes the parens.
/// ```js
/// const x = /** @type {T} */ (
///   value // must stay inside
/// );
/// ```
pub fn cast_target_end(span: Span, f: &JsFormatter<'_, '_>) -> Option<u32> {
    cast_target_parens(span, f).map(|(_, parens)| parens.end)
}

/// Formats a node that is the target of a JSDoc type cast (see [`TypeCast::Target`]):
/// prints the pending cast comments, marks the node
/// (so `NeedsParentheses` rules skip their own parentheses), and wraps it in parentheses,
/// like `/** @type {string} */ (value)` or `/** @type {number} */ ((expression))`.
/// Step 2 of the module's printing protocol: the node's `fmt` is re-entered inside the parens and declines the second offer.
///
/// Returns `true` if the node was formatted as a cast target, `false` otherwise;
/// callers apply their own formatting on `false`.
pub fn format_type_cast_comment_node<'a>(
    node: &(impl Format<'a, JsFormatContext<'a>> + GetSpan),
    is_object_or_array_expression: bool,
    f: &mut JsFormatter<'_, 'a>,
) -> bool {
    // Check if this node is a cast target and get the comments to print
    let TypeCast::Target { pending: type_cast_comments, .. } = classify_type_cast(node.span(), f)
    else {
        return false;
    };

    // Print the type cast comments if any
    if !type_cast_comments.is_empty() {
        write!(f, [FormatLeadingComments::Comments(type_cast_comments)]);
    }

    let span = node.span();
    f.context_mut().comments_mut().mark_as_type_cast_node(node);

    let format_node = format_with(|f| {
        node.fmt(f);
        // Own-line comments left before the cast `)` have no following node inside the parens to lead;
        // deferred, they would lead the next sibling (a call's arguments) outside the parens.
        // Same-line ones are already printed by the node's trailing pass.
        write_comments_before_closing_paren(f, span.end);
    });

    // https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/print/estree.js#L117-L120
    // A comment before the cast `)` (the only thing that can precede it) breaks the hug too
    let hugs = is_object_or_array_expression
        && !f.comments().has_comment_before(span.start)
        && f.source_text().next_non_whitespace_byte_is(span.end, b')');
    if hugs {
        write!(f, group(&format_args!("(", &format_node, ")")));
    } else {
        write!(f, group(&format_args!("(", soft_block_indent(&format_node), ")")));
    }

    true
}

/// Prints a suppressed node that is a cast target keeping its source cast parentheses;
/// returns `false` when it is not one
/// (the caller, `write_suppressed_expression`, prints its plain verbatim range instead).
/// Must run before anything of the node is printed, with every comment still unprinted.
///
/// A cast target's span excludes its own wrapping parentheses (`preserve_parens: false`),
/// so a plain verbatim range would print `/** @type {A} */ x`, silently breaking the cast.
/// Leading comments print only up to the cast comment (`pending` ends with it, see [`TypeCast::Target`]);
/// the in-paren comments (`(/* c */ x)`) print in place via the verbatim text, which also marks them.
/// An empty `pending` (the cast comment printed by an ancestor) cannot reach here suppressed,
/// and falls through regardless.
pub fn write_suppressed_cast_target(span: Span, f: &mut JsFormatter<'_, '_>) -> bool {
    if let Some((pending, verbatim_span)) = cast_target_parens(span, f)
        && !pending.is_empty()
    {
        write!(f, [FormatLeadingComments::Comments(pending)]);
        FormatSuppressedNode(verbatim_span).fmt(f);
        true
    } else {
        false
    }
}

/// The verbatim range for a suppressed cast target:
/// from the first `(` after the cast comment to after the matching `)`.
///
/// The gap is already classified as parens and trivia ([`CastCommentGap::ParensAndTrivia`]);
/// comments are skipped by span ([`gap_segments`]), so `(` `)` bytes inside them don't count.
/// The parser guarantees each counted `(` wraps the node and closes after it (see [`CastCommentGap`]).
#[expect(clippy::cast_possible_truncation)] // Offsets fit in `u32`, source length does
fn cast_parens_span(cast_comment_end: u32, span: Span, f: &JsFormatter<'_, '_>) -> Option<Span> {
    let comments = f.context().comments();
    let source = f.source_text();
    let source_end = source.as_str().len() as u32;

    // Find the first `(` between the cast comment and the node and count them
    let mut open_parens = 0usize;
    let mut first_open_paren = None;
    let backward_comments = comments.comments_in_range(cast_comment_end, span.start);
    for (from, to) in gap_segments(backward_comments, cast_comment_end, span.start) {
        for (offset, &byte) in source.bytes_range(from, to).iter().enumerate() {
            if byte == b'(' {
                open_parens += 1;
                first_open_paren.get_or_insert(from + offset as u32);
            }
        }
    }
    let start = first_open_paren?;

    // Consume the matching `)`s after the node (only trivia sits between them);
    // `)` bytes inside comments don't count.
    for (from, to) in gap_segments(comments.comments_after(span.end), span.end, source_end) {
        for (offset, &byte) in source.bytes_range(from, to).iter().enumerate() {
            if byte == b')' {
                open_parens -= 1;
                if open_parens == 0 {
                    return Some(Span::new(start, from + offset as u32 + 1));
                }
            }
        }
    }
    None
}

/// Byte segments between `start` and `bound` lying outside the given comment spans:
/// one `(gap_start, gap_end)` pair per gap, ending with the tail segment up to `bound`.
fn gap_segments<'c>(
    comments: impl IntoIterator<Item = &'c Comment> + 'c,
    start: u32,
    bound: u32,
) -> impl Iterator<Item = (u32, u32)> + 'c {
    let mut pos = start;
    comments
        .into_iter()
        .map(|comment| comment.span)
        .chain(std::iter::once(Span::empty(bound)))
        .filter_map(move |span| {
            // A comment ending exactly at `pos` lies before the range
            if span.start < pos {
                return None;
            }
            let segment = (pos, span.start);
            pos = span.end;
            Some(segment)
        })
}

/// Prints a node's leading comments and the formatter-added `(` in the correct order.
/// The caller prints the matching `)` when `needs_parentheses` is true.
///
/// `leading_comments_start` bounds the leading-comments query
/// (see `FormatWrite::leading_comments_start`; usually `span.start`).
/// Type-cast classification always uses the node's real `span`.
///
/// When the leading comments end with a cast comment binding into the node (see [`TypeCast::BindsInner`]),
/// printing them all first would insert the added `(` between the comment and its cast target,
/// rebinding the cast and changing the type semantics:
/// ```js
/// x ? y : /** @type {D} */ (a).b ?? c
/// ```
/// must become:
/// ```js
/// x ? y : (/** @type {D} */ (a).b ?? c)
/// ```
/// Not
/// ```js
/// x ? y : /** @type {D} */ ((a).b ?? c)
/// ```
/// So the cast comment is printed inside the added parenthesis.
pub fn format_leading_comments_and_open_paren(
    span: Span,
    leading_comments_start: u32,
    needs_parentheses: bool,
    f: &mut JsFormatter<'_, '_>,
) {
    let leading_span = Span::new(leading_comments_start, span.end);
    if needs_parentheses {
        if let TypeCast::BindsInner(comments) = classify_type_cast(span, f)
            && let Some((cast_comment, rest)) = comments.split_last()
        {
            // Only the cast comment moves inside; earlier comments stay outside.
            write!(
                f,
                [
                    FormatLeadingComments::Comments(rest),
                    "(",
                    FormatLeadingComments::Comments(std::slice::from_ref(cast_comment))
                ]
            );
        } else {
            write!(f, [format_leading_comments(leading_span), "("]);
        }
    } else {
        write!(f, format_leading_comments(leading_span));
    }
}

/// Variant of [`format_leading_comments_and_open_paren`] for nodes
/// that print their own leading comments in `write` (the generator's `AST_NODE_WITHOUT_PRINTING_LEADING_COMMENTS_LIST`):
/// prints the leading comments that belong OUTSIDE the formatter-added `(`,
/// and leaves the rest pending for the node's own leading-comments pass.
///
/// Outside are:
/// - inline comments that preceded the node's source `(`
///   inline comments keep their source side (`X & /* outside */ (/* inside */ A | B)`)
/// - own-line comments, even from inside the source parentheses
///   they stay own-line ABOVE the paren, or the paren line would end in a line comment and force a layout the source never had
///   ```ts
///   type T = X & (
///     // c
///     A | B
///   );
///   // ->
///   type T = X &
///     // c
///     (A | B);
///   ```
///
/// The source `(` is precedence-required wherever such a node needs formatter parentheses,
/// so the split at the last `(`-bearing gap between pending comments reproduces each inline comment's source side.
/// When no gap contains a `(`, the paren precedes every pending comment.
/// Comment bodies are never scanned (a `(` inside a comment cannot false-positive: gaps run between comment bounds).
///
/// NOTE: Prettier normalizes `keyof /* c */ (A | B)` to `keyof (/* c */ A | B)`
/// while keeping the same comment outside in array/indexed-access positions;
/// one source-side rule instead of emulating that inconsistency (Known divergence, layout-only).
///
/// NOTE: The own-line hoist can detach a next-line directive (`@ts-expect-error`, ...) written inside the parens:
/// when the node expands, the added `(` takes the directive's former target line.
/// Matches Prettier; keeping such comments inside would need a `(`-then-break block rendering that does not exist today.
///
/// No type-cast handling ([`TypeCast::BindsInner`]): casts target expressions, and this path currently serves only `TSUnionType`.
pub fn format_outer_leading_comments_and_open_paren(
    span: Span,
    needs_parentheses: bool,
    f: &mut JsFormatter<'_, '_>,
) {
    if !needs_parentheses {
        return;
    }
    let leading = f.context().comments().comments_before(span.start);
    let mut outside_len = 0;
    for (index, comment) in leading.iter().enumerate() {
        let gap_end = leading.get(index + 1).map_or(span.start, |next| next.span.start);
        if f.source_text().bytes_contain(comment.span.end, gap_end, b'(') {
            outside_len = index + 1;
        }
    }
    while leading.get(outside_len).is_some_and(|comment| comment.preceded_by_newline()) {
        outside_len += 1;
    }
    write!(f, [FormatLeadingComments::Comments(&leading[..outside_len]), "("]);
}

/// What the source between a cast comment and the node start contains.
///
/// Where the cast parenthesis closes is derived from this gap plus the span,
/// with no scanning of expression bytes
/// (where a lexical paren count would be confused by regex literals like `/\)/`).
///
/// NOTE: Two parser guarantees make this sound even under `preserve_parens: false`:
/// - a node's own wrapping parentheses are excluded from its span,
///   so a `(` in the gap opened before the node and cannot close inside it
///   (parentheses do not cross node boundaries), the node is the cast target;
/// - a leftmost descendant's parentheses are included in the span (`(a).b ?? c` spans from `(`),
///   so a pure-trivia gap means the cast parenthesis at the span start belongs to a descendant
///   and closes inside the node, the cast binds inner.
enum CastCommentGap {
    /// Only whitespace and comments: the node's extent starts at the cast parenthesis.
    Trivia,
    /// Only `(`s, whitespace, and comments: the parentheses wrap the node.
    ParensAndTrivia,
    /// Anything else: the cast comment does not belong to this node.
    Code,
}

/// Classifies the source between a cast comment end (`start`) and the node start (`end`).
/// Comments in the gap are skipped via their known spans (printed or not, the bytes are the same).
fn classify_cast_comment_gap(start: u32, end: u32, f: &JsFormatter<'_, '_>) -> CastCommentGap {
    let source = f.source_text();
    let comments = f.context().comments().all_comments_in_range(start, end);

    let mut has_paren = false;
    for (from, to) in gap_segments(comments, start, end) {
        let Some(segment_has_paren) = classify_gap_segment(source.bytes_range(from, to)) else {
            return CastCommentGap::Code;
        };
        has_paren |= segment_has_paren;
    }

    if has_paren { CastCommentGap::ParensAndTrivia } else { CastCommentGap::Trivia }
}

/// One comment-free segment of a cast gap: `Some(has_paren)` when it holds only whitespace and `(`s
/// (the only bytes grammatically possible between a cast comment and its target),
/// `None` on anything else, which conservatively ends the cast ([`CastCommentGap::Code`]).
fn classify_gap_segment(bytes: &[u8]) -> Option<bool> {
    let mut has_paren = false;
    for &byte in bytes {
        match byte {
            b'(' => has_paren = true,
            _ if byte.is_ascii_whitespace() => {}
            _ => return None,
        }
    }
    Some(has_paren)
}
