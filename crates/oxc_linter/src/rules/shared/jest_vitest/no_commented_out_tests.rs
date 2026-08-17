use memchr::memchr3_iter;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::context::LintContext;

fn no_commented_out_tests_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Some tests appear to be inside comments.")
        .with_help("Remove or uncomment this test.")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

This rule raises a warning about commented-out tests. It's similar to the
`no-disabled-tests` rule.

### Why is this bad?

You may forget to uncomment some tests. This rule raises a warning about commented-out tests.

It is generally better to skip a test if it's flaky, or remove it if it's no longer needed.

### Examples

Examples of **incorrect** code for this rule:
```javascript
// describe('foo', () => {});
// it('foo', () => {});
// test('foo', () => {});

// describe.skip('foo', () => {});
// it.skip('foo', () => {});
// test.skip('foo', () => {});
```
";

/// Matches: `/^\s*[xf]?(test|it|describe)(\.\w+|\[['"]\w+['"]\])?\s*\(/mu`
///
/// Scans the full comment (not line-by-line) so `\s` can span newlines, matching the
/// original regex without paying for a full regex engine on every comment.
fn is_commented_out_test(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut line_start = 0usize;
    while line_start <= bytes.len() {
        // Skip leading whitespace on this line (and blank lines via loop).
        let mut i = line_start;
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\r') {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if bytes[i] == b'\n' {
            line_start = i + 1;
            continue;
        }

        // Optional x/f prefix only when it forms xit/fit/xdescribe/fdescribe/xtest/ftest
        let start = i;
        if matches!(bytes[i], b'x' | b'f') {
            let rest = &text[i + 1..];
            if rest.starts_with("test") || rest.starts_with("it") || rest.starts_with("describe") {
                i += 1;
            }
        }

        let rest = &text[i..];
        let keyword_len = if rest.starts_with("describe") {
            8
        } else if rest.starts_with("test") {
            4
        } else if rest.starts_with("it") {
            2
        } else {
            // Advance to next line
            line_start = next_line_start(bytes, start);
            continue;
        };
        i += keyword_len;

        // Must not continue as a longer identifier (`item`, `testSomething`)
        if i < bytes.len()
            && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
        {
            line_start = next_line_start(bytes, start);
            continue;
        }

        // Optional `.method` or `['method']` / `["method"]` (no newlines inside in practice,
        // but allow whitespace via the final `\s*` before `(` only)
        if i < bytes.len() && bytes[i] == b'.' {
            i += 1;
            if i >= bytes.len() || !is_regex_word_byte(bytes[i]) {
                line_start = next_line_start(bytes, start);
                continue;
            }
            i += 1;
            while i < bytes.len() && is_regex_word_byte(bytes[i]) {
                i += 1;
            }
        } else if i < bytes.len() && bytes[i] == b'[' {
            i += 1;
            let quote = bytes.get(i).copied();
            if !matches!(quote, Some(b'\'' | b'"')) {
                line_start = next_line_start(bytes, start);
                continue;
            }
            let q = quote.unwrap();
            i += 1;
            let method_start = i;
            while i < bytes.len() && is_regex_word_byte(bytes[i]) {
                i += 1;
            }
            if i == method_start || bytes.get(i) != Some(&q) {
                line_start = next_line_start(bytes, start);
                continue;
            }
            i += 1; // closing quote
            if bytes.get(i) != Some(&b']') {
                line_start = next_line_start(bytes, start);
                continue;
            }
            i += 1;
        }

        // Whitespace (including newlines) then `(`
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'(' {
            return true;
        }

        line_start = next_line_start(bytes, start);
    }
    false
}

#[inline]
fn is_regex_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[inline]
fn next_line_start(bytes: &[u8], from: usize) -> usize {
    match bytes[from..].iter().position(|&b| b == b'\n') {
        Some(rel) => from + rel + 1,
        None => bytes.len() + 1, // terminate outer loop
    }
}

#[inline]
fn might_contain_test_api(text: &str) -> bool {
    memchr3_iter(b't', b'i', b'd', text.as_bytes()).any(|index| {
        let rest = &text[index..];
        rest.starts_with("test") || rest.starts_with("it") || rest.starts_with("describe")
    })
}

pub fn run_once(ctx: &LintContext) {
    for comment in ctx.comments() {
        let text = ctx.source_range(comment.content_span());
        if might_contain_test_api(text) && is_commented_out_test(text) {
            ctx.diagnostic(no_commented_out_tests_diagnostic(comment.content_span()));
        }
    }
}
