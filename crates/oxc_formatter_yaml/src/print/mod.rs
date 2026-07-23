use oxc_formatter_core::{
    Buffer, Format, Formatter,
    builders::{FormatWith, group, hard_line_break, space, text, token},
    write,
};
use oxc_yaml_parser::ast::{Content, Node, Props};

use crate::{
    comments::{
        flush_leading_comments, is_suppressed_last_before, suppression_flush_bound,
        write_single_comment, write_suppressed_node,
    },
    context::YamlFormatContext,
};

mod block;
mod block_collection;
mod document;
mod flow;
mod mapping_item;
mod scalar;
mod span;

pub use block::last_descendant_end;
pub use document::write_root;
pub use span::to_span;

pub type YamlFormatter<'buf, 'a> = Formatter<'buf, 'a, YamlFormatContext<'a>>;

/// `Format` impl for `&'static str` specialized to `YamlFormatContext`.
///
/// Hardcoded to `YamlFormatContext` rather than generic over `C` so the blanket
/// `&T where T: Format` doesn't overlap.
impl<'a> Format<'a, YamlFormatContext<'a>> for &'static str {
    #[inline]
    fn fmt(&self, f: &mut YamlFormatter<'_, 'a>) {
        write!(f, token(self));
    }
}

/// Wraps a re-entrant YAML closure in a [`FormatWith`].
/// The closure's context is pinned to [`YamlFormatContext`] so call sites don't have to annotate it.
#[inline]
pub const fn format_with<'a, T>(formatter: T) -> FormatWith<T>
where
    T: Fn(&mut YamlFormatter<'_, 'a>),
{
    FormatWith::new(formatter)
}

/// 0-based column of `offset` in `source` (bytes since the preceding newline).
pub fn column_of(source: &str, offset: u32) -> u32 {
    let count =
        source.as_bytes()[..offset as usize].iter().rev().take_while(|&&b| b != b'\n').count();
    u32::try_from(count).unwrap_or(u32::MAX)
}

/// A run of `count` newlines with the arena lifetime.
/// Small runs (the overwhelming case: blank-line counts) slice a static string
/// instead of building a `String` and copying it into the arena.
pub fn arena_newlines<'a>(count: usize, f: &YamlFormatter<'_, 'a>) -> &'a str {
    const NEWLINES: &str = "\n\n\n\n\n\n\n\n";
    if count <= NEWLINES.len() {
        &NEWLINES[..count]
    } else {
        f.allocator().alloc_str(&"\n".repeat(count))
    }
}

/// Emits a node's properties (anchor/tag) in their source order.
/// NOTE: Prettier 3.9.5 normalizes to `tag anchor` order; we keep the source order instead.
///
/// The separator after the props is a hard line for block collections and a space otherwise.
fn write_props(props: &Props, is_block_collection: bool, f: &mut YamlFormatter<'_, '_>) {
    let source = f.context().source_text();
    let mut spans = [props.anchor.map(|a| a.span), props.tag.map(|t| t.span)];
    if spans.iter().all(Option::is_none) {
        return;
    }
    spans.sort_by_key(|span| span.map_or(u32::MAX, |s| s.start));
    let mut first = true;
    for span in spans.into_iter().flatten() {
        if !first {
            write!(f, space());
        }
        first = false;
        write!(f, text(source.text_for(&to_span(span))));
    }
    if is_block_collection {
        write!(f, hard_line_break());
    } else {
        write!(f, space());
    }
}

/// Writes `node`, honoring a suppression marker (`# oxfmt-ignore`) in its pending leading comments:
/// a suppressed node is reproduced verbatim.
///
/// A block collection instead leaves the marker pending so the FIRST ITEM's own check claims it,
/// an ignore right above the first key/item freezes that item, never the whole collection
/// (nor, at the document level, the whole document: whole-body suppression needs the explicit head form above `---`).
///
/// Returns `true` when the node was reproduced verbatim
/// (the caller must then skip its own trailing-comment pass; the cursor already moved past the span).
pub fn write_node_or_suppressed<'a>(node: &'a Node<'a>, f: &mut YamlFormatter<'_, 'a>) -> bool {
    let start = node.span.start;
    let is_block_collection = matches!(node.content, Content::Mapping(_) | Content::Sequence(_));
    if !is_block_collection && is_suppressed_last_before(f, start) {
        write_suppressed_node(to_span(node.span), f);
        return true;
    }
    flush_leading_comments(suppression_flush_bound(is_block_collection, start, f), f);
    write_node(node, f);
    false
}

pub fn write_node<'a>(node: &'a Node<'a>, f: &mut YamlFormatter<'_, 'a>) {
    let content = &node.content;
    let content_start = content.span().start;
    let is_block_collection = matches!(content, Content::Mapping(_) | Content::Sequence(_));
    // Middle comments (between the props and the node body, `&a # c`)
    // keep the props separator a space even for block collections.
    let has_middle_comments = f.context().comments().peek().is_some_and(|c| c.end <= content_start);
    write_props(&node.props, is_block_collection && !has_middle_comments, f);

    // Prettier: `[middles.len() == 1 ? "" : hardline, join(hardline, middles), hardline]`.
    // A block collection's trailing suppression marker is NOT a middle comment:
    // it stays pending so the first item's check can claim it.
    let middles_bound = suppression_flush_bound(is_block_collection, content_start, f);
    let middles = f.context().comments().take_before(middles_bound);
    // `i > 0` is subsumed: any later iteration implies more than one middle.
    let multiple_middles = middles.len() > 1;
    for &span in middles {
        if multiple_middles {
            write!(f, hard_line_break());
        }
        write_single_comment(span, f);
    }
    if !middles.is_empty() {
        write!(f, hard_line_break());
    }

    match content {
        Content::Plain(scalar) => scalar::write_plain(to_span(scalar.span), f),
        Content::QuoteSingle(scalar) => {
            scalar::write_quoted(scalar::FlowScalarKind::QuoteSingle, to_span(scalar.span), f);
        }
        Content::QuoteDouble(scalar) => {
            scalar::write_quoted(scalar::FlowScalarKind::QuoteDouble, to_span(scalar.span), f);
        }
        Content::Alias(alias) => write_raw_span(to_span(alias.span), f),
        Content::BlockLiteral(block) => block::write_block_scalar(block, false, f),
        Content::BlockFolded(block) => block::write_block_scalar(block, true, f),
        Content::Mapping(mapping) => {
            // `!!set` on the mapping changes its items' empty-value layout
            let parent_tag_is_set = node.props.tag.is_some_and(|tag| {
                let raw = f.context().source_text().text_for(&to_span(tag.span));
                raw == "!!set" || raw == "!<tag:yaml.org,2002:set>"
            });
            block_collection::write_mapping(mapping, parent_tag_is_set, f);
        }
        Content::Sequence(sequence) => block_collection::write_sequence(sequence, f),
        Content::FlowMapping(flow) => {
            write_flow_collection(flow.span.end, |f| flow::write_flow_mapping(flow, f), f);
        }
        Content::FlowSequence(flow) => {
            write_flow_collection(flow.span.end, |f| flow::write_flow_sequence(flow, f), f);
        }
    }
}

/// Wraps a flow collection in its group,
/// then claims any comments the flow printer's bounded flushes missed so they aren't re-emitted later.
fn write_flow_collection<'a>(
    end: u32,
    inner: impl Fn(&mut YamlFormatter<'_, 'a>),
    f: &mut YamlFormatter<'_, 'a>,
) {
    write!(f, group(&format_with(inner)));
    let _ = f.context().comments().take_before(end);
}

/// v0: verbatim slice of the (newline-normalized) source.
/// Claims any comments inside the span (e.g. inside a verbatim flow collection)
/// so they aren't re-emitted at a later flush point.
fn write_raw_span(span: oxc_span::Span, f: &mut YamlFormatter<'_, '_>) {
    let raw = f.context().source_text().text_for(&span);
    write!(f, text(raw));
    let _ = f.context().comments().take_before(span.end);
}
