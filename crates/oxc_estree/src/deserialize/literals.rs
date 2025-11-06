//! Literal type conversion for ESTree to oxc AST.

use super::error::{ConversionError, ConversionResult, Span};
use super::types::EstreeLiteral;

/// The kind of literal in oxc AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralKind {
    Boolean,
    Numeric,
    String,
    BigInt,
    Null,
    RegExp,
}

/// Convert an ESTree Literal to the appropriate oxc literal type.
///
/// The conversion inspects the `value` field type and the presence of
/// a `regex` property to determine the literal kind.
pub fn convert_literal(
    estree_literal: &EstreeLiteral,
) -> ConversionResult<LiteralKind> {
    // Check for RegExp first (has `regex` property)
    if estree_literal.value.get("regex").is_some() {
        return Ok(LiteralKind::RegExp);
    }

    // Check value type
    match estree_literal.value {
        serde_json::Value::Bool(_) => Ok(LiteralKind::Boolean),
        serde_json::Value::Number(_) => Ok(LiteralKind::Numeric),
        serde_json::Value::String(_) => Ok(LiteralKind::String),
        serde_json::Value::Null => Ok(LiteralKind::Null),
        _ => {
            // Check for BigInt (may be represented as string or object)
            if estree_literal.value.is_string() {
                let s = estree_literal.value.as_str().unwrap_or("");
                if s.ends_with('n') {
                    return Ok(LiteralKind::BigInt);
                }
            }
            Err(ConversionError::LiteralConversionError {
                message: format!("Unknown literal type: {:?}", estree_literal.value),
                span: estree_literal
                    .range
                    .map(|r| (r[0] as u32, r[1] as u32))
                    .unwrap_or((0, 0)),
            })
        }
    }
}

/// Get the boolean value from an ESTree Literal.
pub fn get_boolean_value(estree_literal: &EstreeLiteral) -> ConversionResult<bool> {
    estree_literal.value.as_bool().ok_or_else(|| {
        ConversionError::InvalidFieldType {
            field: "value".to_string(),
            expected: "boolean".to_string(),
            got: format!("{:?}", estree_literal.value),
            span: get_literal_span(estree_literal),
        }
    })
}

/// Get the numeric value from an ESTree Literal.
pub fn get_numeric_value(estree_literal: &EstreeLiteral) -> ConversionResult<f64> {
    estree_literal.value.as_f64().ok_or_else(|| {
        ConversionError::InvalidFieldType {
            field: "value".to_string(),
            expected: "number".to_string(),
            got: format!("{:?}", estree_literal.value),
            span: get_literal_span(estree_literal),
        }
    })
}

/// Get the string value from an ESTree Literal.
pub fn get_string_value(estree_literal: &EstreeLiteral) -> ConversionResult<&str> {
    estree_literal.value.as_str().ok_or_else(|| {
        ConversionError::InvalidFieldType {
            field: "value".to_string(),
            expected: "string".to_string(),
            got: format!("{:?}", estree_literal.value),
            span: get_literal_span(estree_literal),
        }
    })
}

/// Get the span for an ESTree literal as (start, end) byte offsets.
pub fn get_literal_span(estree_literal: &EstreeLiteral) -> Span {
    estree_literal
        .range
        .map(|r| (r[0] as u32, r[1] as u32))
        .unwrap_or((0, 0))
}

