use console::Style;
use similar::{ChangeTag, DiffableStr, TextDiff};

pub fn print_diff_in_terminal<T>(diff: &TextDiff<T>)
where
    T: DiffableStr + ?Sized,
{
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
