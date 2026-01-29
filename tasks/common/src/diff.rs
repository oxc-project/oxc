use std::borrow::Cow;

use console::style;
use similar::Algorithm;
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
/// Simple API that creates a diff from two strings and prints it.
pub fn print_diff_in_terminal(expected: &str, result: &str) {
    let diff = TextDiff::configure().algorithm(Algorithm::Patience).diff_lines(expected, result);
    print_text_diff(&diff);
}

fn render_invisible(s: &str, newlines_matter: bool) -> Cow<'_, str> {
    if newlines_matter || s.find(&['\x1b', '\x07', '\x08', '\x7f'][..]).is_some() {
        Cow::Owned(
            s.replace('\r', "␍\r")
                .replace('\n', "␊\n")
                .replace("␍\r␊\n", "␍␊\r\n")
                .replace('\x07', "␇")
                .replace('\x08', "␈")
                .replace('\x1b', "␛")
                .replace('\x7f', "␡"),
        )
    } else {
        Cow::Borrowed(s)
    }
}

/// Prints an existing `TextDiff` to the terminal.
/// Use this when you need access to the `TextDiff` for other operations (e.g., `diff.ratio()`).
///
/// # Panics
///
/// Panics if the output cannot be printed to the terminal.
///
/// Based on [Cargo insta diff printing](https://github.com/mitsuhiko/insta/blob/8a5b77531f89bc78d00cab17f2ac8b2c69ceadab/insta/src/output.rs#L238-L296).
pub fn print_text_diff<'a, T: DiffableStr + ?Sized>(diff: &'a TextDiff<'a, 'a, 'a, T>) {
    for group in &diff.grouped_ops(CONTEXT_LINES) {
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let missing_newline = change.missing_newline();
                match change.tag() {
                    ChangeTag::Insert => {
                        print!(
                            "{:>5} {:>5} │{}",
                            "",
                            style(change.new_index().unwrap() + 1).cyan().dim().bold(),
                            style("+").green(),
                        );
                        for &(emphasized, change) in change.values() {
                            let change =
                                render_invisible(change.as_str().unwrap(), missing_newline);
                            if emphasized {
                                print!("{}", style(change).green().underlined());
                            } else {
                                print!("{}", style(change).green());
                            }
                        }
                    }
                    ChangeTag::Delete => {
                        print!(
                            "{:>5} {:>5} │{}",
                            style(change.old_index().unwrap() + 1).cyan().dim(),
                            "",
                            style("-").red(),
                        );
                        for &(emphasized, change) in change.values() {
                            let change =
                                render_invisible(change.as_str().unwrap(), missing_newline);
                            if emphasized {
                                print!("{}", style(change).red().underlined());
                            } else {
                                print!("{}", style(change).red());
                            }
                        }
                    }
                    ChangeTag::Equal => {
                        print!(
                            "{:>5} {:>5} │ ",
                            style(change.old_index().unwrap() + 1).cyan().dim(),
                            style(change.new_index().unwrap() + 1).cyan().dim().bold(),
                        );
                        for &(_, change) in change.values() {
                            let change = render_invisible(change.as_str().unwrap(), false);
                            print!("{}", style(change).dim());
                        }
                    }
                }
                if change.missing_newline() {
                    println!();
                }
            }
        }
    }
}
