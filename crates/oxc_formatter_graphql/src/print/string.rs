//! `StringValue` cooking and re-encoding.
//!
//! Prettier prints `StringValue` from graphql-js's *cooked* `node.value`
//! (escapes decoded, block strings dedented), then re-encodes it.
//! `oxc-graphql-parser`'s `StringValue.value` is cooked too, but NOT to spec:
//! no block-string dedent / blank-line trimming, no surrogate pairing, and
//! invalid escapes are silently dropped.
//! So this module cooks from `raw` itself (GraphQL spec string/block-string value algorithms)
//! and then applies Prettier's re-encoding (`"`/`\` escaped, newline as `\n`, `"""` as `\"""`).

use cow_utils::CowUtils;

use oxc_formatter_core::{
    Buffer,
    builders::{hard_line_break, text},
    write,
};
use oxc_graphql_parser::ast::StringValue;

use super::GraphqlFormatter;

pub(super) fn write_string_value<'a>(sv: &StringValue<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    let raw = sv.raw;
    if let Some(body) = raw.strip_prefix("\"\"\"").and_then(|r| r.strip_suffix("\"\"\"")) {
        write_block_string(body, f);
    } else if let Some(body) = raw.strip_prefix('"').and_then(|r| r.strip_suffix('"')) {
        write_regular_string(body, f);
    } else {
        // Defensive: malformed string (parse errors bail out before reaching here).
        write!(f, text(raw));
    }
}

/// Block string: cook per the GraphQL `BlockStringValue` algorithm, re-escape `"""`,
/// then print one line per hard line break (Prettier's shape).
fn write_block_string(body: &str, f: &mut GraphqlFormatter<'_, '_>) {
    // The only escape sequence in block strings.
    let unescaped = body.cow_replace("\\\"\"\"", "\"\"\"");
    let cooked_lines = cook_block_string_lines(&unescaped);

    // Prettier: re-escape, then `lines.length === 1` → trim,
    // all-blank → drop everything.
    // Trailing spaces/tabs are trimmed per line: Prettier's doc printer trims
    // them at every hardline. The core printer does the same, but a line
    // followed by a blank-line run gets its break from raw `\n` text (see
    // `write_block_string_break`), which bypasses that trim — so trim here.
    let mut lines: Vec<String> = cooked_lines
        .iter()
        .map(|l| l.trim_end_matches([' ', '\t']).cow_replace("\"\"\"", "\\\"\"\"").into_owned())
        .collect();
    if lines.len() == 1 {
        lines[0] = lines[0].trim().to_string();
    }
    if lines.iter().all(std::string::String::is_empty) {
        lines.clear();
    }

    write!(f, "\"\"\"");
    // Join with line breaks, preserving runs of blank lines exactly.
    // Consecutive `hard_line_break()`s collapse in the printer (and `empty_line()` caps
    // at one blank), but blank lines inside a block string are part of its VALUE.
    // So a run of `k` blank lines is emitted as `k + 1` raw newlines in a `text()`,
    // followed by a `hard_line_break()` that prints nothing (the line is already empty)
    // but re-arms the pending indent for the next line.
    let mut pending_blanks = 0usize;
    for line in &lines {
        if line.is_empty() {
            pending_blanks += 1;
            continue;
        }
        write_block_string_break(pending_blanks, f);
        pending_blanks = 0;
        write!(f, text(f.allocator().alloc_str(line)));
    }
    write_block_string_break(pending_blanks, f);
    write!(f, "\"\"\"");
}

fn write_block_string_break(blank_lines: usize, f: &mut GraphqlFormatter<'_, '_>) {
    if blank_lines == 0 {
        write!(f, hard_line_break());
    } else {
        write!(f, text(f.allocator().alloc_str(&"\n".repeat(blank_lines + 1))));
        write!(f, hard_line_break());
    }
}

/// GraphQL spec `BlockStringValue`: split lines, strip the common indentation of
/// non-first lines, then drop leading/trailing blank lines.
fn cook_block_string_lines(value: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = split_graphql_lines(value);

    // Common indent over non-blank lines after the first.
    let mut common_indent: Option<usize> = None;
    for line in lines.iter().skip(1) {
        let indent = leading_whitespace_len(line);
        if indent < line.len() && common_indent.is_none_or(|ci| indent < ci) {
            common_indent = Some(indent);
        }
    }
    if let Some(ci) = common_indent {
        for line in lines.iter_mut().skip(1) {
            let cut = ci.min(leading_whitespace_len(line));
            *line = &line[cut..];
        }
    }

    // Strip leading/trailing blank lines.
    let is_blank = |line: &&str| line.bytes().all(|b| b == b' ' || b == b'\t');
    while lines.first().is_some_and(&is_blank) {
        lines.remove(0);
    }
    while lines.last().is_some_and(&is_blank) {
        lines.pop();
    }
    lines
}

/// Splits on GraphQL line terminators: `\r\n`, lone `\r`, `\n`.
fn split_graphql_lines(value: &str) -> Vec<&str> {
    let bytes = value.as_bytes();
    let mut lines = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                lines.push(&value[start..i]);
                i += 1;
                start = i;
            }
            b'\r' => {
                lines.push(&value[start..i]);
                i += if bytes.get(i + 1) == Some(&b'\n') { 2 } else { 1 };
                start = i;
            }
            _ => i += 1,
        }
    }
    lines.push(&value[start..]);
    lines
}

fn leading_whitespace_len(line: &str) -> usize {
    line.bytes().take_while(|&b| b == b' ' || b == b'\t').count()
}

/// Regular string: decode escapes (GraphQL `StringValue` semantics), then re-encode
/// the Prettier way (`"` and `\` escaped, newline as `\n`, everything else verbatim).
fn write_regular_string<'a>(body: &'a str, f: &mut GraphqlFormatter<'_, 'a>) {
    write!(f, "\"");
    if !body.contains('\\') {
        // No escapes: the body cannot contain `"` or raw line terminators either,
        // so it round-trips verbatim.
        if !body.is_empty() {
            write!(f, text(body));
        }
        write!(f, "\"");
        return;
    }

    let cooked = cook_string(body);
    let mut out = String::with_capacity(cooked.len() + 2);
    for c in cooked.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            // DIVERGENCE: Prettier emits a cooked `\r` as a raw CR byte;
            // the core `text()` builder forbids raw `\r` (line-width accounting),
            // so keep it as an escape. The string VALUE is identical.
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    if !out.is_empty() {
        write!(f, text(f.allocator().alloc_str(&out)));
    }
    write!(f, "\"");
}

/// Decodes GraphQL escape sequences. Invalid sequences are kept verbatim
/// (they cannot appear after an error-free parse, but stay lossless just in case).
/// Surrogate pairs (`😀`) are combined; lone surrogates are kept as
/// their original escape text since Rust strings cannot represent them.
fn cook_string(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut chars = body.char_indices().peekable();
    while let Some((_, c)) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        let Some(&(_, esc)) = chars.peek() else {
            out.push('\\');
            break;
        };
        match esc {
            '"' | '\\' | '/' => {
                chars.next();
                out.push(esc);
            }
            'b' => {
                chars.next();
                out.push('\u{0008}');
            }
            'f' => {
                chars.next();
                out.push('\u{000C}');
            }
            'n' => {
                chars.next();
                out.push('\n');
            }
            'r' => {
                chars.next();
                out.push('\r');
            }
            't' => {
                chars.next();
                out.push('\t');
            }
            'u' => {
                chars.next();
                cook_unicode_escape(body, &mut chars, &mut out);
            }
            _ => {
                // Invalid escape: keep verbatim.
                out.push('\\');
            }
        }
    }
    out
}

/// Decodes `\uXXXX` (with surrogate pairing) and `\u{...}` after the `u` has been consumed.
fn cook_unicode_escape(
    body: &str,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    out: &mut String,
) {
    // Variable-width form: `\u{1F600}`.
    if chars.peek().is_some_and(|&(_, c)| c == '{') {
        chars.next();
        let mut value: u32 = 0;
        let mut valid = false;
        while let Some(&(_, c)) = chars.peek() {
            if c == '}' {
                chars.next();
                break;
            }
            let Some(digit) = c.to_digit(16) else {
                valid = false;
                break;
            };
            value = value.saturating_mul(16).saturating_add(digit);
            valid = true;
            chars.next();
        }
        match char::from_u32(value).filter(|_| valid) {
            Some(c) => out.push(c),
            None => out.push_str("\\u"),
        }
        return;
    }

    // Fixed-width form: `\uXXXX`.
    let Some(hi) = take_hex4(chars) else {
        out.push_str("\\u");
        return;
    };
    if (0xD800..=0xDBFF).contains(&hi) {
        // High surrogate: try to pair with a following `\uXXXX` low surrogate.
        let mut lookahead = chars.clone();
        if lookahead.peek().is_some_and(|&(_, c)| c == '\\') {
            lookahead.next();
            if lookahead.peek().is_some_and(|&(_, c)| c == 'u') {
                lookahead.next();
                if let Some(lo) = take_hex4(&mut lookahead)
                    && (0xDC00..=0xDFFF).contains(&lo)
                {
                    *chars = lookahead;
                    let combined = 0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00);
                    if let Some(c) = char::from_u32(combined) {
                        out.push(c);
                        return;
                    }
                }
            }
        }
        // Lone surrogate: not representable in a Rust string, keep the escape verbatim.
        push_verbatim_u_escape(body, hi, out);
        return;
    }
    match char::from_u32(hi) {
        Some(c) => out.push(c),
        None => push_verbatim_u_escape(body, hi, out),
    }
}

fn take_hex4(chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>) -> Option<u32> {
    let mut value: u32 = 0;
    let mut consumed = chars.clone();
    for _ in 0..4 {
        let (_, c) = consumed.next()?;
        value = value * 16 + c.to_digit(16)?;
    }
    *chars = consumed;
    Some(value)
}

fn push_verbatim_u_escape(_body: &str, value: u32, out: &mut String) {
    use std::fmt::Write as _;
    out.push_str("\\u");
    let _ = std::write!(out, "{value:04X}");
}
