use oxc_formatter_core::{
    Buffer, Format, Formatter,
    builders::{FormatWith, align, empty_line, group, hard_line_break, space, text, token},
    write,
};
use oxc_yaml_parser::ast::{
    Chomping, Content, Directive, Document, Mapping, MappingItem, Node, Props, Root, Sequence,
};

use crate::{
    comments::{
        Gap, classify_gap, flush_leading_comments, gap_upper_bound, is_suppressed_last_before,
        suppression_flush_bound, write_single_comment, write_suppressed_node,
        write_trailing_same_line_comment,
    },
    context::YamlFormatContext,
};

mod block;
mod flow;
mod mapping_item;
mod scalar;

pub type YamlFormatter<'buf, 'a> = Formatter<'buf, 'a, YamlFormatContext<'a>>;

/// Bridges the parser's byte-offset span to `oxc_span::Span`.
pub fn to_span(span: oxc_yaml_parser::Span) -> oxc_span::Span {
    oxc_span::Span::new(span.start, span.end)
}

/// `Format` impl for `&'static str` specialized to `YamlFormatContext`.
///
/// Hardcoded to `YamlFormatContext` rather than generic over `C` so the blanket
/// `&T where T: Format` doesn't overlap.
impl<'a> Format<'a, YamlFormatContext<'a>> for &'static str {
    #[inline]
    fn fmt(&self, f: &mut YamlFormatter<'_, 'a>) {
        write!(f, token(self));
    }
}

/// Wraps a re-entrant YAML closure in a [`FormatWith`].
/// The closure's context is pinned to [`YamlFormatContext`] so call sites don't have to annotate it.
#[inline]
pub const fn format_with<'a, T>(formatter: T) -> FormatWith<T>
where
    T: Fn(&mut YamlFormatter<'_, 'a>),
{
    FormatWith::new(formatter)
}

/// 0-based column of `offset` in `source` (bytes since the preceding newline).
pub fn column_of(source: &str, offset: u32) -> u32 {
    let count =
        source.as_bytes()[..offset as usize].iter().rev().take_while(|&&b| b != b'\n').count();
    u32::try_from(count).unwrap_or(u32::MAX)
}

/// A run of `count` newlines with the arena lifetime.
/// Small runs (the overwhelming case: blank-line counts) slice a static string
/// instead of building a `String` and copying it into the arena.
pub fn arena_newlines<'a>(count: usize, f: &YamlFormatter<'_, 'a>) -> &'a str {
    const NEWLINES: &str = "\n\n\n\n\n\n\n\n";
    if count <= NEWLINES.len() {
        &NEWLINES[..count]
    } else {
        f.allocator().alloc_str(&"\n".repeat(count))
    }
}

/// `true` when the source between `from` and `to` holds nothing but
/// whitespace and comments (every line blank or `#`-only after indentation).
fn gap_is_trivia_only(source: &str, from: u32, to: u32) -> bool {
    source[from as usize..to as usize].lines().all(|line| {
        let trimmed = line.trim_start();
        trimmed.is_empty() || trimmed.starts_with('#')
    })
}

/// `true` when only whitespace precedes `offset` on its line
/// (an own-line comment, as opposed to one trailing other content).
pub fn is_own_line(source: &str, offset: u32) -> bool {
    own_line_column(source, offset).is_some()
}

/// The 0-based column of `offset` when only whitespace precedes it on its
/// line, in a single backward scan; `None` when other content does.
fn own_line_column(source: &str, offset: u32) -> Option<u32> {
    let mut column = 0u32;
    for &byte in source.as_bytes()[..offset as usize].iter().rev() {
        match byte {
            b'\n' => break,
            b' ' | b'\t' => column += 1,
            _ => return None,
        }
    }
    Some(column)
}

/// Returns `true` when the stream's last descendant is a keep-chomped (`+`) block scalar.
/// Its verbatim content already ends with the kept newlines,
/// so the caller must not append the usual final `hard_line_break()`
/// (mirrors Prettier's `shouldPrintHardline` gate).
pub fn ends_with_keep_chomped_block(root: &Root<'_>) -> bool {
    root.children
        .last()
        .and_then(|document| document.body.content.as_deref())
        .and_then(last_descendant_block_scalar)
        .is_some_and(|block| block.chomping == Chomping::Keep)
}

/// The block scalar the node's last descendant resolves to, if any.
/// Its span consumes the trailing line breaks,
/// so "same line" checks against the content end are meaningless after one.
fn last_descendant_block_scalar<'b>(
    node: &'b Node<'_>,
) -> Option<&'b oxc_yaml_parser::ast::BlockScalar> {
    match &node.content {
        Content::BlockLiteral(block) | Content::BlockFolded(block) => Some(block),
        Content::Mapping(mapping) => mapping
            .children
            .last()
            .and_then(MappingItem::value_content)
            .and_then(last_descendant_block_scalar),
        Content::Sequence(sequence) => sequence
            .children
            .last()
            .and_then(|item| item.content.as_deref())
            .and_then(last_descendant_block_scalar),
        // Block scalars cannot appear inside flow collections.
        _ => None,
    }
}

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
                let anchor = document_gap_anchor(document, f);
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
                write_trailing_same_line_comment(marker.end, f);
            }
        }
    }

    // Comments after the last node: the last document's end comments
    // (or comments trailing its explicit `...` marker).
    let anchor = documents.last().map(|document| document_gap_anchor(document, f));
    let remaining = f.context().comments().take_remaining();
    write_end_comments(anchor, remaining, f);

    keep_chomped_tail
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

/// One line break, widened to a blank line when the source gap holds one.
fn write_blank_preserving_break(prev_end: u32, upper_bound: u32, f: &mut YamlFormatter<'_, '_>) {
    if prev_end < upper_bound
        && classify_gap(f.context().source_text().bytes_range(prev_end, upper_bound)) == Gap::Blank
    {
        write!(f, empty_line());
    } else {
        write!(f, hard_line_break());
    }
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
        write_trailing_same_line_comment(directive.span.end, f);
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
        write_trailing_same_line_comment(marker.end, f);
        needs_line = true;
    }

    if let Some(node) = &document.body.content {
        if needs_line {
            write!(f, hard_line_break());
        }
        let start = node.span.start;
        // A block collection starts at the same offset as its first item;
        // the suppression marker is left for the item loop, so only the FIRST ITEM is frozen.
        // An ignore right above the first key must not silently freeze the whole document.
        // Whole-body suppression needs the explicit head form (`# oxfmt-ignore` above `---`).
        let body_is_block_collection =
            matches!(node.content, Content::Mapping(_) | Content::Sequence(_));
        if head_ignores_body || (!body_is_block_collection && is_suppressed_last_before(f, start)) {
            write_suppressed_node(to_span(node.span), f);
        } else {
            flush_leading_comments(suppression_flush_bound(body_is_block_collection, start, f), f);
            write_node(node, f);
            // A block scalar's span consumes its trailing breaks;
            // a comment on the next line would look same-line, but none can follow one.
            if last_descendant_block_scalar(node).is_none() {
                write_trailing_same_line_comment(node.span.end, f);
            }
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

/// Emits a node's properties (anchor/tag) in their source order.
/// NOTE: Prettier 3.9.5 normalizes to `tag anchor` order; we keep the source order instead.
///
/// The separator after the props is a hard line for block collections and a space otherwise.
fn write_props(props: &Props, is_block_collection: bool, f: &mut YamlFormatter<'_, '_>) {
    let source = f.context().source_text();
    let mut spans = [props.anchor.map(|a| a.span), props.tag.map(|t| t.span)];
    if spans.iter().all(Option::is_none) {
        return;
    }
    spans.sort_by_key(|span| span.map_or(u32::MAX, |s| s.start));
    let mut first = true;
    for span in spans.into_iter().flatten() {
        if !first {
            write!(f, space());
        }
        first = false;
        write!(f, text(source.text_for(&to_span(span))));
    }
    if is_block_collection {
        write!(f, hard_line_break());
    } else {
        write!(f, space());
    }
}

pub fn write_node<'a>(node: &'a Node<'a>, f: &mut YamlFormatter<'_, 'a>) {
    let content = &node.content;
    let content_start = content.span().start;
    let is_block_collection = matches!(content, Content::Mapping(_) | Content::Sequence(_));
    // Middle comments (between the props and the node body, `&a # c`)
    // keep the props separator a space even for block collections.
    let has_middle_comments = f.context().comments().peek().is_some_and(|c| c.end <= content_start);
    write_props(&node.props, is_block_collection && !has_middle_comments, f);

    // Prettier: `[middles.len() == 1 ? "" : hardline, join(hardline, middles), hardline]`.
    // A block collection's trailing suppression marker is NOT a middle comment:
    // it stays pending so the first item's check can claim it.
    let middles_bound = suppression_flush_bound(is_block_collection, content_start, f);
    let middles = f.context().comments().take_before(middles_bound);
    // `i > 0` is subsumed: any later iteration implies more than one middle.
    let multiple_middles = middles.len() > 1;
    for &span in middles {
        if multiple_middles {
            write!(f, hard_line_break());
        }
        write_single_comment(span, f);
    }
    if !middles.is_empty() {
        write!(f, hard_line_break());
    }

    match content {
        Content::Plain(scalar) => scalar::write_plain(to_span(scalar.span), f),
        Content::QuoteSingle(scalar) => {
            scalar::write_quoted(scalar::FlowScalarKind::QuoteSingle, to_span(scalar.span), f);
        }
        Content::QuoteDouble(scalar) => {
            scalar::write_quoted(scalar::FlowScalarKind::QuoteDouble, to_span(scalar.span), f);
        }
        Content::Alias(alias) => write_raw_span(to_span(alias.span), f),
        Content::BlockLiteral(block) => block::write_block_scalar(block, false, f),
        Content::BlockFolded(block) => block::write_block_scalar(block, true, f),
        Content::Mapping(mapping) => {
            // `!!set` on the mapping changes its items' empty-value layout
            let parent_tag_is_set = node.props.tag.is_some_and(|tag| {
                let raw = f.context().source_text().text_for(&to_span(tag.span));
                raw == "!!set" || raw == "!<tag:yaml.org,2002:set>"
            });
            write_mapping(mapping, parent_tag_is_set, f);
        }
        Content::Sequence(sequence) => write_sequence(sequence, f),
        Content::FlowMapping(flow) => {
            write_flow_collection(flow.span.end, |f| flow::write_flow_mapping(flow, f), f);
        }
        Content::FlowSequence(flow) => {
            write_flow_collection(flow.span.end, |f| flow::write_flow_sequence(flow, f), f);
        }
    }
}

/// Wraps a flow collection in its group,
/// then claims any comments the flow printer's bounded flushes missed so they aren't re-emitted later.
fn write_flow_collection<'a>(
    end: u32,
    inner: impl Fn(&mut YamlFormatter<'_, 'a>),
    f: &mut YamlFormatter<'_, 'a>,
) {
    write!(f, group(&format_with(inner)));
    let _ = f.context().comments().take_before(end);
}

/// v0: verbatim slice of the (newline-normalized) source.
/// Claims any comments inside the span (e.g. inside a verbatim flow collection)
/// so they aren't re-emitted at a later flush point.
fn write_raw_span(span: oxc_span::Span, f: &mut YamlFormatter<'_, '_>) {
    let raw = f.context().source_text().text_for(&span);
    write!(f, text(raw));
    let _ = f.context().comments().take_before(span.end);
}

/// Walks to the end offset of the stream's last descendant node
/// (Prettier's `getLastDescendantNode` on root, used by block scalars).
pub fn last_descendant_end(root: &Root<'_>) -> u32 {
    // Spans nest and every wrapper ends at its last descendant
    // (the parser's `container_span` / `MappingItem` / `Node` span construction),
    // so the last document body's span end IS the last descendant's end.
    root.children
        .last()
        .and_then(|document| document.body.content.as_deref())
        .map_or(0, |node| node.span.end)
}

/// The gap-measurement anchor after an item:
/// a block scalar's span consumes its trailing line breaks (they are part of its VALUE under keep chomping),
/// so blank-line detection must start after the last content character,
/// otherwise the blank line separating it from the next item is invisible.
fn item_gap_anchor(content: Option<&Node<'_>>, end: u32, f: &YamlFormatter<'_, '_>) -> u32 {
    let Some(block) = content.and_then(last_descendant_block_scalar) else {
        return end;
    };
    // Under keep chomping every trailing newline IS the value; the span end is the correct anchor
    if block.chomping == Chomping::Keep {
        return end;
    }
    // The trailing break run (`content_end..span.end`, line breaks plus blank-line indentation).
    // Only the newlines the scalar's own output does NOT consume are the inter-item gap.
    let tail = f.context().source_text().bytes_range(block.content_end, block.span.end);
    // A handful of bytes; not worth a bytecount dependency
    #[expect(clippy::naive_bytecount)]
    let total_newlines = tail.iter().filter(|&&b| b == b'\n').count();
    let is_last_descendant = block.span.end >= f.context().last_descendant_end();
    let kept = block::consumed_trailing_newlines(total_newlines, is_last_descendant);
    // Anchor right after the `kept`-th newline of the tail
    // (the tail is span-bounded, so the saturation never fires).
    tail.iter()
        .enumerate()
        .filter(|&(_, &byte)| byte == b'\n')
        .take(kept)
        .last()
        .map_or(block.content_end, |(i, _)| {
            block.content_end + u32::try_from(i + 1).unwrap_or(u32::MAX)
        })
}

/// Emits the separator between two block-collection items,
/// preserving a blank line from the source (Prettier's `printNextEmptyLine`).
/// The gap is measured up to the next pending comment when it precedes the item,
/// so a blank line in front of a leading comment is still preserved.
fn write_item_separator(prev_end: u32, next_start: u32, f: &mut YamlFormatter<'_, '_>) {
    let upper_bound = gap_upper_bound(next_start, f);
    write_blank_preserving_break(prev_end, upper_bound, f);
}

/// Claims pending comments indented strictly deeper than `item_column` as the preceding item's end comments, printed one indent level in
/// (the placement effect of Prettier's `shouldOwnEndComment` + `mappingValue.endComments`, re-derived positionally).
/// Returns the position after the last claimed comment so the caller can keep measuring gaps from it.
fn flush_container_end_comments(
    item_column: u32,
    prev_end: u32,
    upper_bound: u32,
    f: &mut YamlFormatter<'_, '_>,
) -> u32 {
    let source = f.context().source_text();
    let tab_width = f.options().indent_width.value();
    let mut prev_end = prev_end;
    loop {
        let Some(span) = f.context().comments().peek() else { return prev_end };
        if span.end > upper_bound
            || own_line_column(&source, span.start).is_none_or(|column| column <= item_column)
            // An end-comment run directly follows its container;
            // other tokens in between mean the comment belongs to a LATER node
            // (a nested collection's unbounded tail flush must not jump over the parent's following items).
            || !gap_is_trivia_only(&source, prev_end, span.start)
        {
            return prev_end;
        }
        f.context().comments().take_before(span.end);
        let is_blank = classify_gap(source.bytes_range(prev_end, span.start)) == Gap::Blank;
        // The line break lives INSIDE `align` so the comment line is indented
        let inner = format_with(move |f: &mut YamlFormatter<'_, '_>| {
            if is_blank {
                write!(f, empty_line());
            } else {
                write!(f, hard_line_break());
            }
            write_single_comment(span, f);
        });
        write!(f, align(tab_width, &inner));
        prev_end = span.end;
    }
}

/// Emits the previous item's same-line trailing comment and container end comments,
/// then the separator to the next item (when one follows).
///
/// Both are skipped after a block scalar value:
/// its span consumes the trailing line breaks (a comment on the following line would LOOK same-line, and none can follow one),
/// and comments under its content region lead the next item instead (Prettier's `shouldOwnEndComment` exclusion).
fn finish_previous_item(
    item_column: u32,
    prev_end: u32,
    prev_blocks_end_comments: bool,
    next_start: Option<u32>,
    f: &mut YamlFormatter<'_, '_>,
) {
    let prev_end = if prev_blocks_end_comments {
        prev_end
    } else {
        write_trailing_same_line_comment(prev_end, f);
        flush_container_end_comments(item_column, prev_end, next_start.unwrap_or(u32::MAX), f)
    };
    if let Some(next_start) = next_start {
        write_item_separator(prev_end, next_start, f);
    }
}

fn write_mapping<'a>(
    mapping: &'a Mapping<'a>,
    parent_tag_is_set: bool,
    f: &mut YamlFormatter<'_, 'a>,
) {
    let depth = f.context().collection_depth();
    depth.set(depth.get() + 1);
    let item_column = column_of(&f.context().source_text(), mapping.span.start);
    let mut prev_end: Option<u32> = None;
    let mut prev_blocks_end_comments = false;
    for item in &mapping.children {
        let start = item.span.start;
        if let Some(prev_end) = prev_end {
            finish_previous_item(item_column, prev_end, prev_blocks_end_comments, Some(start), f);
        }
        let value_node = item.value_content();
        prev_end = Some(item_gap_anchor(value_node, item.span.end, f));
        // Descendant-aware, like `item_gap_anchor`:
        // a block scalar ending the item at ANY depth consumes the trailing line breaks,
        // so a following own-line comment would otherwise classify as same-line
        // and be absorbed into the scalar's content (changing the parsed value).
        prev_blocks_end_comments = value_node.and_then(last_descendant_block_scalar).is_some();

        if is_suppressed_last_before(f, start) {
            write_suppressed_node(to_span(item.span), f);
            continue;
        }
        flush_leading_comments(start, f);
        let entry = format_with(|f| {
            mapping_item::write_mapping_item(
                item,
                parent_tag_is_set,
                mapping_item::FlowParent::No,
                f,
            );
        });
        write!(f, group(&entry));
    }
    if let Some(prev_end) = prev_end {
        finish_previous_item(item_column, prev_end, prev_blocks_end_comments, None, f);
    }
    let depth = f.context().collection_depth();
    depth.set(depth.get() - 1);
}

fn write_sequence<'a>(sequence: &'a Sequence<'a>, f: &mut YamlFormatter<'_, 'a>) {
    let depth = f.context().collection_depth();
    depth.set(depth.get() + 1);
    let item_column = column_of(&f.context().source_text(), sequence.span.start);
    let mut prev_end: Option<u32> = None;
    let mut prev_blocks_end_comments = false;
    for item in &sequence.children {
        if let Some(prev_end) = prev_end {
            finish_previous_item(
                item_column,
                prev_end,
                prev_blocks_end_comments,
                Some(item.span.start),
                f,
            );
        }
        prev_end = Some(item_gap_anchor(item.content.as_deref(), item.span.end, f));
        // Descendant-aware for the same reason as in `write_mapping`
        prev_blocks_end_comments =
            item.content.as_deref().and_then(last_descendant_block_scalar).is_some();

        if is_suppressed_last_before(f, item.span.start) {
            write_suppressed_node(to_span(item.span), f);
            continue;
        }
        flush_leading_comments(item.span.start, f);
        write!(f, "- ");
        if let Some(node) = &item.content {
            write!(f, align(2, &format_with(|f| write_node(node, f))));
        }
    }
    if let Some(prev_end) = prev_end {
        finish_previous_item(item_column, prev_end, prev_blocks_end_comments, None, f);
    }
    let depth = f.context().collection_depth();
    depth.set(depth.get() - 1);
}
