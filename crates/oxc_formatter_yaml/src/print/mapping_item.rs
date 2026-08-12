use oxc_formatter_core::{
    Buffer, Format, GroupId, MemoizeFormat, best_fitting,
    builders::{
        align, expand_parent, group, hard_line_break, if_group_breaks, if_group_fits_on_line,
        line_suffix, space,
    },
    format_args, write,
};
use oxc_yaml_parser::ast::{Content, MappingItem, Node};

use crate::{
    comments::{
        Gap, classify_gap, flush_leading_comments, pending_same_line_comment, write_single_comment,
        write_trailing_same_line_comment,
    },
    context::YamlFormatContext,
    options::ProseWrap,
    print::{YamlFormatter, column_of, format_with, to_span, write_node, write_node_or_suppressed},
};

/// Where a mapping item lives; decides the empty-value layout
/// (`{a}` prints the bare key, `[? a]` keeps the explicit form).
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum FlowParent {
    /// Block mapping (`mappingItem`).
    No,
    /// Inside `{...}` (`flowMappingItem` with a `flowMapping` parent).
    Mapping,
    /// A pair inside `[...]` (`flowMappingItem` with a `flowSequence` parent).
    Sequence,
}

/// Ports Prettier's `printMappingItem` for both block (`mappingItem`) and flow (`flowMappingItem`) items.
pub fn write_mapping_item<'a>(
    item: &'a MappingItem<'a>,
    parent_tag_is_set: bool,
    in_flow: FlowParent,
    f: &mut YamlFormatter<'_, 'a>,
) {
    let key_content = item.key_content();
    let value_content = item.value_content();

    let is_empty_key = key_content.is_none();
    let is_empty_value = value_content.is_none();

    if is_empty_key && is_empty_value {
        // The `: `'s own space is the separation for what follows:
        // a same-line comment (`: # c`, Prettier suppresses the line-suffix space for an empty mappingValue)
        // or the rest of a flow collection (`{ : }`).
        // In a block mapping with no comment nothing follows, don't leave it at the line end.
        let same_line_comment = pending_same_line_comment(item.span.end, f);
        if in_flow != FlowParent::No || same_line_comment.is_some() {
            write!(f, ": ");
        } else {
            write!(f, ":");
        }
        if let Some(comment) = same_line_comment {
            f.context().comments().take_before(comment.span.end);
            let content = format_with(move |f: &mut YamlFormatter<'_, 'a>| {
                write_single_comment(comment.span, f);
            });
            write!(f, [line_suffix(&content), expand_parent()]);
        }
        return;
    }

    let space_before_colon = key_content.is_some_and(|k| matches!(k.content, Content::Alias(_)));

    if is_empty_value {
        if in_flow == FlowParent::Mapping {
            // A lone key in a flow mapping prints as just the key
            write_key(item, f);
            return;
        }
        let key_end = item.key.as_ref().expect("a value-less non-empty item has a key").span.end;
        // An explicit `? key` with no value is normalized to `key:` too.
        // Prettier's `!hasTrailingComment(key.content)`:
        // with no `:`, a same-line comment (`? key # c`, whitespace-only gap) attaches to the key content and keeps the explicit form.
        // After `key:` the gap contains the `:`, the comment belongs to the value side, and the implicit form stays
        // (the caller emits it as a line suffix).
        let key_content_trailing_comment = pending_same_line_comment(key_end, f).is_some();
        if in_flow == FlowParent::No
            && is_absolutely_single_line(key_content, f)
            && !parent_tag_is_set
            && !key_content_trailing_comment
        {
            write_key(item, f);
            if space_before_colon {
                write!(f, space());
            }
            write!(f, ":");
            return;
        }
        write!(f, "? ");
        write!(f, align(2, &format_with(|f| write_key(item, f))));
        return;
    }

    if is_empty_key {
        write!(f, ": ");
        write!(f, align(2, &format_with(|f| write_value(item, f))));
        return;
    }

    // Past the empty-key/empty-value returns, both wrappers and their content exist
    let key = item.key.as_ref().expect("empty key handled above");
    let value = item.value.as_ref().expect("empty value handled above");
    let key_node = key_content.expect("empty key handled above");
    let value_node = value_content.expect("empty value handled above");
    let value_start = value_node.span.start;

    // Force explicit key:
    // the key isn't an inline node, or the source was already explicit with a comment between `?` and the key,
    // or between the key and `:` (an implicit `key:` with a comment above the value keeps the implicit form,
    // the comment becomes the value's leading comment instead).
    let explicit_comment_before_key = key.explicit
        && f.context().comments().peek().is_some_and(|c| c.span.end <= key_node.span.start);
    if !is_inline(key_content)
        || explicit_comment_before_key
        || (key.explicit && has_own_line_comment_before_value(value_node, f))
    {
        write!(f, "? ");
        // Comments between the key and `:` that are indented DEEPER than the item are the key's end comments
        // (inside the key's align); comments at the item's own column lead the value (before the `: ` line).
        // Comments AFTER the `:` are the value's middle comments and stay pending,
        // `write_node` prints them right after the `: `.
        let item_column = column_of(&f.context().source_text(), item.span.start);
        // The `:` position: the value span starts at its indicator
        let colon = value.span.start;
        let key_and_comments = format_with(move |f: &mut YamlFormatter<'_, 'a>| {
            write_key(item, f);
            while let Some(comment) = f.context().comments().peek() {
                let span = comment.span;
                if span.end > colon
                    || comment.own_line_column.is_none_or(|column| column <= item_column)
                {
                    break;
                }
                f.context().comments().take_before(span.end);
                write!(f, hard_line_break());
                write_single_comment(span, f);
            }
        });
        write!(f, align(2, &key_and_comments));
        write!(f, hard_line_break());
        let comments = f.context().comments().take_before(colon);
        for comment in comments {
            write_single_comment(comment.span, f);
            write!(f, hard_line_break());
        }
        write!(f, ": ");
        write!(f, align(2, &format_with(|f| write_value(item, f))));
        return;
    }

    // NOTE: In a flow collection Prettier prints a key's same-line trailing comment with `breakParent`,
    // flipping the item to the explicit form while its `conditionalGroup` keeps the enclosing flow FLAT.
    // A newline inside unbroken brackets with no trailing comma (`{ ? "key" # 1` + newline + `  : value }`),
    // the same inconsistency rejected for spec-example-7-20 / 9-4.
    // The comment instead takes the hardline-separator path below and the flow breaks normally,
    // like every other comment inside one (`{` + newline + `  "key": # 1` ...).
    let key_span_end = key.span.end;
    let key_single_line = is_single_line(key_content, f);
    let key_absolutely_single_line = is_absolutely_single_line(key_content, f);
    let key_trailing_same_line = key_has_trailing_comment(key_span_end, value_start, f);

    // Force single line: both sides are definitely single-line and comment-free
    // (a pending comment before the value body is the key's trailing comment or the value's leading/middle comment,
    // all of them break the single line).
    let value_body_start = value_node.content.span().start;
    if key_single_line
        && !key_trailing_same_line
        && !has_pending_comment_before(value_body_start, f)
        && key_absolutely_single_line
        && is_absolutely_single_line(value_content, f)
    {
        write_key(item, f);
        if space_before_colon {
            write!(f, space());
        }
        write!(f, ": ");
        write_value(item, f);
        return;
    }

    // The general case.
    // Prettier decides implicit vs explicit (`? key`) with `conditionalGroup([[groupedKey, ifBreak(explicit, implicit, {groupId})]])`:
    // a key whose group breaks (multiline content or width overflow) flips the item to the explicit form.
    let tab_width = f.options().indent_width.value();

    // A key whose PRINTED form keeps hard line breaks always breaks the key group in Prettier (`breakParent` propagation), i.e. always explicit.
    // With `proseWrap` preserve that is any multiline scalar;
    // under always/never the scalar re-folds, so only a blank line (paragraph breaks survive folding) or a backslash continuation pins the structure.
    // A merely-long key takes the width check below instead (`a\n  true: ...` refolds to `a true: ...`).
    // Flow collections reformat onto one line as well.
    let key_is_multiline_scalar = !key_single_line
        && !key_content.is_some_and(|k| {
            matches!(k.content, Content::FlowMapping(_) | Content::FlowSequence(_))
        })
        && (f.options().prose_wrap == ProseWrap::Preserve
            || has_forced_break_when_folded(key_content, f));
    if key_is_multiline_scalar {
        write!(f, "? ");
        write!(f, align(2, &format_with(|f| write_key(item, f))));
        // `? key # c` keeps the comment on the key's last line, not past the `:`
        // (the source is explicit here, so the gap holds no punctuation)
        write_trailing_same_line_comment(key_span_end, &[], f);
        write!(f, hard_line_break());
        write!(f, ": ");
        write!(f, align(2, &format_with(|f| write_value(item, f))));
        return;
    }

    // Separator between `:` and the value.
    // A props-carrying block collection is NOT pinned:
    // like a block scalar it takes the width-based routing below
    // (Prettier hangs `key:\n  !!tag &a` under an overflowing key too).
    let block_collection_without_props =
        matches!(value_node.content, Content::Mapping(_) | Content::Sequence(_))
            && value_node.props.anchor.is_none()
            && value_node.props.tag.is_none();
    let colon = if space_before_colon { " :" } else { ":" };
    let hardline_separator =
        block_collection_without_props || has_pending_comment_before(value_start, f);

    if hardline_separator {
        // The separator is pinned to a hardline (block collection / comments).
        // A trailing comment drops the implicit guarantee like everywhere else
        // (Prettier's `!hasTrailingComment(key.content)` beside its `isAbsolutelyPrintedAsSingleLineNode`).
        if key_absolutely_single_line && !key_trailing_same_line {
            write_key(item, f);
            write!(f, colon);
            let value = format_with(|f| {
                write_node_or_suppressed(value_node, f);
            });
            write!(f, align(tab_width, &format_args!(hard_line_break(), value)));
            return;
        }
        // A key that may break must not stay implicit: a multi-line implicit key is invalid YAML
        let group_id = write_grouped_key(&format_with(|f| write_key(item, f)), f);
        // The key's same-line comment:
        // the line suffix lands at the end of whichever line the printer is on (`key: # c` flat, `? key # c` broken),
        // so one write serves both forms;
        // its `expand_parent` reaches an enclosing flow collection, not the closed key group.
        write_trailing_same_line_comment(key_span_end, b":", f);
        // The value sits in both conditional bodies
        let value = format_with(|f: &mut YamlFormatter<'_, 'a>| {
            write_node_or_suppressed(value_node, f);
        })
        .memoized();
        // This item is multi-line by definition: expand the enclosing groups explicitly
        // (the item's own group; also an enclosing flow collection,
        // which only reaches this path for a key trailing comment and must break for it anyway).
        // The separators inside the conditionals then don't propagate (see `if_group_breaks` docs).
        write!(f, expand_parent());
        write_explicit_value(group_id, &value, f);
        write!(
            f,
            if_group_fits_on_line(&format_args!(
                colon,
                align(tab_width, &format_args!(hard_line_break().without_expand_parent(), value))
            ))
            .with_group_id(Some(group_id))
        );
        return;
    }

    // Width-dependent layout: two decisions, two mechanisms
    // (Prettier's `conditionalGroup([[groupedKey, ifBreak(explicit, implicit, {groupId})]])`):
    // - implicit vs explicit (`? key`) follows the KEY group via `if_group_breaks`:
    //   width overflow or a forced break inside the key flips the item to the explicit form
    // - the implicit separator (value on the key's line or hanging under it) is a `best_fitting` trial,
    //   measured flat with early-exit at hardlines:
    //   variant 1 fits = `: ` + the value's FIRST line fit after the key
    //   (a multiline value's own hardlines don't re-flow the key line, Prettier's `conditionalGroup` boundary;
    //   for a block scalar that first line is just the `| ` / `>` header, sans any line-suffix trailing comment)
    let key = format_with(|f: &mut YamlFormatter<'_, 'a>| write_key(item, f)).memoized();
    // The group wrapper mirrors Prettier's `genericPrint` (`group(printNode())`)
    // and is what decides variant 1 for a multi-paragraph value:
    // the paragraph hardline expands the group,
    // fits then measures its content in expanded mode and exits `Yes` at the FIRST fill separator,
    // so only `key: ` plus the first word must fit and the fill wraps from the key line.
    // A value with no forced break keeps the group flat and is measured in full.
    let value_content_fmt = format_with(move |f: &mut YamlFormatter<'_, 'a>| {
        flush_leading_comments(value_start, f);
        write!(f, group(&format_with(|f| write_node(value_node, f))));
    })
    .memoized();

    // A definitely-single-line key never flips to the explicit form, no matter how long
    if key_absolutely_single_line && !key_trailing_same_line {
        write!(
            f,
            best_fitting![
                format_args!(key, colon, space(), align(tab_width, &value_content_fmt)),
                format_args!(
                    key,
                    colon,
                    align(tab_width, &format_args!(hard_line_break(), value_content_fmt))
                ),
            ]
        );
        return;
    }

    let group_id = write_grouped_key(&key, f);
    write_explicit_value(group_id, &value_content_fmt, f);
    write!(
        f,
        if_group_fits_on_line(&best_fitting![
            // Everything starting on one line
            format_args!(colon, space(), align(tab_width, &value_content_fmt)),
            // Value on the next line
            format_args!(
                colon,
                align(tab_width, &format_args!(hard_line_break(), value_content_fmt))
            ),
        ])
        .with_group_id(Some(group_id))
    );
}

/// Prettier's `groupedKey` (`group([ifBreak("? "), group(align(2, key), {id})])`):
/// the returned id is what the value side's conditionals watch,
/// the key group breaks on width overflow AND on a forced break inside the key (a comment-pinned flow collection),
/// flipping the item to the explicit form.
/// A `best_fitting` trial could not see the latter: its measurement early-exits `Yes` at the first line break.
fn write_grouped_key<'a>(
    key: &impl Format<'a, YamlFormatContext<'a>>,
    f: &mut YamlFormatter<'_, 'a>,
) -> GroupId {
    let group_id = f.state().group_id("mappingKey");
    write!(
        f,
        group(&format_args!(
            if_group_breaks(&"? "),
            group(&align(2, key)).with_group_id(Some(group_id))
        ))
    );
    group_id
}

/// [write_grouped_key]'s companion: the explicit form's value side (line break + `: value`),
/// printed only when the key group broke (Prettier's `ifBreak(explicitMappingValue, ..., {groupId})`).
/// The separator must not expand anything around it (see the `if_group_breaks` docs).
fn write_explicit_value<'a>(
    group_id: GroupId,
    value: &impl Format<'a, YamlFormatContext<'a>>,
    f: &mut YamlFormatter<'_, 'a>,
) {
    write!(
        f,
        if_group_breaks(&format_args!(
            hard_line_break().without_expand_parent(),
            ": ",
            align(2, value)
        ))
        .with_group_id(Some(group_id))
    );
}

fn write_key<'a>(item: &'a MappingItem<'a>, f: &mut YamlFormatter<'_, 'a>) {
    if let Some(key) = item.key_content() {
        write_node(key, f);
    }
}

fn write_value<'a>(item: &'a MappingItem<'a>, f: &mut YamlFormatter<'_, 'a>) {
    if let Some(value) = item.value_content() {
        write_node(value, f);
    }
}

/// `isInlineNode`: scalars, aliases and flow collections are inline.
fn is_inline(node: Option<&Node<'_>>) -> bool {
    !node.is_some_and(|node| {
        matches!(
            node.content,
            Content::Mapping(_)
                | Content::Sequence(_)
                | Content::BlockLiteral(_)
                | Content::BlockFolded(_)
        )
    })
}

/// The raw source of a flow scalar (plain/quoted); `None` for aliases,
/// collections and block scalars.
fn scalar_raw<'a>(node: &Node<'_>, f: &YamlFormatter<'_, 'a>) -> Option<&'a str> {
    let span = match &node.content {
        Content::Plain(p) => p.span,
        Content::QuoteSingle(s) => s.span,
        Content::QuoteDouble(d) => d.span,
        _ => return None,
    };
    Some(f.context().source_text().text_for(&to_span(span)))
}

/// `isSingleLineNode`: the node occupies one source line.
fn is_single_line(node: Option<&Node<'_>>, f: &YamlFormatter<'_, '_>) -> bool {
    match node {
        None => true,
        Some(node) if matches!(node.content, Content::Alias(_)) => true,
        Some(node) => scalar_raw(node, f).is_some_and(|raw| !raw.contains('\n')),
    }
}

/// `isAbsolutelyPrintedAsSingleLineNode`:
/// the node WILL print on one line regardless of width (so implicit style can be forced).
fn is_absolutely_single_line(node: Option<&Node<'_>>, f: &YamlFormatter<'_, '_>) -> bool {
    let Some(node) = node else { return true };
    if matches!(node.content, Content::Alias(_)) {
        return true;
    }
    let Some(raw) = scalar_raw(node, f) else { return false };

    let prose_wrap = f.options().prose_wrap;
    if prose_wrap == ProseWrap::Preserve {
        return !raw.contains('\n');
    }
    if has_backslash_continuation(raw) {
        return false;
    }
    if prose_wrap == ProseWrap::Never {
        // `never` folds every newline away,
        // so only blank lines (which survive folding) keep it multi-line.
        !has_blank_line(raw)
    } else {
        // `always` may wrap at any space
        !raw.contains('\n') && !raw.contains(' ')
    }
}

fn has_blank_line(raw: &str) -> bool {
    let mut lines = raw.split('\n');
    lines.next();
    lines.any(|l| l.trim().is_empty())
}

/// A backslash at a line end (quoteDouble continuation) pins the line structure.
fn has_backslash_continuation(raw: &str) -> bool {
    raw.split('\n').any(|line| line.ends_with('\\'))
}

/// Whether a scalar keeps a forced line break even after `proseWrap`
/// always/never re-folding: a blank line (paragraph break) survives folding,
/// and a backslash line continuation pins the line structure.
fn has_forced_break_when_folded(node: Option<&Node<'_>>, f: &YamlFormatter<'_, '_>) -> bool {
    let Some(node) = node else { return false };
    // Defensive: non-scalars never re-fold
    let Some(raw) = scalar_raw(node, f) else { return true };
    has_blank_line(raw) || has_backslash_continuation(raw)
}

/// Is there a pending comment on the key's line, BEFORE the value starts?
/// (`key: # comment` with the value on the next line,
/// a comment after the value like `key: value # comment` is the value's trailing comment instead.)
fn key_has_trailing_comment(key_end: u32, value_start: u32, f: &YamlFormatter<'_, '_>) -> bool {
    let Some(comment) = f.context().comments().peek() else { return false };
    let span = comment.span;
    let source = f.context().source_text();
    span.start >= key_end
        && span.end <= value_start
        && classify_gap(source.bytes_range(key_end, span.start)) == Gap::None
}

/// Is there any pending comment before `bound` (a leading comment of the value)?
fn has_pending_comment_before(bound: u32, f: &YamlFormatter<'_, '_>) -> bool {
    f.context().comments().peek().is_some_and(|c| c.span.end <= bound)
}

/// Own-line comment between the key and the value forces the explicit form.
/// A comment trailing the key's line (`? key # comment`) does NOT.
/// It is a trailing comment (`own_line_column: None`), not leading-of-value.
fn has_own_line_comment_before_value(value: &Node<'_>, f: &YamlFormatter<'_, '_>) -> bool {
    f.context()
        .comments()
        .peek()
        .is_some_and(|c| c.span.end <= value.span.start && c.own_line_column.is_some())
}
