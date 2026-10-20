//! What separates siblings (Prettier's `printChildren`), and the line break facts it reads from the source.

use oxc_formatter_core::{
    Format, GroupId,
    builders::{
        empty_line, expand_parent, group, hard_line_break, if_group_fits_on_line, soft_line_break,
        soft_line_break_or_space, space,
    },
    write,
};
use oxc_htmlish_parser::ast::{NodeId, NodeKind};

use super::{
    HtmlishFormatter, block::between_block, format_with, tag::closing_end_borrowed,
    tag::closing_start_borrowed, write_child,
};
use crate::{context::HtmlishFormatContext, tree::Tree};

/// What separates two siblings (Prettier's `printBetweenLine`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Between {
    Nothing,
    Soft,
    Line,
    /// A space that never breaks (the compiler layer keeps a same-line gap).
    Space,
    Hard,
}

impl<'a> Format<'a, HtmlishFormatContext<'a>> for Between {
    fn fmt(&self, f: &mut HtmlishFormatter<'_, 'a>) {
        match self {
            Self::Nothing => {}
            Self::Soft => write!(f, soft_line_break()),
            Self::Line => write!(f, soft_line_break_or_space()),
            Self::Space => write!(f, space()),
            Self::Hard => write!(f, hard_line_break()),
        }
    }
}

fn between(t: &Tree<'_>, prev: NodeId, next: NodeId) -> Between {
    if t.is_block(prev) || t.is_block(next) {
        return between_block(t, prev, next);
    }
    let (p, n) = (&t[prev], &t[next]);
    if p.text_like && n.text_like {
        if p.trailing_sensitive {
            if p.has_trailing_spaces {
                return if prefer_hard_line_as_leading(t, next) {
                    Between::Hard
                } else {
                    Between::Line
                };
            }
            return Between::Nothing;
        }
        return if prefer_hard_line_as_leading(t, next) { Between::Hard } else { Between::Soft };
    }

    if t[next].borrows.prev_closing_end
        && t.element(prev).is_some()
        && p.only_child().is_some_and(|only| t.is_interpolation(only))
        && p.has_leading_spaces
    {
        return Between::Nothing;
    }

    if (t[prev].borrows.next_opening_start
        && (n.prettier_ignore
            // 123<a
            //      ~
            //   ><b>
            || n.first_child.is_some()
            // 123<!--
            //        ~
            // -->
            || n.self_closing
            // 123<span
            //         ~
            //   attr
            || t.element(next).is_some_and(|el| !el.attributes.is_empty())))
        // <img
        //   src="long"
        //             ~
        // />123
        || (t.element(prev).is_some() && p.self_closing && t[next].borrows.prev_closing_end)
    {
        return Between::Nothing;
    }

    // <div>{{ x }}<!-- comment --></div>
    //             ~
    if matches!(t.kind(next), NodeKind::Comment(_)) && n.leading_sensitive && !n.has_leading_spaces
    {
        return Between::Soft;
    }

    // Past a hidden element, insensitive whitespace between two non-blocks is free to stay on the line,
    // as written (DIVERGENCES.md#script-display-none): `<p>x<meta /></p>`, `<link /> <link />`
    if !n.leading_sensitive
        && (t.is_transparent(prev) || t.is_transparent(next))
        && p.display.sides_sensitive()
        && n.display.sides_sensitive()
        && !prefer_hard_line_as_leading(t, next)
    {
        return if n.has_leading_spaces { Between::Line } else { Between::Soft };
    }

    if !n.leading_sensitive
        || prefer_hard_line_as_leading(t, next)
        // Want to write us a letter? Use our<a
        //   ><b><a>mailing address</a></b></a
        //                                    ~
        // >.
        || (t[next].borrows.prev_closing_end
            && p.last_child.is_some_and(|last| {
                t[last].borrows.parent_closing_start && closing_start_borrowed(t, last)
            }))
    {
        return Between::Hard;
    }

    if !n.has_leading_spaces {
        return Between::Soft;
    }
    // The compiler layer: between two elements, whitespace that renders keeps its kind (DIVERGENCES.md#vue-element-gap-kind).
    if t.rules().keeps_element_gap_kind && t.element(prev).is_some() && t.element(next).is_some() {
        return if has_trailing_line_break(t, prev) { Between::Hard } else { Between::Space };
    }
    Between::Line
}

pub fn write_children<'a>(
    t: &'a Tree<'a>,
    id: NodeId,
    force_break: bool,
    f: &mut HtmlishFormatter<'_, 'a>,
) {
    if force_break {
        write!(f, expand_parent());
        for child in t.children(id) {
            if let Some(prev) = t[child].prev {
                write_between(between(t, prev, child), force_next_empty_line(t, prev), f);
            }
            write_child(t, child, f);
        }
        return;
    }

    // Each pair's separator and blank-line fact, computed once and carried to the next child.
    let mut carried = (Between::Nothing, false);
    let mut prev_group_id: Option<GroupId> = None;
    for child in t.children(id) {
        let node = &t[child];
        let (next_between, next_blank) = match node.next {
            Some(next) => (between(t, child, next), force_next_empty_line(t, child)),
            None => (Between::Nothing, false),
        };
        let (prev_between, prev_blank) =
            std::mem::replace(&mut carried, (next_between, next_blank));

        if node.text_like {
            if node.prev.is_some_and(|prev| t[prev].text_like) {
                write_between(prev_between, prev_blank, f);
            }
            write_child(t, child, f);
            continue;
        }

        // A line break on a side is printed outside the child's group; a softer separator inside it.
        let breaks =
            |between, blank| between != Between::Nothing && (blank || between == Between::Hard);
        let prev_breaks = breaks(prev_between, prev_blank);
        let leading = match node.prev {
            // After a `Space` (the previous group's trailing), no break before the next child.
            Some(prev)
                if !matches!(prev_between, Between::Nothing | Between::Space) && !prev_breaks =>
            {
                if t[prev].text_like {
                    Some(Leading::Between(prev_between))
                } else {
                    prev_group_id.map(Leading::IfPrevFlat)
                }
            }
            _ => None,
        };
        let next_breaks = breaks(next_between, next_blank);
        let trailing = if next_breaks { Between::Nothing } else { next_between };

        if prev_breaks {
            write_between(prev_between, prev_blank, f);
        }
        let group_id = f.state().group_id("htmlChild");
        write!(
            f,
            group(&format_with(|f| {
                match leading {
                    Some(Leading::Between(between)) => write!(f, between),
                    Some(Leading::IfPrevFlat(prev_id)) => write!(
                        f,
                        if_group_fits_on_line(&soft_line_break()).with_group_id(Some(prev_id))
                    ),
                    None => {}
                }
                write!(
                    f,
                    group(&format_with(|f| {
                        write_child(t, child, f);
                        write!(f, trailing);
                    }))
                    .with_group_id(Some(group_id))
                );
            }))
        );
        prev_group_id = Some(group_id);
        if next_breaks && node.next.is_some_and(|next| t[next].text_like) {
            write_between(next_between, next_blank, f);
        }
    }
}

/// A separator between siblings; a kept blank line is `empty_line`
/// (Prettier's two `hardline`s, which would collapse into one here).
fn write_between(between: Between, blank: bool, f: &mut HtmlishFormatter<'_, '_>) {
    match between {
        Between::Nothing => {}
        _ if blank => write!(f, empty_line()),
        _ => write!(f, between),
    }
}

#[derive(Clone, Copy)]
enum Leading {
    Between(Between),
    /// A soft line break printed only while the previous sibling's group stays flat.
    IfPrevFlat(GroupId),
}

fn force_next_empty_line(t: &Tree<'_>, id: NodeId) -> bool {
    t[id].next.is_some_and(|next| t.line_breaks(t.source_end(id), t[next].span.start) >= 2)
}

fn prefer_hard_line_as_leading(t: &Tree<'_>, id: NodeId) -> bool {
    prefer_hard_line_as_surrounding(t, id)
        || t[id].prev.is_some_and(|prev| prefer_hard_line_as_trailing(t, prev))
        || has_surrounding_line_break(t, id)
}

fn prefer_hard_line_as_trailing(t: &Tree<'_>, id: NodeId) -> bool {
    prefer_hard_line_as_surrounding(t, id) || t.tag(id) == "br" || has_surrounding_line_break(t, id)
}

fn prefer_hard_line_as_surrounding(t: &Tree<'_>, id: NodeId) -> bool {
    match t.kind(id) {
        NodeKind::ConditionalComment(_) | NodeKind::Comment(_) => true,
        NodeKind::Element(_) => matches!(t.tag(id), "script" | "select"),
        _ => false,
    }
}

fn has_surrounding_line_break(t: &Tree<'_>, id: NodeId) -> bool {
    has_leading_line_break(t, id) && has_trailing_line_break(t, id)
}

pub fn has_leading_line_break(t: &Tree<'_>, id: NodeId) -> bool {
    let item = &t[id];
    if !item.has_leading_spaces {
        return false;
    }
    if let Some(prev) = item.prev {
        return t.line_breaks(t.source_end(prev), item.span.start) > 0;
    }
    let parent = t.parent(id);
    parent == NodeId::DOCUMENT || t.line_breaks(t.content_start(parent), item.span.start) > 0
}

pub fn has_trailing_line_break(t: &Tree<'_>, id: NodeId) -> bool {
    let item = &t[id];
    if !item.has_trailing_spaces {
        return false;
    }
    if let Some(next) = item.next {
        return t.line_breaks(t.source_end(id), t[next].span.start) > 0;
    }
    let parent = t.parent(id);
    parent == NodeId::DOCUMENT
        // A line break before a borrowed `>` cannot be printed (`b </p\n>`, `b</p\n>`),
        // so it is not read either: the next pass sees what this one did (DIVERGENCES.md#line-break-before-borrowed-end).
        || (!closing_end_borrowed(t, parent)
            && t.end_tag_start(parent).is_some_and(|end| t.line_breaks(t.source_end(id), end) > 0))
}
