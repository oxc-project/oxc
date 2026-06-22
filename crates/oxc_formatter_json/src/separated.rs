//! Light-weight separated-list helper for JSON,
//! modelled after `oxc_formatter::formatter::separated::FormatSeparatedIter` but trimmed for JSON's needs:
//! - only one separator character (`,`),
//! - one inter-entry break style (`soft_line_break_or_space`),
//! - and an optional trailing separator that only materializes when the surrounding group breaks

use oxc_formatter_core::{
    Buffer,
    builders::{
        empty_line, hard_line_break, if_group_breaks, line_suffix, soft_line_break_or_space, space,
    },
    write,
};
use oxc_span::Span;

use crate::{
    comments::{count_newlines, write_single_comment},
    print::{JsonFormatter, format_with},
};

/// Whether a trailing `,` should follow the last entry.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum TrailingSeparator {
    /// Never emit a trailing separator.
    /// Used for the `json` / `json-stringify` variants (and elision-forced arrays).
    #[default]
    Disallowed,
    /// Emit `,` only when the enclosing group breaks (multi-line).
    /// Used for the `jsonc` / `json5` variants when the user option allows it.
    AllowedWhenBreaking,
}

impl TrailingSeparator {
    /// `AllowedWhenBreaking` when `allow` (the variant + user option permit a trailing comma),
    /// otherwise `Disallowed`.
    pub fn when_breaking(allow: bool) -> Self {
        if allow { Self::AllowedWhenBreaking } else { Self::Disallowed }
    }
}

/// Writes entries with `, ` (or a soft line break) between them.
/// If the source between consecutive entries contains a blank line,
/// that blank line is preserved in the output.
/// This matches Prettier's behavior of keeping user-authored grouping.
///
/// `emit_entry(index, f)` is invoked once per entry; the first call receives no leading separator.
/// `spans` provides per-entry source spans used for the blank-line detection.
pub fn write_separated<'a, F>(
    f: &mut JsonFormatter<'_, 'a>,
    spans: &[Span],
    trailing: TrailingSeparator,
    upper_bound: u32,
    mut emit_entry: F,
) where
    F: FnMut(usize, &mut JsonFormatter<'_, 'a>),
{
    let count = spans.len();
    for i in 0..count {
        emit_entry(i, f);
        if i + 1 < count {
            write_inter_entry_separator(spans[i], spans[i + 1], f);
        }
    }

    if count > 0 {
        match trailing {
            TrailingSeparator::Disallowed => {}
            TrailingSeparator::AllowedWhenBreaking => {
                write_trailing_separator(upper_bound, f);
            }
        }
    }
}

/// Emits the optional trailing `,` for the last entry,
/// threading the last entry's same-line trailing block comments before the comma.
///
/// Mirrors [`write_inter_entry_separator`]'s rule applied to the final position:
/// a block comment that sits on the same line as the last value is part of that value's trailing,
/// so it prints before the separator (`value /* block */,`).
/// Line comments and own-line comments stay pending for the caller's [`crate::comments::FormatTrailingInsideComments`] pass.
/// Landing after the comma (`value, // line`) which is correct,
/// since a line comment terminates the line.
///
/// `upper_bound` (the container's closing-delimiter position) bounds the comment scan
/// so nested/outer comments aren't pulled in.
fn write_trailing_separator(upper_bound: u32, f: &mut JsonFormatter<'_, '_>) {
    // Leading run of same-line block comments (no preceding newline, not a line comment).
    let block_end = f
        .context()
        .comments()
        .iter_before(upper_bound)
        .take_while(|c| !c.preceded_by_newline() && !c.is_line())
        .last()
        .map(|c| c.span.end);

    if let Some(block_end) = block_end {
        for c in f.context().comments().take_before(block_end) {
            write!(f, space());
            write_single_comment(c, f);
        }
    }

    write!(f, if_group_breaks(&","));
}

/// Writes the `,` separator between two entries,
/// threading any "trailing-of-prev" comments around the comma.
///
/// Mirrors Prettier's comment placement:
/// a comment that physically sits on the same line as the previous entry's value end is
/// treated as trailing on that entry, not as leading on the next.
/// - block comments emit *before* the comma (`value /* block */,`)
/// - a line comment emits *after* the comma followed by a hard line break
///   (`value, // line\n`); since line comments terminate the line,
///   at most one can appear per gap and it's necessarily the last
///
/// Own-line comments (newline between prev value end and comment start) stay
/// pending so the next entry's leading-comment pass picks them up.
fn write_inter_entry_separator(prev: Span, curr: Span, f: &mut JsonFormatter<'_, '_>) {
    // Same-line trailing-of-prev comments are exactly those with no preceding newline
    let trailing_end = f
        .context()
        .comments()
        .iter_before(curr.start)
        .take_while(|c| !c.preceded_by_newline())
        .last()
        .map(|c| c.span.end);

    let Some(trailing_end) = trailing_end else {
        write!(f, ",");
        write_break_after(prev.end, curr.start, f);
        return;
    };

    // Drain only the trailing-of-prev comments;
    // the rest stay pending so the next entry's `FormatLeadingComments` picks them up.
    let trailing_comments = f.context().comments().take_before(trailing_end);

    // A line comment can only be the last entry in the group (its `\n` ends the line)
    let (block_comments, line_comment) = match trailing_comments.last() {
        Some(c) if c.is_line() => (&trailing_comments[..trailing_comments.len() - 1], Some(c)),
        _ => (trailing_comments, None),
    };

    for c in block_comments {
        write!(f, space());
        write_single_comment(c, f);
    }
    write!(f, ",");

    if let Some(lc) = line_comment {
        // Defer the line comment via `line_suffix`,
        // so its width doesn't count against the preceding value's group budget.
        // Without this, `"k": [a, b], // long...`
        // would force the array to expand even when it fits on its own.
        let lc = *lc;
        let suffix = format_with(move |f: &mut JsonFormatter<'_, '_>| {
            write!(f, space());
            write_single_comment(&lc, f);
        });
        write!(f, line_suffix(&suffix));
        // Promote to `empty_line` when the source preserves a blank line after the trailing comment;
        // otherwise a hard break (also flushes the line_suffix).
        if has_blank_line(lc.span.end, curr.start, f) {
            write!(f, empty_line());
        } else {
            write!(f, hard_line_break());
        }
    } else {
        write_break_after(trailing_end, curr.start, f);
    }
}

/// Emits `empty_line()` when:
/// the gap from `start` to the next own-line content has a user-authored blank line,
/// otherwise `soft_line_break_or_space()`.
///
/// `start` is `prev.end` in the no-trailing-comments path (gap still contains the source `,`)
/// and `trailing_end` after trailing blocks (gap is past the source `,`).
/// The blank-line policy differs between those two cases, but the bounding logic is shared.
fn write_break_after(start: u32, curr_start: u32, f: &mut JsonFormatter<'_, '_>) {
    if has_blank_line(start, curr_start, f) {
        write!(f, empty_line());
    } else {
        write!(f, soft_line_break_or_space());
    }
}

/// Returns `true` if the source gap from `start` to the next own-line content
/// (first pending leading comment of `curr` or `curr_start`) contains a blank line.
///
/// When the gap still includes the source `,` (no-trailing-comments path),
/// newlines before that comma are ignored (`1\n,2` is layout style, not grouping).
/// After the comma the slice doesn't contain it and we count raw newlines.
fn has_blank_line(start: u32, curr_start: u32, f: &JsonFormatter<'_, '_>) -> bool {
    let Some(slice) = gap_slice(start, curr_start, f) else { return false };
    if slice.contains(&b',') { blank_line_after_comma(slice) } else { count_newlines(slice) >= 2 }
}

/// Returns the source byte slice from `start` to the first pending leading comment of `curr`
/// (or `curr_start` if none).
/// Bounding by the first leading comment is what prevents newlines *inside* comment bodies
/// from being misread as blank-line markers in the inter-entry gap.
fn gap_slice<'a>(start: u32, curr_start: u32, f: &JsonFormatter<'_, 'a>) -> Option<&'a [u8]> {
    let source = f.context().source_text();
    let end = f
        .context()
        .comments()
        .iter_before(curr_start)
        .find(|c| c.span.start >= start)
        .map_or(curr_start, |c| c.span.start);
    if end <= start || end as usize > source.len() {
        return None;
    }
    Some(source.bytes_range(start, end))
}

/// Returns `true` if `between` (the source slice between two adjacent entries) places a blank line
/// after the separator comma.
/// Newlines before the comma (e.g. `1\n,2`) don't count,
/// they represent the user spacing the comma off from the value, not grouping entries.
pub fn blank_line_after_comma(between: &[u8]) -> bool {
    // Count only the line terminators following the first (separator) comma.
    // `count_newlines` is CR/CRLF-aware, keeping this consistent with core newline detection.
    let Some(comma) = between.iter().position(|&b| b == b',') else { return false };
    count_newlines(&between[comma + 1..]) >= 2
}
