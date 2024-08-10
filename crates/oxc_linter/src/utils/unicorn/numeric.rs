use lazy_static::lazy_static;
use regex::Regex;

// Determine whether this node is a decimal integer literal.
// Copied from https://github.com/eslint/eslint/blob/cc4871369645c3409dc56ded7a555af8a9f63d51/lib/rules/utils/ast-utils.js#L1237
lazy_static! {
    static ref DECIMAL_INTEGER_PATTERN: Regex =
        Regex::new(r"^(?:0|0[0-7]*[89]\d*|[1-9](?:_?\d)*)$").unwrap();
}

pub fn is_decimal_integer(text: &str) -> bool {
    DECIMAL_INTEGER_PATTERN.is_match(text)
}
