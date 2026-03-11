use std::fmt::Write as _;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::ExternalCallbacks;
use crate::options::TrailingCommas;
use crate::{FormatOptions, Formatter, LineWidth, get_parse_options};

use super::serialize::truncate_trim_end;

/// Map fenced code block language tags to external formatter language identifiers.
/// Returns `None` if the language should be handled by the native JS/TS formatter.
pub(super) fn fenced_lang_to_external_language(lang: &str) -> Option<&'static str> {
    match lang {
        "css" | "scss" | "less" => Some("tagged-css"),
        "html" => Some("tagged-html"),
        "graphql" | "gql" => Some("tagged-graphql"),
        "markdown" | "md" | "mdx" => Some("tagged-markdown"),
        "yaml" | "yml" => Some("tagged-yaml"),
        _ => None,
    }
}

/// Returns `true` if the fenced code block language is JS/TS/JSX/TSX.
pub(super) fn is_js_ts_lang(lang: &str) -> bool {
    matches!(lang, "js" | "javascript" | "jsx" | "ts" | "typescript" | "tsx")
}

/// Format code using the external formatter (Prettier) for non-JS/TS languages.
/// Returns `Some(formatted)` on success, `None` if no callback is available or formatting fails.
pub(super) fn format_external_language(
    code: &str,
    language: &str,
    _wrap_width: usize,
    external_callbacks: &ExternalCallbacks,
) -> Option<String> {
    let result = external_callbacks.format_embedded(language, code)?;
    match result {
        Ok(mut formatted) => {
            truncate_trim_end(&mut formatted);
            Some(formatted)
        }
        Err(_) => None,
    }
}

/// Count unescaped backticks on a line and update template literal depth.
/// Returns the new depth after processing the line.
pub(super) fn update_template_depth(line: &str, mut depth: u32) -> u32 {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2; // skip escaped character
            continue;
        }
        if bytes[i] == b'`' {
            if depth == 0 {
                depth += 1;
            } else {
                depth -= 1;
            }
        }
        i += 1;
    }
    depth
}

/// Try to format JS/TS/JSX code using the formatter.
/// Returns `Some(formatted)` on success, `None` if parsing fails.
/// The `print_width` is the available width for the formatted code.
/// Uses the parent `format_options` to ensure consistent formatting behavior.
pub(super) fn format_embedded_js(
    code: &str,
    print_width: usize,
    format_options: &FormatOptions,
    allocator: &Allocator,
) -> Option<String> {
    let width = u16::try_from(print_width).unwrap_or(80).clamp(1, 320);
    let line_width = LineWidth::try_from(width).unwrap();

    // Null out sort_imports/sort_tailwindcss — they're top-level concerns irrelevant
    // to embedded code, and their Vec fields make cloning expensive.
    let base_options = FormatOptions {
        line_width,
        jsdoc: None,
        sort_imports: None,
        sort_tailwindcss: None,
        ..format_options.clone()
    };

    // Try to parse and format with the given source type
    let try_format = |code: &str, source_type: SourceType| -> Option<String> {
        let ret =
            Parser::new(allocator, code, source_type).with_options(get_parse_options()).parse();
        if ret.panicked || !ret.errors.is_empty() {
            return None;
        }
        let mut formatted = Formatter::new(allocator, base_options.clone()).build(&ret.program);
        truncate_trim_end(&mut formatted);
        Some(formatted)
    };

    // If code starts with `{`, try expression wrapping FIRST to handle object
    // literals like `{ key: value }` that would otherwise parse as block statements
    // with labels. The upstream plugin uses `parser: "json"` for this case.
    let trimmed = code.trim();
    if trimmed.starts_with('{') {
        let wrapped = allocator.alloc_concat_strs_array(["(", trimmed, ")"]);
        // Use TrailingCommas::None for object literals since JSON-like code
        // shouldn't have trailing commas
        let obj_options =
            FormatOptions { trailing_commas: TrailingCommas::None, ..base_options.clone() };

        let try_format_obj = |code: &str, source_type: SourceType| -> Option<String> {
            let ret =
                Parser::new(allocator, code, source_type).with_options(get_parse_options()).parse();
            if ret.panicked || !ret.errors.is_empty() {
                return None;
            }
            let formatted = Formatter::new(allocator, obj_options.clone()).build(&ret.program);
            let formatted = formatted.trim_end();
            // Remove the wrapping parens and trailing semicolon
            if let Some(inner) = formatted.strip_prefix('(')
                && let Some(inner) = inner.strip_suffix(");")
            {
                return Some(String::from(inner));
            }
            Some(String::from(formatted))
        };

        if let Some(result) = try_format_obj(wrapped, SourceType::jsx()) {
            return Some(result);
        }
        if let Some(result) = try_format_obj(wrapped, SourceType::tsx()) {
            return Some(result);
        }
    }

    // Try JSX first (most @example code in React projects uses JSX),
    // then TSX (for TypeScript code with JSX).
    if let Some(result) = try_format(code, SourceType::jsx()) {
        return Some(result);
    }
    if let Some(result) = try_format(code, SourceType::tsx()) {
        return Some(result);
    }

    None
}

/// Format a JSDoc type expression using the formatter (simulating upstream's `formatType()`).
///
/// Wraps the type as `type __t = {type_str};`, parses as TSX, formats, then extracts
/// the formatted type. Handles `...Type` rest params by formatting the inner type
/// separately. Returns `None` on parse/format failure.
/// `type_options` must already have `jsdoc: None` to prevent recursive formatting.
///
/// String literals (`"..."` and `'...'`) are protected from the formatter by replacing
/// them with placeholder identifiers before formatting, then restoring afterwards.
/// This matches the upstream prettier-plugin-jsdoc's `withoutStrings()` approach,
/// preventing the formatter from changing quote style inside type expressions.
pub(super) fn format_type_via_formatter(
    type_str: &str,
    type_options: &FormatOptions,
    allocator: &Allocator,
) -> Option<String> {
    if type_str.is_empty() {
        return None;
    }

    // Handle rest/spread prefix: convert `...Type` to `(Type)[]`, format, then strip
    // the trailing `[]` and prepend `...`. Same approach as upstream's `formatType()`.
    if let Some(rest) = type_str.strip_prefix("...") {
        let rest = rest.trim_start();
        if rest.is_empty() {
            return None;
        }
        let wrapped = allocator.alloc_concat_strs_array(["(", rest, ")[]"]);
        let formatted = format_type_via_formatter(wrapped, type_options, allocator)?;
        let inner = formatted.strip_suffix("[]")?;
        let mut result = String::with_capacity(inner.len() + 3);
        result.push_str("...");
        result.push_str(inner);
        return Some(result);
    }

    // Fast path: skip the expensive parse+format cycle for types that the TS
    // formatter won't change. Types without union/intersection operators, object
    // literals, function arrows, or parenthesized expressions are already in
    // their final form after normalize_type().
    if !needs_formatter_pass(type_str) {
        return None;
    }

    // Protect string literals from the formatter by replacing them with placeholder
    // identifiers. This prevents the TS formatter from changing quote style.
    let (protected, string_literals) = protect_string_literals(type_str);
    let type_to_format = if string_literals.is_empty() { type_str } else { &protected };

    let input = allocator.alloc_concat_strs_array(["type __t = ", type_to_format, ";"]);

    let ret =
        Parser::new(allocator, input, SourceType::tsx()).with_options(get_parse_options()).parse();
    if ret.panicked || !ret.errors.is_empty() {
        // If parsing fails with placeholders, try without protection as fallback
        if !string_literals.is_empty() {
            let input = allocator.alloc_concat_strs_array(["type __t = ", type_str, ";"]);
            let ret = Parser::new(allocator, input, SourceType::tsx())
                .with_options(get_parse_options())
                .parse();
            if ret.panicked || !ret.errors.is_empty() {
                return None;
            }
            let formatted = Formatter::new(allocator, type_options.clone()).build(&ret.program);
            let formatted = formatted.trim_end();
            let result = formatted.get("type __t = ".len()..)?;
            let result = result.trim_start();
            let result = result.trim_end_matches([';', '\n']);
            let result = result.strip_prefix('|').unwrap_or(result);
            let result = result.trim();
            let result = if result.contains('\n') {
                collapse_multiline_type(result)
            } else {
                String::from(result)
            };
            if result.is_empty() || result == type_str {
                return None;
            }
            return Some(result);
        }
        return None;
    }

    let formatted = Formatter::new(allocator, type_options.clone()).build(&ret.program);
    let formatted = formatted.trim_end();

    // Strip the `type __t = ` prefix (11 chars) using slice, matching upstream's
    // `pretty.slice(TYPE_START.length)` approach. This handles both same-line and
    // wrapped output (e.g. `type __t =\n  | ...`).
    let result = formatted.get("type __t = ".len()..)?;

    // Upstream cleanup: strip leading whitespace, trailing `;` and newlines,
    // leading `|`, then trim. Matches upstream's regex:
    //   .replace(/^\s*/g, "")     — strip leading whitespace
    //   .replace(/[;\n]*$/g, "")  — strip trailing `;` and newlines
    //   .replace(/^\|/g, "")      — strip leading pipe
    //   .trim()
    // Interior newlines are preserved — the TS formatter's multi-line output
    // for complex types (object literals, function types) is kept as-is.
    let result = result.trim_start();
    let result = result.trim_end_matches([';', '\n']);
    let result = result.strip_prefix('|').unwrap_or(result);
    let mut result = String::from(result.trim());

    // Restore original string literals from placeholders
    if !string_literals.is_empty() {
        result = restore_string_literals(&result, &string_literals);
    }

    if result.is_empty() || result == type_str {
        return None;
    }

    Some(result)
}

/// Replace string literals (`"..."` and `'...'`) with placeholder identifiers
/// (`__str0__`, `__str1__`, etc.) so the TS formatter doesn't modify their quotes.
/// Returns the modified string and the list of original string literals.
fn protect_string_literals(type_str: &str) -> (String, Vec<String>) {
    if !type_str.contains('"') && !type_str.contains('\'') {
        return (type_str.to_string(), Vec::new());
    }

    let mut literals: Vec<String> = Vec::new();
    let mut result = String::with_capacity(type_str.len());
    let bytes = type_str.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let ch = bytes[i];
        if ch == b'"' || ch == b'\'' {
            let quote = ch;
            let start = i;
            i += 1;
            // Scan to closing quote, handling backslash escapes
            while i < len {
                if bytes[i] == b'\\' {
                    i += 1; // skip backslash
                    if i < len {
                        i += 1; // skip escaped char
                    }
                    continue;
                }
                if bytes[i] == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            let matched = &type_str[start..i.min(len)];
            literals.push(matched.to_string());
            write!(result, "__str{}__", literals.len() - 1).unwrap();
        } else {
            let c = type_str[i..].chars().next().unwrap();
            result.push(c);
            i += c.len_utf8();
        }
    }

    (result, literals)
}

/// Restore original string literals from `__strN__` placeholders.
fn restore_string_literals(formatted: &str, literals: &[String]) -> String {
    if literals.is_empty() {
        return formatted.to_string();
    }

    let mut result = String::with_capacity(formatted.len());
    let bytes = formatted.as_bytes();
    let len = bytes.len();
    let prefix = b"__str";
    let prefix_len = prefix.len();
    let mut i = 0;

    while i < len {
        // Check for `__str` prefix
        if i + prefix_len < len && &bytes[i..i + prefix_len] == prefix {
            let digit_start = i + prefix_len;
            let mut digit_end = digit_start;
            while digit_end < len && bytes[digit_end].is_ascii_digit() {
                digit_end += 1;
            }
            // Must have digits followed by `__`
            if digit_end > digit_start
                && digit_end + 1 < len
                && bytes[digit_end] == b'_'
                && bytes[digit_end + 1] == b'_'
                && let Ok(idx) = formatted[digit_start..digit_end].parse::<usize>()
                && idx < literals.len()
            {
                result.push_str(&literals[idx]);
                i = digit_end + 2; // skip past trailing `__`
                continue;
            }
        }
        let ch = formatted[i..].chars().next().unwrap();
        result.push(ch);
        i += ch.len_utf8();
    }

    result
}

/// Collapse a multi-line TS-formatted type back to a single-line JSDoc type.
///
/// The TS formatter may wrap complex types (e.g., object types inside generics)
/// across multiple lines. JSDoc type expressions are single-line, so we collapse
/// newlines + indentation into single spaces while preserving the content
/// (including semicolons which are valid JSDoc object type member separators).
fn collapse_multiline_type(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for line in s.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !result.is_empty() {
            // Add space between collapsed lines, but avoid double spaces
            // and avoid space before closing brackets
            let last_char = result.as_bytes().last().copied().unwrap_or(b' ');
            let first_char = trimmed.as_bytes().first().copied().unwrap_or(b' ');
            if last_char != b' '
                && first_char != b' '
                && first_char != b'>'
                && first_char != b'}'
                && first_char != b']'
                && first_char != b')'
            {
                result.push(' ');
            }
        }
        result.push_str(trimmed);
    }
    result
}

/// Check if a type expression needs to go through the TS formatter.
///
/// Returns `false` for types that are already in their final form after
/// `normalize_type()` — simple identifiers, dotted names, array shorthand (`T[]`),
/// and generic types without complex structure.
///
/// The TS formatter can only change types that contain:
/// - `|` or `&`: union/intersection types may need wrapping
/// - `{` or `}`: object literal types need spacing
/// - `(` or `)`: parenthesized/function types need formatting
/// - `=>`: function type arrows
/// - Newlines: multi-line types need reformatting
pub(super) fn needs_formatter_pass(type_str: &str) -> bool {
    let bytes = type_str.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'|' | b'&' | b'{' | b'}' | b'(' | b')' | b'\n' => return true,
            b'=' if bytes.get(i + 1) == Some(&b'>') => return true,
            _ => {}
        }
        i += 1;
    }
    false
}
