//! The fill of one phrasing content: every whitespace that may break is a separator,
//! everything else (words, markers, code spans, hard breaks) is content.

use oxc_allocator::{Allocator, ArenaVec, StringBuilder};
use oxc_formatter_core::{
    Format,
    builders::{
        dedent_to_root, hard_line_break, literal_line_break, mark_as_root,
        soft_line_break_or_space, text,
    },
    write,
};

use crate::{context::MarkdownFormatContext, print::Mark};

use super::{MarkdownFormatter, format_with};

/// A piece of fill content.
#[derive(Clone, Copy)]
pub enum Atom<'a> {
    Str(&'a str),
    /// A forced break inside content (backslash hard break), not a separator:
    /// a fill measures an item only up to a hard break,
    /// so the first two words after one always share a line, whatever the width (Prettier does the same).
    HardLine,
    /// A line break that keeps trailing whitespace and resumes at the current indention (two-space hard break).
    LiteralLine,
    /// A line break inside a verbatim inline construct (HTML, liquid, code span):
    /// the next line resumes at the container's content column, the only indention the parser strips.
    VerbatimLine,
}

impl<'a> Format<'a, MarkdownFormatContext<'a>> for Atom<'a> {
    fn fmt(&self, f: &mut MarkdownFormatter<'_, 'a>) {
        match self {
            Atom::Str(s) => write!(f, text(s)),
            Atom::LiteralLine => write!(f, mark_as_root(&literal_line_break())),
            // Containers mark their content column as root; a list item's extra alignment
            // (checkbox, tab width) is not stripped by the parser and must not be printed here
            Atom::VerbatimLine => write!(f, dedent_to_root(&hard_line_break())),
            Atom::HardLine => write!(f, hard_line_break()),
        }
    }
}

/// A fill separator.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sep {
    Line,
    HardLine,
}

impl<'a> Format<'a, MarkdownFormatContext<'a>> for Sep {
    fn fmt(&self, f: &mut MarkdownFormatter<'_, 'a>) {
        match self {
            Sep::Line => write!(f, soft_line_break_or_space()),
            Sep::HardLine => write!(f, hard_line_break()),
        }
    }
}

#[derive(Clone, Copy)]
enum Item<'a> {
    Atom(Atom<'a>),
    Sep(Sep),
}

/// The alternating content / separator list of one fill, flat (no per-word allocation);
/// runs of atoms are grouped when the fill is written.
pub struct Parts<'a> {
    items: ArenaVec<'a, Item<'a>>,
    /// The source the texts are sliced from: adjacent slices of it join into one atom.
    source: &'a str,
    /// Bytes of `flat` pushed so far: the printed offset of what comes next.
    len: u32,
    /// The delimiter nodes, opaque text and link texts pushed, by printed offset (`pairing`).
    pub marks: Vec<Mark>,
}

impl<'a> Item<'a> {
    /// The item as the parser will read it: a break is a newline, a line a space.
    fn flat(&self) -> &'a str {
        match self {
            Item::Atom(Atom::Str(s)) => s,
            Item::Atom(_) | Item::Sep(Sep::HardLine) => "\n",
            Item::Sep(Sep::Line) => " ",
        }
    }
}

impl<'a> Parts<'a> {
    pub fn new(allocator: &'a Allocator, source: &'a str) -> Self {
        Self { items: ArenaVec::new_in(&allocator), source, len: 0, marks: Vec::new() }
    }

    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn push_str(&mut self, s: &'a str) {
        // A raw newline in content resumes at column 0 (`push_lines` places continuation lines)
        debug_assert!(!s.contains('\n'), "multi-line content must go through `push_lines`: {s:?}");
        if !s.is_empty() {
            self.push_atom(Atom::Str(s));
        }
    }

    pub fn push_atom(&mut self, atom: Atom<'a>) {
        self.push(Item::Atom(atom));
    }

    /// Adjacent separators collapse to the strongest (a text edge next to a soft break).
    pub fn push_sep(&mut self, sep: Sep) {
        if let Some(Item::Sep(existing)) = self.items.last_mut() {
            *existing = (*existing).max(sep);
        } else {
            self.push(Item::Sep(sep));
        }
    }

    fn push(&mut self, item: Item<'a>) {
        self.len += u32::try_from(item.flat().len()).unwrap_or(u32::MAX);
        // Words and the spaces between them as written print as one text
        if let Item::Atom(Atom::Str(s)) = item
            && let Some(Item::Atom(Atom::Str(last))) = self.items.last_mut()
            && let Some(joined) = join_in_source(self.source, last, s)
        {
            *last = joined;
            return;
        }
        self.items.push(item);
    }

    pub fn mark(&mut self, mark: Mark) {
        self.marks.push(mark);
    }

    /// The content as the parser will read it.
    pub fn flat(&self, allocator: &'a Allocator) -> &'a str {
        let mut out = StringBuilder::with_capacity_in(self.len as usize, allocator);
        for item in &self.items {
            out.push_str(item.flat());
        }
        out.into_str()
    }

    /// `value` split into lines, `separator` between them.
    pub fn push_lines(&mut self, value: &'a str, separator: Atom<'a>) {
        for (i, line) in value.split('\n').enumerate() {
            if i > 0 {
                self.push_atom(separator);
            }
            self.push_str(line);
        }
    }

    /// The content on one line (table cells): every separator is a space.
    pub fn into_str(self, allocator: &'a Allocator) -> &'a str {
        if let [Item::Atom(Atom::Str(s))] = self.items.as_slice() {
            return s;
        }
        let mut out = StringBuilder::new_in(allocator);
        for item in &self.items {
            match item {
                Item::Atom(Atom::Str(s)) => out.push_str(s),
                _ => out.push(' '),
            }
        }
        out.into_str()
    }

    pub fn write_fill(self, f: &mut MarkdownFormatter<'_, 'a>) {
        // One line of text: a fill with no separator and no line break decides nothing
        if self.items.iter().all(|item| matches!(item, Item::Atom(Atom::Str(_)))) {
            write_atoms(&self.items, f);
            return;
        }
        let mut fill = f.fill();
        let mut sep = Sep::Line;
        let mut run_start = 0;
        for (i, item) in self.items.iter().enumerate() {
            if let Item::Sep(next) = item {
                // A fill alternates content / separator;
                // a separator with no content before it gets an empty entry.
                let run = &self.items[run_start..i];
                fill.entry(&sep, &format_with(|f| write_atoms(run, f)));
                sep = *next;
                run_start = i + 1;
            }
        }
        let run = &self.items[run_start..];
        fill.entry(&sep, &format_with(|f| write_atoms(run, f)));
        fill.finish();
    }
}

/// `a` and `b` as one slice of `source`, when `b` follows `a` there,
/// and the joined text prints and measures exactly as the two did.
///
/// The width: a character [ending every width sequence](ends_width_sequence) on either side of the boundary
/// makes the widths add up.
/// A host escapes each text on its own (markdown-in-js's `escape_backticks_raw` doubles the `\`s before a backtick,
/// other template guests get `${` escaped likewise), so a `\` or `$` never ends the first.
fn join_in_source<'a>(source: &'a str, a: &str, b: &str) -> Option<&'a str> {
    // `b` starts where `a` ends in memory: within `source` (bounds-checked by `get`), the source continuing
    if a.as_bytes().as_ptr_range().end != b.as_ptr() {
        return None;
    }
    let (&last, &first) = (a.as_bytes().last()?, b.as_bytes().first()?);
    if !(ends_width_sequence(last) || ends_width_sequence(first)) || matches!(last, b'\\' | b'$') {
        return None;
    }
    let start = a.as_ptr().addr().checked_sub(source.as_ptr().addr())?;
    source.get(start..start + a.len() + b.len())
}

/// An ASCII character other than a keycap base (`#`, `*`, a digit) ends every Unicode width sequence:
/// nothing next to it changes its width, and it changes no neighbor's.
fn ends_width_sequence(byte: u8) -> bool {
    byte.is_ascii() && !matches!(byte, b'#' | b'*' | b'0'..=b'9')
}

fn write_atoms<'a>(items: &[Item<'a>], f: &mut MarkdownFormatter<'_, 'a>) {
    for item in items {
        if let Item::Atom(atom) = item {
            atom.fmt(f);
        }
    }
}

#[cfg(test)]
mod tests {
    use oxc_formatter_core::{IndentWidth, text_width};

    use super::ends_width_sequence;

    /// The width rule `join_in_source` relies on, against the measurement itself
    #[test]
    fn widths_add_up_next_to_a_sequence_end() {
        let width = |s: &str| text_width(s, IndentWidth::default());
        // Variation selector, keycap, ZWJ, skin tone, combining mark, regional indicator, emoji, keycap bases
        let neighbors = [
            "\u{FE0F}",
            "\u{20E3}",
            "\u{200D}",
            "\u{1F3FB}",
            "\u{301}",
            "\u{1F1E6}",
            "\u{1F600}",
            "#",
            "1",
        ];
        for byte in (0x20..0x7F).filter(|&byte| ends_width_sequence(byte)) {
            let c = char::from(byte).to_string();
            for neighbor in neighbors {
                assert_eq!(
                    width(&(c.clone() + neighbor)),
                    width(&c) + width(neighbor),
                    "{c:?}{neighbor:?}"
                );
                assert_eq!(
                    width(&(neighbor.to_string() + &c)),
                    width(neighbor) + width(&c),
                    "{neighbor:?}{c:?}"
                );
            }
        }
    }
}
