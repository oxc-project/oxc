//! `StringValue` printing: block-string dedent (GraphQL spec `BlockStringValue`) and Prettier's re-encoding,
//! see `AGENTS.md` "Strings".

use std::borrow::Cow;

use cow_utils::CowUtils;

use oxc_allocator::StringBuilder;
use oxc_formatter_core::{
    arena_cow_str,
    builders::{exact_line_breaks, hard_line_break, text},
    write,
};
use oxc_graphql_parser::ast::StringValue;

use super::GraphqlFormatter;

pub(super) fn write_string_value<'a>(sv: &StringValue<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    if sv.block {
        write_block_string(sv.value, f);
    } else {
        write_regular_string(sv, f);
    }
}

/// Block string: cook per the GraphQL `BlockStringValue` algorithm, re-escape `"""`,
/// then print one line per hard line break (Prettier's shape).
fn write_block_string<'a>(body: &'a str, f: &mut GraphqlFormatter<'_, 'a>) {
    // `\"""` is the only escape in block strings.
    // Precheck its `\`: `cow_replace` builds a substring searcher per call
    let unescaped = if body.contains('\\') {
        body.cow_replace("\\\"\"\"", "\"\"\"")
    } else {
        Cow::Borrowed(body)
    };
    // Without one, no line holds a `"""` to re-escape: the lines print borrowed from the source.
    // Re-escaping touches no whitespace, so it is done when printing
    let escapes = matches!(unescaped, Cow::Owned(_));
    let mut lines = cook_block_string_lines(arena_cow_str(&unescaped, f));

    // Prettier: `lines.length === 1` → trim, all-blank → drop everything.
    // Trailing spaces/tabs are trimmed per line.
    for line in &mut lines {
        *line = line.trim_end_matches([' ', '\t']);
    }
    if let [line] = lines.as_mut_slice() {
        *line = line.trim();
    }
    if lines.iter().all(|line| line.is_empty()) {
        lines.clear();
    }

    write!(f, "\"\"\"");
    // Join with line breaks, preserving runs of blank lines exactly:
    // blank lines inside a block string are part of its VALUE,
    // so a run of `k` blank lines is emitted as `exact_line_breaks(k + 1)`.
    let mut pending_blanks = 0usize;
    for line in lines {
        if line.is_empty() {
            pending_blanks += 1;
            continue;
        }
        write_block_string_break(pending_blanks, f);
        pending_blanks = 0;
        let line =
            if escapes { arena_cow_str(&line.cow_replace("\"\"\"", "\\\"\"\""), f) } else { line };
        write!(f, text(line));
    }
    write_block_string_break(pending_blanks, f);
    write!(f, "\"\"\"");
}

fn write_block_string_break(blank_lines: usize, f: &mut GraphqlFormatter<'_, '_>) {
    if blank_lines == 0 {
        write!(f, hard_line_break());
    } else {
        write!(f, exact_line_breaks(blank_lines + 1));
    }
}

/// GraphQL spec `BlockStringValue`: split lines, strip the common indentation of
/// non-first lines, then drop leading/trailing blank lines.
fn cook_block_string_lines(value: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = split_lines(value);

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
    while lines.first().is_some_and(is_blank) {
        lines.remove(0);
    }
    while lines.last().is_some_and(is_blank) {
        lines.pop();
    }
    lines
}

/// `value.split('\n')`, by the SIMD `memchr` rather than core's
fn split_lines(value: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    for i in memchr::memchr_iter(b'\n', value.as_bytes()) {
        lines.push(&value[start..i]);
        start = i + 1;
    }
    lines.push(&value[start..]);
    lines
}

fn leading_whitespace_len(line: &str) -> usize {
    line.bytes().take_while(|&b| b == b' ' || b == b'\t').count()
}

/// Regular string: the cooked value re-encoded the Prettier way
/// (`"` and `\` escaped, newline as `\n`, everything else verbatim).
fn write_regular_string<'a>(sv: &StringValue<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    write!(f, "\"");
    // Without an escape the value is the source text, which holds none of these either
    let needs_escape = sv.raw.contains('\\')
        && sv.value.bytes().any(|b| matches!(b, b'"' | b'\\' | b'\n' | b'\r'));
    if !needs_escape {
        if !sv.value.is_empty() {
            write!(f, text(sv.value));
        }
        write!(f, "\"");
        return;
    }

    let mut out = StringBuilder::with_capacity_in(sv.value.len() + 2, f.allocator());
    for c in sv.value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    write!(f, [text(out.into_str()), "\""]);
}
