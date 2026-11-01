//! Value printers (scalars, variables, lists, objects).
//!
//! `StringValue` cooking and re-encoding lives in the sibling [`super::string`] module.

use oxc_formatter_core::{
    FormatContext,
    builders::{group, soft_block_indent, soft_block_indent_with_maybe_space, text},
    write,
};
use oxc_graphql_parser::ast::{ListValue, Name, ObjectValue, Value};
use oxc_span::Span;

use crate::comments::{flush_leading_comments, flush_trailing_inside_comments};

use super::{
    GraphqlFormatter, SeparatorKind, common, flush_trailing_before_literal, format_with,
    span::{close_delim_start, to_span},
    string, write_sequence,
};

pub(super) fn write_value<'a>(value: &'a Value<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    flush_leading_comments(value.span().start, f);
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

/// Source span of `value` when [write_value] prints it as its own source text (see `common::write_verbatim_colon_pair`).
pub(super) fn verbatim_value_span(value: &Value<'_>) -> Option<Span> {
    let verbatim = match value {
        Value::Int(_) | Value::Float(_) | Value::Boolean(_) | Value::Null(_) | Value::Enum(_) => {
            true
        }
        Value::String(sv) => string::is_verbatim(sv),
        Value::Variable(variable) => {
            common::verbatim_sigil_span(variable.span, "$", &variable.name).is_some()
        }
        Value::List(_) | Value::Object(_) | Value::Missing(_) => false,
    };
    verbatim.then(|| to_span(value.span()))
}

/// `name: value` of an argument or an object field, one source slice when it reads as printed.
pub(super) fn write_name_value<'a>(
    name: &Name<'a>,
    value: Option<&'a Value<'a>>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let name_span = to_span(name.span);
    if common::write_verbatim_colon_pair(name_span, value.and_then(verbatim_value_span), f) {
        return;
    }
    common::write_name(name, f);
    flush_trailing_before_literal(name_span.end, f);
    write!(f, ": ");
    if let Some(value) = value {
        write_value(value, f);
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

    // `[ # c`: the comment leads the first value on its own line, as after every opener.
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

    // `{ # c`: the comment leads the first field on its own line, as after every opener.
    let bracket_spacing = f.context().options().bracket_spacing.value();
    let body = format_with(|f: &mut GraphqlFormatter<'_, 'a>| {
        let last_end = write_sequence(f, fields, SeparatorKind::CommaSoftline, false, |i, f| {
            write_name_value(&fields[i].name, fields[i].value.as_ref(), f);
        });
        if let Some(last_end) = last_end {
            flush_trailing_inside_comments(last_end, r_curly, f);
        }
    });
    write!(f, [group(&soft_block_indent_with_maybe_space(&body, bracket_spacing)), "}"]);
}
