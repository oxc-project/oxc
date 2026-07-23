use std::borrow::Cow;

use oxc_formatter_core::{
    Buffer,
    builders::{
        align, dedent, dedent_to_root, hard_line_break, literal_line_break, mark_as_root,
        soft_line_break_or_space, space, text,
    },
    write,
};
use oxc_yaml_parser::ast::{BlockScalar, Chomping, Content, MappingItem, Node, Root};

use crate::{
    comments::{Gap, classify_gap, write_single_comment},
    options::ProseWrap,
    print::{YamlFormatter, format_with, scalar::split_with_single_space, to_span},
};

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
    // Indicator comment: same line as the header (`| # comment`)
    if let Some(span) = f.context().comments().peek()
        && span.end <= block.content_start
        && classify_gap(f.context().source_text().bytes_range(block.span.start, span.start))
            == Gap::None
    {
        f.context().comments().take_before(span.end);
        write!(f, space());
        write_single_comment(span, f);
    }
    // Any other comments inside the header region are consumed silently
    // (they cannot be represented in a block scalar).
    let _ = f.context().comments().take_before(block.content_start);

    // Words fold to the arena lifetime: borrowed words already slice the arena-backed source,
    // only owned (merged) words need an arena copy.
    let line_groups: Vec<Vec<&'a str>> =
        block_value_line_contents(block, is_folded, parent_indent, is_last_descendant, f)
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

    // Blank runs are emitted as raw `\n` text followed by a hardline that only re-arms indentation
    // (consecutive hard lines would otherwise be collapsed by the printer,
    // same technique as `oxc_formatter_graphql`'s `write_block_string_break`).
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
                write!(f, text(crate::print::arena_newlines(blanks + 1, f)));
                write!(f, hard_line_break());
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
        // Trailing blank lines (kept by chomping / blank-line preservation),
        // plus the final newline for a keep-chomped last descendant.
        // The extra `\n` also covers the following hardline being collapsed.
        if blanks > 0 || (chomping_keep && is_last_descendant && wrote_any) {
            write!(f, dedent_to_root(&text(crate::print::arena_newlines(blanks + 1, f))));
        }
    });

    if let Some(indent) = block.indent {
        let width = u8::try_from(indent - 1 + parent_indent).unwrap_or(u8::MAX);
        write!(f, dedent_to_root(&align(width, &contents)));
    } else {
        let tab_width = f.options().indent_width.value();
        write!(f, dedent(&align(tab_width, &contents)));
    }

    // Claim any comments the scanner collected inside the scalar's range
    let _ = f.context().comments().take_before(block.span.end);
}

/// Ports Prettier's `getBlockValueLineContents`.
fn block_value_line_contents<'s>(
    block: &BlockScalar,
    is_folded: bool,
    parent_indent: u32,
    is_last_descendant: bool,
    f: &YamlFormatter<'_, 's>,
) -> Vec<Vec<Cow<'s, str>>> {
    if block.content_start >= block.span.end {
        return Vec::new();
    }
    let source = f.context().source_text();
    let content = source.text_for(&to_span(oxc_yaml_parser::Span {
        start: block.content_start,
        end: block.span.end,
    }));

    let raw_lines: Vec<&str> = content.split('\n').collect();

    // Leading indentation to strip from every line
    let leading_space_count = if let Some(indent) = block.indent {
        (indent - 1 + parent_indent) as usize
    } else {
        raw_lines
            .iter()
            .find_map(|l| {
                let spaces = l.len() - l.trim_start_matches(' ').len();
                // First line containing a non-space, non-CR character
                l[spaces..].chars().next().filter(|c| *c != '\r').map(|_| spaces)
            })
            .unwrap_or(usize::MAX)
    };

    let stripped: Vec<&str> =
        raw_lines.iter().map(|l| l.get(leading_space_count.min(l.len())..).unwrap_or("")).collect();

    let prose_wrap = f.options().prose_wrap;
    // Literal blocks (`|`) are never re-flowed; folded blocks only under `proseWrap` always/never.
    let no_reflow = prose_wrap == ProseWrap::Preserve || !is_folded;

    let lines: Vec<Vec<Cow<'s, str>>> = if no_reflow {
        stripped
            .iter()
            .map(|l| if l.is_empty() { vec![] } else { vec![Cow::Borrowed(*l)] })
            .collect()
    } else {
        fold_lines(&stripped, prose_wrap)
    };

    remove_unnecessary_trailing_newlines(block, content, is_last_descendant, lines)
}

fn fold_lines<'s>(stripped: &[&'s str], prose_wrap: ProseWrap) -> Vec<Vec<Cow<'s, str>>> {
    let mut lines: Vec<Vec<&'s str>> = Vec::with_capacity(stripped.len());
    for (index, line) in stripped.iter().enumerate() {
        // NOTE: a more-indented line keeps its line breaks literally per YAML folding,
        // so re-flowing it at the print width would change the parsed value (and break idempotency).
        // Prettier wraps it like any other paragraph; here it stays one unbreakable word.
        // The guards below already keep such a line in its own group, so this only removes the soft breaks inside it.
        if line.starts_with(char::is_whitespace) && !line.trim().is_empty() {
            lines.push(vec![*line]);
            continue;
        }
        let mut words = split_with_single_space(line).peekable();
        // NOTE: Prettier tests `/^\s|\s$/` against the previous array (a JS quirk, `Array::toString` joins with commas);
        // the effective check is on the previous group's first word start / last word end.
        let prev_group_has_boundary_space = lines.last().is_some_and(|prev| {
            prev.first().is_some_and(|w| w.starts_with(char::is_whitespace))
                || prev.last().is_some_and(|w| w.ends_with(char::is_whitespace))
        });
        let merge = index > 0
            && !line.is_empty()
            && !stripped[index - 1].is_empty()
            && !words.peek().is_some_and(|w| w.starts_with(char::is_whitespace))
            && !prev_group_has_boundary_space;
        if merge {
            lines.last_mut().unwrap().extend(words);
        } else {
            lines.push(words.collect());
        }
    }

    // Merge words into their predecessor when it ends with whitespace (trailing spaces are not allowed at a soft break)
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
        merged = merged.into_iter().map(|words| vec![Cow::Owned(words.join(" "))]).collect();
    }
    merged
}

/// Mirrors Prettier's `removeUnnecessaryTrailingNewlines`.
fn remove_unnecessary_trailing_newlines<'s>(
    block: &BlockScalar,
    content: &str,
    is_last_descendant: bool,
    mut lines: Vec<Vec<Cow<'s, str>>>,
) -> Vec<Vec<Cow<'s, str>>> {
    if block.chomping == Chomping::Keep {
        if content.ends_with('\n') && lines.last().is_some_and(Vec::is_empty) {
            lines.pop();
        }
        return lines;
    }

    let trailing_newline_count = lines
        .iter()
        .rev()
        .take_while(|words| words.iter().all(|w| w.trim_end_matches([' ', '\t']).is_empty()))
        .count();

    if trailing_newline_count == 0 {
        return lines;
    }
    let keep = if trailing_newline_count >= 2 && !is_last_descendant {
        // Preserve one blank line.
        lines.len() - (trailing_newline_count - 1)
    } else {
        lines.len() - trailing_newline_count
    };
    lines.truncate(keep);
    lines
}

/// Returns `true` when the stream's last descendant is a keep-chomped (`+`) block scalar.
/// Its verbatim content already ends with the kept newlines,
/// so the caller must not append the usual final `hard_line_break()`
/// (mirrors Prettier's `shouldPrintHardline` gate).
pub fn ends_with_keep_chomped_block(root: &Root<'_>) -> bool {
    root.children
        .last()
        .and_then(|document| document.body.content.as_deref())
        .and_then(last_descendant_block_scalar)
        .is_some_and(|block| block.chomping == Chomping::Keep)
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
        // Block scalars cannot appear inside flow collections.
        _ => None,
    }
}

/// Walks to the end offset of the stream's last descendant node
/// (Prettier's `getLastDescendantNode` on root, used by block scalars).
pub fn last_descendant_end(root: &Root<'_>) -> u32 {
    // Spans nest and every wrapper ends at its last descendant
    // (the parser's `container_span` / `MappingItem` / `Node` span construction),
    // so the last document body's span end IS the last descendant's end.
    root.children
        .last()
        .and_then(|document| document.body.content.as_deref())
        .map_or(0, |node| node.span.end)
}

/// The gap-measurement anchor after an item:
/// a block scalar's span consumes its trailing line breaks (they are part of its VALUE under keep chomping),
/// so blank-line detection must start after the last content character,
/// otherwise the blank line separating it from the next item is invisible.
pub fn item_gap_anchor(content: Option<&Node<'_>>, end: u32, f: &YamlFormatter<'_, '_>) -> u32 {
    let Some(block) = content.and_then(last_descendant_block_scalar) else {
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
