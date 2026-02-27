use std::fmt;

use oxc_allocator::Allocator;
use oxc_formatter_core::formatter::prelude::*;
use oxc_formatter_core::formatter::{Argument, Arguments, Format, Formatter};
use oxc_formatter_core::write;
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::{JsonFormatContext, JsonFormatOptions};

#[derive(Debug)]
pub enum JsonFormatError {
    Parse(String),
}

impl JsonFormatError {
    pub fn is_parse_error(&self) -> bool {
        matches!(self, Self::Parse(_))
    }
}

impl fmt::Display for JsonFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(err) => {
                f.write_str("JSON parse error: ")?;
                f.write_str(err)
            }
        }
    }
}

impl std::error::Error for JsonFormatError {}

#[derive(Debug, Clone)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(JsonNumber),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

#[derive(Debug, Clone, Copy)]
enum JsonNumber {
    I64(i64),
    U64(u64),
    F64(f64),
    Infinity,
    NegInfinity,
    NaN,
}

impl JsonNumber {
    fn from_f64(value: f64) -> Self {
        if value.is_nan() {
            return Self::NaN;
        }
        if value == f64::INFINITY {
            return Self::Infinity;
        }
        if value == f64::NEG_INFINITY {
            return Self::NegInfinity;
        }
        Self::F64(value)
    }
}

impl<'de> Deserialize<'de> for JsonValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct JsonValueVisitor;

        impl<'de> Visitor<'de> for JsonValueVisitor {
            type Value = JsonValue;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON/JSON5 value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                Ok(JsonValue::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(JsonValue::Number(JsonNumber::I64(value)))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                Ok(JsonValue::Number(JsonNumber::U64(value)))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
                Ok(JsonValue::Number(JsonNumber::from_f64(value)))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
                Ok(JsonValue::String(value.to_string()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                Ok(JsonValue::String(value))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(JsonValue::Null)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(JsonValue::Null)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element::<JsonValue>()? {
                    values.push(value);
                }
                Ok(JsonValue::Array(values))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut entries = Vec::new();
                while let Some((key, value)) = map.next_entry::<String, JsonValue>()? {
                    entries.push((key, value));
                }
                Ok(JsonValue::Object(entries))
            }
        }

        deserializer.deserialize_any(JsonValueVisitor)
    }
}

fn format_finite_number(value: f64) -> String {
    let mut number = serde_json::Number::from_f64(value)
        .map(|number| {
            serde_json::to_string(&number)
                .expect("serializing a JSON number into string should not fail")
        })
        .expect("finite f64 must be representable as a JSON number");

    // `json5` can deserialize integer-like values as floats (`1e10` -> `10000000000.0`).
    // Match Prettier's json-stringify behavior by removing the trailing `.0`.
    if number.ends_with(".0") {
        number.truncate(number.len() - 2);
    }

    number
}

pub fn format_json(
    source_text: &str,
    options: JsonFormatOptions,
) -> Result<String, JsonFormatError> {
    let value: JsonValue =
        json5::from_str(source_text).map_err(|err| JsonFormatError::Parse(err.to_string()))?;

    Ok(format_json_value(&value, options))
}

fn format_json_value(value: &JsonValue, options: JsonFormatOptions) -> String {
    let allocator = Allocator::default();
    let context = JsonFormatContext::new(&allocator, options);

    let formatted =
        oxc_formatter_core::formatter::format(context, Arguments::new(&[Argument::new(value)]));
    let printed = formatted.print().expect("printing JSON formatter output should not fail");

    let mut code = printed.into_code();
    code.push_str(match options.line_ending {
        oxc_formatter_core::LineEnding::Lf => "\n",
        oxc_formatter_core::LineEnding::Crlf => "\r\n",
        oxc_formatter_core::LineEnding::Cr => "\r",
    });
    code
}

impl<'ast> Format<'ast, JsonFormatContext<'ast>> for JsonValue {
    fn fmt(&self, f: &mut Formatter<'_, 'ast, JsonFormatContext<'ast>>) {
        match self {
            JsonValue::Null => write!(f, [token("null")]),
            JsonValue::Bool(value) => {
                if *value {
                    write!(f, [token("true")]);
                } else {
                    write!(f, [token("false")]);
                }
            }
            JsonValue::Number(number) => {
                let number = match number {
                    JsonNumber::I64(value) => value.to_string(),
                    JsonNumber::U64(value) => value.to_string(),
                    JsonNumber::F64(value) => format_finite_number(*value),
                    JsonNumber::Infinity => "Infinity".to_string(),
                    JsonNumber::NegInfinity => "-Infinity".to_string(),
                    JsonNumber::NaN => "NaN".to_string(),
                };
                let number = f.context().allocator().alloc_str(&number);
                write!(f, [text(number)]);
            }
            JsonValue::String(value) => {
                let string = serde_json::to_string(value)
                    .expect("serializing a JSON string into string should not fail");
                let string = f.context().allocator().alloc_str(&string);
                write!(f, [text(string)]);
            }
            JsonValue::Array(items) => {
                if items.is_empty() {
                    write!(f, [token("[]")]);
                    return;
                }

                if f.options().always_expand {
                    write!(
                        f,
                        [
                            token("["),
                            block_indent(&format_with(
                                |f: &mut Formatter<'_, 'ast, JsonFormatContext<'ast>>| {
                                    for (index, value) in items.iter().enumerate() {
                                        value.fmt(f);
                                        if index + 1 != items.len() {
                                            write!(f, [token(","), hard_line_break()]);
                                        }
                                    }
                                }
                            )),
                            token("]")
                        ]
                    );
                    return;
                }

                write!(
                    f,
                    [group(&oxc_formatter_core::format_args!(
                        token("["),
                        soft_block_indent(&format_with(
                            |f: &mut Formatter<'_, 'ast, JsonFormatContext<'ast>>| {
                                for (index, value) in items.iter().enumerate() {
                                    value.fmt(f);
                                    if index + 1 != items.len() {
                                        write!(f, [token(","), soft_line_break_or_space()]);
                                    }
                                }
                            }
                        )),
                        token("]")
                    ))]
                );
            }
            JsonValue::Object(entries) => {
                if entries.is_empty() {
                    write!(f, [token("{}")]);
                    return;
                }

                if f.options().always_expand {
                    write!(
                        f,
                        [
                            token("{"),
                            block_indent(&format_with(
                                |f: &mut Formatter<'_, 'ast, JsonFormatContext<'ast>>| {
                                    for (index, (key, value)) in entries.iter().enumerate() {
                                        let key_string = serde_json::to_string(key).expect(
                                            "serializing an object key into string should not fail",
                                        );
                                        let key_string =
                                            f.context().allocator().alloc_str(&key_string);
                                        write!(f, [text(key_string), token(": ")]);
                                        value.fmt(f);

                                        if index + 1 != entries.len() {
                                            write!(f, [token(","), hard_line_break()]);
                                        }
                                    }
                                }
                            )),
                            token("}")
                        ]
                    );
                    return;
                }

                write!(
                    f,
                    [group(&oxc_formatter_core::format_args!(
                        token("{"),
                        soft_block_indent(&format_with(
                            |f: &mut Formatter<'_, 'ast, JsonFormatContext<'ast>>| {
                                for (index, (key, value)) in entries.iter().enumerate() {
                                    let key_string = serde_json::to_string(key).expect(
                                        "serializing an object key into string should not fail",
                                    );
                                    let key_string = f.context().allocator().alloc_str(&key_string);
                                    write!(f, [text(key_string), token(": ")]);
                                    value.fmt(f);

                                    if index + 1 != entries.len() {
                                        write!(f, [token(","), soft_line_break_or_space()]);
                                    }
                                }
                            }
                        )),
                        token("}")
                    ))]
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_json5_comments_and_trailing_commas() {
        let input = r#"
        {
          // Comment
          "a": 1,
          "b": [2, 3,],
        }
        "#;

        let output = format_json(input, JsonFormatOptions::default()).unwrap();
        assert_eq!(output, "{\"a\": 1, \"b\": [2, 3]}\n");
    }

    #[test]
    fn formats_primitives() {
        let input = r#"{"s":"\n","n":1.50,"b":true,"z":null}"#;
        let output = format_json(input, JsonFormatOptions::default()).unwrap();
        assert_eq!(output, "{\"s\": \"\\n\", \"n\": 1.5, \"b\": true, \"z\": null}\n");
    }

    #[test]
    fn formats_json_stringify_style() {
        let input = r#"{"a":1,"b":[2,3]}"#;
        let output = format_json(
            input,
            JsonFormatOptions { always_expand: true, ..JsonFormatOptions::default() },
        )
        .unwrap();
        assert_eq!(output, "{\n  \"a\": 1,\n  \"b\": [\n    2,\n    3\n  ]\n}\n");
    }

    #[test]
    fn normalizes_integer_like_float_numbers() {
        let input = r#"{"n":1e10,"m":1e-3}"#;
        let output = format_json(
            input,
            JsonFormatOptions { always_expand: true, ..JsonFormatOptions::default() },
        )
        .unwrap();
        assert_eq!(output, "{\n  \"n\": 10000000000,\n  \"m\": 0.001\n}\n");
    }

    #[test]
    fn preserves_non_finite_numbers_and_duplicate_keys() {
        let input = r#"{Infinity: NaN, NaN: Infinity, NaN: -Infinity}"#;
        let output = format_json(
            input,
            JsonFormatOptions { always_expand: true, ..JsonFormatOptions::default() },
        )
        .unwrap();
        assert_eq!(
            output,
            "{\n  \"Infinity\": NaN,\n  \"NaN\": Infinity,\n  \"NaN\": -Infinity\n}\n"
        );
    }
}
