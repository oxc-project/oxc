//! Elements (and conditional comments): the tags around the children, or the content as written.

use oxc_formatter_core::{
    Format,
    builders::{
        dedent_to_root, expand_parent, group, if_group_breaks, indent, indent_if_group_breaks,
        soft_line_break, soft_line_break_or_space, space, text,
    },
    write,
};
use oxc_htmlish_parser::ast::{NodeId, NodeKind};

use super::{
    HtmlishFormatter,
    children::{has_leading_line_break, has_trailing_line_break, write_children},
    format_with,
    tag::{
        closing_end_borrowed, closing_end_marker, closing_start_borrowed, closing_start_marker,
        opening_end_borrowed, opening_end_marker, write_closing_tag, write_opening_tag,
    },
};
use crate::{
    HtmlishFormatOptions,
    context::HtmlishFormatContext,
    tree::{Display, Tree},
};

pub fn write_element<'a>(t: &'a Tree<'a>, id: NodeId, f: &mut HtmlishFormatter<'_, 'a>) {
    let item = &t[id];

    if should_preserve_content(t, id) {
        // The opening tag prints the borrowed prefix, once (DIVERGENCES.md#pre-after-borrowed-closing-end)
        write!(f, group(&format_with(|f| write_opening_tag(t, id, f))));
        write!(f, text(node_content(t, id)));
        write_closing_tag(t, id, f);
        return;
    }

    let attr_group_id = f.state().group_id("htmlAttributes");
    let print_tag = |f: &mut HtmlishFormatter<'_, 'a>,
                     body: &dyn Format<'a, HtmlishFormatContext<'a>>| {
        write!(
            f,
            group(&format_with(|f| {
                write!(
                    f,
                    group(&format_with(|f| write_opening_tag(t, id, f)))
                        .with_group_id(Some(attr_group_id))
                );
                write!(f, body);
                write_closing_tag(t, id, f);
            }))
        );
    };

    let (Some(first), Some(last)) = (item.first_child, item.last_child) else {
        let dangling = item.has_dangling_spaces && item.dangling_sensitive;
        print_tag(
            f,
            &format_with(|f| {
                if dangling {
                    write!(f, soft_line_break_or_space());
                }
            }),
        );
        return;
    };

    let hug = item.only_child().is_some()
        && t.is_interpolation(first)
        && t[first].leading_sensitive
        && !t[first].has_leading_spaces
        && t[last].trailing_sensitive
        && !t[last].has_trailing_spaces;

    let line_before = format_with(|f| {
        if hug {
            write!(f, if_group_breaks(&soft_line_break()).with_group_id(Some(attr_group_id)));
        } else if t[first].has_leading_spaces && t[first].leading_sensitive {
            write!(f, soft_line_break_or_space());
        } else if t.is_text(first) && item.pre_like {
            write!(f, dedent_to_root(&soft_line_break()));
        } else {
            write!(f, soft_line_break());
        }
    });

    let line_after = format_with(|f| {
        if closing_end_borrowed(t, id) {
            if item.script_like {
                // The body renders nothing, so its end tag takes its own line (DIVERGENCES.md#script-display-none)
                write!(f, soft_line_break());
            } else if t[last].has_trailing_spaces && t[last].trailing_sensitive {
                write!(f, space());
            }
        } else if item.pre_like && t[last].borrows.parent_closing_start {
            // `<pre>` whose last text prints `</pre`: a line break would be content
        } else if hug {
            write!(f, if_group_breaks(&soft_line_break()).with_group_id(Some(attr_group_id)));
        } else if t[last].has_trailing_spaces && t[last].trailing_sensitive {
            write!(f, soft_line_break_or_space());
        } else if t[last].ends_in_pre {
            // `<div><pre />⏎<b>x</b></div>`: the unclosed `pre` ends here, so a line break would be its content
            // (DIVERGENCES.md#pre-content-without-end-tag).
        } else if !ends_at_indentation(t, id, last, f.options()) {
            write!(f, soft_line_break());
        }
    });

    let force_break = force_break_children(t, id);
    let children = format_with(|f| {
        write!(f, line_before);
        write_children(t, id, force_break, f);
    });
    print_tag(
        f,
        &format_with(|f| {
            if force_break || force_break_content(t, id) {
                write!(f, expand_parent());
            }
            if hug {
                write!(f, indent_if_group_breaks(&children, attr_group_id));
            } else {
                write!(f, indent(&children));
            }
            write!(f, line_after);
        }),
    );
}

/// The last child is a comment or `pre` text ending in a line break plus exactly the element's indentation,
/// counted in the indent unit (DIVERGENCES.md#end-tag-indentation-as-printed):
/// the end tag already sits where it would be printed.
fn ends_at_indentation(
    t: &Tree<'_>,
    id: NodeId,
    last: NodeId,
    options: &HtmlishFormatOptions,
) -> bool {
    let value = match t.kind(last) {
        NodeKind::Comment(comment) => comment.content.slice(t.source()),
        NodeKind::Text(_) | NodeKind::Cdata if t[id].pre_like => t[last].span.slice(t.source()),
        _ => return false,
    };
    let Some((_, tail)) = value.rsplit_once('\n') else { return false };
    let (unit, per_level) = if options.indent_style.is_tab() {
        (b'\t', 1)
    } else {
        (b' ', usize::from(options.indent_width.value()))
    };
    if !tail.bytes().all(|b| b == unit) {
        return false;
    }
    // The element's own indentation: one level per ancestor below the document
    let mut depth = 0;
    let mut ancestor = t.parent(id);
    while ancestor != NodeId::DOCUMENT {
        depth += 1;
        ancestor = t.parent(ancestor);
    }
    tail.len() == per_level * depth
}

/// Children Prettier prints as written between the tags.
fn should_preserve_content(t: &Tree<'_>, id: NodeId) -> bool {
    let item = &t[id];
    match t.kind(id) {
        NodeKind::ConditionalComment(comment) => {
            !comment.parsed
                // `<!--[if lt IE 9]><html><![endif]-->`: an unclosed element inside
                || item.last_child.is_some_and(|last| t.is_unclosed(last))
        }
        // `<pre>` with markup inside
        _ => {
            item.pre_like
                && t.children(id).any(|child| !(t.is_text(child) || t.is_interpolation(child)))
        }
    }
}

/// The source between the tags, adjusted for markers the edges borrow (Prettier's `getNodeContent`).
fn node_content<'a>(t: &'a Tree<'a>, id: NodeId) -> &'a str {
    let item = &t[id];
    let mut start = t.content_start(id) as usize;
    // An unclosed element's content runs to where the parser closed it
    let mut end = t.end_tag_start(id).unwrap_or(item.span.end) as usize;
    if opening_end_borrowed(t, id) {
        start -= opening_end_marker(t, id).len();
    }
    if closing_start_borrowed(t, id) {
        end += closing_start_marker(t, id).len();
    } else if t[id].borrows.last_child_closing_end {
        end -= closing_end_marker(t, item.last_child.expect("borrowed by the last child")).len();
    }
    &t.source()[start..end]
}

fn force_break_children(t: &Tree<'_>, id: NodeId) -> bool {
    let item = &t[id];
    t.element(id).is_some()
        && item.first_child.is_some()
        && (matches!(t.tag(id), "html" | "head" | "ul" | "ol" | "select")
            || item.display == Display::Table)
}

/// Besides `force_break_children`:
/// the content of `body`, `script`, `style`, nested markup, or a lone child on its own lines.
fn force_break_content(t: &Tree<'_>, id: NodeId) -> bool {
    let item = &t[id];
    if t.element(id).is_some()
        && item.first_child.is_some()
        && (item.script_like
            || t.tag(id) == "body"
            || t.children(id)
                .any(|child| t.children(child).any(|grandchild| !t.is_text(grandchild))))
    {
        return true;
    }
    item.only_child().is_some_and(|only| {
        !t.is_text(only)
            && has_leading_line_break(t, only)
            && (!t[only].trailing_sensitive || has_trailing_line_break(t, only))
    })
}
