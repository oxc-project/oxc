//! Postprocessing pass for embedded-language IRs before they integrate into
//! the parent template literal.
//!
//! Three entry points share a common set of private helpers
//! ([`escape_template_characters`], [`escape_backticks_raw_str`], [`count_placeholders`]):
//! - [`postprocess`]: Doc-converted IRs (HTML/Angular/Markdown). Trims trailing
//!   hardlines, collapses double-hardlines, merges Text runs, escapes template
//!   characters per the requested mode, counts surviving placeholders.
//! - [`escape_template_characters_in_ir`]: graphql-in-js IRs from `oxc_formatter_graphql`.
//! - [`merge_texts_and_count_css_placeholders`]: css-in-js IRs from `oxc_formatter_css`.

use oxc_allocator::{Allocator, ArenaStringBuilder, ArenaVec};
use oxc_formatter_core::{FormatElement, IndentWidth, LineMode, TextWidth};
use oxc_formatter_css::{TEMPLATE_PLACEHOLDER_PREFIX, TEMPLATE_PLACEHOLDER_SUFFIX};

/// Escape template-literal characters (`` ` ``, `${`, `\`) in every `Text`
/// element of an embedded IR.
///
/// graphql-in-js is re-inserted into a JS template literal built from `.cooked`
/// values, so these characters must be re-escaped before integration.
///
/// Only top-level `Text` elements are visited; the IR produced by formatter
/// crates is a flat tag stream (no nested element containers).
pub fn escape_template_characters_in_ir<'a>(
    ir: &mut [FormatElement<'a>],
    allocator: &'a Allocator,
    indent_width: IndentWidth,
) {
    for element in ir.iter_mut() {
        if let FormatElement::Text { text, .. } = element {
            let escaped = escape_template_characters(text, allocator);
            // `escape_template_characters` returns the input slice when nothing needed escaping
            if !std::ptr::eq(escaped, *text) {
                let width = TextWidth::from_text(escaped, indent_width);
                *element = FormatElement::Text { text: escaped, width };
            }
        }
    }
}

/// Merge consecutive text-like elements (`Text` / `Token` / `Space`) of a
/// css-in-js IR and count `@prettier-placeholder-N-id` occurrences.
///
/// The parent (`oxc_formatter::embed/css.rs`) replaces placeholders per `Text`
/// element, so a placeholder split across elements (e.g. `Token("@")` +
/// `Text("prettier-placeholder-0-id")` from the at-rule printer) must be
/// fused into one `Text` to be detectable. `oxc_formatter_css` emits a mix of
/// `Text`/`Token`/`Space`, so all three join the run.
pub fn merge_texts_and_count_css_placeholders<'a>(
    ir: &mut ArenaVec<'a, FormatElement<'a>>,
    allocator: &'a Allocator,
    indent_width: IndentWidth,
) -> usize {
    fn text_like<'i>(element: &'i FormatElement<'_>) -> Option<&'i str> {
        match element {
            FormatElement::Text { text, .. } | FormatElement::Token { text } => Some(text),
            FormatElement::Space => Some(" "),
            _ => None,
        }
    }

    let (prefix, suffix) = (TEMPLATE_PLACEHOLDER_PREFIX, TEMPLATE_PLACEHOLDER_SUFFIX);
    let mut placeholder_count = 0;
    let mut write = 0;
    let mut read = 0;
    while read < ir.len() {
        if text_like(&ir[read]).is_some() {
            let run_start = read;
            read += 1;
            while read < ir.len() && text_like(&ir[read]).is_some() {
                read += 1;
            }

            if read - run_start == 1 {
                // Single element: keep it (and its width) as-is.
                // A lone `Token` (static strings only) or `Space` can never
                // contain a placeholder, so only `Text` is worth counting.
                if let FormatElement::Text { text, .. } = &ir[run_start] {
                    placeholder_count += count_placeholders(text, prefix, suffix);
                }
                if write != run_start {
                    ir[write] = ir[run_start].clone();
                }
            } else {
                let mut sb = ArenaStringBuilder::new_in(allocator);
                for element in &ir[run_start..read] {
                    sb.push_str(text_like(element).unwrap());
                }
                let text = sb.into_str();
                placeholder_count += count_placeholders(text, prefix, suffix);
                let width = TextWidth::from_text(text, indent_width);
                ir[write] = FormatElement::Text { text, width };
            }
            write += 1;
        } else {
            if write != read {
                ir[write] = ir[read].clone();
            }
            write += 1;
            read += 1;
        }
    }
    ir.truncate(write);
    placeholder_count
}

// ---

#[derive(Clone, Copy)]
pub enum TemplateEscape {
    /// Full escaping: `\` → `\\`, `` ` `` → `` \` ``, `${` → `\${`.
    Full,
    /// Raw backtick escaping: `(\\*)\`` → `$1$1\\\``.
    RawBacktick,
}

/// Post-process FormatElements in a single compaction pass:
/// - strip trailing hardline (useless for embedded parts)
/// - collapse double-hardlines `[Hard, ExpandParent, Hard, ExpandParent]` → `[Empty, ExpandParent]`
/// - merge consecutive Text nodes (SCSS emits split strings like `"@"` + `"prettier-placeholder-0-id"`)
/// - escape template characters (mode determined by [`TemplateEscape`])
/// - count placeholders matching `(prefix)(digits)(_digits)?(suffix)` pattern
///
/// Returns the placeholder count (0 when `placeholder` is `None`).
pub fn postprocess<'a>(
    ir: &mut ArenaVec<'a, FormatElement<'a>>,
    allocator: &'a Allocator,
    escape: TemplateEscape,
    placeholder: Option<(&str, &str)>,
) -> usize {
    // Strip trailing hardline
    if ir.len() >= 2
        && matches!(ir[ir.len() - 1], FormatElement::ExpandParent)
        && matches!(ir[ir.len() - 2], FormatElement::Line(LineMode::Hard))
    {
        let new_len = ir.len() - 2;
        ir.truncate(new_len);
    }

    let mut placeholder_count = 0;
    let mut write = 0;
    let mut read = 0;
    while read < ir.len() {
        // Collapse double-hardline → empty line
        if read + 3 < ir.len()
            && matches!(ir[read], FormatElement::Line(LineMode::Hard))
            && matches!(ir[read + 1], FormatElement::ExpandParent)
            && matches!(ir[read + 2], FormatElement::Line(LineMode::Hard))
            && matches!(ir[read + 3], FormatElement::ExpandParent)
        {
            ir[write] = FormatElement::Line(LineMode::Empty);
            ir[write + 1] = FormatElement::ExpandParent;
            write += 2;
            read += 4;
        } else if matches!(ir[read], FormatElement::Text { .. }) {
            // Merge consecutive Text nodes + escape + count placeholders
            let run_start = read;
            read += 1;
            while read < ir.len() && matches!(ir[read], FormatElement::Text { .. }) {
                read += 1;
            }

            let text = if read - run_start == 1 {
                let FormatElement::Text { text, .. } = &ir[run_start] else { unreachable!() };
                text
            } else {
                let mut sb = ArenaStringBuilder::new_in(allocator);
                for element in &ir[run_start..read] {
                    if let FormatElement::Text { text, .. } = element {
                        sb.push_str(text);
                    }
                }
                sb.into_str()
            };
            let text = match escape {
                TemplateEscape::Full => escape_template_characters(text, allocator),
                TemplateEscape::RawBacktick => escape_backticks_raw_str(text, allocator),
            };
            let width = TextWidth::from_text(text, IndentWidth::default());
            ir[write] = FormatElement::Text { text, width };
            write += 1;

            // Count placeholders for this text if needed
            if let Some((prefix, suffix)) = placeholder {
                placeholder_count += count_placeholders(text, prefix, suffix);
            }
        } else {
            if write != read {
                ir[write] = ir[read].clone();
            }
            write += 1;
            read += 1;
        }
    }
    ir.truncate(write);
    placeholder_count
}

/// Count placeholder occurrences matching `{prefix}{digits}(_{digits})?{suffix}` in text.
///
/// The optional `_{digits}` group allows matching both formats:
/// - CSS: `@prettier-placeholder-0-id` (no counter)
/// - HTML: `PRETTIER_HTML_PLACEHOLDER_0_0_IN_JS` (with counter)
fn count_placeholders(text: &str, prefix: &str, suffix: &str) -> usize {
    let mut count = 0;
    let mut remaining = text;
    while let Some(start) = remaining.find(prefix) {
        let after_prefix = &remaining[start + prefix.len()..];
        let digit_end =
            after_prefix.bytes().position(|b| !b.is_ascii_digit()).unwrap_or(after_prefix.len());
        if digit_end > 0 {
            let mut after_digits = &after_prefix[digit_end..];
            // Skip optional `_{digits}` (e.g., HTML counter)
            if let Some(after_underscore) = after_digits.strip_prefix('_') {
                let c = after_underscore
                    .bytes()
                    .position(|b| !b.is_ascii_digit())
                    .unwrap_or(after_underscore.len());
                if c > 0 {
                    after_digits = &after_underscore[c..];
                }
            }
            if let Some(rest) = after_digits.strip_prefix(suffix) {
                count += 1;
                remaining = rest;
                continue;
            }
        }
        remaining = &remaining[start + prefix.len()..];
    }
    count
}

/// Escape characters that would break template literal syntax.
///
/// Equivalent to Prettier's `uncookTemplateElementValue`:
/// `cookedValue.replaceAll(/([\\`]|\$\{)/gu, String.raw`\$1`);`
fn escape_template_characters<'a>(s: &'a str, allocator: &'a Allocator) -> &'a str {
    let bytes = s.as_bytes();
    let len = bytes.len();

    // Fast path: scan for the first character that needs escaping.
    // All characters of interest (`\`, `` ` ``, `$`, `{`) are single-byte ASCII,
    // so byte-indexed access is safe and avoids multi-byte decode overhead.
    let first_escape = (0..len).find(|&i| {
        let ch = bytes[i];
        ch == b'\\' || ch == b'`' || (ch == b'$' && i + 1 < len && bytes[i + 1] == b'{')
    });

    let Some(first) = first_escape else {
        return s;
    };

    // Slow path: build escaped string in the arena, reusing the clean prefix.
    let mut result = ArenaStringBuilder::with_capacity_in(len + 1, allocator);
    result.push_str(&s[..first]);

    // Iterate by chars (not bytes) to correctly handle multi-byte UTF-8.
    // All escape targets (`\`, `` ` ``, `${`) are ASCII, so this is straightforward.
    let mut chars = s[first..].chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' || ch == '`' {
            result.push('\\');
            result.push(ch);
        } else if ch == '$' && chars.peek() == Some(&'{') {
            result.push_str("\\${");
            chars.next(); // skip '{'
        } else {
            result.push(ch);
        }
    }

    result.into_str()
}

/// Escape backticks in raw mode for markdown-in-JS template literals.
///
/// Equivalent to Prettier's `escapeTemplateCharacters(doc, /* raw */ true)`:
/// <https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/print/template-literal.js#L277-L287>
/// `str.replaceAll(/(\\*)`/g, "$1$1\\`")`
///
/// For each backtick, doubles the preceding backslashes and adds `\` before the backtick:
/// - `` ` `` → `` \` ``
/// - `` \` `` → `` \\\` ``
/// - `` \\` `` → `` \\\\\` ``
fn escape_backticks_raw_str<'a>(s: &'a str, allocator: &'a Allocator) -> &'a str {
    if !s.contains('`') {
        return s;
    }
    let mut result = ArenaStringBuilder::with_capacity_in(s.len() + 1, allocator);
    let mut bs_count: usize = 0;
    for ch in s.chars() {
        if ch == '\\' {
            bs_count += 1;
            result.push('\\');
        } else if ch == '`' {
            // The backslash branch already emitted `bs_count` backslashes.
            // Emit another `bs_count` to double them, then add `\``.
            for _ in 0..bs_count {
                result.push('\\');
            }
            result.push('\\');
            result.push('`');
            bs_count = 0;
        } else {
            bs_count = 0;
            result.push(ch);
        }
    }
    result.into_str()
}
