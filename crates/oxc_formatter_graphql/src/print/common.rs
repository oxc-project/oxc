//! Shared printers: names, descriptions, directives, argument lists,
//! variable definitions, types, and input value definitions.

use oxc_formatter_core::{
    Buffer, Format,
    builders::{
        block_indent, group, hard_line_break, indent, soft_block_indent, soft_line_break,
        soft_line_break_or_space, space, text,
    },
    write,
};
use oxc_graphql_parser::ast::{
    Argument, Directive, InputValueDefinition, Name, NamedType, StringValue, Type, Value, Variable,
    VariableDefinition,
};

use crate::comments::{
    flush_leading_comments, flush_trailing_inside_comments, write_dangling_comments,
};

use super::{
    GraphqlFormatter, SeparatorKind, format_with,
    span::{Spanned, find_close_after, to_span},
    string, value, write_sequence,
};

pub(super) fn write_name<'a>(name: &Name<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    write!(f, text(name.value));
}

/// Description followed by a hard line break (the default placement).
pub(super) fn write_description<'a>(
    description: Option<&StringValue<'a>>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let Some(sv) = description else { return };
    string::write_string_value(sv, f);
    write!(f, hard_line_break());
}

/// Mirrors Prettier's `printDescription`.
/// Description placement for `InputValueDefinition`:
/// non-block descriptions are followed by a soft line
/// (they may stay inline in an argument list),
/// block descriptions by a hard line break.
fn write_description_input_value<'a>(
    description: Option<&StringValue<'a>>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let Some(sv) = description else { return };
    let is_block = sv.block;
    string::write_string_value(sv, f);
    if is_block {
        write!(f, hard_line_break());
    } else {
        write!(f, soft_line_break_or_space());
    }
}

/// Directive placement style. Mirrors Prettier's `printDirectives`.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum DirectivesStyle {
    /// On `OperationDefinition` / `FragmentDefinition`: `group([line, joined])`.
    Definition,
    /// Everywhere else: `[" ", group(indent([softline, joined]))]`.
    Attached,
}

pub(super) fn write_directives<'a>(
    directives: &'a [Directive<'a>],
    style: DirectivesStyle,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    if directives.is_empty() {
        return;
    }

    let joined = format_with(move |f: &mut GraphqlFormatter<'_, 'a>| {
        for (i, directive) in directives.iter().enumerate() {
            if i > 0 {
                write!(f, soft_line_break_or_space());
            }
            write_directive(directive, f);
        }
    });

    match style {
        DirectivesStyle::Definition => {
            let content = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
                write!(f, soft_line_break_or_space());
                joined.fmt(f);
            });
            write!(f, group(&content));
        }
        DirectivesStyle::Attached => {
            let content = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
                write!(f, soft_line_break());
                joined.fmt(f);
            });
            write!(f, [space(), group(&indent(&content))]);
        }
    }
}

fn write_directive<'a>(directive: &'a Directive<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    write!(f, "@");
    write_name(&directive.name, f);
    write_arguments(&directive.arguments, f);
}

/// Close an empty delimited container (`[]`, `{}`, `{ }` selection set): drains any comments
/// pending before `close_start`, emits them block-indented when present, then writes `close`.
/// The caller has already written the opening delimiter. Sibling of [`write_paren_list`] /
/// `write_braced_body` for the empty case.
pub(super) fn write_empty_delimited<'a>(
    close_start: u32,
    close: &'static str,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let dangling = f.context().comments().take_before(close_start);
    if !dangling.is_empty() {
        write!(
            f,
            block_indent(&format_with(move |f: &mut GraphqlFormatter<'_, 'a>| {
                write_dangling_comments(dangling, f);
            }))
        );
    }
    write!(f, text(close));
}

/// A parenthesized, comma-soft-separated list:
/// `group(["(", indent([softline, join(...)]), softline, ")"])`.
/// Emits nothing for an empty list.
/// Comments pending before `)` are flushed inside the group body;
/// the `)` position is derived by scanning past trivia
/// from the last item's end (paren lists have no wrapper node carrying it).
pub(super) fn write_paren_list<'a, T, F>(
    f: &mut GraphqlFormatter<'_, 'a>,
    items: &[T],
    preserve_blank: bool,
    write_item: F,
) where
    T: Spanned,
    F: Fn(usize, &mut GraphqlFormatter<'_, 'a>),
{
    let Some(last) = items.last() else { return };
    let r_paren_start = find_close_after(&f.context().source_text(), last.span().end, b')');

    write!(f, "(");
    let body = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
        let last_end =
            write_sequence(f, items, SeparatorKind::CommaSoftline, preserve_blank, &write_item);
        if let Some(last_end) = last_end {
            flush_trailing_inside_comments(last_end, r_paren_start, f);
        }
    });
    write!(f, [group(&soft_block_indent(&body)), ")"]);
}

/// `(arg: value, ...)` on fields, directives, and fragment spreads.
/// Blank lines between arguments are preserved (Prettier routes these through `printSequence`).
pub(super) fn write_arguments<'a>(arguments: &'a [Argument<'a>], f: &mut GraphqlFormatter<'_, 'a>) {
    write_paren_list(f, arguments, true, |i, f| {
        let argument = &arguments[i];
        write_name(&argument.name, f);
        write!(f, ": ");
        if let Some(v) = argument.value.as_ref() {
            value::write_value(v, f);
        }
    });
}

/// `($var: Type = default, ...)` on operations.
/// No blank-line preservation (Prettier uses a plain `path.map` here).
pub(super) fn write_variable_definitions<'a>(
    variable_definitions: &'a [VariableDefinition<'a>],
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    write_paren_list(f, variable_definitions, false, |i, f| {
        write_variable_definition(&variable_definitions[i], f);
    });
}

fn write_variable_definition<'a>(
    variable_definition: &'a VariableDefinition<'a>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    write_description(variable_definition.description.as_ref(), f);
    write_variable(&variable_definition.variable, f);
    write!(f, ": ");
    if let Some(ty) = variable_definition.ty.as_ref() {
        write_type(ty, f);
    }
    write_default_value(variable_definition.default_value.as_ref(), f);
    write_directives(&variable_definition.directives, DirectivesStyle::Attached, f);
}

pub(super) fn write_variable<'a>(variable: &Variable<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    write!(f, "$");
    write_name(&variable.name, f);
}

fn write_default_value<'a>(default_value: Option<&'a Value<'a>>, f: &mut GraphqlFormatter<'_, 'a>) {
    let Some(v) = default_value else { return };
    write!(f, " = ");
    value::write_value(v, f);
}

pub(super) fn write_type<'a>(ty: &Type<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    match ty {
        Type::Named(named) => write_named_type(named, f),
        Type::List(list) => {
            write!(f, "[");
            write_type(&list.ty, f);
            write!(f, "]");
        }
        Type::NonNull(non_null) => {
            write_type(&non_null.ty, f);
            write!(f, "!");
        }
        Type::Missing(_) => {}
    }
}

pub(super) fn write_named_type<'a>(named: &NamedType<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    write_name(&named.name, f);
}

/// `name: Type = default @dir` (+ leading description) inside
/// `ArgumentsDefinition` / `InputFieldsDefinition`.
pub(super) fn write_input_value_definition<'a>(
    input_value: &'a InputValueDefinition<'a>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    write_description_input_value(input_value.description.as_ref(), f);
    write_name(&input_value.name, f);
    write!(f, ": ");
    if let Some(ty) = input_value.ty.as_ref() {
        write_type(ty, f);
    }
    write_default_value(input_value.default_value.as_ref(), f);
    write_directives(&input_value.directives, DirectivesStyle::Attached, f);
}

/// ` implements A & B`, mirroring Prettier 3.8.4's `printInterfaces`:
/// names joined by plain `" & "` — the list NEVER breaks on line width (no group, no indent).
/// TODO: We will revisit this once Prettier 3.9.x released, as it changed the behavior.
///
/// A `line` replaces the space only when a comment sits between two names;
/// outside any group it always prints as a newline,
/// so the list breaks exactly at the comment position (at zero extra indentation).
pub(super) fn write_implements_interfaces<'a>(
    interfaces: &'a [NamedType<'a>],
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    if interfaces.is_empty() {
        return;
    }
    write!(f, " implements ");
    for (i, named) in interfaces.iter().enumerate() {
        let start = to_span(named.name.span).start;
        if i > 0 {
            write!(f, " &");
            // Pending comments before this name = comments between the two names
            // (Prettier checks `textBetween.includes("#")`).
            let has_comment = f.context().comments().iter_before(start).next().is_some();
            if has_comment {
                write!(f, soft_line_break_or_space());
            } else {
                write!(f, space());
            }
        }
        flush_leading_comments(start, f);
        write_named_type(named, f);
    }
}
