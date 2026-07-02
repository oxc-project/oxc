//! Value printers (scalars, variables, lists, objects).
//!
//! `StringValue` cooking and re-encoding lives in the sibling [`super::string`] module.

use oxc_formatter_core::{
    Buffer, FormatContext,
    builders::{group, soft_block_indent, soft_block_indent_with_maybe_space, text},
    write,
};
use oxc_graphql_parser::ast::{ListValue, ObjectValue, Value};

use crate::comments::flush_trailing_inside_comments;

use super::{
    GraphqlFormatter, SeparatorKind, common, format_with, span::close_delim_start, string,
    write_sequence,
};

pub(super) fn write_value<'a>(value: &'a Value<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    match value {
        Value::Variable(variable) => common::write_variable(variable, f),
        Value::String(sv) => string::write_string_value(sv, f),
        Value::Float(v) => write!(f, text(v.raw)),
        Value::Int(v) => write!(f, text(v.raw)),
        Value::Boolean(v) => write!(f, text(if v.value { "true" } else { "false" })),
        Value::Null(_) => write!(f, "null"),
        Value::Enum(v) => write!(f, text(v.name.value)),
        Value::List(list) => write_list_value(list, f),
        Value::Object(object) => write_object_value(object, f),
        Value::Missing(_) => {}
    }
}

fn write_list_value<'a>(list: &'a ListValue<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    let values = list.values.as_slice();
    let r_brack = close_delim_start(list.span);

    write!(f, "[");
    if values.is_empty() {
        common::write_empty_delimited(r_brack, "]", f);
        return;
    }

    let body = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
        // No blank-line preservation (Prettier uses a plain `path.map` for list values).
        let last_end = write_sequence(f, values, SeparatorKind::CommaSoftline, false, |i, f| {
            write_value(&values[i], f);
        });
        if let Some(last_end) = last_end {
            flush_trailing_inside_comments(last_end, r_brack, f);
        }
    });
    write!(f, [group(&soft_block_indent(&body)), "]"]);
}

fn write_object_value<'a>(object: &'a ObjectValue<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    let fields = object.fields.as_slice();
    let r_curly = close_delim_start(object.span);

    write!(f, "{");
    if fields.is_empty() {
        common::write_empty_delimited(r_curly, "}", f);
        return;
    }

    let bracket_spacing = f.context().options().bracket_spacing.value();
    let body = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
        let last_end = write_sequence(f, fields, SeparatorKind::CommaSoftline, false, |i, f| {
            let field = &fields[i];
            common::write_name(&field.name, f);
            write!(f, ": ");
            if let Some(v) = field.value.as_ref() {
                write_value(v, f);
            }
        });
        if let Some(last_end) = last_end {
            flush_trailing_inside_comments(last_end, r_curly, f);
        }
    });
    write!(f, [group(&soft_block_indent_with_maybe_space(&body, bracket_spacing)), "}"]);
}
