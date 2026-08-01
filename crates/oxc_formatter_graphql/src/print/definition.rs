//! Document-level definition printers: operations, fragments, and the type system.

use oxc_formatter_core::{
    Buffer,
    builders::{
        block_indent, group, hard_line_break, if_group_breaks, if_group_fits_on_line, indent,
        soft_line_break_or_space, space, text,
    },
    write,
};
use oxc_graphql_parser::ast::{
    self, Definition, Directive, DirectiveDefinition, DirectiveExtension, EnumValueDefinition,
    FieldDefinition, FragmentDefinition, Name, OperationDefinition, OperationType,
    RootOperationTypeDefinition, SchemaDefinition, SchemaExtension, StringValue,
};

use crate::comments::{
    flush_trailing_comment_before, flush_trailing_comment_before_break,
    flush_trailing_inside_comments,
};

use super::{
    GraphqlFormatter, SeparatorKind, common,
    common::DirectivesStyle,
    flush_trailing_before_literal, flush_trailing_before_literal_break, format_with, selection,
    span::{Spanned, close_delim_start, to_span},
    write_sequence,
};

pub(super) fn write_definition<'a>(
    definition: &'a Definition<'a>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    match definition {
        Definition::Operation(d) => write_operation_definition(d, f),
        Definition::Fragment(d) => write_fragment_definition(d, f),
        Definition::Directive(d) => write_directive_definition(d, f),
        Definition::DirectiveExtension(d) => write_directive_extension(d, f),
        Definition::Schema(d) => write_schema_definition(d, f),
        Definition::SchemaExtension(d) => write_schema_extension(d, f),
        Definition::ScalarType(d) => {
            write_scalar_type(d.description.as_deref(), false, &d.name, &d.directives, f);
        }
        Definition::ScalarTypeExtension(d) => {
            write_scalar_type(None, true, &d.name, &d.directives, f);
        }
        Definition::ObjectType(d) => write_object_like(
            d.description.as_deref(),
            false,
            "type",
            &d.name,
            d.implements.as_ref(),
            &d.directives,
            d.fields.as_ref(),
            f,
        ),
        Definition::ObjectTypeExtension(d) => write_object_like(
            None,
            true,
            "type",
            &d.name,
            d.implements.as_ref(),
            &d.directives,
            d.fields.as_ref(),
            f,
        ),
        Definition::InterfaceType(d) => write_object_like(
            d.description.as_deref(),
            false,
            "interface",
            &d.name,
            d.implements.as_ref(),
            &d.directives,
            d.fields.as_ref(),
            f,
        ),
        Definition::InterfaceTypeExtension(d) => write_object_like(
            None,
            true,
            "interface",
            &d.name,
            d.implements.as_ref(),
            &d.directives,
            d.fields.as_ref(),
            f,
        ),
        Definition::InputObjectType(d) => write_input_object_like(
            d.description.as_deref(),
            false,
            &d.name,
            &d.directives,
            d.fields.as_ref(),
            f,
        ),
        Definition::InputObjectTypeExtension(d) => {
            write_input_object_like(None, true, &d.name, &d.directives, d.fields.as_ref(), f);
        }
        Definition::UnionType(d) => {
            write_union_like(
                d.description.as_deref(),
                false,
                &d.name,
                &d.directives,
                d.members.as_ref(),
                f,
            );
        }
        Definition::UnionTypeExtension(d) => {
            write_union_like(None, true, &d.name, &d.directives, d.members.as_ref(), f);
        }
        Definition::EnumType(d) => write_enum_like(
            d.description.as_deref(),
            false,
            &d.name,
            &d.directives,
            d.values.as_ref(),
            f,
        ),
        Definition::EnumTypeExtension(d) => {
            write_enum_like(None, true, &d.name, &d.directives, d.values.as_ref(), f);
        }
    }
}

fn operation_type_keyword(kind: OperationType) -> &'static str {
    match kind {
        OperationType::Query => "query",
        OperationType::Mutation => "mutation",
        OperationType::Subscription => "subscription",
    }
}

fn write_operation_definition<'a>(
    operation: &'a OperationDefinition<'a>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    common::write_description(operation.description.as_deref(), f);

    // Direct-AST `operation_type` is always set (shorthand `{ ... }` parses as Query).
    // Shorthand means the operation's first significant token IS the selection set's `{`,
    // so their span starts coincide. Mirrors Prettier's
    // `locStart(node) !== locStart(node.selectionSet)` check.
    let is_shorthand =
        operation.selection_set.as_ref().is_some_and(|ss| ss.span.start == operation.span.start);
    let has_operation = !is_shorthand;
    let has_name = operation.name.is_some();

    if has_operation {
        write!(f, text(operation_type_keyword(operation.operation_type)));
    }
    if let Some(name) = operation.name.as_ref() {
        write!(f, space());
        common::write_name(name, f);
    }
    if has_operation && !has_name && operation.variable_definitions.is_some() {
        write!(f, space());
    }
    common::write_variable_definitions(operation.variable_definitions.as_ref(), f);
    common::write_directives(&operation.directives, DirectivesStyle::Definition, f);
    if has_operation || has_name {
        write!(f, space());
    }
    if let Some(selection_set) = operation.selection_set.as_ref() {
        if has_operation || has_name {
            flush_trailing_comment_before(to_span(selection_set.span).start, f);
        }
        selection::write_selection_set(selection_set, f);
    }
}

fn write_fragment_definition<'a>(
    fragment: &'a FragmentDefinition<'a>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    common::write_description(fragment.description.as_deref(), f);
    write!(f, "fragment ");
    common::write_name(&fragment.name, f);
    // Fragment arguments
    common::write_variable_definitions(fragment.variable_definitions.as_ref(), f);
    common::write_spaced_keyword(to_span(fragment.type_condition.span).start, "on ", f);
    common::write_named_type(&fragment.type_condition.named_type, f);
    common::write_directives(&fragment.directives, DirectivesStyle::Definition, f);
    write!(f, space());
    if let Some(selection_set) = fragment.selection_set.as_ref() {
        flush_trailing_comment_before(to_span(selection_set.span).start, f);
        selection::write_selection_set(selection_set, f);
    }
}

fn write_directive_definition<'a>(
    directive: &'a DirectiveDefinition<'a>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    common::write_description(directive.description.as_deref(), f);
    write!(f, "directive @");
    common::write_name(&directive.name, f);
    write_arguments_definition(directive.arguments.as_ref(), f);
    common::write_directives(&directive.directives, DirectivesStyle::Attached, f);
    // `repeatable` and `on` are bare keywords with no node;
    // scan from the last positioned token to bound the trailing claims.
    let keyword_from = directive
        .directives
        .last()
        .map(|d| to_span(d.span).end)
        .or_else(|| directive.arguments.as_ref().map(|a| to_span(a.span).end))
        .unwrap_or_else(|| to_span(directive.name.span).end);
    if directive.repeatable {
        write!(f, space());
        flush_trailing_before_literal(keyword_from, f);
        write!(f, "repeatable");
    }
    write!(f, space());
    if !directive.repeatable {
        // With `repeatable` present there is no cheap anchor for the `on`;
        // a comment between the two keywords drains via the sequence-level fallback.
        flush_trailing_before_literal(keyword_from, f);
    }
    write!(f, "on ");
    let locations = directive.locations.as_slice();
    for (i, location) in locations.iter().enumerate() {
        if i > 0 {
            write!(f, " | ");
        }
        write!(f, text(location.name));
    }
}

fn write_directive_extension<'a>(
    directive: &'a DirectiveExtension<'a>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    write!(f, "extend directive @");
    common::write_name(&directive.name, f);
    common::write_directives(&directive.directives, DirectivesStyle::Attached, f);
}

fn write_schema_definition<'a>(schema: &'a SchemaDefinition<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    common::write_description(schema.description.as_deref(), f);
    write!(f, "schema");
    common::write_directives(&schema.directives, DirectivesStyle::Attached, f);
    write!(f, space());
    // The `{` has no node (the spec names no production for the root-operation braces);
    // scan from the last directive when there is one to anchor from.
    // Directive-less schemas have no anchor (`schema` is a bare keyword)
    // and fall through to the braced-body claim, which lands after the `{`.
    if let Some(d) = schema.directives.last() {
        flush_trailing_before_literal(to_span(d.span).end, f);
    }
    write!(f, "{");
    let operation_types = schema.root_operations.as_slice();
    if operation_types.is_empty() {
        write!(f, [hard_line_break(), "}"]);
    } else {
        write_braced_body(f, operation_types, close_delim_start(schema.span), |i, f| {
            write_root_operation_type_definition(&operation_types[i], f);
        });
    }
}

fn write_schema_extension<'a>(schema: &'a SchemaExtension<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    write!(f, "extend schema");
    common::write_directives(&schema.directives, DirectivesStyle::Attached, f);
    let operation_types = schema.root_operations.as_slice();
    if !operation_types.is_empty() {
        write!(f, space());
        // See `write_schema_definition` on the directive-less fall-through
        if let Some(d) = schema.directives.last() {
            flush_trailing_before_literal(to_span(d.span).end, f);
        }
        write!(f, "{");
        write_braced_body(f, operation_types, close_delim_start(schema.span), |i, f| {
            write_root_operation_type_definition(&operation_types[i], f);
        });
    }
}

fn write_root_operation_type_definition<'a>(
    def: &'a RootOperationTypeDefinition<'a>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    write!(f, text(operation_type_keyword(def.operation_type)));
    write!(f, ": ");
    common::write_named_type(&def.named_type, f);
}

/// ` { items... }` for a grammar-production body node (`FieldsDefinition`, ...):
/// claims a trailing comment in front of the `{`
/// (the space is written first so the boundary's break discards it as pending), then the braced body.
/// Emits nothing for an empty list (`{}` parses only on the error path).
fn write_braced_section<'a, T, F>(
    f: &mut GraphqlFormatter<'_, 'a>,
    items: &'a [T],
    list_span: ast::Span,
    write_item: F,
) where
    T: Spanned,
    F: Fn(usize, &mut GraphqlFormatter<'_, 'a>),
{
    if items.is_empty() {
        return;
    }
    write!(f, space());
    flush_trailing_comment_before(to_span(list_span).start, f);
    write!(f, "{");
    write_braced_body(f, items, close_delim_start(list_span), write_item);
}

/// The body of an already-opened `{`: a hard-line sequence (blank lines preserved),
/// comments pending before the closing brace, then the `}` itself.
fn write_braced_body<'a, T, F>(
    f: &mut GraphqlFormatter<'_, 'a>,
    items: &[T],
    r_curly_start: u32,
    write_item: F,
) where
    T: Spanned,
    F: Fn(usize, &mut GraphqlFormatter<'_, 'a>),
{
    // `{ # c`: keep the comment on the `{` line; the block indent breaks after it
    flush_trailing_comment_before_break(items[0].span().start, f);
    let body = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
        let last_end = write_sequence(f, items, SeparatorKind::Hard, true, &write_item);
        if let Some(last_end) = last_end {
            flush_trailing_inside_comments(last_end, r_curly_start, f);
        }
    });
    write!(f, [block_indent(&body), "}"]);
}

fn write_scalar_type<'a>(
    description: Option<&StringValue<'a>>,
    extend: bool,
    name: &Name<'a>,
    directives: &'a [Directive<'a>],
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    common::write_description(description, f);
    if extend {
        write!(f, "extend ");
    }
    write!(f, "scalar ");
    common::write_name(name, f);
    common::write_directives(directives, DirectivesStyle::Attached, f);
}

#[expect(clippy::too_many_arguments)]
fn write_object_like<'a>(
    description: Option<&StringValue<'a>>,
    extend: bool,
    keyword: &'static str,
    name: &Name<'a>,
    implements: Option<&'a ast::ImplementsInterfaces<'a>>,
    directives: &'a [Directive<'a>],
    fields: Option<&'a ast::FieldsDefinition<'a>>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    common::write_description(description, f);
    if extend {
        write!(f, "extend ");
    }
    write!(f, [keyword, space()]);
    common::write_name(name, f);
    common::write_implements_interfaces(implements, f);
    common::write_directives(directives, DirectivesStyle::Attached, f);

    if let Some(fields) = fields {
        write_braced_section(f, &fields.fields, fields.span, |i, f| {
            write_field_definition(&fields.fields[i], f);
        });
    }
}

fn write_input_object_like<'a>(
    description: Option<&StringValue<'a>>,
    extend: bool,
    name: &Name<'a>,
    directives: &'a [Directive<'a>],
    fields: Option<&'a ast::InputFieldsDefinition<'a>>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    common::write_description(description, f);
    if extend {
        write!(f, "extend ");
    }
    write!(f, "input ");
    common::write_name(name, f);
    common::write_directives(directives, DirectivesStyle::Attached, f);

    if let Some(fields) = fields {
        write_braced_section(f, &fields.fields, fields.span, |i, f| {
            common::write_input_value_definition(&fields.fields[i], f);
        });
    }
}

fn write_enum_like<'a>(
    description: Option<&StringValue<'a>>,
    extend: bool,
    name: &Name<'a>,
    directives: &'a [Directive<'a>],
    values: Option<&'a ast::EnumValuesDefinition<'a>>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    common::write_description(description, f);
    if extend {
        write!(f, "extend ");
    }
    write!(f, "enum ");
    common::write_name(name, f);
    common::write_directives(directives, DirectivesStyle::Attached, f);

    if let Some(values) = values {
        write_braced_section(f, &values.values, values.span, |i, f| {
            write_enum_value_definition(&values.values[i], f);
        });
    }
}

fn write_enum_value_definition<'a>(
    value: &'a EnumValueDefinition<'a>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    common::write_description(value.description.as_deref(), f);
    common::write_name(&value.value.name, f);
    common::write_directives(&value.directives, DirectivesStyle::Attached, f);
}

fn write_union_like<'a>(
    description: Option<&StringValue<'a>>,
    extend: bool,
    name: &'a Name<'a>,
    directives: &'a [Directive<'a>],
    member_types: Option<&'a ast::UnionMemberTypes<'a>>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let content = format_with(move |f: &mut GraphqlFormatter<'_, 'a>| {
        common::write_description(description, f);
        let inner = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
            if extend {
                write!(f, "extend ");
            }
            write!(f, "union ");
            common::write_name(name, f);
            common::write_directives(directives, DirectivesStyle::Attached, f);
            if let Some(member_types) = member_types
                && !member_types.members.is_empty()
            {
                let members = member_types.members.as_slice();
                write!(f, space());
                // The node's span starts at the `=`
                flush_trailing_comment_before(to_span(member_types.span).start, f);
                write!(f, "=");
                write!(f, if_group_fits_on_line(&space()));
                let body = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
                    let leader = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
                        write!(f, [soft_line_break_or_space(), "| "]);
                    });
                    write!(f, if_group_breaks(&leader));
                    for (i, named) in members.iter().enumerate() {
                        if i > 0 {
                            // `A # c` + newline + `| B`: the separator's break flushes it;
                            // bounded at the `|` so a comment trailing it stays put.
                            flush_trailing_before_literal_break(
                                to_span(members[i - 1].name.span).end,
                                f,
                            );
                            write!(f, [soft_line_break_or_space(), "| "]);
                        }
                        common::write_named_type(named, f);
                    }
                });
                write!(f, indent(&body));
            }
        });
        write!(f, group(&inner));
    });
    write!(f, group(&content));
}

fn write_field_definition<'a>(field: &'a FieldDefinition<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    common::write_description(field.description.as_deref(), f);
    common::write_name(&field.name, f);
    write_arguments_definition(field.arguments.as_ref(), f);
    // The `:` has no node; scan from the `)` (or the name without arguments)
    let colon_from =
        field.arguments.as_ref().map_or(to_span(field.name.span).end, |a| to_span(a.span).end);
    flush_trailing_before_literal(colon_from, f);
    write!(f, ": ");
    if let Some(ty) = field.ty.as_ref() {
        common::write_type(ty, f);
    }
    common::write_directives(&field.directives, DirectivesStyle::Attached, f);
}

/// `(name: Type = default @dir, ...)` on field and directive definitions.
/// Blank lines between entries are preserved (Prettier routes these through `printSequence`).
fn write_arguments_definition<'a>(
    arguments: Option<&'a ast::ArgumentsDefinition<'a>>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let Some(arguments) = arguments else { return };
    let items = arguments.items.as_slice();
    common::write_paren_list(f, items, arguments.span, true, |i, f| {
        common::write_input_value_definition(&items[i], f);
    });
}
