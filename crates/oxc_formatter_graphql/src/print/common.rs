//! Shared printers: names, descriptions, directives, argument lists,
//! variable definitions, types, and input value definitions.

use apollo_parser::{cst, cst::CstNode};

use oxc_formatter_core::{
    Buffer, Format,
    builders::{
        block_indent, group, hard_line_break, indent, soft_block_indent, soft_line_break,
        soft_line_break_or_space, space, text,
    },
    write,
};

use crate::comments::{
    flush_leading_comments, flush_trailing_inside_comments, write_dangling_comments,
};

use super::{
    GraphqlFormatter, SeparatorKind, format_with,
    sig::{closing_token_start, node_text},
    string, value, write_sequence,
};

pub fn write_name(name: &cst::Name, f: &mut GraphqlFormatter<'_, '_>) {
    write!(f, text(node_text(f, name.syntax())));
}

/// Whether a description string is a block string (`"""..."""`).
fn is_block_string(sv: &cst::StringValue, f: &GraphqlFormatter<'_, '_>) -> bool {
    node_text(f, sv.syntax()).starts_with("\"\"\"")
}

/// Description followed by a hard line break (the default placement).
pub fn write_description(description: Option<cst::Description>, f: &mut GraphqlFormatter<'_, '_>) {
    let Some(description) = description else { return };
    let Some(sv) = description.string_value() else { return };
    string::write_string_value(&sv, f);
    write!(f, hard_line_break());
}

/// Description placement for `InputValueDefinition`:
/// non-block descriptions are followed by a soft line (they may stay inline in an
/// argument list), block descriptions by a hard line break.
/// Mirrors Prettier's `printDescription`.
fn write_description_input_value(
    description: Option<cst::Description>,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    let Some(description) = description else { return };
    let Some(sv) = description.string_value() else { return };
    let is_block = is_block_string(&sv, f);
    string::write_string_value(&sv, f);
    if is_block {
        write!(f, hard_line_break());
    } else {
        write!(f, soft_line_break_or_space());
    }
}

/// Directive placement style. Mirrors Prettier's `printDirectives`.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum DirectivesStyle {
    /// On `OperationDefinition` / `FragmentDefinition`: `group([line, joined])`.
    Definition,
    /// Everywhere else: `[" ", group(indent([softline, joined]))]`.
    Attached,
}

pub fn write_directives<'a>(
    directives: Option<cst::Directives>,
    style: DirectivesStyle,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let Some(directives) = directives else { return };
    let list: Vec<cst::Directive> = directives.directives().collect();
    if list.is_empty() {
        return;
    }

    let joined = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
        for (i, directive) in list.iter().enumerate() {
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

fn write_directive(directive: &cst::Directive, f: &mut GraphqlFormatter<'_, '_>) {
    write!(f, "@");
    if let Some(name) = directive.name() {
        write_name(&name, f);
    }
    write_arguments(directive.arguments(), f);
}

/// Close an empty delimited container (`[]`, `{}`, `{ }` selection set): drains any comments
/// pending before `close_start`, emits them block-indented when present, then writes `close`.
/// The caller has already written the opening delimiter. Sibling of [`write_paren_list`] /
/// `write_braced_body` for the empty case.
pub fn write_empty_delimited<'a>(
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
/// Comments pending before `)` are flushed inside the group body.
pub fn write_paren_list<'a, T, F>(
    f: &mut GraphqlFormatter<'_, 'a>,
    items: &[T],
    r_paren_start: u32,
    preserve_blank: bool,
    write_item: F,
) where
    T: CstNode,
    F: Fn(usize, &mut GraphqlFormatter<'_, 'a>),
{
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
pub fn write_arguments(arguments: Option<cst::Arguments>, f: &mut GraphqlFormatter<'_, '_>) {
    let Some(arguments) = arguments else { return };
    let items: Vec<cst::Argument> = arguments.arguments().collect();
    if items.is_empty() {
        return;
    }
    let r_paren = closing_token_start(arguments.r_paren_token(), arguments.syntax());
    write_paren_list(f, &items, r_paren, true, |i, f| {
        let argument = &items[i];
        if let Some(name) = argument.name() {
            write_name(&name, f);
        }
        write!(f, ": ");
        if let Some(v) = argument.value() {
            value::write_value(&v, f);
        }
    });
}

/// `($var: Type = default, ...)` on operations.
/// No blank-line preservation (Prettier uses a plain `path.map` here).
pub fn write_variable_definitions(
    variable_definitions: Option<cst::VariableDefinitions>,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    let Some(variable_definitions) = variable_definitions else { return };
    let items: Vec<cst::VariableDefinition> = variable_definitions.variable_definitions().collect();
    if items.is_empty() {
        return;
    }
    let r_paren =
        closing_token_start(variable_definitions.r_paren_token(), variable_definitions.syntax());
    write_paren_list(f, &items, r_paren, false, |i, f| {
        write_variable_definition(&items[i], f);
    });
}

fn write_variable_definition(
    variable_definition: &cst::VariableDefinition,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    write_description(variable_definition.description(), f);
    if let Some(variable) = variable_definition.variable() {
        write_variable(&variable, f);
    }
    write!(f, ": ");
    if let Some(ty) = variable_definition.ty() {
        write_type(&ty, f);
    }
    write_default_value(variable_definition.default_value(), f);
    write_directives(variable_definition.directives(), DirectivesStyle::Attached, f);
}

pub fn write_variable(variable: &cst::Variable, f: &mut GraphqlFormatter<'_, '_>) {
    write!(f, "$");
    if let Some(name) = variable.name() {
        write_name(&name, f);
    }
}

fn write_default_value(default_value: Option<cst::DefaultValue>, f: &mut GraphqlFormatter<'_, '_>) {
    let Some(default_value) = default_value else { return };
    write!(f, " = ");
    if let Some(v) = default_value.value() {
        value::write_value(&v, f);
    }
}

pub fn write_type(ty: &cst::Type, f: &mut GraphqlFormatter<'_, '_>) {
    match ty {
        cst::Type::NamedType(named) => write_named_type(named, f),
        cst::Type::ListType(list) => write_list_type(list, f),
        cst::Type::NonNullType(non_null) => {
            if let Some(named) = non_null.named_type() {
                write_named_type(&named, f);
            } else if let Some(list) = non_null.list_type() {
                write_list_type(&list, f);
            }
            write!(f, "!");
        }
    }
}

pub fn write_named_type(named: &cst::NamedType, f: &mut GraphqlFormatter<'_, '_>) {
    if let Some(name) = named.name() {
        write_name(&name, f);
    }
}

fn write_list_type(list: &cst::ListType, f: &mut GraphqlFormatter<'_, '_>) {
    write!(f, "[");
    if let Some(inner) = list.ty() {
        write_type(&inner, f);
    }
    write!(f, "]");
}

/// `name: Type = default @dir` (+ leading description) inside
/// `ArgumentsDefinition` / `InputFieldsDefinition`.
pub fn write_input_value_definition(
    input_value: &cst::InputValueDefinition,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    write_description_input_value(input_value.description(), f);
    if let Some(name) = input_value.name() {
        write_name(&name, f);
    }
    write!(f, ": ");
    if let Some(ty) = input_value.ty() {
        write_type(&ty, f);
    }
    write_default_value(input_value.default_value(), f);
    write_directives(input_value.directives(), DirectivesStyle::Attached, f);
}

/// ` implements A & B`, mirroring Prettier 3.8.4's `printInterfaces`:
/// names joined by plain `" & "` — the list NEVER breaks on line width (no group, no indent).
/// TODO: We will revisit this once Prettier 3.9.x released, as it changed the behavior.
///
/// A `line` replaces the space only when a comment sits between two names;
/// outside any group it always prints as a newline,
/// so the list breaks exactly at the comment position (at zero extra indentation).
pub fn write_implements_interfaces(
    implements: Option<cst::ImplementsInterfaces>,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    let Some(implements) = implements else { return };
    let names: Vec<cst::NamedType> = implements.named_types().collect();
    if names.is_empty() {
        return;
    }
    write!(f, " implements ");
    for (i, named) in names.iter().enumerate() {
        let start = super::sig::sig_start(named.syntax());
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
