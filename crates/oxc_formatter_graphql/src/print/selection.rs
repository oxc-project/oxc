//! Selection set printers: fields, fragment spreads, inline fragments.

use oxc_formatter_core::{
    Buffer,
    builders::{block_indent, group, space},
    write,
};
use oxc_graphql_parser::ast::{Field, FragmentSpread, InlineFragment, Selection, SelectionSet};

use crate::comments::{
    flush_trailing_comment_before, flush_trailing_comment_before_break,
    flush_trailing_inside_comments,
};

use super::{
    GraphqlFormatter, SeparatorKind, common,
    common::DirectivesStyle,
    flush_trailing_before_literal, format_with,
    span::{close_delim_start, to_span},
    write_sequence,
};

/// `{ selections... }`, always multi-line.
pub(super) fn write_selection_set<'a>(
    selection_set: &'a SelectionSet<'a>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let selections = selection_set.selections.as_slice();
    let r_curly = close_delim_start(selection_set.span);

    write!(f, "{");
    if selections.is_empty() {
        // Grammar requires at least one selection; reachable only defensively.
        common::write_empty_delimited(r_curly, "}", f);
        return;
    }

    flush_trailing_comment_before_break(selections[0].span().start, f);
    let body = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
        let last_end = write_sequence(f, selections, SeparatorKind::Hard, true, |i, f| {
            write_selection(&selections[i], f);
        });
        if let Some(last_end) = last_end {
            flush_trailing_inside_comments(last_end, r_curly, f);
        }
    });
    write!(f, [block_indent(&body), "}"]);
}

fn write_selection<'a>(selection: &'a Selection<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    match selection {
        Selection::Field(field) => write_field(field, f),
        Selection::FragmentSpread(spread) => write_fragment_spread(spread, f),
        Selection::InlineFragment(inline) => write_inline_fragment(inline, f),
    }
}

fn write_field<'a>(field: &'a Field<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    let content = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
        if let Some(alias) = field.alias.as_ref() {
            common::write_name(alias, f);
            flush_trailing_before_literal(to_span(alias.span).end, f);
            write!(f, ": ");
        }
        common::write_name(&field.name, f);
        common::write_arguments(field.arguments.as_ref(), f);
        common::write_directives(&field.directives, DirectivesStyle::Attached, f);
        if let Some(selection_set) = field.selection_set.as_ref() {
            write!(f, space());
            flush_trailing_comment_before(to_span(selection_set.span).start, f);
            write_selection_set(selection_set, f);
        }
    });
    write!(f, group(&content));
}

fn write_fragment_spread<'a>(spread: &'a FragmentSpread<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    write!(f, "...");
    common::write_name(&spread.name, f);
    common::write_arguments(spread.arguments.as_ref(), f);
    common::write_directives(&spread.directives, DirectivesStyle::Attached, f);
}

fn write_inline_fragment<'a>(inline: &'a InlineFragment<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    write!(f, "...");
    if let Some(type_condition) = inline.type_condition.as_ref() {
        common::write_spaced_keyword(to_span(type_condition.span).start, "on ", f);
        common::write_named_type(&type_condition.named_type, f);
    }
    common::write_directives(&inline.directives, DirectivesStyle::Attached, f);
    if let Some(selection_set) = inline.selection_set.as_ref() {
        write!(f, space());
        flush_trailing_comment_before(to_span(selection_set.span).start, f);
        write_selection_set(selection_set, f);
    }
}
