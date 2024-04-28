use oxc_ast::ast::PropertyKey;
use oxc_syntax::identifier::is_identifier_name;

pub(super) fn is_property_key_has_quote(key: &PropertyKey<'_>) -> bool {
    matches!(key, PropertyKey::StringLiteral(literal) if is_string_prop_safe_to_unquote(literal.value.as_str()))
}

pub(super) fn is_string_prop_safe_to_unquote(value: &str) -> bool {
    !is_identifier_name(value) && !is_simple_number(value)
}

// Matches “simple” numbers like `123` and `2.5` but not `1_000`, `1e+100` or `0b10`.
pub(super) fn is_simple_number(str: &str) -> bool {
    let mut bytes = str.as_bytes().iter();
    let mut has_dot = false;
    bytes.next().is_some_and(u8::is_ascii_digit)
        && bytes.all(|c| {
            if c == &b'.' {
                if has_dot {
                    return false;
                }
                has_dot = true;
                return true;
            }
            c.is_ascii_digit()
        })
}
