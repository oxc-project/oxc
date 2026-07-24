use oxc_formatter_core::{
    Buffer,
    builders::{align, group},
    write,
};
use oxc_yaml_parser::ast::{Content, Mapping, Sequence};

use crate::{
    comments::{
        flush_container_end_comments, flush_leading_comments, gap_upper_bound,
        is_suppressed_last_before, pending_same_line_comment, write_blank_preserving_break,
        write_suppressed_node, write_trailing_same_line_comment,
    },
    print::{
        YamlFormatter,
        block::{item_gap_anchor, last_descendant_block_scalar},
        column_of, format_with, mapping_item, to_span, write_node,
    },
};

/// Emits the separator between two block-collection items,
/// preserving a blank line from the source (Prettier's `printNextEmptyLine`).
/// The gap is measured up to the next pending comment when it precedes the item,
/// so a blank line in front of a leading comment is still preserved.
fn write_item_separator(prev_end: u32, next_start: u32, f: &mut YamlFormatter<'_, '_>) {
    let upper_bound = gap_upper_bound(next_start, f);
    write_blank_preserving_break(prev_end, upper_bound, f);
}

/// How an item's tail interacts with the comments that follow it.
#[derive(Clone, Copy, PartialEq)]
enum ItemTail {
    /// No block scalar: same-line trailing comment, then container end comments.
    Plain,
    /// The item ends in a block scalar at SOME depth, whose span consumes the trailing line breaks:
    /// a following own-line comment would LOOK same-line,
    /// and absorbing it onto the header would change the parsed value, skip the same-line check, still claim end comments.
    /// Also every sequence-item block scalar, direct or not
    /// (Prettier's `shouldOwnEndComment` has no block-scalar exclusion for sequence items).
    EndsInBlockScalar,
    /// The item's VALUE is a block scalar (mapping only):
    /// it cannot own end comments either
    /// (Prettier's `shouldOwnEndComment` exclusion, an ancestor whose value is a collection still can);
    /// skip both, the comments fall through to the enclosing container or the next node's leading position.
    ValueIsBlockScalar,
}

/// Emits the previous item's same-line trailing comment and container end comments
/// (as far as its [`ItemTail`] allows), then the separator to the next item (when one follows).
///
/// `align_width` is forwarded to [`flush_container_end_comments`] (see its doc for the per-container value).
fn finish_previous_item(
    item_column: u32,
    align_width: u8,
    mut prev_end: u32,
    prev_tail: ItemTail,
    next_start: Option<u32>,
    f: &mut YamlFormatter<'_, '_>,
) {
    if prev_tail == ItemTail::Plain {
        write_trailing_same_line_comment(prev_end, &[], f);
    }
    if prev_tail != ItemTail::ValueIsBlockScalar {
        prev_end = flush_container_end_comments(
            item_column,
            align_width,
            prev_end,
            next_start.unwrap_or(u32::MAX),
            f,
        );
    }
    if let Some(next_start) = next_start {
        write_item_separator(prev_end, next_start, f);
    }
}

pub fn write_mapping<'a>(
    mapping: &'a Mapping<'a>,
    parent_tag_is_set: bool,
    f: &mut YamlFormatter<'_, 'a>,
) {
    let depth = f.context().collection_depth();
    depth.set(depth.get() + 1);
    let item_column = column_of(&f.context().source_text(), mapping.span.start);
    let align_width = f.options().indent_width.value();
    let mut prev_end: Option<u32> = None;
    let mut prev_tail = ItemTail::Plain;
    for item in &mapping.children {
        let start = item.span.start;
        if let Some(prev_end) = prev_end {
            finish_previous_item(item_column, align_width, prev_end, prev_tail, Some(start), f);
        }
        let value_node = item.value_content();
        let last_block = value_node.and_then(last_descendant_block_scalar);
        prev_end = Some(item_gap_anchor(last_block, item.span.end, f));
        prev_tail = match value_node.map(|node| &node.content) {
            Some(Content::BlockLiteral(_) | Content::BlockFolded(_)) => {
                ItemTail::ValueIsBlockScalar
            }
            _ if last_block.is_some() => ItemTail::EndsInBlockScalar,
            _ => ItemTail::Plain,
        };

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
        finish_previous_item(item_column, align_width, prev_end, prev_tail, None, f);
    }
    let depth = f.context().collection_depth();
    depth.set(depth.get() - 1);
}

pub fn write_sequence<'a>(sequence: &'a Sequence<'a>, f: &mut YamlFormatter<'_, 'a>) {
    // Sequence item content sits `- ` (2 columns) in, regardless of the tab width
    const SEQ_CONTENT_ALIGN: u8 = 2;

    let depth = f.context().collection_depth();
    depth.set(depth.get() + 1);

    let item_column = column_of(&f.context().source_text(), sequence.span.start);
    let mut prev_end: Option<u32> = None;
    let mut prev_tail = ItemTail::Plain;
    for item in &sequence.children {
        if let Some(prev_end) = prev_end {
            finish_previous_item(
                item_column,
                SEQ_CONTENT_ALIGN,
                prev_end,
                prev_tail,
                Some(item.span.start),
                f,
            );
        }
        let last_block = item.content.as_deref().and_then(last_descendant_block_scalar);
        prev_end = Some(item_gap_anchor(last_block, item.span.end, f));
        // Never `ValueIsBlockScalar` here: see its doc on [`ItemTail`]
        prev_tail =
            if last_block.is_some() { ItemTail::EndsInBlockScalar } else { ItemTail::Plain };

        if is_suppressed_last_before(f, item.span.start) {
            write_suppressed_node(to_span(item.span), f);
            continue;
        }
        flush_leading_comments(item.span.start, f);
        // The content-area space stays when something follows it:
        // the content, or a same-line comment
        // (whose own line-suffix space makes it `-  # c`, two spaces, Prettier's shape).
        // For a bare null item don't leave it at the line end.
        if item.content.is_some() || pending_same_line_comment(item.span.end, f).is_some() {
            write!(f, "- ");
        } else {
            write!(f, "-");
        }
        if let Some(node) = &item.content {
            write!(f, align(SEQ_CONTENT_ALIGN, &format_with(|f| write_node(node, f))));
        }
    }
    if let Some(prev_end) = prev_end {
        finish_previous_item(item_column, SEQ_CONTENT_ALIGN, prev_end, prev_tail, None, f);
    }
    let depth = f.context().collection_depth();
    depth.set(depth.get() - 1);
}
