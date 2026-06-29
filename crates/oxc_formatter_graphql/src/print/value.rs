//! Value printers (scalars, variables, lists, objects).
//!
//! `StringValue` cooking and re-encoding lives in the sibling [`super::string`] module.

use oxc_graphql_parser::{cst, cst::CstNode};

use oxc_formatter_core::{
    Buffer, FormatContext,
    builders::{group, soft_block_indent_with_maybe_space, text},
    write,
};

use crate::comments::flush_trailing_inside_comments;

use super::{
    GraphqlFormatter, SeparatorKind, common, format_with,
    sig::{closing_token_start, node_text},
    string, write_sequence,
};

pub fn write_value(value: &cst::Value, f: &mut GraphqlFormatter<'_, '_>) {
    match value {
        cst::Value::Variable(variable) => common::write_variable(variable, f),
        cst::Value::StringValue(sv) => string::write_string_value(sv, f),
        cst::Value::FloatValue(v) => write!(f, text(node_text(f, v.syntax()))),
        cst::Value::IntValue(v) => write!(f, text(node_text(f, v.syntax()))),
        cst::Value::BooleanValue(v) => write!(f, text(node_text(f, v.syntax()))),
        cst::Value::NullValue(_) => write!(f, "null"),
        cst::Value::EnumValue(v) => write!(f, text(node_text(f, v.syntax()))),
        cst::Value::ListValue(list) => write_list_value(list, f),
        cst::Value::ObjectValue(object) => write_object_value(object, f),
    }
}

fn write_list_value<'a>(list: &cst::ListValue, f: &mut GraphqlFormatter<'_, 'a>) {
    let values: Vec<cst::Value> = list.values().collect();
    let r_brack = closing_token_start(list.r_brack_token(), list.syntax());

    write!(f, "[");
    if values.is_empty() {
        common::write_empty_delimited(r_brack, "]", f);
        return;
    }

    let body = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
        // No blank-line preservation (Prettier uses a plain `path.map` for list values).
        let last_end = write_sequence(f, &values, SeparatorKind::CommaSoftline, false, |i, f| {
            write_value(&values[i], f);
        });
        if let Some(last_end) = last_end {
            flush_trailing_inside_comments(last_end, r_brack, f);
        }
    });
    write!(f, [group(&oxc_formatter_core::builders::soft_block_indent(&body)), "]"]);
}

fn write_object_value<'a>(object: &cst::ObjectValue, f: &mut GraphqlFormatter<'_, 'a>) {
    let fields: Vec<cst::ObjectField> = object.object_fields().collect();
    let r_curly = closing_token_start(object.r_curly_token(), object.syntax());

    write!(f, "{");
    if fields.is_empty() {
        common::write_empty_delimited(r_curly, "}", f);
        return;
    }

    let bracket_spacing = f.context().options().bracket_spacing.value();
    let body = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
        let last_end = write_sequence(f, &fields, SeparatorKind::CommaSoftline, false, |i, f| {
            let field = &fields[i];
            if let Some(name) = field.name() {
                common::write_name(&name, f);
            }
            write!(f, ": ");
            if let Some(v) = field.value() {
                write_value(&v, f);
            }
        });
        if let Some(last_end) = last_end {
            flush_trailing_inside_comments(last_end, r_curly, f);
        }
    });
    write!(f, [group(&soft_block_indent_with_maybe_space(&body, bracket_spacing)), "}"]);
}
