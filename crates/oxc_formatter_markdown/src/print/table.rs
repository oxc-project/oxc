//! GFM tables.
//!
//! Every cell is printed to a string first (Prettier's `printDocToString` at infinite width),
//! then each column is padded to its widest cell (3 at least, the delimiter's `---`).
//! With `proseWrap: never`, a table whose header row does not fit is printed compact instead.

use oxc_allocator::{Allocator, ArenaVec, StringBuilder};
use oxc_formatter_core::{
    Buffer, TextWidth,
    builders::{group, hard_line_break, if_group_breaks, if_group_fits_on_line},
    format_args, write,
};
use oxc_markdown_parser::ast::{Table, TableAlign};

use crate::options::ProseWrap;

use super::{
    MarkdownFormatter, format_with,
    inline::{InlineParent, Parts, collect_inlines},
    write_lines,
};

struct Cell<'a> {
    text: &'a str,
    width: usize,
}

pub fn write_table<'a>(table: &'a Table<'a>, f: &mut MarkdownFormatter<'_, 'a>) {
    let allocator = f.allocator();
    let columns = table.children.iter().map(|row| row.children.len()).max().unwrap_or(0);
    let mut widths = ArenaVec::from_iter_in(std::iter::repeat_n(3, columns), &allocator);
    let mut rows = ArenaVec::with_capacity_in(table.children.len(), &allocator);
    for row in &table.children {
        let mut cells = ArenaVec::with_capacity_in(row.children.len(), &allocator);
        for (i, cell) in row.children.iter().enumerate() {
            let mut parts = Parts::new(allocator);
            collect_inlines(&cell.children, InlineParent::default(), &mut parts, f);
            let text = parts.into_str(allocator);
            let width = TextWidth::from_text(text, f.options().indent_width).value() as usize;
            widths[i] = widths[i].max(width);
            cells.push(Cell { text, width });
        }
        rows.push(cells);
    }

    let aligned = print_lines(&rows, &widths, &table.align, false, allocator);
    if f.options().prose_wrap != ProseWrap::Never {
        write_lines(aligned, hard_line_break(), f);
        return;
    }
    // Only the header row is measured: `fits` stops at the first line break
    let compact = print_lines(&rows, &widths, &table.align, true, allocator);
    let separator = hard_line_break().without_expand_parent();
    write!(
        f,
        group(&format_args!(
            if_group_breaks(&format_with(|f| write_lines(compact.iter().copied(), separator, f))),
            if_group_fits_on_line(&format_with(|f| write_lines(
                aligned.iter().copied(),
                separator,
                f
            ))),
        ))
    );
}

/// Header row, delimiter row, body rows.
/// The delimiter row has the header's cell count (`align`): more, and it is not a table.
fn print_lines<'a>(
    rows: &[ArenaVec<'a, Cell<'a>>],
    widths: &[usize],
    align: &[TableAlign],
    compact: bool,
    allocator: &'a Allocator,
) -> ArenaVec<'a, &'a str> {
    let row_line = |cells: &[Cell<'a>]| {
        let mut out = StringBuilder::with_capacity_in(
            1 + widths.len() * 3 + widths.iter().sum::<usize>(),
            allocator,
        );
        out.push('|');
        for (i, cell) in cells.iter().enumerate() {
            let spaces = if compact { 0 } else { widths[i] - cell.width };
            let before = match align.get(i) {
                Some(TableAlign::Right) => spaces,
                Some(TableAlign::Center) => spaces / 2,
                _ => 0,
            };
            out.push(' ');
            out.push_ascii_byte_repeat(b' ', before);
            out.push_str(cell.text);
            out.push_ascii_byte_repeat(b' ', spaces - before);
            out.push_str(" |");
        }
        out.into_str()
    };
    let delimiter_line = || {
        let mut out = StringBuilder::new_in(allocator);
        out.push('|');
        for (align, width) in align.iter().zip(widths) {
            let (first, last) = match align {
                TableAlign::None => ('-', '-'),
                TableAlign::Left => (':', '-'),
                TableAlign::Center => (':', ':'),
                TableAlign::Right => ('-', ':'),
            };
            out.push(' ');
            out.push(first);
            out.push_ascii_byte_repeat(b'-', if compact { 1 } else { width - 2 });
            out.push(last);
            out.push_str(" |");
        }
        out.into_str()
    };

    let mut lines = ArenaVec::with_capacity_in(rows.len() + 1, &allocator);
    lines.push(row_line(&rows[0]));
    lines.push(delimiter_line());
    lines.extend(rows[1..].iter().map(|cells| row_line(cells)));
    lines
}
