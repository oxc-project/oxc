use oxc_formatter_core::{
    Buffer,
    builders::{align, group},
    write,
};
use oxc_yaml_parser::ast::{Mapping, Sequence};

use crate::{
    comments::{
        flush_container_end_comments, flush_leading_comments, gap_upper_bound,
        is_suppressed_last_before, write_blank_preserving_break, write_suppressed_node,
        write_trailing_same_line_comment,
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
        write_trailing_same_line_comment(prev_end, &[], f);
        flush_container_end_comments(item_column, prev_end, next_start.unwrap_or(u32::MAX), f)
    };
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

pub fn write_sequence<'a>(sequence: &'a Sequence<'a>, f: &mut YamlFormatter<'_, 'a>) {
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
