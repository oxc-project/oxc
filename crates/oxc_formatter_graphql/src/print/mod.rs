use apollo_parser::{SyntaxKind, SyntaxNode, cst, cst::CstNode};
use oxc_formatter_core::{
    Buffer, Format, Formatter,
    builders::{FormatWith, empty_line, hard_line_break, if_group_fits_on_line, soft_line_break},
    write,
};
use oxc_span::Span;

use crate::{
    comments::{
        flush_leading_comments, flush_trailing_inside_comments, is_suppressed_before,
        write_dangling_comments, write_suppressed_node, write_trailing_same_line_comment,
    },
    context::GraphqlFormatContext,
};

pub mod common;
pub mod definition;
pub mod selection;
pub mod value;

pub type GraphqlFormatter<'buf, 'a> = Formatter<'buf, 'a, GraphqlFormatContext<'a>>;

/// `Format` impl for `&'static str` specialized to `GraphqlFormatContext`.
///
/// Hardcoded to `GraphqlFormatContext` rather than generic over `C` so the blanket
/// `&T where T: Format` doesn't overlap.
impl<'a> Format<'a, GraphqlFormatContext<'a>> for &'static str {
    #[inline]
    fn fmt(&self, f: &mut GraphqlFormatter<'_, 'a>) {
        write!(f, oxc_formatter_core::builders::token(self));
    }
}

/// Wraps a re-entrant GraphQL closure in a [`FormatWith`]. The closure's context is
/// pinned to [`GraphqlFormatContext`] so call sites don't have to annotate it.
#[inline]
pub const fn format_with<'a, T>(formatter: T) -> FormatWith<T>
where
    T: Fn(&mut GraphqlFormatter<'_, 'a>),
{
    FormatWith::new(formatter)
}

/// Whether `kind` is a trivia token. In GraphQL, commas are trivia too.
pub fn is_trivia(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::WHITESPACE | SyntaxKind::COMMENT | SyntaxKind::COMMA)
}

/// Start offset of the first significant (non-trivia) token within `node`.
///
/// apollo-parser attaches pending trivia to the node that is open when the next
/// significant token is consumed, so `node.text_range().start()` may point at a
/// comment that logically precedes the node. All layout decisions use significant
/// token positions instead.
pub fn sig_start(node: &SyntaxNode) -> u32 {
    let node_end = u32::from(node.text_range().end());
    let mut tok = node.first_token();
    while let Some(t) = tok {
        if u32::from(t.text_range().start()) >= node_end {
            break;
        }
        if !is_trivia(t.kind()) {
            return t.text_range().start().into();
        }
        tok = t.next_token();
    }
    node.text_range().start().into()
}

/// End offset of the last significant (non-trivia) token within `node`.
pub fn sig_end(node: &SyntaxNode) -> u32 {
    let node_start = u32::from(node.text_range().start());
    let mut tok = node.last_token();
    while let Some(t) = tok {
        if u32::from(t.text_range().end()) <= node_start {
            break;
        }
        if !is_trivia(t.kind()) {
            return t.text_range().end().into();
        }
        tok = t.prev_token();
    }
    node.text_range().end().into()
}

/// Significant-token span of `node`.
pub fn sig_span(node: &SyntaxNode) -> Span {
    Span::new(sig_start(node), sig_end(node))
}

/// Start offset of a closing delimiter: the token's start when present,
/// the container's significant end otherwise (error-resilient fallback).
pub fn closing_token_start(
    token: Option<apollo_parser::SyntaxToken>,
    container: &SyntaxNode,
) -> u32 {
    token.map_or_else(|| sig_end(container), |t| t.text_range().start().into())
}

/// Source slice of `node`'s significant span, carrying the arena lifetime.
pub fn node_text<'a>(f: &GraphqlFormatter<'_, 'a>, node: &SyntaxNode) -> &'a str {
    let span = sig_span(node);
    f.context().source_text().slice_range(span.start, span.end)
}

/// How consecutive sequence items are separated.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SeparatorKind {
    /// One item per line (definitions, selections, fields, enum values, ...).
    Hard,
    /// Prettier's `join([ifBreak("", ", "), softline], items)`
    /// (arguments, variable definitions, list values, object fields).
    CommaSoftline,
}

/// Core sequence engine: emits `items` with separators, source blank-line preservation
/// (Prettier's `printSequence`), leading-comment flushing, trailing same-line comments,
/// and `# prettier-ignore` suppression.
///
/// Returns the significant end of the last item (`None` for an empty sequence),
/// so callers can seed `flush_trailing_inside_comments` without re-scanning.
pub fn write_sequence<'a, T, F>(
    f: &mut GraphqlFormatter<'_, 'a>,
    items: &[T],
    separator: SeparatorKind,
    preserve_blank: bool,
    mut write_item: F,
) -> Option<u32>
where
    T: CstNode,
    F: FnMut(usize, &mut GraphqlFormatter<'_, 'a>),
{
    let mut prev_end: Option<u32> = None;
    for (i, item) in items.iter().enumerate() {
        let node = item.syntax();
        let start = sig_start(node);
        if let Some(pe) = prev_end {
            write_trailing_same_line_comment(pe, f);
            // Measure the gap up to the next pending comment (if it precedes the item),
            // so a blank line in front of a leading comment is still preserved.
            let anchor = f
                .context()
                .comments()
                .peek()
                .filter(|c| c.start < start)
                .map_or(start, |c| c.start);
            let is_blank = preserve_blank
                && crate::comments::classify_gap(f.context().source_text().bytes_range(pe, anchor))
                    == crate::comments::Gap::Blank;
            match separator {
                SeparatorKind::Hard => {
                    if is_blank {
                        write!(f, empty_line());
                    } else {
                        write!(f, hard_line_break());
                    }
                }
                SeparatorKind::CommaSoftline => {
                    // Mirrors Prettier's `[printed, hardline]` + `[ifBreak("", ", "), softline]`.
                    // The hard line forces the group to break, so the `ifBreak` comma never
                    // materializes; an `empty_line` reproduces the blank line directly.
                    if is_blank {
                        write!(f, empty_line());
                    } else {
                        write!(f, [if_group_fits_on_line(&", "), soft_line_break()]);
                    }
                }
            }
        }
        let end = sig_end(node);
        if is_suppressed_before(f, start) {
            write_suppressed_node(Span::new(start, end), f);
        } else {
            flush_leading_comments(start, f);
            write_item(i, f);
        }
        prev_end = Some(end);
    }
    prev_end
}

/// Top-level document: definitions joined by hard lines (blank lines preserved),
/// then any comments trailing the last definition.
pub fn write_document(document: &cst::Document, f: &mut GraphqlFormatter<'_, '_>) {
    let defs: Vec<cst::Definition> = document.definitions().collect();
    if defs.is_empty() {
        // Defensive: apollo-parser errors on empty/comments-only documents,
        // so this branch is unreachable on the normal path.
        let remaining = f.context().comments().take_remaining();
        write_dangling_comments(remaining, f);
        return;
    }

    let last_end = write_sequence(f, &defs, SeparatorKind::Hard, true, |i, f| {
        definition::write_definition(&defs[i], f);
    });
    if let Some(last_end) = last_end {
        flush_trailing_inside_comments(last_end, u32::MAX, f);
    }
}
