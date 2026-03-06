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

    // Clone once upfront — subsequent clones of base_options are cheap since
    // the Vec fields (sort_imports, sort_tailwindcss) are already owned.
    let base_options = FormatOptions { line_width, jsdoc: None, ..format_options.clone() };

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

    let input = allocator.alloc_concat_strs_array(["type __t = ", type_str, ";"]);

    let ret =
        Parser::new(allocator, input, SourceType::tsx()).with_options(get_parse_options()).parse();
    if ret.panicked || !ret.errors.is_empty() {
        return None;
    }

    let formatted = Formatter::new(allocator, type_options.clone()).build(&ret.program);
    let formatted = formatted.trim_end();

    // Strip the `type __t = ` prefix (11 chars) using slice, matching upstream's
    // `pretty.slice(TYPE_START.length)` approach. This handles both same-line and
    // wrapped output (e.g. `type __t =\n  | ...`).
    let result = formatted.get("type __t = ".len()..)?;

    // Upstream cleanup: strip leading whitespace, trailing `;` and newlines,
    // leading `|`, then trim.
    let result = result.trim_start();
    let result = result.trim_end_matches([';', '\n']);
    let result = result.strip_prefix('|').unwrap_or(result);
    let result = result.trim();

    if result.is_empty() {
        return None;
    }

    Some(String::from(result))
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
    for &b in type_str.as_bytes() {
        match b {
            b'|' | b'&' | b'{' | b'}' | b'(' | b')' | b'\n' => return true,
            _ => {}
        }
    }
    type_str.contains("=>")
}
