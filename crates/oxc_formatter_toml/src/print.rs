//! Builds the IR from `oxc-toml`'s CST, see AGENTS.md "Layout".

use oxc_formatter_core::{
    Formatter, SourceText, arena_cow_str,
    builders::{
        FormatWith, empty_line, group, hard_line_break, if_group_breaks, indent, soft_line_break,
        soft_line_break_or_space, space, text, token,
    },
    normalize_newlines,
    spec::{Gap, classify_gap, is_suppression_marker},
    write,
};
use oxc_toml::{
    SyntaxElement,
    SyntaxKind::{
        ARRAY, BRACE_START, BRACKET_START, COMMA, COMMENT, KEY, NEWLINE, TABLE_ARRAY_HEADER,
        TABLE_HEADER, WHITESPACE,
    },
    SyntaxNode, TextRange,
};

use crate::{context::TomlFormatContext, options::TrailingCommas};

pub type TomlFormatter<'buf, 'a> = Formatter<'buf, 'a, TomlFormatContext<'a>>;

/// Wraps a re-entrant TOML closure in a [`FormatWith`],
/// pinning its context so call sites don't have to annotate it.
#[inline]
const fn format_with<'a, T>(formatter: T) -> FormatWith<T>
where
    T: Fn(&mut TomlFormatter<'_, 'a>),
{
    FormatWith::new(formatter)
}

/// A line of the root or of a multi-line array.
enum Item<'n, 'a> {
    Entry(&'n SyntaxNode),
    Header(&'n SyntaxNode),
    /// An array value, with the comment trailing it on its line.
    Value {
        node: &'n SyntaxNode,
        comment: Option<&'a str>,
        last: bool,
    },
    /// A comment on its own line.
    Comment(&'a str),
}

impl Item<'_, '_> {
    fn is_suppression(&self) -> bool {
        matches!(self, Self::Comment(c) if is_suppression_marker(c.strip_prefix('#').unwrap_or(c)))
    }
}

struct Line<'n, 'a> {
    item: Item<'n, 'a>,
    /// A blank line (whitespace-only lines included) separates it from the previous line.
    blank_before: bool,
}

/// Collects the lines in source order.
struct Lines<'n, 'a> {
    rows: Vec<Line<'n, 'a>>,
    source: SourceText<'a>,
    /// Where the previous line's item ends
    prev_end: Option<u32>,
}

impl<'n, 'a> Lines<'n, 'a> {
    const fn new(source: SourceText<'a>) -> Self {
        Self { rows: Vec::new(), source, prev_end: None }
    }

    fn push(&mut self, item: Item<'n, 'a>, range: &TextRange) {
        // No blank line at the start, nor between a table header and its first entry
        let after_header = matches!(self.rows.last(), Some(Line { item: Item::Header(_), .. }));
        let blank_before = !(after_header && matches!(item, Item::Entry(_)))
            && self.prev_end.is_some_and(|end| {
                classify_gap(self.source.bytes_range(end, range.start)) == Gap::Blank
            });
        self.rows.push(Line { item, blank_before });
        self.prev_end = Some(range.end);
    }

    /// Writes the lines, each after a line break (`empty_line` for a blank one) but the first.
    ///
    /// `write_item` is told whether the previous line is a suppression comment, to print the item as written.
    fn write(
        &self,
        f: &mut TomlFormatter<'_, 'a>,
        write_item: impl Fn(&Item<'n, 'a>, bool, &mut TomlFormatter<'_, 'a>),
    ) {
        for (i, line) in self.rows.iter().enumerate() {
            if i != 0 {
                if line.blank_before {
                    write!(f, empty_line());
                } else {
                    write!(f, hard_line_break());
                }
            }
            let suppressed = i != 0 && self.rows[i - 1].item.is_suppression();
            write_item(&line.item, suppressed, f);
        }
    }
}

/// A comment's text, without trailing whitespace.
fn comment_text<'a>(element: &SyntaxElement, source: &'a str) -> &'a str {
    element.text(source).trim_end()
}

/// Writes `node` as written (`# prettier-ignore`).
fn write_verbatim(node: &SyntaxNode, f: &mut TomlFormatter<'_, '_>) {
    let source = f.context().source_text().as_str();
    write_source_text(node.text(source).trim_end(), f);
}

/// Source text prints as written, but its line breaks follow `endOfLine`.
fn write_source_text<'a>(source_text: &'a str, f: &mut TomlFormatter<'_, 'a>) {
    let normalized = normalize_newlines(source_text, ['\r']);
    write!(f, text(arena_cow_str(&normalized, f)));
}

fn write_trailing_comment<'a>(comment: Option<&'a str>, f: &mut TomlFormatter<'_, 'a>) {
    if let Some(comment) = comment {
        write!(f, [space(), text(comment)]);
    }
}

// ---

/// Writes the root: entries, table headers and comments, one per line, keeping up to 1 blank line.
///
/// `final_newline: false` leaves the line after the last one to the parent (embedded use).
pub fn write_root(root: &SyntaxNode, final_newline: bool, f: &mut TomlFormatter<'_, '_>) {
    let source = f.context().source_text();
    let mut lines = Lines::new(source);
    for element in root.children_with_tokens() {
        let item = match element {
            SyntaxElement::Node(node)
                if matches!(node.kind(), TABLE_HEADER | TABLE_ARRAY_HEADER) =>
            {
                Item::Header(node)
            }
            SyntaxElement::Node(node) => Item::Entry(node),
            SyntaxElement::Token(t) if t.kind() == COMMENT => {
                Item::Comment(comment_text(element, source.as_str()))
            }
            SyntaxElement::Token(_) => continue,
        };
        lines.push(item, &element.text_range());
    }

    lines.write(f, |item, suppressed, f| match item {
        // A suppressed entry keeps its comment as written too
        Item::Entry(node) | Item::Header(node) if suppressed => write_verbatim(node, f),
        Item::Entry(node) => {
            let comment = write_entry(node, f);
            write_trailing_comment(comment, f);
        }
        Item::Header(node) => write_header(node, f),
        Item::Comment(comment) => write!(f, text(comment)),
        Item::Value { .. } => unreachable!("only arrays hold values"),
    });
    // A whitespace-only file formats to empty, like the other languages
    if final_newline && !lines.rows.is_empty() {
        write!(f, hard_line_break());
    }
}

/// Writes `[key]` or `[[key]]` with its trailing comment.
fn write_header(node: &SyntaxNode, f: &mut TomlFormatter<'_, '_>) {
    let source = f.context().source_text().as_str();
    let mut comment = None;
    for element in node.children_with_tokens() {
        match element {
            SyntaxElement::Node(key) => write_key(key, f),
            SyntaxElement::Token(t) => match t.kind() {
                WHITESPACE | NEWLINE => {}
                COMMENT => comment = Some(comment_text(element, source)),
                _ => write!(f, text(t.text(source))),
            },
        }
    }
    write_trailing_comment(comment, f);
}

/// Writes the key's parts without the whitespace around its dots.
fn write_key(node: &SyntaxNode, f: &mut TomlFormatter<'_, '_>) {
    let source = f.context().source_text().as_str();
    for element in node.children_with_tokens() {
        if let SyntaxElement::Token(t) = element
            && !matches!(t.kind(), WHITESPACE | NEWLINE)
        {
            write!(f, text(t.text(source)));
        }
    }
}

/// Writes `key = value`, returning the entry's comment.
fn write_entry<'a>(node: &SyntaxNode, f: &mut TomlFormatter<'_, 'a>) -> Option<&'a str> {
    let source = f.context().source_text().as_str();
    let mut comment = None;
    for element in node.children_with_tokens() {
        match element {
            SyntaxElement::Node(n) if n.kind() == KEY => {
                write_key(n, f);
                write!(f, token(" = "));
            }
            SyntaxElement::Node(n) => write_value(n, f),
            SyntaxElement::Token(t) if t.kind() == COMMENT => {
                comment = comment.or(Some(comment_text(element, source)));
            }
            SyntaxElement::Token(_) => {}
        }
    }
    comment
}

/// Writes a `VALUE` node.
fn write_value(node: &SyntaxNode, f: &mut TomlFormatter<'_, '_>) {
    let source = f.context().source_text().as_str();
    for element in node.children_with_tokens() {
        match element {
            SyntaxElement::Node(n) if n.kind() == ARRAY => write_array(n, f),
            SyntaxElement::Node(n) => write_inline_table(n, f),
            SyntaxElement::Token(t) => match t.kind() {
                NEWLINE | WHITESPACE => {}
                // The trivia after a scalar is the parent's (`oxc-toml` >= 0.15.2)
                COMMENT => debug_assert!(false, "a comment in a `VALUE` node would be dropped"),
                _ => write_source_text(t.text(source), f),
            },
        }
    }
}

// ---

fn write_array(node: &SyntaxNode, f: &mut TomlFormatter<'_, '_>) {
    let source = f.context().source_text();
    let trailing_comma = f.options().trailing_commas == TrailingCommas::Always;

    let mut lines = Lines::new(source);
    // A comment right after the `[`, on its line
    let mut bracket_comment = None;
    // The end of the last `[`, value, `,` or comment, to tell whether a comment is on its own line
    let mut token_end = 0;
    for element in node.children_with_tokens() {
        let range = element.text_range();
        match element {
            SyntaxElement::Node(value) => {
                lines.push(Item::Value { node: value, comment: None, last: false }, &range);
            }
            SyntaxElement::Token(t) => match t.kind() {
                BRACKET_START | COMMA => {}
                COMMENT => {
                    let comment = comment_text(element, source.as_str());
                    let on_own_line =
                        classify_gap(source.bytes_range(token_end, range.start)) != Gap::None;
                    match lines.rows.last_mut() {
                        Some(Line {
                            item: Item::Value { comment: trailing @ None, .. }, ..
                        }) if !on_own_line => {
                            *trailing = Some(comment);
                        }
                        None if !on_own_line => bracket_comment = Some(comment),
                        // On its own line, or after a value that already has one (`1 # a` + newline + `, # b`)
                        _ => lines.push(Item::Comment(comment), &range),
                    }
                }
                _ => continue,
            },
        }
        token_end = range.end;
    }

    if lines.rows.is_empty() && bracket_comment.is_none() {
        write!(f, token("[]"));
        return;
    }
    if let Some(Item::Value { last, .. }) = lines
        .rows
        .iter_mut()
        .rev()
        .map(|line| &mut line.item)
        .find(|item| matches!(item, Item::Value { .. }))
    {
        *last = true;
    }

    if has_comment(node) {
        // A comment runs to the end of its line, so the array keeps one value or comment per line
        let content = format_with(|f| {
            lines.write(f, |item, suppressed, f| match item {
                Item::Value { node, comment, last } => {
                    if suppressed {
                        write_verbatim(node, f);
                    } else {
                        write_value(node, f);
                    }
                    if !last || trailing_comma {
                        write!(f, token(","));
                    }
                    write_trailing_comment(*comment, f);
                }
                Item::Comment(comment) => write!(f, text(comment)),
                Item::Entry(_) | Item::Header(_) => unreachable!("arrays hold values"),
            });
        });
        write!(f, token("["));
        write_trailing_comment(bracket_comment, f);
        write!(
            f,
            [
                indent(&format_with(|f| write!(f, [hard_line_break(), content]))),
                hard_line_break(),
                token("]")
            ]
        );
        return;
    }

    // `[a, b]` when it fits, one value per line otherwise
    write!(
        f,
        group(&format_with(|f| {
            write!(f, token("["));
            write!(
                f,
                indent(&format_with(|f| {
                    write!(f, soft_line_break());
                    for (index, line) in lines.rows.iter().enumerate() {
                        if index != 0 {
                            // A blank line only when it expands, without making it expand
                            if line.blank_before {
                                write!(f, if_group_breaks(&empty_line().without_expand_parent()));
                            }
                            write!(f, soft_line_break_or_space());
                        }
                        if let Item::Value { node, last, .. } = line.item {
                            write_value(node, f);
                            if !last {
                                write!(f, token(","));
                            } else if trailing_comma {
                                write!(f, if_group_breaks(&token(",")));
                            }
                        }
                    }
                }))
            );
            write!(f, [soft_line_break(), token("]")]);
        }))
    );
}

// ---

fn write_inline_table(node: &SyntaxNode, f: &mut TomlFormatter<'_, '_>) {
    if has_own_comment(node) {
        write_multiline_inline_table(node, f);
        return;
    }

    let mut entries = node.children().iter().filter_map(SyntaxElement::as_node).peekable();
    if entries.peek().is_none() {
        write!(f, token("{}"));
        return;
    }
    write!(f, token("{ "));
    for (i, entry) in entries.enumerate() {
        if i != 0 {
            write!(f, token(", "));
        }
        write_entry(entry, f);
    }
    write!(f, token(" }"));
}

/// Whether `node` has a comment anywhere inside.
///
/// NOTE: Each nested array walks its subtree again (depth x size), shallow in TOML.
fn has_comment(node: &SyntaxNode) -> bool {
    node.children().iter().any(|e| match e {
        SyntaxElement::Node(n) => has_comment(n),
        SyntaxElement::Token(t) => t.kind() == COMMENT,
    })
}

/// Whether the inline table has a comment of its own, not one inside a value.
fn has_own_comment(node: &SyntaxNode) -> bool {
    node.children().iter().any(|e| e.kind() == COMMENT)
}

/// A comment runs to the end of its line, so joining the lines would comment out the rest:
/// one entry or own-line comment per line (TOML 1.1), blank lines dropped.
fn write_multiline_inline_table(node: &SyntaxNode, f: &mut TomlFormatter<'_, '_>) {
    let source = f.context().source_text();
    let entry_count = node.children().iter().filter_map(SyntaxElement::as_node).count();
    let content = format_with(|f| {
        let mut entry_index = 0;
        // The end of the last `{`, entry, `,` or comment, to tell whether a comment is on its own line
        let mut token_end = 0;
        for element in node.children_with_tokens() {
            let range = element.text_range();
            match element {
                SyntaxElement::Node(entry) => {
                    write!(f, hard_line_break());
                    write_entry(entry, f);
                    entry_index += 1;
                    if entry_index < entry_count {
                        write!(f, token(","));
                    }
                }
                SyntaxElement::Token(t) => match t.kind() {
                    BRACE_START | COMMA => {}
                    COMMENT => {
                        if classify_gap(source.bytes_range(token_end, range.start)) == Gap::None {
                            write!(f, space());
                        } else {
                            write!(f, hard_line_break());
                        }
                        write!(f, text(comment_text(element, source.as_str())));
                    }
                    _ => continue,
                },
            }
            token_end = range.end;
        }
    });
    write!(f, [token("{"), indent(&content), hard_line_break(), token("}")]);
}
