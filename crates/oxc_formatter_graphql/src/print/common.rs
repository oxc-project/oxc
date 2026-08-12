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
    self, Directive, InputValueDefinition, Name, NamedType, StringValue, Type, Variable,
    VariableDefinition,
};

use crate::comments::{
    flush_leading_comments, flush_trailing_comment_before, flush_trailing_comment_before_break,
    flush_trailing_inside_comments, write_adjacent_trailing_comment, write_dangling_comments,
};

use super::{
    GraphqlFormatter, SeparatorKind, flush_trailing_before_literal, format_with,
    span::{Spanned, close_delim_start, next_token_start_after, to_span},
    string, value, write_sequence,
};

pub(super) fn write_name<'a>(name: &Name<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    flush_leading_comments(to_span(name.span).start, f);
    write!(f, text(name.value));
}

/// Description followed by a hard line break (the default placement).
/// A comment on the description's own line stays there (`"desc" # c`),
/// deferred past the break via `line_suffix`.
pub(super) fn write_description<'a>(
    description: Option<&StringValue<'a>>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let Some(sv) = description else { return };
    string::write_string_value(sv, f);
    write_adjacent_trailing_comment(to_span(sv.span).end, f);
    write!(f, hard_line_break());
    // Own-line comments between the description and what follows stay in place, at line start.
    // Without this, a following bare keyword (`type`, ...) offers no claim site
    // and the comment would cross it forward to the name's flush.
    let next_token_start = next_token_start_after(&f.context().source_text(), to_span(sv.span).end);
    flush_leading_comments(next_token_start, f);
}

/// Mirrors Prettier's `printDescription`.
/// Description placement for `InputValueDefinition`:
/// non-block descriptions are followed by a soft line
/// (they may stay inline in an argument list),
/// block descriptions by a hard line break.
/// No bounded flush after the break (unlike [`write_description`]):
/// the next token is always the name, whose own leading flush claims at the same position.
fn write_description_input_value<'a>(
    description: Option<&StringValue<'a>>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let Some(sv) = description else { return };
    let is_block = sv.block;
    string::write_string_value(sv, f);
    write_adjacent_trailing_comment(to_span(sv.span).end, f);
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
    // Claim at the construct's span start so a leading comment lands before the `@`,
    // not between the sigil and the name.
    flush_leading_comments(to_span(directive.span).start, f);
    write!(f, "@");
    write_name(&directive.name, f);
    write_arguments(directive.arguments.as_ref(), f);
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
/// Emits nothing for an empty list (`f()` parses only on the error path).
/// Comments pending before `)` are flushed inside the group body.
/// `list_span` covers `(`..`)`.
pub(super) fn write_paren_list<'a, T, F>(
    f: &mut GraphqlFormatter<'_, 'a>,
    items: &[T],
    list_span: ast::Span,
    preserve_blank: bool,
    write_item: F,
) where
    T: Spanned,
    F: Fn(usize, &mut GraphqlFormatter<'_, 'a>),
{
    let Some(first) = items.first() else { return };
    let first_start = first.span().start;
    let r_paren_start = close_delim_start(list_span);

    // `name # c (…)`: pin the comment to the name's line, in front of the `(`
    flush_trailing_comment_before(to_span(list_span).start, f);
    write!(f, "(");
    // `( # c`: keep the comment on the `(` line; the body's soft indent breaks after it
    flush_trailing_comment_before_break(first_start, f);
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
pub(super) fn write_arguments<'a>(
    arguments: Option<&'a ast::Arguments<'a>>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let Some(arguments) = arguments else { return };
    let items = arguments.items.as_slice();
    write_paren_list(f, items, arguments.span, true, |i, f| {
        let argument = &items[i];
        write_name(&argument.name, f);
        flush_trailing_before_literal(to_span(argument.name.span).end, f);
        write!(f, ": ");
        if let Some(v) = argument.value.as_ref() {
            value::write_value(v, f);
        }
    });
}

/// `($var: Type = default, ...)` on operations.
/// No blank-line preservation (Prettier uses a plain `path.map` here).
pub(super) fn write_variable_definitions<'a>(
    variable_definitions: Option<&'a ast::VariableDefinitions<'a>>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let Some(variable_definitions) = variable_definitions else { return };
    let items = variable_definitions.items.as_slice();
    write_paren_list(f, items, variable_definitions.span, false, |i, f| {
        write_variable_definition(&items[i], f);
    });
}

fn write_variable_definition<'a>(
    variable_definition: &'a VariableDefinition<'a>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    write_description(variable_definition.description.as_deref(), f);
    write_variable(&variable_definition.variable, f);
    flush_trailing_before_literal(to_span(variable_definition.variable.span).end, f);
    write!(f, ": ");
    if let Some(ty) = variable_definition.ty.as_ref() {
        write_type(ty, f);
    }
    write_default_value(variable_definition.default_value.as_ref(), f);
    write_directives(&variable_definition.directives, DirectivesStyle::Attached, f);
}

pub(super) fn write_variable<'a>(variable: &Variable<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    // Claim at the construct's span start so a leading comment lands before the `$`,
    // not between the sigil and the name.
    flush_leading_comments(to_span(variable.span).start, f);
    write!(f, "$");
    write_name(&variable.name, f);
}

fn write_default_value<'a>(
    default_value: Option<&'a ast::DefaultValue<'a>>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let Some(default_value) = default_value else { return };
    // The node's span starts at the `=`
    write_spaced_keyword(to_span(default_value.span).start, "= ", f);
    value::write_value(&default_value.value, f);
}

/// ` <keyword>` with a trailing-comment claim in front (`name # c` + break + `keyword`):
/// the space is written first so a claimed comment's boundary break discards it as pending,
/// then the claim bounded at `anchor` (the keyword's source position), then the keyword.
pub(super) fn write_spaced_keyword(
    anchor: u32,
    keyword: &'static str,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    write!(f, space());
    flush_trailing_comment_before(anchor, f);
    write!(f, text(keyword));
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
    write_description_input_value(input_value.description.as_deref(), f);
    write_name(&input_value.name, f);
    flush_trailing_before_literal(to_span(input_value.name.span).end, f);
    write!(f, ": ");
    if let Some(ty) = input_value.ty.as_ref() {
        write_type(ty, f);
    }
    write_default_value(input_value.default_value.as_ref(), f);
    write_directives(&input_value.directives, DirectivesStyle::Attached, f);
}

/// ` implements A & B`, mirroring Prettier's  `printInterfaces`:
/// `indent(group(join([" &", line], names)))`.
///
/// The whole list is one group so it breaks together on width overflow.
/// A leading comment between two names emits a `hard_line_break` which forces the group to expand.
pub(super) fn write_implements_interfaces<'a>(
    implements: Option<&'a ast::ImplementsInterfaces<'a>>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let Some(implements) = implements else { return };
    let interfaces = implements.interfaces.as_slice();
    write_spaced_keyword(to_span(implements.span).start, "implements ", f);
    let joined = format_with(move |f: &mut GraphqlFormatter<'_, 'a>| {
        for (i, named) in interfaces.iter().enumerate() {
            if i > 0 {
                write!(f, [" &", soft_line_break_or_space()]);
            }
            write_named_type(named, f);
        }
    });
    write!(f, group(&indent(&joined)));
}
