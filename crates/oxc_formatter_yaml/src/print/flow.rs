use oxc_formatter_core::{
    Buffer,
    builders::{
        align, group, hard_line_break, if_group_breaks, soft_line_break, soft_line_break_or_space,
        text,
    },
    write,
};
use oxc_span::Span;
use oxc_yaml_parser::ast::{FlowMapping, FlowSequence, FlowSequenceEntry};

use crate::{
    comments::{
        Gap, classify_gap, flush_leading_comments, gap_upper_bound, is_suppressed_last_before,
        write_single_comment, write_suppressed_node, write_trailing_same_line_comment,
    },
    print::{YamlFormatter, format_with, mapping_item, write_node},
};

/// Ports Prettier's `printFlowMapping` / `printFlowSequence`.
/// The caller wraps the result in a group.
pub fn write_flow_mapping<'a>(flow: &'a FlowMapping<'a>, f: &mut YamlFormatter<'_, 'a>) {
    let is_empty_last_item = flow
        .children
        .last()
        .is_some_and(|item| item.key_content().is_none() && item.value_content().is_none());
    // `{ a: b }` gets inner spaces (sequences never do)
    let spaced = !flow.children.is_empty() && f.options().bracket_spacing.value();

    write!(f, "{");
    let inner = format_with(|f: &mut YamlFormatter<'_, 'a>| {
        write_bracket_spacing(spaced, f);
        let count = flow.children.len();
        for (i, item) in flow.children.iter().enumerate() {
            let start = item.span.start;
            if is_suppressed_last_before(f, start) {
                write_suppressed_node(Span::new(start, item.span.end), f);
            } else {
                flush_leading_comments(start, f);
                let entry = format_with(|f| {
                    mapping_item::write_mapping_item(
                        item,
                        false,
                        mapping_item::FlowParent::Mapping,
                        f,
                    );
                });
                write!(f, group(&entry));
            }
            if i + 1 < count {
                write!(f, ",");
            }
            write_trailing_same_line_comment(item.span.end, f);
            if i + 1 < count {
                write_entry_separator(item.span.end, flow.children[i + 1].span.start, f);
            }
        }
        write_trailing_comma_and_end_comments(flow.span.end, f);
    });
    write!(f, align(f.options().indent_width.value(), &inner));
    if !is_empty_last_item {
        write_bracket_spacing(spaced, f);
    }
    write!(f, "}");
}

pub fn write_flow_sequence<'a>(flow: &'a FlowSequence<'a>, f: &mut YamlFormatter<'_, 'a>) {
    write!(f, "[");
    let inner = format_with(|f: &mut YamlFormatter<'_, 'a>| {
        write!(f, soft_line_break());
        let count = flow.children.len();
        for (i, entry) in flow.children.iter().enumerate() {
            let (start, end) = flow_entry_bounds(entry);
            if is_suppressed_last_before(f, start) {
                write_suppressed_node(Span::new(start, end), f);
            } else {
                flush_leading_comments(start, f);
                let entry_content = format_with(|f: &mut YamlFormatter<'_, 'a>| match entry {
                    FlowSequenceEntry::Item(node) => write_node(node, f),
                    FlowSequenceEntry::Pair(item) => {
                        mapping_item::write_mapping_item(
                            item,
                            false,
                            mapping_item::FlowParent::Sequence,
                            f,
                        );
                    }
                });
                write!(f, group(&entry_content));
            }
            if i + 1 < count {
                write!(f, ",");
            }
            write_trailing_same_line_comment(end, f);
            if i + 1 < count {
                write_entry_separator(end, flow_entry_bounds(&flow.children[i + 1]).0, f);
            }
        }
        write_trailing_comma_and_end_comments(flow.span.end, f);
    });
    write!(f, align(f.options().indent_width.value(), &inner));
    write!(f, soft_line_break());
    write!(f, "]");
}

fn flow_entry_bounds(entry: &FlowSequenceEntry<'_>) -> (u32, u32) {
    match entry {
        FlowSequenceEntry::Item(node) => (node.span.start, node.span.end),
        FlowSequenceEntry::Pair(item) => (item.span.start, item.span.end),
    }
}

fn write_bracket_spacing(spaced: bool, f: &mut YamlFormatter<'_, '_>) {
    if spaced {
        write!(f, soft_line_break_or_space());
    } else {
        write!(f, soft_line_break());
    }
}

/// `line` between entries, plus an extra newline when the source had a blank line
/// (Prettier's `printNextEmptyLine` inside flow children).
///
/// The blank line must appear only when the collection breaks and must not force it to break,
/// so it is a raw `\n` behind `if_group_breaks` with expand-propagation suppressed
/// (the following `line` re-arms indentation: after the raw newline the printer sees an empty line and emits none itself).
fn write_entry_separator(prev_end: u32, next_start: u32, f: &mut YamlFormatter<'_, '_>) {
    let anchor = gap_upper_bound(next_start, f);
    if anchor > prev_end {
        let gap = f.context().source_text().bytes_range(prev_end, anchor);
        if classify_gap(gap) == Gap::Blank {
            // The raw text supplies BOTH newlines;
            // the following `line` then finds itself at a fresh line and only re-arms indentation.
            write!(f, if_group_breaks(&text("\n\n").without_expand_parent()));
        }
    }
    write!(f, soft_line_break_or_space());
}

/// Trailing comma when broken, then any comments left before the closing bracket as own-line end comments.
fn write_trailing_comma_and_end_comments(close_end: u32, f: &mut YamlFormatter<'_, '_>) {
    if f.options().allow_trailing_comma() {
        write!(f, if_group_breaks(&","));
    }
    // Comments inside the brackets after the last entry
    let close_start = close_end.saturating_sub(1);
    let comments = f.context().comments().take_before(close_start);
    for &span in comments {
        write!(f, hard_line_break());
        write_single_comment(span, f);
    }
}
