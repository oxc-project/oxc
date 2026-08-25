use std::borrow::Cow;

use oxc_formatter_core::{
    Buffer,
    builders::{
        align, dedent, dedent_to_root, exact_line_breaks, hard_line_break, literal_line_break,
        mark_as_root, soft_line_break_or_space, text,
    },
    write,
};
use oxc_yaml_parser::ast::{BlockScalar, Chomping, Content, MappingItem, Node, Root};

use crate::{
    comments::write_comment_line_suffix,
    options::ProseWrap,
    print::{
        YamlFormatter, format_with,
        scalar::{join_lines_for_never_wrap, split_with_single_space},
        to_span,
    },
};

/// A run of `count` newlines with the arena lifetime, for the keep-chomping tail.
/// Small runs (the overwhelming case) slice a static string,
/// instead of building a `String` and copying it into the arena.
fn arena_newlines<'a>(count: usize, f: &YamlFormatter<'_, 'a>) -> &'a str {
    const NEWLINES: &str = "\n\n\n\n\n\n\n\n";
    if count <= NEWLINES.len() {
        &NEWLINES[..count]
    } else {
        f.allocator().alloc_str(&"\n".repeat(count))
    }
}

/// How many of a block scalar's trailing source newlines its own OUTPUT consumes
/// (the effect of [`remove_unnecessary_trailing_newlines`]):
/// none as the stream's last descendant (the output is truncated right after the last content character),
/// otherwise its final line ending plus the one preserved blank line when the source had two or more.
///
/// Zero for the last descendant because the FILE's trailing newline is owned by `FormatYamlRoot` (`format.rs`), not the scalar:
/// it appends the POSIX final newline itself, EXCEPT after a keep-chomped (`+`) tail whose verbatim value already carries its trailing newlines,
/// emitting any here would double them.
fn consumed_trailing_newlines(total_newlines: usize, is_last_descendant: bool) -> usize {
    if is_last_descendant {
        0
    } else {
        let blanks = total_newlines.saturating_sub(1);
        (1 + usize::from(blanks >= 2)).min(total_newlines)
    }
}

/// Ports Prettier's `printBlock` for `|` / `>` scalars.
pub fn write_block_scalar<'a>(
    block: &'a BlockScalar,
    is_folded: bool,
    f: &mut YamlFormatter<'_, 'a>,
) {
    let parent_indent = f.context().collection_depth().get();
    let is_last_descendant = block.span.end >= f.context().last_descendant_end();

    // Header: indicator, explicit indent digit, chomping indicator
    write!(f, if is_folded { ">" } else { "|" });
    if let Some(indent) = block.indent {
        // The explicit indentation indicator is a single digit 1-9
        let digit = match indent {
            1 => "1",
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            _ => "9",
        };
        write!(f, text(digit));
    }
    match block.chomping {
        Chomping::Keep => write!(f, "+"),
        Chomping::Strip => write!(f, "-"),
        Chomping::Clip => {}
    }
    // Indicator comment: same line as the header (`| # comment`).
    // The parser guarantees it is the ONLY comment within the scalar's span
    // (see the guarantee on `BlockScalar`), so nothing else needs draining here;
    // trailing-ness (`own_line_column: None`) pins it to the header line.
    // No `expand_parent()`: the scalar's leading hardline already breaks the container,
    // and expansion must not leak into the enclosing `best_fitting` measurement.
    if let Some(comment) = f.context().comments().peek()
        && comment.span.end <= block.content_start
        && comment.own_line_column.is_none()
    {
        f.context().comments().take_before(comment.span.end);
        write_comment_line_suffix(comment.span, f);
    }

    let line_groups: Vec<Vec<&'a str>> =
        block_value_line_contents(block, is_folded, parent_indent, is_last_descendant, f);

    // Blank runs are the scalar's VALUE
    let chomping_keep = block.chomping == Chomping::Keep;
    let contents = format_with(move |f: &mut YamlFormatter<'_, 'a>| {
        let mut blanks = 0usize;
        let mut wrote_any = false;
        for words in &line_groups {
            if words.is_empty() {
                blanks += 1;
                continue;
            }
            if blanks > 0 {
                // One newline entering this segment
                // (after the header for the first, the separator otherwise) + one per blank line.
                write!(f, exact_line_breaks(blanks + 1));
            } else if wrote_any {
                write!(f, mark_as_root(&literal_line_break()));
            } else {
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
        if chomping_keep {
            // Keep chomping: the trailing newlines are the VALUE
            // (plus the final newline for a last descendant, which `FormatYamlRoot` skips after a keep tail).
            // Raw text keeps them exempt from EVERY layout normalization,
            // including the state effects a line element would have on the following separator.
            // `ends_with_keep_chomped_block` mirrors this guard for the root's final newline.
            if blanks > 0 || (is_last_descendant && wrote_any) {
                write!(f, dedent_to_root(&text(arena_newlines(blanks + 1, f))));
            }
        } else if blanks > 0 {
            // The trailing blank preserved by blank-line preservation
            write!(f, exact_line_breaks(blanks + 1));
        }
    });

    if let Some(indent) = block.indent {
        let width = u8::try_from(indent - 1 + parent_indent).unwrap_or(u8::MAX);
        write!(f, dedent_to_root(&align(width, &contents)));
    } else {
        let tab_width = f.options().indent_width.value();
        write!(f, dedent(&align(tab_width, &contents)));
    }
}

/// Ports Prettier's `getBlockValueLineContents`.
///
/// Return contract: an EMPTY word group IS a blank line, and nothing else is.
/// A line still holding spaces/tabs after indentation stripping is more-indented,
/// hence CONTENT per YAML: its whitespace is part of the value (prettier#19764),
/// and the core printer never trims it away.
fn block_value_line_contents<'s>(
    block: &BlockScalar,
    is_folded: bool,
    parent_indent: u32,
    is_last_descendant: bool,
    f: &YamlFormatter<'_, 's>,
) -> Vec<Vec<&'s str>> {
    if block.content_start >= block.span.end {
        return Vec::new();
    }
    let source = f.context().source_text();
    let content = source.text_for(&to_span(oxc_yaml_parser::Span {
        start: block.content_start,
        end: block.span.end,
    }));

    // Leading indentation to strip from every line
    let leading_space_count = if let Some(indent) = block.indent {
        (indent - 1 + parent_indent) as usize
    } else {
        content
            .split('\n')
            .find_map(|l| {
                let spaces = l.len() - l.trim_start_matches(' ').len();
                // First line containing a non-space, non-CR character
                l[spaces..].chars().next().filter(|c| *c != '\r').map(|_| spaces)
            })
            .unwrap_or(usize::MAX)
    };
    let strip = |l: &'s str| l.get(leading_space_count.min(l.len())..).unwrap_or("");

    let prose_wrap = f.options().prose_wrap;
    // Literal blocks (`|`) are never re-flowed;
    // folded blocks only under `proseWrap` always/never.
    let no_reflow = prose_wrap == ProseWrap::Preserve || !is_folded;

    let lines: Vec<Vec<&'s str>> = if no_reflow {
        content
            .split('\n')
            .map(strip)
            .map(|l| if l.is_empty() { vec![] } else { vec![l] })
            .collect()
    } else {
        // Borrowed words already slice the arena-backed source;
        // only the words fold_lines merged into owned strings need an arena copy.
        let stripped: Vec<&'s str> = content.split('\n').map(strip).collect();
        fold_lines(&stripped, prose_wrap)
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
            .collect()
    };

    remove_unnecessary_trailing_newlines(block, is_last_descendant, lines)
}

fn fold_lines<'s>(stripped: &[&'s str], prose_wrap: ProseWrap) -> Vec<Vec<Cow<'s, str>>> {
    let mut lines: Vec<Vec<&'s str>> = Vec::with_capacity(stripped.len());
    for line in stripped {
        // NOTE: a more-indented line keeps its line breaks literally per YAML folding,
        // so re-flowing it at the print width would change the parsed value (and break idempotency).
        // Prettier wraps it like any other paragraph; here it stays one unbreakable word
        // (space-only lines included: their whitespace is value).
        if line.starts_with(char::is_whitespace) {
            lines.push(vec![*line]);
            continue;
        }
        let mut words = split_with_single_space(line).peekable();
        let prev_group_has_boundary_space = lines.last().is_some_and(|prev| {
            prev.first().is_some_and(|w| w.starts_with(char::is_whitespace))
                || prev.last().is_some_and(|w| w.ends_with(char::is_whitespace))
        });
        let merge = !line.is_empty()
            && lines.last().is_some_and(|prev| !prev.is_empty())
            && !words.peek().is_some_and(|w| w.starts_with(char::is_whitespace))
            && !prev_group_has_boundary_space;
        if merge {
            lines.last_mut().unwrap().extend(words);
        } else {
            lines.push(words.collect());
        }
    }

    // Merge words into their predecessor when it ends with whitespace
    // (a soft break after it would re-fold to a different value).
    let mut merged: Vec<Vec<Cow<'s, str>>> = lines
        .into_iter()
        .map(|original| {
            let mut words: Vec<Cow<'s, str>> = Vec::with_capacity(original.len());
            for word in original {
                if let Some(last) = words.last_mut()
                    && last.ends_with(char::is_whitespace)
                {
                    // `take` avoids re-copying an already-owned word
                    let mut owned = std::mem::take(last).into_owned();
                    owned.push(' ');
                    owned.push_str(word);
                    *last = Cow::Owned(owned);
                } else {
                    words.push(Cow::Borrowed(word));
                }
            }
            words
        })
        .collect();

    if prose_wrap == ProseWrap::Never {
        merged = join_lines_for_never_wrap(merged);
    }
    merged
}

/// Mirrors Prettier's `removeUnnecessaryTrailingNewlines`.
fn remove_unnecessary_trailing_newlines<'s>(
    block: &BlockScalar,
    is_last_descendant: bool,
    mut lines: Vec<Vec<&'s str>>,
) -> Vec<Vec<&'s str>> {
    if block.chomping == Chomping::Keep {
        // NOTE: The fragment after the last break holds no line break, so it is not a kept line:
        // either the empty artifact `split('\n')` yields after a final break,
        // or a break-less EOF line of at-or-below-indent spaces (a known divergence: Prettier counts it);
        // a more-indented space run stays a content group.
        lines.pop_if(|words| words.is_empty());
        return lines;
    }

    let trailing_blanks = lines.iter().rev().take_while(|words| words.is_empty()).count();
    // Preserve one blank line when the source had two or more
    lines.truncate(
        lines.len() - trailing_blanks + usize::from(trailing_blanks >= 2 && !is_last_descendant),
    );
    lines
}

/// Returns `true` when the stream's last descendant is a keep-chomped (`+`) block scalar
/// whose verbatim content ends with the kept newlines,
/// so the caller must not append the usual final `hard_line_break()`
/// (mirrors Prettier's `shouldPrintHardline` gate).
///
/// Returns `false` when the scalar emits no tail of its own,
/// no content characters and no kept line break (empty, or one break-less EOF line of at-or-below-indent spaces);
/// the caller's final newline stands.
pub fn ends_with_keep_chomped_block(root: &Root<'_>, f: &YamlFormatter<'_, '_>) -> bool {
    root.children
        .last()
        .and_then(|document| document.body.content.as_deref())
        .and_then(last_descendant_block_scalar)
        .is_some_and(|block| {
            // Content characters guarantee a tail;
            // an empty content region emits one exactly when its trailing run holds a line break.
            block.chomping == Chomping::Keep
                && (block.content_end > block.content_start
                    || f.context()
                        .source_text()
                        .bytes_range(block.content_end, block.span.end)
                        .contains(&b'\n'))
        })
}

/// The block scalar the node's last descendant resolves to, if any.
/// Its span consumes the trailing line breaks,
/// so "same line" checks against the content end are meaningless after one.
pub fn last_descendant_block_scalar<'b>(node: &'b Node<'_>) -> Option<&'b BlockScalar> {
    match &node.content {
        Content::BlockLiteral(block) | Content::BlockFolded(block) => Some(block),
        Content::Mapping(mapping) => mapping
            .children
            .last()
            .and_then(MappingItem::value_content)
            .and_then(last_descendant_block_scalar),
        Content::Sequence(sequence) => sequence
            .children
            .last()
            .and_then(|item| item.content.as_deref())
            .and_then(last_descendant_block_scalar),
        // Block scalars cannot appear inside flow collections
        _ => None,
    }
}

/// Walks to the end offset of the stream's last descendant node.
///
/// Spans nest and every wrapper ends at its last descendant
/// (the parser's `container_span` / `MappingItem` / `Node` span construction),
/// so the last document body's span end IS the last descendant's end.
pub fn last_descendant_end(root: &Root<'_>) -> u32 {
    root.children
        .last()
        .and_then(|document| document.body.content.as_deref())
        .map_or(0, |node| node.span.end)
}

/// The gap-measurement anchor after an item:
/// a block scalar's span consumes its trailing line breaks (they are part of its VALUE under keep chomping),
/// so blank-line detection must start after the last content character,
/// otherwise the blank line separating it from the next item is invisible.
///
/// `block` is the item's resolved [`last_descendant_block_scalar`];
/// callers that need the walk's result themselves pass it in instead of paying for it twice.
pub fn item_gap_anchor(block: Option<&BlockScalar>, end: u32, f: &YamlFormatter<'_, '_>) -> u32 {
    let Some(block) = block else {
        return end;
    };
    // Under keep chomping every trailing newline IS the value; the span end is the correct anchor
    if block.chomping == Chomping::Keep {
        return end;
    }
    // The trailing break run (`content_end..span.end`, line breaks plus blank-line indentation).
    // Only the newlines the scalar's own output does NOT consume are the inter-item gap.
    let tail = f.context().source_text().bytes_range(block.content_end, block.span.end);
    // A handful of bytes; not worth a bytecount dependency
    #[expect(clippy::naive_bytecount)]
    let total_newlines = tail.iter().filter(|&&b| b == b'\n').count();
    let is_last_descendant = block.span.end >= f.context().last_descendant_end();
    let kept = consumed_trailing_newlines(total_newlines, is_last_descendant);
    // Anchor right after the `kept`-th newline of the tail
    // (the tail is span-bounded, so the saturation never fires).
    tail.iter()
        .enumerate()
        .filter(|&(_, &byte)| byte == b'\n')
        .take(kept)
        .last()
        .map_or(block.content_end, |(i, _)| {
            block.content_end + u32::try_from(i + 1).unwrap_or(u32::MAX)
        })
}
