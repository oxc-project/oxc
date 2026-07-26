use oxc_formatter_core::{
    Buffer,
    builders::{empty_line, hard_line_break, space, text},
    write,
};
use oxc_yaml_parser::ast::{Directive, Document, Root};

use crate::{
    comments::{
        Gap, classify_gap, flush_leading_comments, gap_upper_bound, is_suppressed_last_before,
        write_blank_preserving_break, write_single_comment, write_suppressed_node,
        write_trailing_same_line_comment,
    },
    print::{
        YamlFormatter,
        block::{ends_with_keep_chomped_block, item_gap_anchor, last_descendant_block_scalar},
        to_span, write_node_or_suppressed,
    },
};

/// Top-level stream: documents joined by hard lines with `---`/`...` marker logic,
/// then any comments trailing the last document.
///
/// Returns whether the stream ends with a keep-chomped block scalar (computed here anyway),
/// so `format()` doesn't re-walk the tree for its final newline.
pub fn write_root<'a>(root: &'a Root<'a>, f: &mut YamlFormatter<'_, 'a>) -> bool {
    let keep_chomped_tail = ends_with_keep_chomped_block(root);
    let documents = root.children.as_slice();

    for (i, document) in documents.iter().enumerate() {
        if i > 0 {
            write_document_separator(&documents[i - 1], document, f);
        }
        write_document(document, f);

        let next = documents.get(i + 1);
        if should_print_document_end_marker(document, next) {
            if let Some(marker) = document.document_end_marker {
                // The body's end comments, printed between it and the `...`
                let anchor = document_end_comment_anchor(document, f);
                let comments = f.context().comments().take_before(marker.start);
                write_end_comments(Some(anchor), comments, f);
            }
            // After a keep-chomped block scalar the verbatim content already
            // ends with a newline; `...` starts a fresh line without one.
            if !(keep_chomped_tail && i + 1 == documents.len()) {
                write!(f, hard_line_break());
            }
            write!(f, "...");
            if let Some(marker) = document.document_end_marker {
                write_trailing_same_line_comment(marker.end, &[], f);
            }
        }
    }

    // Comments after the last node: the last document's end comments
    // (or comments trailing its explicit `...` marker).
    let anchor = documents.last().map(|document| document_end_comment_anchor(document, f));
    let remaining = f.context().comments().take_remaining();
    write_end_comments(anchor, remaining, f);

    keep_chomped_tail
}

/// The end-comment gap anchor for `document`, clamped past comments an inner container already consumed.
/// Must be taken BEFORE the end comments themselves are drained,
/// that take advances the consumed cursor past them, corrupting the clamp.
fn document_end_comment_anchor(document: &Document<'_>, f: &YamlFormatter<'_, '_>) -> u32 {
    f.context().comments().gap_anchor_after_consumed(document_gap_anchor(document, f))
}

/// The gap-measurement anchor after a document: its `...` marker,
/// or the end of its body (adjusted for a block scalar tail, whose span consumes the trailing line breaks).
fn document_gap_anchor(document: &Document<'_>, f: &YamlFormatter<'_, '_>) -> u32 {
    document.document_end_marker.map_or_else(
        || item_gap_anchor(document.body.content.as_deref(), document.body.span.end, f),
        |marker| marker.end,
    )
}

/// The separator between two documents.
///
/// NOTE: Whether Prettier keeps a blank line here depends on the previous body's node kind and on what follows (prettier#15528).
/// (`printNextEmptyLine` covers block collections only;
/// its `documentBody` end-comments separator covers block mappings, and unconditionally INSERTS one after a non-keep block scalar)
/// The consistent rule instead: a blank line in the source is preserved (normalized to one), never invented, for every node kind alike.
fn write_document_separator<'a>(
    prev: &'a Document<'a>,
    next: &'a Document<'a>,
    f: &mut YamlFormatter<'_, 'a>,
) {
    let anchor = document_gap_anchor(prev, f);
    let upper_bound = gap_upper_bound(next.span.start, f);
    write_blank_preserving_break(anchor, upper_bound, f);
}

/// Writes a document body's end comments, one per line, preserving the source's blank lines (normalized to one).
/// In front of the first comment (measured from `anchor`) and between consecutive comments alike.
/// See [`write_document_separator`] for the deliberate non-follow.
fn write_end_comments(
    anchor: Option<u32>,
    comments: &[oxc_span::Span],
    f: &mut YamlFormatter<'_, '_>,
) {
    let source = f.context().source_text();
    for (i, &span) in comments.iter().enumerate() {
        let prev_end = if i == 0 { anchor } else { Some(comments[i - 1].end) };
        let blank = prev_end.is_some_and(|prev_end| {
            prev_end < span.start
                && classify_gap(source.bytes_range(prev_end, span.start)) == Gap::Blank
        });
        if blank {
            write!(f, empty_line());
        } else {
            write!(f, hard_line_break());
        }
        write_single_comment(span, f);
    }
}

/// Prettier's `shouldPrintDocumentEndMarker`:
/// an explicit `...`, or a following document whose head has directives or leading comments
/// (which would otherwise be ambiguous without the marker).
fn should_print_document_end_marker(document: &Document<'_>, next: Option<&Document<'_>>) -> bool {
    if document.document_end_marker.is_some() {
        return true;
    }
    // NOTE: Prettier also checks `hasEndComments(nextDocument.head)`,
    // but a comment only lands in the next head when THIS document has a `...` marker (already handled above);
    // without one it belongs to this document's body, so no `...` is introduced for it.
    next.is_some_and(|next| !next.head.directives.is_empty())
}

fn write_document<'a>(document: &'a Document<'a>, f: &mut YamlFormatter<'_, 'a>) {
    let mut needs_line = false;

    for directive in &document.head.directives {
        if needs_line {
            write!(f, hard_line_break());
        }
        flush_leading_comments(directive.span.start, f);
        write_directive(directive, f);
        write_trailing_same_line_comment(directive.span.end, &[], f);
        needs_line = true;
    }

    // `# prettier-ignore` as the head's LAST end comment suppresses the whole
    // document body (Prettier's `hasPrettierIgnore` for `documentBody`).
    let mut head_ignores_body = false;
    if let Some(marker) = document.directives_end_marker {
        if needs_line {
            write!(f, hard_line_break());
        }
        head_ignores_body = is_suppressed_last_before(f, marker.start);
        // Comments before `---` (the head's end comments)
        flush_leading_comments(marker.start, f);
        write!(f, "---");
        write_trailing_same_line_comment(marker.end, &[], f);
        needs_line = true;
    }

    if let Some(node) = &document.body.content {
        if needs_line {
            write!(f, hard_line_break());
        }
        // Whole-body suppression needs the explicit head form (`# oxfmt-ignore` above `---`);
        // an ignore right above the body is `write_node_or_suppressed`'s per-node rule.
        let suppressed = if head_ignores_body {
            write_suppressed_node(to_span(node.span), f);
            true
        } else {
            write_node_or_suppressed(node, f)
        };
        // A block scalar's span consumes its trailing breaks;
        // a comment on the next line would look same-line, but none can follow one.
        if !suppressed && last_descendant_block_scalar(node).is_none() {
            write_trailing_same_line_comment(node.span.end, &[], f);
        }
    }
}

/// Re-joins directive words with single spaces (whitespace runs between them are normalized).
fn write_directive<'a>(directive: &'a Directive<'a>, f: &mut YamlFormatter<'_, 'a>) {
    write!(f, "%");
    write!(f, text(directive.name));
    for parameter in &directive.parameters {
        write!(f, space());
        write!(f, text(parameter));
    }
}
