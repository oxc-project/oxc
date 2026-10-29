//! The printer: dispatch by node kind; elements, text and blocks get their own module,
//! besides the separators between siblings (`children`), the tag markers (`tag`) and attributes (`attribute`).

use oxc_formatter_core::{
    Format, Formatter,
    builders::{FormatWith, group, soft_space_or_block_indent, space, text, token},
    write,
};
use oxc_htmlish_parser::ast::{NodeId, NodeKind};

use crate::{context::HtmlishFormatContext, tree::Tree};

mod attribute;
mod block;
mod children;
mod element;
mod embed;
mod tag;
mod text;

pub use embed::{CssInHtmlStyleAttribute, JsInHtmlEventHandler};

use block::write_block;
use children::write_children;
use element::write_element;
use tag::{
    after_html_word, closing_end_borrowed, closing_end_marker, closing_tag_suffix,
    opening_start_borrowed, opening_start_marker, opening_tag_prefix, write_closing_tag_end,
    write_opening_tag_start,
};
use text::{write_dedented_lines, write_text};

pub type HtmlishFormatter<'buf, 'a> = Formatter<'buf, 'a, HtmlishFormatContext<'a>>;

/// `Format` impl for `&'static str` specialized to `HtmlishFormatContext`.
impl<'a> Format<'a, HtmlishFormatContext<'a>> for &'static str {
    #[inline]
    fn fmt(&self, f: &mut HtmlishFormatter<'_, 'a>) {
        write!(f, token(self));
    }
}

/// Wraps a re-entrant closure in a [`FormatWith`], its context pinned to [`HtmlishFormatContext`].
#[inline]
pub const fn format_with<'a, T>(formatter: T) -> FormatWith<T>
where
    T: Fn(&mut HtmlishFormatter<'_, 'a>),
{
    FormatWith::new(formatter)
}

pub fn write_root(f: &mut HtmlishFormatter<'_, '_>) {
    let t = f.context().tree();
    write!(f, group(&format_with(|f| write_children(t, NodeId::DOCUMENT, false, f))));
}

fn write_child<'a>(t: &'a Tree<'a>, id: NodeId, f: &mut HtmlishFormatter<'_, 'a>) {
    if !t[id].prettier_ignore {
        write_node(t, id, f);
        return;
    }
    let mut start = t[id].span.start as usize;
    if opening_start_borrowed(t, id) {
        start += opening_start_marker(t, id).len();
    }
    let mut end = ignored_end(t, id) as usize;
    // Whoever borrows the closing `>` prints it,
    // the parent included (DIVERGENCES.md#ignored-closing-end-once)
    if closing_end_borrowed(t, id) {
        end -= closing_end_marker(t, id).len();
    }
    let raw = t.source()[start..end].trim_ascii_end();
    write!(f, [opening_tag_prefix(t, id), text(raw), closing_tag_suffix(t, id)]);
}

/// Where an ignored node's source ends: an unclosed element extends to its last child.
fn ignored_end(t: &Tree<'_>, id: NodeId) -> u32 {
    let end = t.source_end(id);
    match t.kind(id) {
        NodeKind::Element(el) if el.end_tag.is_none() => {
            t[id].last_child.map_or(end, |last| end.max(ignored_end(t, last)))
        }
        _ => end,
    }
}

fn write_node<'a>(t: &'a Tree<'a>, id: NodeId, f: &mut HtmlishFormatter<'_, 'a>) {
    let item = &t[id];
    match t.kind(id) {
        NodeKind::Element(_) | NodeKind::ConditionalComment(_) => write_element(t, id, f),
        NodeKind::ConditionalStart(_) | NodeKind::ConditionalEnd => {
            write_opening_tag_start(t, id, f);
            write_closing_tag_end(t, id, f);
        }
        NodeKind::Text(_) | NodeKind::Cdata => write_text(t, id, f),
        // `{{ }}`; the expression keeps its lines until it is formatted
        NodeKind::Interpolation(interpolation) => {
            write_opening_tag_start(t, id, f);
            let expression = interpolation.expression.slice(t.source());
            if !expression.trim_ascii().is_empty() {
                write!(
                    f,
                    soft_space_or_block_indent(&format_with(|f| write_dedented_lines(
                        expression, f
                    )))
                );
            }
            write_closing_tag_end(t, id, f);
        }
        NodeKind::DocType(doctype) => {
            let value = t.collapsed(doctype.value);
            write!(
                f,
                group(&format_with(|f| {
                    write_opening_tag_start(t, id, f);
                    write!(f, space());
                    // Its leading `html` lowercased (`<!DOCTYPE HTML PUBLIC …>`)
                    if let Some(rest) = after_html_word(value) {
                        write!(f, [text("html"), text(rest)]);
                    } else {
                        write!(f, text(value));
                    }
                }))
            );
            write_closing_tag_end(t, id, f);
        }
        // Comments are printed as written,
        // a `</ div>` too (DIVERGENCES.md#end-tag-with-space-is-comment), and an omitted element's end tag (`</body>`).
        NodeKind::Comment(_) | NodeKind::EndTag(_) => write!(
            f,
            [
                opening_tag_prefix(t, id),
                text(item.span.slice(t.source())),
                closing_tag_suffix(t, id)
            ]
        ),
        NodeKind::Block(block) => write_block(t, id, block, f),
        NodeKind::Document => unreachable!("the root is written by `write_root`"),
    }
}
