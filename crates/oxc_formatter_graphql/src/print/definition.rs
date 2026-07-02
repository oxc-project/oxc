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
    Definition, Directive, DirectiveDefinition, EnumValueDefinition, FieldDefinition,
    FragmentDefinition, InputValueDefinition, Name, NamedType, OperationDefinition, OperationType,
    RootOperationTypeDefinition, SchemaDefinition, SchemaExtension, StringValue,
};

use crate::comments::flush_trailing_inside_comments;

use super::{
    GraphqlFormatter, SeparatorKind, common,
    common::DirectivesStyle,
    format_with, selection,
    span::{Spanned, close_delim_start},
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
        Definition::Schema(d) => write_schema_definition(d, f),
        Definition::SchemaExtension(d) => write_schema_extension(d, f),
        Definition::ScalarType(d) => {
            write_scalar_type(d.description.as_ref(), false, &d.name, &d.directives, f);
        }
        Definition::ScalarTypeExtension(d) => {
            write_scalar_type(None, true, &d.name, &d.directives, f);
        }
        Definition::ObjectType(d) => write_object_like(
            d.description.as_ref(),
            false,
            "type",
            &d.name,
            &d.interfaces,
            &d.directives,
            &d.fields,
            close_delim_start(d.span),
            f,
        ),
        Definition::ObjectTypeExtension(d) => write_object_like(
            None,
            true,
            "type",
            &d.name,
            &d.interfaces,
            &d.directives,
            &d.fields,
            close_delim_start(d.span),
            f,
        ),
        Definition::InterfaceType(d) => write_object_like(
            d.description.as_ref(),
            false,
            "interface",
            &d.name,
            &d.interfaces,
            &d.directives,
            &d.fields,
            close_delim_start(d.span),
            f,
        ),
        Definition::InterfaceTypeExtension(d) => write_object_like(
            None,
            true,
            "interface",
            &d.name,
            &d.interfaces,
            &d.directives,
            &d.fields,
            close_delim_start(d.span),
            f,
        ),
        Definition::InputObjectType(d) => write_input_object_like(
            d.description.as_ref(),
            false,
            &d.name,
            &d.directives,
            &d.fields,
            close_delim_start(d.span),
            f,
        ),
        Definition::InputObjectTypeExtension(d) => write_input_object_like(
            None,
            true,
            &d.name,
            &d.directives,
            &d.fields,
            close_delim_start(d.span),
            f,
        ),
        Definition::UnionType(d) => {
            write_union_like(d.description.as_ref(), false, &d.name, &d.directives, &d.members, f);
        }
        Definition::UnionTypeExtension(d) => {
            write_union_like(None, true, &d.name, &d.directives, &d.members, f);
        }
        Definition::EnumType(d) => write_enum_like(
            d.description.as_ref(),
            false,
            &d.name,
            &d.directives,
            &d.values,
            close_delim_start(d.span),
            f,
        ),
        Definition::EnumTypeExtension(d) => {
            write_enum_like(
                None,
                true,
                &d.name,
                &d.directives,
                &d.values,
                close_delim_start(d.span),
                f,
            );
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
    common::write_description(operation.description.as_ref(), f);

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
    let has_variable_definitions = !operation.variable_definitions.is_empty();
    if has_operation && !has_name && has_variable_definitions {
        write!(f, space());
    }
    common::write_variable_definitions(&operation.variable_definitions, f);
    common::write_directives(&operation.directives, DirectivesStyle::Definition, f);
    if has_operation || has_name {
        write!(f, space());
    }
    if let Some(selection_set) = operation.selection_set.as_ref() {
        selection::write_selection_set(selection_set, f);
    }
}

fn write_fragment_definition<'a>(
    fragment: &'a FragmentDefinition<'a>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    common::write_description(fragment.description.as_ref(), f);
    write!(f, "fragment ");
    common::write_name(&fragment.name, f);
    // Legacy fragment variables (graphql-js `allowLegacyFragmentVariables`).
    common::write_variable_definitions(&fragment.variable_definitions, f);
    write!(f, " on ");
    common::write_named_type(&fragment.type_condition, f);
    common::write_directives(&fragment.directives, DirectivesStyle::Definition, f);
    write!(f, space());
    if let Some(selection_set) = fragment.selection_set.as_ref() {
        selection::write_selection_set(selection_set, f);
    }
}

fn write_directive_definition<'a>(
    directive: &'a DirectiveDefinition<'a>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    common::write_description(directive.description.as_ref(), f);
    write!(f, "directive @");
    common::write_name(&directive.name, f);
    write_arguments_definition(&directive.arguments, f);
    if directive.repeatable {
        write!(f, " repeatable");
    }
    write!(f, " on ");
    let locations = directive.locations.as_slice();
    for (i, location) in locations.iter().enumerate() {
        if i > 0 {
            write!(f, " | ");
        }
        write!(f, text(location.name));
    }
}

fn write_schema_definition<'a>(schema: &'a SchemaDefinition<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    common::write_description(schema.description.as_ref(), f);
    write!(f, "schema");
    common::write_directives(&schema.directives, DirectivesStyle::Attached, f);
    write!(f, " {");
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
        write!(f, " {");
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
    interfaces: &'a [NamedType<'a>],
    directives: &'a [Directive<'a>],
    fields: &'a [FieldDefinition<'a>],
    r_curly_start: u32,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    common::write_description(description, f);
    if extend {
        write!(f, "extend ");
    }
    write!(f, [keyword, space()]);
    common::write_name(name, f);
    common::write_implements_interfaces(interfaces, f);
    common::write_directives(directives, DirectivesStyle::Attached, f);

    if !fields.is_empty() {
        write!(f, " {");
        write_braced_body(f, fields, r_curly_start, |i, f| {
            write_field_definition(&fields[i], f);
        });
    }
}

fn write_input_object_like<'a>(
    description: Option<&StringValue<'a>>,
    extend: bool,
    name: &Name<'a>,
    directives: &'a [Directive<'a>],
    fields: &'a [InputValueDefinition<'a>],
    r_curly_start: u32,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    common::write_description(description, f);
    if extend {
        write!(f, "extend ");
    }
    write!(f, "input ");
    common::write_name(name, f);
    common::write_directives(directives, DirectivesStyle::Attached, f);

    if !fields.is_empty() {
        write!(f, " {");
        write_braced_body(f, fields, r_curly_start, |i, f| {
            common::write_input_value_definition(&fields[i], f);
        });
    }
}

fn write_enum_like<'a>(
    description: Option<&StringValue<'a>>,
    extend: bool,
    name: &Name<'a>,
    directives: &'a [Directive<'a>],
    values: &'a [EnumValueDefinition<'a>],
    r_curly_start: u32,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    common::write_description(description, f);
    if extend {
        write!(f, "extend ");
    }
    write!(f, "enum ");
    common::write_name(name, f);
    common::write_directives(directives, DirectivesStyle::Attached, f);

    if !values.is_empty() {
        write!(f, " {");
        write_braced_body(f, values, r_curly_start, |i, f| {
            write_enum_value_definition(&values[i], f);
        });
    }
}

fn write_enum_value_definition<'a>(
    value: &'a EnumValueDefinition<'a>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    common::write_description(value.description.as_ref(), f);
    common::write_name(&value.value.name, f);
    common::write_directives(&value.directives, DirectivesStyle::Attached, f);
}

fn write_union_like<'a>(
    description: Option<&StringValue<'a>>,
    extend: bool,
    name: &'a Name<'a>,
    directives: &'a [Directive<'a>],
    members: &'a [NamedType<'a>],
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
            if !members.is_empty() {
                write!(f, " =");
                write!(f, if_group_fits_on_line(&space()));
                let body = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
                    let leader = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
                        write!(f, [soft_line_break_or_space(), "| "]);
                    });
                    write!(f, if_group_breaks(&leader));
                    for (i, named) in members.iter().enumerate() {
                        if i > 0 {
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
    common::write_description(field.description.as_ref(), f);
    common::write_name(&field.name, f);
    write_arguments_definition(&field.arguments, f);
    write!(f, ": ");
    if let Some(ty) = field.ty.as_ref() {
        common::write_type(ty, f);
    }
    common::write_directives(&field.directives, DirectivesStyle::Attached, f);
}

/// `(name: Type = default @dir, ...)` on field and directive definitions.
/// Blank lines between entries are preserved (Prettier routes these through `printSequence`).
fn write_arguments_definition<'a>(
    arguments: &'a [InputValueDefinition<'a>],
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    common::write_paren_list(f, arguments, true, |i, f| {
        common::write_input_value_definition(&arguments[i], f);
    });
}
