use std::borrow::Cow;

use cow_utils::CowUtils;

use oxc_formatter_core::{
    Buffer, arena_cow_str,
    builders::{hard_line_break, soft_line_break_or_space, text},
    write,
};
use oxc_span::Span;

use crate::{
    options::ProseWrap,
    print::{YamlFormatter, arena_newlines},
};

/// Which flow scalar shape is being folded (affects word-merge rules and quoting).
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum FlowScalarKind {
    Plain,
    QuoteSingle,
    QuoteDouble,
}

/// Emits a plain scalar: raw slice content, line-folded per `proseWrap`.
pub fn write_plain(span: Span, f: &mut YamlFormatter<'_, '_>) {
    let content = f.context().source_text().text_for(&span);
    write_flow_scalar_content(FlowScalarKind::Plain, content, f);
}

/// Emits a quoted scalar with Prettier's `quoteDouble`/`quoteSingle` case.
pub fn write_quoted(kind: FlowScalarKind, span: Span, f: &mut YamlFormatter<'_, '_>) {
    let source = f.context().source_text();
    // Inner content between the quotes
    let raw = source.text_for(&Span::new(span.start + 1, span.end - 1));

    let has_unreencodable_escape = match kind {
        // quoteSingle cannot represent backslash escapes; keep as-is when any appear
        FlowScalarKind::QuoteSingle => raw.contains('\\'),
        // A backslash escaping anything but `"` requires staying double-quoted
        FlowScalarKind::QuoteDouble => {
            raw.as_bytes().windows(2).any(|w| w[0] == b'\\' && w[1] != b'"')
        }
        FlowScalarKind::Plain => unreachable!("plain scalars are not quoted"),
    };

    if has_unreencodable_escape {
        let quote = if kind == FlowScalarKind::QuoteDouble { "\"" } else { "'" };
        write!(f, text(quote));
        write_flow_scalar_content(kind, raw, f);
        write!(f, text(quote));
        return;
    }

    if raw.contains('"') {
        // Re-quote with single quotes
        write!(f, text("'"));
        if kind == FlowScalarKind::QuoteDouble {
            // `\"` unescapes to `"`; `'` must be doubled in single quotes.
            // (The guarding `contains('"')` means the first replace always matches,
            // so the chain is owned; copy it into the arena once.)
            let content = raw.cow_replace("\\\"", "\"").cow_replace('\'', "''").into_owned();
            write_flow_scalar_content(kind, f.allocator().alloc_str(&content), f);
        } else {
            write_flow_scalar_content(kind, raw, f);
        }
        write!(f, text("'"));
        return;
    }

    if raw.contains('\'') {
        // Re-quote with double quotes
        write!(f, text("\""));
        if kind == FlowScalarKind::QuoteSingle {
            // `''` unescapes to `'` (nothing else needs escaping in double quotes here)
            let content = raw.cow_replace("''", "'");
            write_flow_scalar_content(kind, arena_cow_str(&content, f), f);
        } else {
            write_flow_scalar_content(kind, raw, f);
        }
        write!(f, text("\""));
        return;
    }

    let quote = f.options().single_quote.as_str();
    write!(f, text(quote));
    write_flow_scalar_content(kind, raw, f);
    write!(f, text(quote));
}

/// Prettier's `printFlowScalarContent`:
/// each reflow group becomes a `fill(join(line, words))`, groups joined by hardlines.
///
/// Empty groups (blank lines) are emitted as raw `\n` text,
/// so consecutive breaks survive the printer's newline collapsing (same technique as block scalars);
/// a following hardline only re-arms indentation.
pub fn write_flow_scalar_content<'a>(
    kind: FlowScalarKind,
    content: &'a str,
    f: &mut YamlFormatter<'_, 'a>,
) {
    let prose_wrap = f.options().prose_wrap;
    // Fast path:
    // a single-line scalar under `preserve` (the default) prints verbatim,
    // skip the line-contents machinery and its allocations.
    if prose_wrap == ProseWrap::Preserve && !content.contains('\n') {
        if !content.is_empty() {
            write!(f, text(content));
        }
        return;
    }
    // Words fold to the arena lifetime:
    // borrowed words already slice the arena-backed source, only owned (merged) words need an arena copy.
    let line_groups: Vec<Vec<&'a str>> = flow_scalar_line_contents(kind, content, prose_wrap)
        .into_iter()
        .map(|words| {
            words
                .into_iter()
                .map(|word| match word {
                    Cow::Borrowed(word) => word,
                    Cow::Owned(word) => f.allocator().alloc_str(&word),
                })
                .collect()
        })
        .collect();

    let mut blanks = 0usize;
    let mut wrote_any = false;
    for words in &line_groups {
        if words.is_empty() {
            blanks += 1;
            continue;
        }
        if blanks > 0 {
            // N blank groups = N separators before this group
            // (plus the one separating it from the previous non-empty group, if any).
            write!(f, text(arena_newlines(blanks + usize::from(wrote_any), f)));
            write!(f, hard_line_break());
        } else if wrote_any {
            write!(f, hard_line_break());
        }
        blanks = 0;
        wrote_any = true;
        let mut fill = f.fill();
        for &word in words {
            fill.entry(&soft_line_break_or_space(), &text(word));
        }
        fill.finish();
    }
    if blanks > 0 {
        // Trailing blank groups contribute join-separators only BETWEEN groups:
        // N after a non-empty group, N-1 when every group is empty
        // (a single empty group — e.g. the empty string `""` — emits nothing).
        let separators = if wrote_any { blanks } else { blanks - 1 };
        if separators > 0 {
            write!(f, text(arena_newlines(separators, f)));
            write!(f, hard_line_break());
        }
    }
}

/// Prettier's `getFlowScalarLineContents`:
/// position-dependent trimming, then paragraph merging (unless `preserve`),
/// then `never` collapses each paragraph onto one line.
fn flow_scalar_line_contents<'s>(
    kind: FlowScalarKind,
    content: &'s str,
    prose_wrap: ProseWrap,
) -> Vec<Vec<Cow<'s, str>>> {
    let raw_lines: Vec<&str> = content.split('\n').collect();
    let count = raw_lines.len();
    let trimmed: Vec<&str> = raw_lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            if count == 1 {
                *l
            } else if i != 0 && i != count - 1 {
                l.trim()
            } else if i == 0 {
                l.trim_end()
            } else {
                l.trim_start()
            }
        })
        .collect();

    if prose_wrap == ProseWrap::Preserve {
        return trimmed
            .iter()
            .map(|l| if l.is_empty() { vec![] } else { vec![Cow::Borrowed(*l)] })
            .collect();
    }

    let mut lines: Vec<Vec<Cow<'s, str>>> = Vec::with_capacity(trimmed.len());
    for (i, line) in trimmed.iter().enumerate() {
        let merge_into_previous = i > 0
            && !trimmed[i - 1].is_empty()
            && !line.is_empty()
            // A trailing backslash in quoteDouble is a line continuation; never merge across it
            && (kind != FlowScalarKind::QuoteDouble
                || !lines
                    .last()
                    .and_then(|prev| prev.last())
                    .is_some_and(|w| w.ends_with('\\')));
        let words = split_with_single_space(line).map(Cow::Borrowed);
        if merge_into_previous {
            lines.last_mut().unwrap().extend(words);
        } else {
            lines.push(words.collect());
        }
    }

    if prose_wrap == ProseWrap::Never {
        lines = lines.into_iter().map(|words| vec![Cow::Owned(words.join(" "))]).collect();
    }
    lines
}

/// Prettier's `splitWithSingleSpace` (`/(?<!^| ) (?! |$)/`):
/// split on single spaces that are not at a boundary and not adjacent to another space,
/// so multi-space runs stay inside one "word".
/// Empty input yields no words; anything else yields at least one.
pub fn split_with_single_space(text: &str) -> impl Iterator<Item = &str> {
    let bytes = text.as_bytes();
    let mut start = 0;
    let mut done = text.is_empty();
    std::iter::from_fn(move || {
        if done {
            return None;
        }
        // A split position needs a non-space on both sides,
        // so resuming at `start + 1` (skipping the first byte of the new word) is safe.
        for i in start + 1..bytes.len().saturating_sub(1) {
            if bytes[i] == b' ' && bytes[i - 1] != b' ' && bytes[i + 1] != b' ' {
                let word = &text[start..i];
                start = i + 1;
                return Some(word);
            }
        }
        done = true;
        Some(&text[start..])
    })
}

#[cfg(test)]
mod tests {
    use super::split_with_single_space;

    #[test]
    fn split_preserves_multi_space_runs() {
        assert_eq!(
            split_with_single_space(" a   b c   d e   f ").collect::<Vec<_>>(),
            vec![" a   b", "c   d", "e   f "]
        );
        assert_eq!(split_with_single_space("a b").collect::<Vec<_>>(), vec!["a", "b"]);
        assert_eq!(split_with_single_space("").collect::<Vec<_>>(), Vec::<&str>::new());
    }
}
