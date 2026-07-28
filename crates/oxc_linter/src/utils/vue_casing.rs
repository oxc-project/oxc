//! Mirror of upstream `eslint-plugin-vue/lib/utils/casing.js`. Shared by the
//! vue lint rules that need to check whether an identifier (component name,
//! prop name, etc.) is in a particular casing.

use convert_case::{Boundary, Case, Converter};

/// Returns true if `s` contains any character that is not allowed in any of
/// the casings recognised by eslint-plugin-vue.
///
/// Mirrors upstream `hasSymbols`. The character class excludes ` `, `$`,
/// `-`, `_` deliberately — `-` and `_` are case-specific word separators
/// and `$` is allowed in JavaScript identifiers (e.g. `$actionEl`).
pub fn has_symbols(s: &str) -> bool {
    s.chars().any(|c| {
        matches!(
            c,
            '!' | '"'
                | '#'
                | '%'
                | '&'
                | '\''
                | '('
                | ')'
                | '*'
                | '+'
                | ','
                | '.'
                | '/'
                | ':'
                | ';'
                | '<'
                | '='
                | '>'
                | '?'
                | '@'
                | '['
                | '\\'
                | ']'
                | '^'
                | '`'
                | '{'
                | '|'
                | '}'
        )
    })
}

/// Returns true if `s` contains any ASCII uppercase letter.
pub fn has_upper(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_uppercase())
}

pub fn is_pascal_case(s: &str) -> bool {
    !has_symbols(s)
        && !s.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && !s.chars().any(|c| matches!(c, '-' | '_') || c.is_whitespace())
}

pub fn is_kebab_case(s: &str) -> bool {
    if has_upper(s) || has_symbols(s) || s.starts_with('-') {
        return false;
    }
    if s.contains('_') || s.contains("--") || s.chars().any(char::is_whitespace) {
        return false;
    }
    true
}

pub fn is_camel_case(s: &str) -> bool {
    !has_symbols(s)
        && !s.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        && !s.chars().any(|c| matches!(c, '-' | '_') || c.is_whitespace())
}

pub fn is_snake_case(s: &str) -> bool {
    if has_upper(s) || has_symbols(s) {
        return false;
    }
    if s.contains('-') || s.contains("__") || s.chars().any(char::is_whitespace) {
        return false;
    }
    true
}

pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn is_regex_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn regex_word_before_upper(graphemes: &[&str]) -> bool {
    let Some(prev) = graphemes.first().and_then(|s| s.chars().next()) else {
        return false;
    };
    let Some(current) = graphemes.get(1).and_then(|s| s.chars().next()) else {
        return false;
    };
    is_regex_word(prev) && current.is_ascii_uppercase()
}

/// Mirror of upstream `camelCase`:
/// - if input is already PascalCase: lowercase the first char
/// - else: replace `[-_](\w)` with `\w` uppercased
pub fn camel_case(s: &str) -> String {
    if is_pascal_case(s) {
        let mut chars = s.chars();
        return match chars.next() {
            Some(c) => c.to_lowercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        };
    }

    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if matches!(c, '-' | '_')
            && let Some(next) = chars.peek()
            && is_regex_word(*next)
        {
            if let Some(next) = chars.next() {
                out.extend(next.to_uppercase());
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn pascal_case(s: &str) -> String {
    capitalize(&camel_case(s))
}

pub fn kebab_case(s: &str) -> String {
    let word_before_upper =
        Boundary::Custom { condition: regex_word_before_upper, start: 1, len: 0 };
    Converter::new()
        .set_boundaries(&[Boundary::Underscore, word_before_upper])
        .to_case(Case::Kebab)
        .convert(s)
}
