//! Markers normalize: bullets alternate `-` / `*`
//! and ordered delimiters `.` / `)` between adjacent sibling lists (so two lists stay two lists);
//! numbers renumber from the first item's
//! (all the same when the second item repeats `1`, "git-diff friendly").

use oxc_formatter_core::{
    Buffer,
    builders::{align, text, token},
    write,
};
use oxc_markdown_parser::ast::{Block, CodeBlockKind, List, ListItem, ListMarker, TaskCheckbox};

use crate::context::ListFrame;

use super::{MarkdownFormatter, block, column_of, format_with, write_gap};

fn is_ordered(list: &List<'_>) -> bool {
    matches!(list.marker, ListMarker::Ordered { .. })
}

fn is_indented_code(block: &Block<'_>) -> bool {
    matches!(block, Block::CodeBlock(code) if matches!(code.kind, CodeBlockKind::Indented))
}

/// The maximum number CommonMark parses as a list item number.
const MAXIMUM_ORDERED_LIST_MARKER: u64 = 999_999_999;

pub fn write_list<'a>(
    list: &'a List<'a>,
    siblings: &'a [Block<'a>],
    index: usize,
    f: &mut MarkdownFormatter<'_, 'a>,
) {
    let alternate_marker = uses_alternate_marker(list, siblings, index, f);
    let git_diff_friendly = has_git_diff_friendly_ordered_list(list, f);
    let next = siblings.get(index + 1);
    let aligned = is_aligned(list, next, f);
    let min_indent = required_indent(next, f);
    let indent_width = usize::from(f.options().indent_width.value());
    let ordered = is_ordered(list);
    let start = list.children.first().map_or(1, |item| marker_number(item, f));

    f.context().lists().borrow_mut().push(ListFrame { alternate_marker, aligned });

    let mut previous_loose = false;
    for (i, item) in list.children.iter().enumerate() {
        if i > 0 {
            write_gap(previous_loose, f);
        }
        let loose = is_loose_item(item, list.children.get(i + 1), f);
        previous_loose = loose;
        let checkbox = match item.checkbox {
            Some(TaskCheckbox { checked: true, .. }) => "[x] ",
            Some(TaskCheckbox { checked: false, .. }) => "[ ] ",
            None => "",
        };

        let mut prefix = String::new();
        if ordered {
            let number = if i == 0 {
                start
            } else if git_diff_friendly {
                1
            } else {
                (start + i as u64).min(MAXIMUM_ORDERED_LIST_MARKER)
            };
            prefix.push_str(&number.to_string());
            prefix.push_str(if alternate_marker { ") " } else { ". " });
            if aligned {
                align_list_prefix(&mut prefix, indent_width);
            }
        } else {
            prefix.push_str(if alternate_marker { "* " } else { "- " });
        }
        pad_to_min_indent(&mut prefix, min_indent);

        // An empty item is just its marker (the printer never trims trailing whitespace).
        if item.children.is_empty() {
            let marker = f.allocator().alloc_concat_strs_array([&prefix, checkbox]);
            write!(f, text(marker.trim_end()));
            continue;
        }

        let prefix: &'a str = f.allocator().alloc_str(&prefix);
        write!(f, text(prefix));

        // Prettier skips the alignment for `[paragraph, html]` items whose html starts at another column
        let skip_align =
            item.children.len() == 2 && matches!(item.children[1], Block::HtmlBlock(_)) && {
                let source = f.context().source_text().as_str();
                column_of(source, item.children[0].span().start)
                    != column_of(source, item.children[1].span().start)
            };
        let body = format_with(|f| write_list_item(item, checkbox, prefix.len(), loose, f));
        let width = if skip_align { 0 } else { u8::try_from(prefix.len()).unwrap_or(u8::MAX) };
        write!(f, align(width, &body));
    }

    f.context().lists().borrow_mut().pop();
}

fn write_list_item<'a>(
    item: &'a ListItem<'a>,
    checkbox: &'static str,
    list_prefix_len: usize,
    loose: bool,
    f: &mut MarkdownFormatter<'_, 'a>,
) {
    write!(f, token(checkbox));

    // Non-first children (and a first child that is a list) sit `tabWidth - prefix` further in,
    // capped at 3 (4+ would read as indented code).
    let indent_width = usize::from(f.options().indent_width.value());
    let alignment = indent_width.saturating_sub(list_prefix_len).min(3);
    let parent = block::Parent::ListItem { loose };

    for (i, child) in item.children.iter().enumerate() {
        if i > 0 {
            block::write_gap(&item.children, i, parent, f);
        }
        let body = format_with(|f| block::write_block(child, &item.children, i, parent, f));
        // An indented code block's content is whatever follows the item's content column + 4:
        // any extra alignment lands inside the code on the next parse.
        // See DIVERGENCES `list-indented-code-alignment`.
        // (Prettier's checkbox align before #19647, its tab-width alignment since)
        if is_indented_code(child) {
            write!(f, body);
            continue;
        }
        // The first child (unless a list) sits right after the checkbox;
        // other children get the tab-width alignment.
        // `checkbox` is `[x] ` / `[ ] ` / empty, `alignment` is at most 3
        #[expect(clippy::cast_possible_truncation)]
        let (checkbox_width, alignment_width) = (checkbox.len() as u8, alignment as u8);
        if i == 0 && !matches!(child, Block::List(_)) {
            write!(f, align(checkbox_width, &body));
        } else {
            write!(f, [token(&"   "[..alignment]), align(alignment_width, &body)]);
        }
    }
}

/// Loose: spread, or a blank line before the next item.
fn is_loose_item(
    item: &ListItem<'_>,
    next: Option<&ListItem<'_>>,
    f: &MarkdownFormatter<'_, '_>,
) -> bool {
    item.spread
        || next.is_some_and(|next| f.context().has_blank_between(item.span.end, next.span.start))
}

/// Whether this list prints the alternate marker (`*` / `)`).
///
/// Adjacent lists of the same kind alternate markers so they stay separate lists.
/// A list kept verbatim (`prettier-ignore`) shows its own marker,
/// so when it opens the run the alternation continues from that marker rather than from the default.
fn uses_alternate_marker(
    list: &List<'_>,
    siblings: &[Block<'_>],
    index: usize,
    f: &MarkdownFormatter<'_, '_>,
) -> bool {
    let ordered = is_ordered(list);
    let same_kind = |block: &Block<'_>| matches!(block, Block::List(other) if matches!(other.marker, ListMarker::Ordered { .. }) == ordered);
    let mut start = index;
    while start > 0 && same_kind(&siblings[start - 1]) {
        start -= 1;
    }
    // Only the run's first list can be verbatim:
    // an ignore comment before any later one would sit between two lists and end the run.
    let first_alternate = match &siblings[start] {
        Block::List(first) if block::is_ignored(siblings, start, f) => match first.marker {
            ListMarker::Bullet { marker } => marker == b'*',
            ListMarker::Ordered { delimiter } => delimiter == b')',
        },
        _ => false,
    };
    first_alternate ^ ((index - start) % 2 == 1)
}

/// The item's number, leading zeros dropped.
fn marker_number(item: &ListItem<'_>, f: &MarkdownFormatter<'_, '_>) -> u64 {
    let raw = f.context().slice(item.marker);
    raw.trim_end_matches(['.', ')']).parse().unwrap_or(0)
}

/// `1. 1. 1.` (or `0. 1. 1.`) numbering: every item keeps `1` so edits do not renumber the rest.
fn has_git_diff_friendly_ordered_list(list: &List<'_>, f: &MarkdownFormatter<'_, '_>) -> bool {
    if !is_ordered(list) || list.children.len() < 2 {
        return false;
    }
    if marker_number(&list.children[1], f) != 1 {
        return false;
    }
    if marker_number(&list.children[0], f) != 0 {
        return true;
    }
    list.children.get(2).is_some_and(|third| marker_number(third, f) == 1)
}

/// Whether ordered markers are padded to the tab width.
fn is_aligned(list: &List<'_>, next: Option<&Block<'_>>, f: &MarkdownFormatter<'_, '_>) -> bool {
    if list.children.is_empty() {
        return false;
    }
    // An unaligned ancestor list makes every descendant unaligned
    // (each frame carries its ancestors' verdict)
    if f.context().lists().borrow().last().is_some_and(|frame| !frame.aligned) {
        return false;
    }
    // Hard to align when followed by indented code
    if next.is_some_and(is_indented_code) {
        return false;
    }
    if !is_ordered(list) {
        return true;
    }

    let source = f.context().source_text().as_str();
    let item_start = |item: &ListItem<'_>| -> Option<usize> {
        item.children.first().map(|child| column_of(source, child.span().start))
    };
    let indent_width = usize::from(f.options().indent_width.value());

    let first = &list.children[0];
    if first.padding > 1 {
        return true;
    }
    let Some(first_start) = item_start(first) else { return false };
    // `x % 0` is `NaN` in Prettier, never equal to 0
    let at_tab_stop = |column: usize| indent_width != 0 && column.is_multiple_of(indent_width);
    let Some(second) = list.children.get(1) else {
        return at_tab_stop(first_start);
    };
    if item_start(second) != Some(first_start) {
        return false;
    }
    if at_tab_stop(first_start) {
        return true;
    }
    second.padding > 1
}

/// Pads the prefix so the content starts at a tab stop
/// (never 4+ extra: that would be indented code).
fn align_list_prefix(prefix: &mut String, indent_width: usize) {
    if indent_width == 0 {
        return;
    }
    let rest = prefix.len() % indent_width;
    let additional = if rest == 0 { 0 } else { indent_width - rest };
    if additional < 4 {
        prefix.extend(std::iter::repeat_n(' ', additional));
    }
}

/// A list followed by an indented code block must indent its content deeper than the code
/// (4 + the code's own leading whitespace + 1), or the code would join the list.
fn required_indent(next: Option<&Block<'_>>, f: &MarkdownFormatter<'_, '_>) -> usize {
    let Some(Block::CodeBlock(code)) = next else { return 0 };
    if !matches!(code.kind, CodeBlockKind::Indented) {
        return 0;
    }
    let Some(first) = code.lines.first() else { return 0 };
    let raw = f.context().slice(first.span);
    let leading: usize = raw
        .bytes()
        .take_while(|b| matches!(b, b' ' | b'\t'))
        .map(|b| if b == b'\t' { 4 } else { 1 })
        .sum();
    4 + usize::from(first.padding) + leading + 1
}

/// Pads the prefix to `min_indent`:
/// trailing spaces first (max 4), then leading spaces (max 3).
fn pad_to_min_indent(prefix: &mut String, min_indent: usize) {
    if prefix.len() >= min_indent {
        return;
    }
    let trimmed_len = prefix.trim_end().len();
    prefix.truncate(trimmed_len);
    let trailing = (min_indent - prefix.len()).min(4);
    prefix.extend(std::iter::repeat_n(' ', trailing));
    let leading = min_indent.saturating_sub(prefix.len()).min(3);
    if leading > 0 {
        prefix.insert_str(0, &" ".repeat(leading));
    }
}
