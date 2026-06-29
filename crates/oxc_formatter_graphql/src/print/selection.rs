//! Selection set printers: fields, fragment spreads, inline fragments.

use oxc_graphql_parser::{cst, cst::CstNode};

use oxc_formatter_core::{
    Buffer,
    builders::{block_indent, group, space},
    write,
};

use crate::comments::flush_trailing_inside_comments;

use super::{
    GraphqlFormatter, SeparatorKind, common, common::DirectivesStyle, format_with,
    sig::closing_token_start, write_sequence,
};

/// `{ selections... }`, always multi-line.
pub fn write_selection_set<'a>(
    selection_set: &cst::SelectionSet,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let selections: Vec<cst::Selection> = selection_set.selections().collect();
    let r_curly = closing_token_start(selection_set.r_curly_token(), selection_set.syntax());

    write!(f, "{");
    if selections.is_empty() {
        // Grammar requires at least one selection; reachable only defensively.
        common::write_empty_delimited(r_curly, "}", f);
        return;
    }

    let body = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
        let last_end = write_sequence(f, &selections, SeparatorKind::Hard, true, |i, f| {
            write_selection(&selections[i], f);
        });
        if let Some(last_end) = last_end {
            flush_trailing_inside_comments(last_end, r_curly, f);
        }
    });
    write!(f, [block_indent(&body), "}"]);
}

fn write_selection(selection: &cst::Selection, f: &mut GraphqlFormatter<'_, '_>) {
    match selection {
        cst::Selection::Field(field) => write_field(field, f),
        cst::Selection::FragmentSpread(spread) => write_fragment_spread(spread, f),
        cst::Selection::InlineFragment(inline) => write_inline_fragment(inline, f),
    }
}

fn write_field<'a>(field: &cst::Field, f: &mut GraphqlFormatter<'_, 'a>) {
    let content = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
        if let Some(alias) = field.alias()
            && let Some(name) = alias.name()
        {
            common::write_name(&name, f);
            write!(f, ": ");
        }
        if let Some(name) = field.name() {
            common::write_name(&name, f);
        }
        common::write_arguments(field.arguments(), f);
        common::write_directives(field.directives(), DirectivesStyle::Attached, f);
        if let Some(selection_set) = field.selection_set() {
            write!(f, space());
            write_selection_set(&selection_set, f);
        }
    });
    write!(f, group(&content));
}

fn write_fragment_spread(spread: &cst::FragmentSpread, f: &mut GraphqlFormatter<'_, '_>) {
    write!(f, "...");
    if let Some(fragment_name) = spread.fragment_name()
        && let Some(name) = fragment_name.name()
    {
        common::write_name(&name, f);
    }
    common::write_directives(spread.directives(), DirectivesStyle::Attached, f);
}

fn write_inline_fragment(inline: &cst::InlineFragment, f: &mut GraphqlFormatter<'_, '_>) {
    write!(f, "...");
    if let Some(type_condition) = inline.type_condition()
        && let Some(named) = type_condition.named_type()
    {
        write!(f, " on ");
        common::write_named_type(&named, f);
    }
    common::write_directives(inline.directives(), DirectivesStyle::Attached, f);
    if let Some(selection_set) = inline.selection_set() {
        write!(f, space());
        write_selection_set(&selection_set, f);
    }
}
