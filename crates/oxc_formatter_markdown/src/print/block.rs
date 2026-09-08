//! Block dispatch and the blank-line policy between siblings.

use oxc_formatter_core::{
    Buffer,
    builders::{
        align, group, hard_line_break, literal_line_break, mark_as_root, soft_line_break_or_space,
        space, text, token,
    },
    format_args,
    spec::is_suppression_marker,
    write,
};
use oxc_markdown_parser::ast::{
    Block, FootnoteDefinition, Heading, HeadingKind, HtmlBlock, Liquid, MathBlock,
};

use crate::{context::MarkdownFormatContext, options::ProseWrap};

use super::{
    MarkdownFormatter, blockquote, code, format_with, inline, join_pieces, link, list, table,
    with_depth, write_indented_lines, write_lines, write_verbatim,
};

/// What contains the blocks being written; the gap policy depends on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Parent {
    Root,
    /// `loose`: spread, or a blank line before the next item.
    ListItem {
        loose: bool,
    },
    /// Blockquote or container directive.
    Container,
}

/// Emits sibling blocks with a blank line between them, except where one line break is the rule
/// (tight list items, adjacent HTML, consecutive definitions, ...).
///
/// `<!-- prettier-ignore -->` (or `oxfmt-ignore`) freezes the next block;
/// `-start` / `-end` pairs freeze the range between them.
pub fn write_blocks<'a>(
    blocks: &'a [Block<'a>],
    parent: Parent,
    f: &mut MarkdownFormatter<'_, 'a>,
) {
    let mut index = 0;
    while index < blocks.len() {
        let block = &blocks[index];
        if index > 0 {
            write_gap(blocks, index, parent, f);
        }

        // `<!-- prettier-ignore-start -->` ... `<!-- prettier-ignore-end -->`: the range is source text.
        // Only at the document root; nested, the comments are plain HTML.
        if parent == Parent::Root
            && let Some(end) = ignore_range_end(blocks, index, f)
        {
            let source = f.context().source_text();
            let start_comment = block.span();
            let end_comment = blocks[end].span();
            write_verbatim(start_comment, f);
            write!(f, text(source.slice_range(start_comment.end, end_comment.start)));
            write_verbatim(end_comment, f);
            index = end + 1;
            continue;
        }

        write_block(block, blocks, index, parent, f);
        index += 1;
    }
}

pub fn write_gap(
    siblings: &[Block<'_>],
    index: usize,
    parent: Parent,
    f: &mut MarkdownFormatter<'_, '_>,
) {
    super::write_gap(double_gap(siblings, index, parent, f), f);
}

/// Whether `siblings[index]` gets a blank line before it.
fn double_gap(
    siblings: &[Block<'_>],
    index: usize,
    parent: Parent,
    f: &MarkdownFormatter<'_, '_>,
) -> bool {
    let (previous, node) = (&siblings[index - 1], &siblings[index]);
    let ctx = f.context();
    let adjacent = !ctx.has_blank_between(previous.span().end, node.span().start);

    // ```
    // [a]: b
    // [c]: d
    // Text
    // ===
    // ```
    // mdast starts the heading at the first definition (they were the heading paragraph's first lines),
    // and Prettier keeps them together only when that start lies on an earlier line than the last definition's end:
    // the run has two definitions, or its one definition spans lines.
    if adjacent
        && matches!(previous, Block::Definition(_))
        && matches!(node, Block::Heading(h) if matches!(h.kind, HeadingKind::Setext { .. }))
    {
        let after_definition = index >= 2
            && matches!(siblings[index - 2], Block::Definition(_))
            && !ctx.has_blank_between(siblings[index - 2].span().end, previous.span().start);
        let multi_line =
            ctx.source_text().bytes_contain(previous.span().start, previous.span().end, b'\n');
        return !after_definition && !multi_line;
    }

    // A nested list after a paragraph / code keeps its blank line (prettier#17746)
    if matches!(node, Block::List(_))
        && matches!(parent, Parent::ListItem { .. })
        && matches!(previous, Block::CodeBlock(_) | Block::Paragraph(_))
        && !adjacent
    {
        return true;
    }

    let is_sibling_node = matches!((previous, node), (Block::Definition(_), Block::Definition(_)));
    let in_tight_list_item = match parent {
        Parent::ListItem { loose } => matches!(node, Block::List(_)) || !loose,
        _ => false,
    };
    let prev_is_ignore = ignore_kind(previous, f) == Some(IgnoreKind::Next);
    let html_after_html = matches!((previous, node), (Block::HtmlBlock(_), Block::HtmlBlock(_)));
    let html_after_paragraph =
        matches!((previous, node), (Block::Paragraph(_), Block::HtmlBlock(_)));
    let liquid = matches!(previous, Block::Liquid(_)) || matches!(node, Block::Liquid(_));

    !(is_sibling_node
        || in_tight_list_item
        || prev_is_ignore
        || (adjacent && (html_after_html || html_after_paragraph || liquid)))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum IgnoreKind {
    Next,
    Start,
    End,
}

/// `<!-- prettier-ignore -->` / `<!-- oxfmt-ignore -->` and their `-start` / `-end` forms as a whole HTML block.
/// Asked several times per block, so it bails on the first byte before joining anything.
fn ignore_kind(block: &Block<'_>, f: &MarkdownFormatter<'_, '_>) -> Option<IgnoreKind> {
    let Block::HtmlBlock(html) = block else { return None };

    let source = f.context().source_text().as_str();
    let (first, last) = (html.lines.first()?, html.lines.last()?);
    if !first.span.slice(source).starts_with("<!--") || !last.span.slice(source).ends_with("-->") {
        return None;
    }
    let value = join_pieces(&html.lines, f);
    let inner = value.strip_prefix("<!--")?.strip_suffix("-->")?.trim();
    if is_suppression_marker(inner) {
        return Some(IgnoreKind::Next);
    }
    if let Some(marker) = inner.strip_suffix("-start")
        && is_suppression_marker(marker)
    {
        return Some(IgnoreKind::Start);
    }
    if let Some(marker) = inner.strip_suffix("-end")
        && is_suppression_marker(marker)
    {
        return Some(IgnoreKind::End);
    }
    None
}

/// The index of the `-end` comment closing an ignore range opened at `index`, if any.
fn ignore_range_end(
    blocks: &[Block<'_>],
    index: usize,
    f: &MarkdownFormatter<'_, '_>,
) -> Option<usize> {
    if ignore_kind(&blocks[index], f) != Some(IgnoreKind::Start) {
        return None;
    }
    (index + 1..blocks.len()).find(|&i| ignore_kind(&blocks[i], f) == Some(IgnoreKind::End))
}

/// `<!-- prettier-ignore -->` right before `siblings[index]`: the block is printed as source text.
pub fn is_ignored(siblings: &[Block<'_>], index: usize, f: &MarkdownFormatter<'_, '_>) -> bool {
    index > 0 && ignore_kind(&siblings[index - 1], f) == Some(IgnoreKind::Next)
}

/// `siblings[index]` is `block`; some blocks look at their neighbors.
pub fn write_block<'a>(
    block: &'a Block<'a>,
    siblings: &'a [Block<'a>],
    index: usize,
    parent: Parent,
    f: &mut MarkdownFormatter<'_, 'a>,
) {
    if is_ignored(siblings, index, f) {
        write_verbatim(block.span(), f);
        return;
    }
    match block {
        Block::Paragraph(p) => {
            let parent = inline::InlineParent {
                paragraph: true,
                first_in_container: parent == Parent::Container && index == 0,
                ..inline::InlineParent::default()
            };
            inline::write_inlines(&p.children, parent, f);
        }
        Block::Heading(h) => write_heading(h, f),
        Block::ThematicBreak(_) => write_thematic_break(index, parent, f),
        Block::CodeBlock(c) => code::write_code_block(c, f),
        Block::HtmlBlock(h) => {
            let is_last_in_root = parent == Parent::Root && index + 1 == siblings.len();
            write_html_block(h, is_last_in_root, f);
        }
        Block::Blockquote(q) => blockquote::write_blockquote(q, f),
        Block::List(l) => list::write_list(l, siblings, index, f),
        Block::Definition(d) => link::write_definition(d, f),
        Block::MathBlock(m) => write_math_block(m, f),
        Block::Liquid(l) => write_liquid_block(l, f),
        Block::ContainerDirective(d) => {
            // Fence lines are verbatim (dialects disagree on their grammar); children are markdown
            write_verbatim(d.opening, f);
            if !d.children.is_empty() {
                write!(f, hard_line_break());
                write_blocks(&d.children, Parent::Container, f);
            }
            if let Some(closing) = d.closing {
                write!(f, hard_line_break());
                write_verbatim(closing, f);
            }
        }
        Block::Table(t) => table::write_table(t, f),
        Block::FootnoteDefinition(d) => write_footnote_definition(d, f),
        // MDX stays verbatim
        Block::MdxEsm(_) | Block::MdxExpression(_) | Block::MdxJsx(_) => {
            write_verbatim(block.span(), f);
        }
    }
}

/// ATX headings normalize to `#`s + one space;
/// setext headings keep their underline verbatim (length included).
fn write_heading<'a>(heading: &'a Heading<'a>, f: &mut MarkdownFormatter<'_, 'a>) {
    match &heading.kind {
        HeadingKind::Atx => {
            const HASHES: &str = "######";
            // `space` dies at the line break, so an empty heading is just its `#`s
            write!(f, [text(&HASHES[..usize::from(heading.level)]), space()]);
            // An ATX heading never wraps
            with_depth(f, MarkdownFormatContext::no_wrap_depth, |f| {
                inline::write_inlines(&heading.children, inline::InlineParent::default(), f);
            });
        }
        HeadingKind::Setext { underline, .. } => {
            inline::write_inlines(&heading.children, inline::InlineParent::default(), f);
            let raw = f.context().slice(*underline);
            write!(f, [hard_line_break(), text(raw.trim_end())]);
        }
    }
}

/// `[^label]: ` + content.
/// A lone paragraph stays on the label's line under `never` (and under `preserve` when it is one source line);
/// otherwise the content is aligned by 4 and the first block moves to its own line when it does not fit
/// (or is multi-line).
fn write_footnote_definition<'a>(
    def: &'a FootnoteDefinition<'a>,
    f: &mut MarkdownFormatter<'_, 'a>,
) {
    write!(f, [token("[^"), text(f.context().slice(def.label)), token("]:")]);
    let children = &def.children;
    let inline = match children.as_slice() {
        [Block::Paragraph(p)] => match f.options().prose_wrap {
            ProseWrap::Never => true,
            ProseWrap::Preserve => {
                !f.context().source_text().bytes_contain(p.span.start, p.span.end, b'\n')
            }
            ProseWrap::Always => false,
        },
        _ => false,
    };
    if inline {
        write!(f, space());
        write_blocks(children, Parent::Container, f);
        return;
    }
    let parent = Parent::Container;
    write!(
        f,
        align(
            4,
            &format_with(|f| {
                let first = format_with(|f| write_block(&children[0], children, 0, parent, f));
                write!(f, group(&format_args!(soft_line_break_or_space(), first)));
                for index in 1..children.len() {
                    write_gap(children, index, parent, f);
                    write_block(&children[index], children, index, parent, f);
                }
            })
        )
    );
}

/// `---`, except inside a list
/// (alternating with `***` by the list's sibling parity so the marker never collides with a bullet)
/// and as the first block of the document (`***`, so it never reads as front matter).
fn write_thematic_break(index: usize, parent: Parent, f: &mut MarkdownFormatter<'_, '_>) {
    let frame = f.context().lists().borrow().last().copied();
    let star = match frame {
        None => index == 0 && parent == Parent::Root,
        Some(frame) => !frame.alternate_marker,
    };
    write!(f, token(if star { "***" } else { "---" }));
}

/// HTML blocks are verbatim, line by line.
///
/// Comments re-indent at the current indention;
/// anything else goes through literal lines anchored at the current indention, which never trim.
fn write_html_block<'a>(
    html: &'a HtmlBlock<'a>,
    is_last_in_root: bool,
    f: &mut MarkdownFormatter<'_, 'a>,
) {
    let mut value = join_pieces(&html.lines, f);
    if is_last_in_root {
        value = value.trim_end();
    }
    if is_html_comment(value) {
        write_indented_lines(value, f);
        return;
    }
    write!(
        f,
        mark_as_root(&format_with(|f| {
            write_lines(value.split('\n'), literal_line_break(), f);
            // micromark keeps the closing line ending as content in a few shapes;
            // Prettier prints it, so the block gains an extra line.
            if html.trailing_newline && !is_last_in_root {
                write!(f, literal_line_break());
            }
        }))
    );
}

/// `$$` fences are normalized; the content is verbatim.
fn write_math_block<'a>(math: &'a MathBlock<'a>, f: &mut MarkdownFormatter<'_, 'a>) {
    write!(f, token("$$"));
    if let Some(meta) = math.meta {
        let raw = f.context().slice(meta);
        write!(f, [token(" "), text(raw)]);
    }
    write!(f, hard_line_break());
    if !math.lines.is_empty() {
        write_indented_lines(join_pieces(&math.lines, f), f);
        write!(f, hard_line_break());
    }
    write!(f, token("$$"));
}

/// A `{% %}` / `{{ }}` tag standing alone is verbatim, line by line.
fn write_liquid_block<'a>(liquid: &'a Liquid<'a>, f: &mut MarkdownFormatter<'_, 'a>) {
    write_indented_lines(join_pieces(&liquid.pieces, f), f);
}

/// Prettier's comment test for HTML values: `<!--` ... `-->` as the whole value.
pub fn is_html_comment(value: &str) -> bool {
    value.starts_with("<!--") && value.ends_with("-->")
}
