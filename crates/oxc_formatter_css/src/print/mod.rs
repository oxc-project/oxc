use cow_utils::CowUtils;
use raffia::ast::{Placeholder, Stylesheet};

use oxc_formatter_core::{
    Buffer, Format, FormatElement, Formatter, arena_cow_str,
    builders::{FormatWith, text, token},
    write,
};

use crate::{
    comments::{write_gap, write_leading_comments},
    context::CssFormatContext,
};

pub mod at_rule;
pub mod less;
pub mod scss;
pub mod selector;
pub mod statement;
pub mod value;

pub type CssFormatter<'buf, 'a> = Formatter<'buf, 'a, CssFormatContext<'a>>;

/// `Format` impl for `&'static str` specialized to `CssFormatContext`.
///
/// Hardcoded to `CssFormatContext` rather than generic over `C` so the blanket
/// `&T where T: Format` doesn't overlap.
impl<'a> Format<'a, CssFormatContext<'a>> for &'static str {
    #[inline]
    fn fmt(&self, f: &mut CssFormatter<'_, 'a>) {
        write!(f, token(self));
    }
}

/// Wraps a re-entrant CSS closure in a [`FormatWith`]. The closure's context is
/// pinned to [`CssFormatContext`] so call sites don't have to annotate it.
#[inline]
pub const fn format_with<'a, T>(formatter: T) -> FormatWith<T>
where
    T: Fn(&mut CssFormatter<'_, 'a>),
{
    FormatWith::new(formatter)
}

/// Emits a css-in-js placeholder:
/// the typed `EmbedPlaceholder` marker (the host substitutes `${expr}` for it)
/// may be followed by any glued literal suffix
/// (`@placeholder-0-idpx` -> marker + `px`), so `${x}px` reads as one value.
fn write_placeholder<'a>(placeholder: &Placeholder<'a>, f: &mut CssFormatter<'_, 'a>) {
    f.write_element(FormatElement::EmbedPlaceholder(placeholder.index));
    if !placeholder.suffix.is_empty() {
        write!(f, text(placeholder.suffix));
    }
}

/// Collapses any whitespace run in `raw` to a single space.
pub fn normalize_whitespace(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut iter = raw.split_whitespace();
    if let Some(first) = iter.next() {
        out.push_str(first);
        for word in iter {
            out.push(' ');
            out.push_str(word);
        }
    }
    out
}

/// Mirrors Prettier's `maybeToLowerCase`.
/// Lowercase unless the identifier contains variable/interpolation markers.
pub fn write_maybe_lowercase<'a>(value: &'a str, f: &mut CssFormatter<'_, 'a>) {
    if value.contains('$')
        || value.contains('@')
        || value.contains('#')
        || value.starts_with('%')
        || value.starts_with("--")
        || value.starts_with(":--")
        || (value.contains('(') && value.contains(')'))
    {
        write!(f, text(value));
        return;
    }
    let lower = value.cow_to_ascii_lowercase();
    write!(f, text(arena_cow_str(&lower, f)));
}

/// Emits the whole stylesheet: top-level statements separated by hard lines
/// (blank lines preserved, max one), then any trailing comments.
///
/// Mirrors Prettier's `css-root` + `printSequence`.
pub fn write_stylesheet<'a>(stylesheet: &Stylesheet<'a>, f: &mut CssFormatter<'_, 'a>) {
    statement::write_statement_sequence_bounded(&stylesheet.statements, u32::MAX, f);

    // Comments after the last statement (or in an otherwise empty file).
    let remaining = f.context().comments().take_remaining();
    if !remaining.is_empty() {
        if !stylesheet.statements.is_empty() {
            let source = f.context().source_text();
            let prev_end = stylesheet.statements.last().map_or(0, |s| statement::stmt_end(s, f));
            write_gap(source.bytes_range(prev_end, remaining[0].span.start), f);
        }
        let last_end = remaining.last().unwrap().span.end;
        write_leading_comments(remaining, last_end, f);
    }
}
