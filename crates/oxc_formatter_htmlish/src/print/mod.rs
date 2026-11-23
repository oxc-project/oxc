//! The printer: dispatch by node kind; elements, text and blocks get their own module,
//! besides the separators between siblings (`children`), the tag markers (`tag`) and attributes (`attribute`).

use oxc_allocator::ArenaVec;
use oxc_formatter_core::{
    Format, FormatElement, Formatter,
    builders::{FormatOnce, FormatWith, group, space, text, token},
    write,
};
use oxc_htmlish_parser::ast::{Interpolation, NodeId, NodeKind};

use crate::{context::HtmlishFormatContext, rules::TemplateExpressions, tree::Tree};

mod angular;
mod attribute;
mod block;
mod children;
mod element;
mod embed;
mod tag;
mod text;
mod vue;

pub use angular::JsInAngularTemplate;
pub use embed::{CssInHtmlStyleAttribute, JsInHtmlEventHandler};
pub use vue::JsInVueTemplate;

use block::write_block;
use children::write_children;
use element::write_element;
use tag::{
    after_html_word, closing_end_borrowed, closing_end_marker, closing_tag_suffix,
    opening_start_borrowed, opening_start_marker, opening_tag_prefix, write_closing_tag_end,
    write_opening_tag_start,
};
use text::write_text;

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

/// Wraps a closure run once in a [`FormatOnce`], its context pinned to [`HtmlishFormatContext`].
#[inline]
pub const fn format_once<'a, T>(formatter: T) -> FormatOnce<T>
where
    T: FnOnce(&mut HtmlishFormatter<'_, 'a>),
{
    FormatOnce::new(formatter)
}

/// Writes a child's IR (a dispatched part), once.
pub fn format_ir<'a>(
    ir: ArenaVec<'a, FormatElement<'a>>,
) -> FormatOnce<impl FnOnce(&mut HtmlishFormatter<'_, 'a>)> {
    FormatOnce::new(move |f: &mut HtmlishFormatter<'_, 'a>| f.write_elements(ir))
}

pub fn write_root(f: &mut HtmlishFormatter<'_, '_>) {
    let t = f.context().tree();
    write!(f, group(&format_with(|f| write_children(t, NodeId::DOCUMENT, false, f))));
}

fn write_child<'a>(t: &'a Tree<'a>, id: NodeId, f: &mut HtmlishFormatter<'_, 'a>) {
    if !t[id].suppressed {
        write_node(t, id, f);
        return;
    }
    let mut start = t[id].span.start as usize;
    if opening_start_borrowed(t, id) {
        start += opening_start_marker(t, id).len();
    }
    let mut end = end_with_content(t, id) as usize;
    // Whoever borrows the closing `>` prints it,
    // the parent included (DIVERGENCES.md#ignored-closing-end-once)
    if closing_end_borrowed(t, id) {
        end -= closing_end_marker(t, id).len();
    }
    let raw = t.source()[start..end].trim_ascii_end();
    write!(f, [opening_tag_prefix(t, id), text(raw), closing_tag_suffix(t, id)]);
}

/// Where a node's source ends, its content included: an unclosed element extends to its last child.
fn end_with_content(t: &Tree<'_>, id: NodeId) -> u32 {
    let end = t.source_end(id);
    match t.kind(id) {
        NodeKind::Element(el) if el.end_tag.is_none() => {
            t[id].last_child.map_or(end, |last| end.max(end_with_content(t, last)))
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
        NodeKind::Text(_) => write_text(t, id, f),
        NodeKind::Interpolation(interpolation) => {
            let expression = Interpolation::expression(item.span).slice(t.source());
            // TODO: A character reference is not checked, unlike in an attribute's value (`embed::dispatch_attribute_value`):
            // Vue and Angular decode it before they read the code, so `{{ '&quot;' }}` printed `"&quot;"` reads `"""`.
            // Left for now as rare (one, harmless, in the corpus), and the check's cost unmeasured
            // Each its own type, borrowed as one
            let (vue, angular);
            let formatted: Option<&dyn Format<'a, _>> = match t.rules().template_expressions {
                TemplateExpressions::Vue => {
                    vue = vue::dispatch_interpolation(t, expression, f).map(format_ir);
                    vue.as_ref().map(|ir| ir as _)
                }
                TemplateExpressions::Angular => {
                    angular = angular::format_interpolation(interpolation.angular.as_deref(), f);
                    angular.as_ref().map(|ir| ir as _)
                }
                // The parser reads `{{ }}` in Vue and Angular only
                TemplateExpressions::None => None,
            };
            write_opening_tag_start(t, id, f);
            embed::write_interpolation(t, id, expression, formatted, f);
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
        NodeKind::LetDeclaration(declaration) => {
            write!(f, opening_tag_prefix(t, id));
            if !angular::write_let(declaration, f) {
                write!(f, text(item.span.slice(t.source())));
            }
            write!(f, closing_tag_suffix(t, id));
        }
        // Comments and CDATA sections are printed as written,
        // a `</ div>` too (DIVERGENCES.md#end-tag-with-space-is-comment), and an omitted element's end tag (`</body>`).
        // An Angular ICU expression too, whose message ID hashes it (DIVERGENCES.md#angular-i18n-content-as-written).
        NodeKind::Comment(_)
        | NodeKind::Cdata
        | NodeKind::EndTag(_)
        | NodeKind::Icu(_)
        | NodeKind::IcuCase(_) => write!(
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
