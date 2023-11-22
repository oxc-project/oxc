use console::Style;
use similar::{ChangeTag, TextDiff};

pub fn print_diff_in_terminal(expected: &str, actual: &str) {
    let diff = TextDiff::from_lines(expected, actual);

    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            let (sign, style) = match change.tag() {
                ChangeTag::Delete => ("-", Style::new().red()),
                ChangeTag::Insert => ("+", Style::new().green()),
                ChangeTag::Equal => (" ", Style::new()),
            };
            print!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
        }
    }
}
