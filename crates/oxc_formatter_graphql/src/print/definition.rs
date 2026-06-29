//! Document-level definition printers: operations, fragments, and the type system.

use apollo_parser::{cst, cst::CstNode};
use oxc_formatter_core::{
    Buffer,
    builders::{
        block_indent, group, hard_line_break, if_group_breaks, if_group_fits_on_line, indent,
        soft_line_break_or_space, space, text,
    },
    write,
};

use crate::comments::flush_trailing_inside_comments;

use super::{
    GraphqlFormatter, SeparatorKind, closing_token_start, common, common::DirectivesStyle,
    format_with, node_text, selection, write_sequence,
};

pub fn write_definition(definition: &cst::Definition, f: &mut GraphqlFormatter<'_, '_>) {
    match definition {
        cst::Definition::OperationDefinition(d) => write_operation_definition(d, f),
        cst::Definition::FragmentDefinition(d) => write_fragment_definition(d, f),
        cst::Definition::DirectiveDefinition(d) => write_directive_definition(d, f),
        cst::Definition::SchemaDefinition(d) => write_schema_definition(d, f),
        cst::Definition::ScalarTypeDefinition(d) => {
            write_scalar_type(d.description(), false, d.name(), d.directives(), f);
        }
        cst::Definition::ScalarTypeExtension(d) => {
            write_scalar_type(None, true, d.name(), d.directives(), f);
        }
        cst::Definition::ObjectTypeDefinition(d) => write_object_like(
            d.description(),
            false,
            "type",
            d.name(),
            d.implements_interfaces(),
            d.directives(),
            d.fields_definition(),
            f,
        ),
        cst::Definition::ObjectTypeExtension(d) => write_object_like(
            None,
            true,
            "type",
            d.name(),
            d.implements_interfaces(),
            d.directives(),
            d.fields_definition(),
            f,
        ),
        cst::Definition::InterfaceTypeDefinition(d) => write_object_like(
            d.description(),
            false,
            "interface",
            d.name(),
            d.implements_interfaces(),
            d.directives(),
            d.fields_definition(),
            f,
        ),
        cst::Definition::InterfaceTypeExtension(d) => write_object_like(
            None,
            true,
            "interface",
            d.name(),
            d.implements_interfaces(),
            d.directives(),
            d.fields_definition(),
            f,
        ),
        cst::Definition::InputObjectTypeDefinition(d) => write_input_object_like(
            d.description(),
            false,
            d.name(),
            d.directives(),
            d.input_fields_definition(),
            f,
        ),
        cst::Definition::InputObjectTypeExtension(d) => write_input_object_like(
            None,
            true,
            d.name(),
            d.directives(),
            d.input_fields_definition(),
            f,
        ),
        cst::Definition::UnionTypeDefinition(d) => write_union_like(
            d.description(),
            false,
            d.name(),
            d.directives(),
            d.union_member_types(),
            f,
        ),
        cst::Definition::UnionTypeExtension(d) => {
            write_union_like(None, true, d.name(), d.directives(), d.union_member_types(), f);
        }
        cst::Definition::EnumTypeDefinition(d) => write_enum_like(
            d.description(),
            false,
            d.name(),
            d.directives(),
            d.enum_values_definition(),
            f,
        ),
        cst::Definition::EnumTypeExtension(d) => {
            write_enum_like(None, true, d.name(), d.directives(), d.enum_values_definition(), f);
        }
        cst::Definition::SchemaExtension(d) => write_schema_extension(d, f),
    }
}

fn write_operation_definition(
    operation: &cst::OperationDefinition,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    let has_operation = operation.operation_type().is_some();
    let has_name = operation.name().is_some();

    if let Some(operation_type) = operation.operation_type() {
        write!(f, text(node_text(f, operation_type.syntax())));
    }
    if let Some(name) = operation.name() {
        write!(f, space());
        common::write_name(&name, f);
    }
    let has_variable_definitions = operation
        .variable_definitions()
        .is_some_and(|vd| vd.variable_definitions().next().is_some());
    if has_operation && !has_name && has_variable_definitions {
        write!(f, space());
    }
    common::write_variable_definitions(operation.variable_definitions(), f);
    common::write_directives(operation.directives(), DirectivesStyle::Definition, f);
    if has_operation || has_name {
        write!(f, space());
    }
    if let Some(selection_set) = operation.selection_set() {
        selection::write_selection_set(&selection_set, f);
    }
}

fn write_fragment_definition(fragment: &cst::FragmentDefinition, f: &mut GraphqlFormatter<'_, '_>) {
    write!(f, "fragment ");
    if let Some(fragment_name) = fragment.fragment_name()
        && let Some(name) = fragment_name.name()
    {
        common::write_name(&name, f);
    }
    if let Some(type_condition) = fragment.type_condition()
        && let Some(named) = type_condition.named_type()
    {
        write!(f, " on ");
        common::write_named_type(&named, f);
    }
    common::write_directives(fragment.directives(), DirectivesStyle::Definition, f);
    write!(f, space());
    if let Some(selection_set) = fragment.selection_set() {
        selection::write_selection_set(&selection_set, f);
    }
}

fn write_directive_definition(
    directive: &cst::DirectiveDefinition,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    common::write_description(directive.description(), f);
    write!(f, "directive @");
    if let Some(name) = directive.name() {
        common::write_name(&name, f);
    }
    write_arguments_definition(directive.arguments_definition(), f);
    if directive.repeatable_token().is_some() {
        write!(f, " repeatable");
    }
    write!(f, " on ");
    if let Some(locations) = directive.directive_locations() {
        let list: Vec<cst::DirectiveLocation> = locations.directive_locations().collect();
        for (i, location) in list.iter().enumerate() {
            if i > 0 {
                write!(f, " | ");
            }
            write!(f, text(node_text(f, location.syntax())));
        }
    }
}

fn write_schema_definition(schema: &cst::SchemaDefinition, f: &mut GraphqlFormatter<'_, '_>) {
    common::write_description(schema.description(), f);
    write!(f, "schema");
    common::write_directives(schema.directives(), DirectivesStyle::Attached, f);
    write!(f, " {");
    let operation_types: Vec<cst::RootOperationTypeDefinition> =
        schema.root_operation_type_definitions().collect();
    if operation_types.is_empty() {
        write!(f, [hard_line_break(), "}"]);
    } else {
        let r_curly = closing_token_start(schema.r_curly_token(), schema.syntax());
        write_braced_body(f, &operation_types, r_curly, |i, f| {
            write_root_operation_type_definition(&operation_types[i], f);
        });
    }
}

fn write_schema_extension(schema: &cst::SchemaExtension, f: &mut GraphqlFormatter<'_, '_>) {
    write!(f, "extend schema");
    common::write_directives(schema.directives(), DirectivesStyle::Attached, f);
    let operation_types: Vec<cst::RootOperationTypeDefinition> =
        schema.root_operation_type_definitions().collect();
    if !operation_types.is_empty() {
        write!(f, " {");
        let r_curly = closing_token_start(schema.r_curly_token(), schema.syntax());
        write_braced_body(f, &operation_types, r_curly, |i, f| {
            write_root_operation_type_definition(&operation_types[i], f);
        });
    }
}

fn write_root_operation_type_definition(
    operation_type_definition: &cst::RootOperationTypeDefinition,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    if let Some(operation_type) = operation_type_definition.operation_type() {
        write!(f, text(node_text(f, operation_type.syntax())));
    }
    write!(f, ": ");
    if let Some(named) = operation_type_definition.named_type() {
        common::write_named_type(&named, f);
    }
}

/// The body of an already-opened `{`: a hard-line sequence (blank lines preserved),
/// comments pending before the closing brace, then the `}` itself.
fn write_braced_body<'a, T, F>(
    f: &mut GraphqlFormatter<'_, 'a>,
    items: &[T],
    r_curly_start: u32,
    write_item: F,
) where
    T: CstNode,
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

fn write_scalar_type(
    description: Option<cst::Description>,
    extend: bool,
    name: Option<cst::Name>,
    directives: Option<cst::Directives>,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    common::write_description(description, f);
    if extend {
        write!(f, "extend ");
    }
    write!(f, "scalar ");
    if let Some(name) = name {
        common::write_name(&name, f);
    }
    common::write_directives(directives, DirectivesStyle::Attached, f);
}

#[expect(clippy::too_many_arguments)]
fn write_object_like(
    description: Option<cst::Description>,
    extend: bool,
    keyword: &'static str,
    name: Option<cst::Name>,
    implements: Option<cst::ImplementsInterfaces>,
    directives: Option<cst::Directives>,
    fields: Option<cst::FieldsDefinition>,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    common::write_description(description, f);
    if extend {
        write!(f, "extend ");
    }
    write!(f, [keyword, space()]);
    if let Some(name) = name {
        common::write_name(&name, f);
    }
    common::write_implements_interfaces(implements, f);
    common::write_directives(directives, DirectivesStyle::Attached, f);

    if let Some(fields) = fields {
        let items: Vec<cst::FieldDefinition> = fields.field_definitions().collect();
        if !items.is_empty() {
            write!(f, " {");
            let r_curly = closing_token_start(fields.r_curly_token(), fields.syntax());
            write_braced_body(f, &items, r_curly, |i, f| {
                write_field_definition(&items[i], f);
            });
        }
    }
}

fn write_input_object_like(
    description: Option<cst::Description>,
    extend: bool,
    name: Option<cst::Name>,
    directives: Option<cst::Directives>,
    fields: Option<cst::InputFieldsDefinition>,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    common::write_description(description, f);
    if extend {
        write!(f, "extend ");
    }
    write!(f, "input ");
    if let Some(name) = name {
        common::write_name(&name, f);
    }
    common::write_directives(directives, DirectivesStyle::Attached, f);

    if let Some(fields) = fields {
        let items: Vec<cst::InputValueDefinition> = fields.input_value_definitions().collect();
        if !items.is_empty() {
            write!(f, " {");
            let r_curly = closing_token_start(fields.r_curly_token(), fields.syntax());
            write_braced_body(f, &items, r_curly, |i, f| {
                common::write_input_value_definition(&items[i], f);
            });
        }
    }
}

fn write_enum_like(
    description: Option<cst::Description>,
    extend: bool,
    name: Option<cst::Name>,
    directives: Option<cst::Directives>,
    values: Option<cst::EnumValuesDefinition>,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    common::write_description(description, f);
    if extend {
        write!(f, "extend ");
    }
    write!(f, "enum ");
    if let Some(name) = name {
        common::write_name(&name, f);
    }
    common::write_directives(directives, DirectivesStyle::Attached, f);

    if let Some(values) = values {
        let items: Vec<cst::EnumValueDefinition> = values.enum_value_definitions().collect();
        if !items.is_empty() {
            write!(f, " {");
            let r_curly = closing_token_start(values.r_curly_token(), values.syntax());
            write_braced_body(f, &items, r_curly, |i, f| {
                write_enum_value_definition(&items[i], f);
            });
        }
    }
}

fn write_enum_value_definition(value: &cst::EnumValueDefinition, f: &mut GraphqlFormatter<'_, '_>) {
    common::write_description(value.description(), f);
    if let Some(enum_value) = value.enum_value()
        && let Some(name) = enum_value.name()
    {
        common::write_name(&name, f);
    }
    common::write_directives(value.directives(), DirectivesStyle::Attached, f);
}

// The CST handles are captured by `Fn` closures (callable multiple times),
// so they are cloned inside; pass-by-value keeps the call sites uniform.
#[expect(clippy::needless_pass_by_value)]
fn write_union_like<'a>(
    description: Option<cst::Description>,
    extend: bool,
    name: Option<cst::Name>,
    directives: Option<cst::Directives>,
    members: Option<cst::UnionMemberTypes>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let content = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
        common::write_description(description.clone(), f);
        let inner = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
            if extend {
                write!(f, "extend ");
            }
            write!(f, "union ");
            if let Some(name) = name.clone() {
                common::write_name(&name, f);
            }
            common::write_directives(directives.clone(), DirectivesStyle::Attached, f);
            if let Some(members) = members.clone() {
                let types: Vec<cst::NamedType> = members.named_types().collect();
                if !types.is_empty() {
                    write!(f, " =");
                    write!(f, if_group_fits_on_line(&space()));
                    let body = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
                        let leader = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
                            write!(f, [soft_line_break_or_space(), "| "]);
                        });
                        write!(f, if_group_breaks(&leader));
                        for (i, named) in types.iter().enumerate() {
                            if i > 0 {
                                write!(f, [soft_line_break_or_space(), "| "]);
                            }
                            common::write_named_type(named, f);
                        }
                    });
                    write!(f, indent(&body));
                }
            }
        });
        write!(f, group(&inner));
    });
    write!(f, group(&content));
}

fn write_field_definition(field: &cst::FieldDefinition, f: &mut GraphqlFormatter<'_, '_>) {
    common::write_description(field.description(), f);
    if let Some(name) = field.name() {
        common::write_name(&name, f);
    }
    write_arguments_definition(field.arguments_definition(), f);
    write!(f, ": ");
    if let Some(ty) = field.ty() {
        common::write_type(&ty, f);
    }
    common::write_directives(field.directives(), DirectivesStyle::Attached, f);
}

/// `(name: Type = default @dir, ...)` on field and directive definitions.
/// Blank lines between entries are preserved (Prettier routes these through `printSequence`).
fn write_arguments_definition(
    arguments: Option<cst::ArgumentsDefinition>,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    let Some(arguments) = arguments else { return };
    let items: Vec<cst::InputValueDefinition> = arguments.input_value_definitions().collect();
    if items.is_empty() {
        return;
    }
    let r_paren = closing_token_start(arguments.r_paren_token(), arguments.syntax());
    common::write_paren_list(f, &items, r_paren, true, |i, f| {
        common::write_input_value_definition(&items[i], f);
    });
}
