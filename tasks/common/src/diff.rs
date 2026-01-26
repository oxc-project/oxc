use std::collections::VecDeque;

use console::Style;
use similar::{ChangeTag, DiffableStr, TextDiff};

const CONTEXT_LINES: usize = 3;

/// Prints a colored diff to the terminal with context lines around changes.
///
/// Features:
/// - 3 lines of context above and below each change
/// - Line numbers in a gutter with `│` separator
/// - Blank line between adjacent hunks
/// - `...` separator when lines are skipped between hunks
/// - Colors: red (`-`) for deletions, green (`+`) for insertions, dim for context
///
/// Example output:
/// ```text
///     1 │fn main() {
///     2 │    let x = 1;
/// -   3 │    let y = 2;
/// +   3 │    let y = 3;
///     4 │    println!("{}", x + y);
///     5 │}
/// ...
///    42 │fn other() {
/// -  43 │    old_code();
/// +  43 │    new_code();
///    44 │}
/// ```
/// Simple API that creates a diff from two strings and prints it.
pub fn print_diff_in_terminal(expected: &str, result: &str) {
    let diff = TextDiff::from_lines(expected, result);
    print_text_diff(&diff);
}

/// Prints an existing `TextDiff` to the terminal.
/// Use this when you need access to the `TextDiff` for other operations (e.g., `diff.ratio()`).
pub fn print_text_diff<T: DiffableStr + ?Sized>(diff: &TextDiff<T>) {
    let mut context_buffer: VecDeque<_> = VecDeque::with_capacity(CONTEXT_LINES);
    let mut trailing_remaining = 0;
    let mut has_printed = false;
    let mut had_gap = false;
    let mut needs_separator = false;

    for change in diff.ops().iter().flat_map(|op| diff.iter_changes(op)) {
        let tag = change.tag();

        if tag == ChangeTag::Equal {
            if trailing_remaining > 0 {
                print_line(tag, change.new_index(), &change.to_string());
                trailing_remaining -= 1;
                needs_separator = true;
            } else {
                if context_buffer.len() == CONTEXT_LINES {
                    context_buffer.pop_front();
                    had_gap = true;
                }
                context_buffer.push_back((tag, change.new_index(), change.to_string()));
            }
            continue;
        }

        if has_printed && (needs_separator || !context_buffer.is_empty()) {
            if had_gap {
                println!("{}", Style::new().cyan().apply_to("..."));
            } else {
                println!();
            }
        }

        while let Some((tag, line_num, content)) = context_buffer.pop_front() {
            print_line(tag, line_num, &content);
        }

        let line_num =
            if tag == ChangeTag::Delete { change.old_index() } else { change.new_index() };
        print_line(tag, line_num, &change.to_string());

        has_printed = true;
        had_gap = false;
        needs_separator = false;
        trailing_remaining = CONTEXT_LINES;
    }
}

fn print_line(tag: ChangeTag, line_num: Option<usize>, content: &str) {
    let (sign, style) = match tag {
        ChangeTag::Delete => ("-", Style::new().red()),
        ChangeTag::Insert => ("+", Style::new().green()),
        ChangeTag::Equal => (" ", Style::new().dim()),
    };
    print!(
        "{}{} │{}",
        style.apply_to(sign).bold(),
        Style::new().dim().apply_to(format!("{:>4}", line_num.map_or(0, |n| n + 1))),
        style.apply_to(content)
    );
}
