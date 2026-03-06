use oxc_allocator::{Allocator, StringBuilder};
use oxc_ast::ast::*;

use crate::{
    IndentWidth,
    ast_nodes::AstNode,
    external_formatter::EmbeddedDocResult,
    format_args,
    formatter::{FormatElement, Formatter, format_element::TextWidth, prelude::*},
    write,
};

// This prefix and suffix are used by Prettier's css formatting,
// so we need to use the same pattern.
const PLACEHOLDER_PREFIX: &str = "@prettier-placeholder-";
const PLACEHOLDER_SUFFIX: &str = "-id";

/// Format a CSS-in-JS template literal via the Doc→IR path with placeholder replacement.
///
/// Joins quasis with special `@prettier-placeholder-N-id` markers, formats as SCSS,
/// then replaces placeholder occurrences in the resulting IR with `${expr}` Docs.
pub(super) fn format_css_doc<'a>(
    quasi: &AstNode<'a, TemplateLiteral<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> bool {
    let quasis = &quasi.quasis;
    let expressions: Vec<_> = quasi.expressions().iter().collect();

    // Phase 0: No expressions
    // format the single quasi text directly
    if expressions.is_empty() {
        // Use `.raw` (not `.cooked`), CSS/SCSS needs the original escape sequences
        let raw = quasis[0].value.raw.as_str();

        if raw.trim().is_empty() {
            write!(f, ["``"]);
            return true;
        }

        let allocator = f.allocator();
        let group_id_builder = f.group_id_builder();
        let Some(Ok(EmbeddedDocResult::DocWithPlaceholders(ir, _))) = f
            .context()
            .external_callbacks()
            .format_embedded_doc(allocator, group_id_builder, "tagged-css", &[raw])
        else {
            return false;
        };

        write!(f, ["`", block_indent(&format_once(|f| f.write_elements(ir))), "`"]);
        return true;
    }

    // Phase 1: Build joined text
    // quasis[0].raw + "@prettier-placeholder-0-id" + quasis[1].raw + ...
    let allocator = f.context().allocator();
    let joined = {
        let mut sb = StringBuilder::new_in(allocator);
        for (idx, quasi_elem) in quasis.iter().enumerate() {
            if idx > 0 {
                sb.push_str(PLACEHOLDER_PREFIX);
                let _ = std::fmt::Write::write_fmt(&mut sb, std::format_args!("{}", idx - 1));
                sb.push_str(PLACEHOLDER_SUFFIX);
            }
            // Use `.raw` (not `.cooked`), CSS/SCSS needs the original escape sequences
            sb.push_str(quasi_elem.value.raw.as_str());
        }
        sb.into_str()
    };

    // Phase 2: Format via the Doc→IR path
    let allocator = f.allocator();
    let group_id_builder = f.group_id_builder();
    let Some(Ok(EmbeddedDocResult::DocWithPlaceholders(ir, placeholder_count))) = f
        .context()
        .external_callbacks()
        .format_embedded_doc(allocator, group_id_builder, "tagged-css", &[joined])
    else {
        return false;
    };

    // Verify all placeholders survived SCSS formatting.
    // Some edge cases (e.g. `/* prettier-ignore */` before a placeholder without semicolon)
    // cause SCSS to drop placeholders.
    // In that case, fall back to regular template formatting
    // (same behavior as Prettier's `replacePlaceholders()` returning `null`).
    // https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/embed/css.js#L42
    if placeholder_count != expressions.len() {
        return false;
    }

    // Phase 3: Replace placeholders in IR with expressions
    let indent_width = f.options().indent_width;
    let format_content = format_once(move |f: &mut Formatter<'_, 'a>| {
        for element in ir {
            match &element {
                FormatElement::Text { text, .. } if text.contains(PLACEHOLDER_PREFIX) => {
                    let parts = split_on_placeholders(text);
                    for (i, part) in parts.iter().enumerate() {
                        if i % 2 == 0 {
                            if !part.is_empty() {
                                write_text_with_line_breaks(f, part, allocator, indent_width);
                            }
                        } else if let Some(idx) = part.parse::<usize>().ok()
                            && let Some(expr) = expressions.get(idx)
                        {
                            // Format `${expr}` directly to preserve soft line breaks
                            // so the printer can decide line breaks based on `printWidth`.
                            // (Regular template expressions use `RemoveSoftLinesBuffer`
                            // which forces single-line layout.)
                            write!(
                                f,
                                [group(&format_args!("${", expr, line_suffix_boundary(), "}"))]
                            );
                        }
                    }
                }
                _ => {
                    f.write_element(element);
                }
            }
        }
    });

    write!(f, ["`", block_indent(&format_content), "`"]);
    true
}

// ---

/// Split text on `@prettier-placeholder-N-id` patterns.
///
/// Returns alternating parts: `[literal, index_str, literal, index_str, ...]`
/// Similar to JavaScript `String.split(/(@prettier-placeholder-(\d+)-id)/)`
/// but only captures the digit group (index).
fn split_on_placeholders(text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut remaining = text;

    loop {
        let Some(start) = remaining.find(PLACEHOLDER_PREFIX) else {
            result.push(remaining);
            break;
        };

        // Push the literal before the placeholder
        result.push(&remaining[..start]);

        // Skip past the prefix
        let after_prefix = &remaining[start + PLACEHOLDER_PREFIX.len()..];

        // Find the digits
        let digit_end =
            after_prefix.bytes().position(|b| !b.is_ascii_digit()).unwrap_or(after_prefix.len());

        if digit_end == 0 {
            // No digits found after prefix — not a valid placeholder, treat as literal
            if let Some(last) = result.last_mut() {
                let end = start + PLACEHOLDER_PREFIX.len();
                *last = &remaining[..end];
            }
            remaining = &remaining[start + PLACEHOLDER_PREFIX.len()..];
            continue;
        }

        let digits = &after_prefix[..digit_end];
        let after_digits = &after_prefix[digit_end..];

        // Check for the `-id` suffix
        if let Some(after_suffix) = after_digits.strip_prefix(PLACEHOLDER_SUFFIX) {
            // Valid placeholder - push the digit index
            result.push(digits);
            remaining = after_suffix;
        } else {
            // Not a valid placeholder, include in the literal
            let end = start + PLACEHOLDER_PREFIX.len() + digit_end;
            if let Some(last) = result.last_mut() {
                *last = &remaining[..end];
            }
            remaining = &remaining[end..];
        }
    }

    result
}

/// Emit text with newlines converted to literal line breaks (`replaceEndOfLine()` equivalent).
///
/// Uses `Text("\n") + ExpandParent` (= `literalline()`)
/// instead of `hard_line_break()` to avoid adding indentation.
///
/// The SCSS formatter has already computed proper indentation in the text content,
/// so we must not add extra indent from the surrounding `block_indent`.
fn write_text_with_line_breaks<'a>(
    f: &mut Formatter<'_, 'a>,
    text: &str,
    allocator: &'a Allocator,
    indent_width: IndentWidth,
) {
    let mut first = true;
    // Splitting on `\n` is safe because `Doc` only contains normalized linebreaks.
    for line in text.split('\n') {
        if !first {
            // Emit literalline: Text("\n") + ExpandParent
            let newline = allocator.alloc_str("\n");
            f.write_element(FormatElement::Text { text: newline, width: TextWidth::multiline(0) });
            f.write_element(FormatElement::ExpandParent);
        }
        first = false;
        if !line.is_empty() {
            let arena_text = allocator.alloc_str(line);
            let width = TextWidth::from_text(arena_text, indent_width);
            f.write_element(FormatElement::Text { text: arena_text, width });
        }
    }
}
