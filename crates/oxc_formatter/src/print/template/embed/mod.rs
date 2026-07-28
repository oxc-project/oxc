mod css;
mod graphql;
mod html;
mod markdown;

use oxc_allocator::{Allocator, ArenaStringBuilder};
use oxc_ast::ast::*;
use oxc_formatter_core::IndentWidth;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::{FormatElement, format_element::TextWidth, prelude::*},
    write,
};

/// Try to format a tagged template with the embedded formatter if supported.
/// Returns `true` if formatting was performed, `false` if not applicable.
pub(super) fn try_format_embedded_template<'a>(
    tagged: &AstNode<'a, TaggedTemplateExpression<'a>>,
    f: &mut JsFormatter<'_, 'a>,
) -> bool {
    match get_tag_name(&tagged.tag) {
        Some("css" | "styled") => css::format_css_doc(tagged.quasi(), f),
        Some("gql" | "graphql") => graphql::format_graphql_doc(tagged.quasi(), f),
        Some("html") => html::format_html_doc(tagged.quasi(), f, false),
        // Markdown never supports `${}` (Prettier doesn't either)
        Some("md" | "markdown") if tagged.quasi.is_no_substitution_template() => {
            markdown::try_embed_markdown(tagged, f)
        }
        _ => false,
    }
}

fn get_tag_name<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    let expr = expr.get_inner_expression();
    match expr {
        Expression::Identifier(ident) => Some(ident.name.as_str()),
        Expression::StaticMemberExpression(member) => get_tag_name(&member.object),
        Expression::ComputedMemberExpression(exp) => get_tag_name(&exp.object),
        Expression::CallExpression(call) => get_tag_name(&call.callee),
        _ => None,
    }
}

/// Try to format a template literal inside a `graphql()` function call.
/// Returns `true` if formatting was performed, `false` if not applicable.
///
/// NOTE: when this fires for a single-argument call,
/// `arguments.rs` also applies a "hugging" layout (`graphql(`…`)` with no trailing comma).
/// See `is_graphql_call_with_single_template_arg()` in `arguments.rs`.
pub(super) fn try_format_graphql_call<'a>(
    template: &AstNode<'a, TemplateLiteral<'a>>,
    f: &mut JsFormatter<'_, 'a>,
) -> bool {
    let AstNodes::CallExpression(call) = template.parent() else { return false };
    let Expression::Identifier(ident) = &call.callee else { return false };
    if ident.name.as_str() != "graphql" {
        return false;
    }
    graphql::format_graphql_doc(template, f)
}

/// Try to format a template literal with a language comment (e.g., `/* HTML */`).
/// Returns `true` if formatting was performed, `false` if not applicable.
///
/// Supported languages:
/// - HTML
/// - GraphQL
pub(super) fn try_format_comment_embedded<'a>(
    template: &AstNode<'a, TemplateLiteral<'a>>,
    f: &mut JsFormatter<'_, 'a>,
) -> bool {
    // By the time `TemplateLiteral::write()` runs, parent nodes have already printed
    // leading comments via the cursor-based system. So `/* HTML */` is the last printed comment.
    let Some(comment) = f.context().comments().printed_comments().last() else {
        return false;
    };
    if !comment.is_block() || comment.span.end > template.span.start {
        return false;
    }

    // Ensure there's nothing but whitespace between the comment and the template literal.
    // This prevents matching `const html /* HTML */ = \`...\`` where `=` is between them.
    if !f
        .source_text()
        .all_bytes_match(comment.span.end, template.span.start, |b| b.is_ascii_whitespace())
    {
        return false;
    }

    let text = f.source_text().text_for(&comment.content_span());
    match text {
        " HTML " => html::format_html_doc(template, f, false),
        " GraphQL " => graphql::format_graphql_doc(template, f),
        _ => false,
    }
}

/// Try to format a template literal inside css prop or styled-jsx with the embedded formatter.
/// Returns `true` if formatting was attempted, `false` if not applicable.
pub(super) fn try_format_css_template<'a>(
    template_literal: &AstNode<'a, TemplateLiteral<'a>>,
    f: &mut JsFormatter<'_, 'a>,
) -> bool {
    if !is_in_css_jsx(template_literal) {
        return false;
    }
    css::format_css_doc(template_literal, f)
}

/// Check if the template literal is inside a `css` prop or `<style jsx>` element.
fn is_in_css_jsx<'a>(node: &AstNode<'a, TemplateLiteral<'a>>) -> bool {
    let AstNodes::JSXExpressionContainer(container) = node.parent() else {
        return false;
    };

    match container.parent() {
        AstNodes::JSXAttribute(attribute) => {
            if let JSXAttributeName::Identifier(ident) = &attribute.name
                && ident.name == "css"
            {
                return true;
            }
        }
        AstNodes::JSXElement(element) => {
            if let JSXElementName::Identifier(ident) = &element.opening_element.name
                && ident.name == "style"
                && element.opening_element.attributes.iter().any(|attr| {
                    matches!(attr.as_attribute().and_then(|a| a.name.as_identifier()), Some(name) if name.name == "jsx")
                })
            {
                return true;
            }
        }
        _ => {}
    }
    false
}

/// Try to format a template literal inside Angular @Component's template/styles property.
/// Returns `true` if formatting was performed, `false` if not applicable.
pub(super) fn try_format_angular_component<'a>(
    template_literal: &AstNode<'a, TemplateLiteral<'a>>,
    f: &mut JsFormatter<'_, 'a>,
) -> bool {
    match get_angular_component_property(template_literal) {
        Some("template") => html::format_html_doc(template_literal, f, true),
        Some("styles") => css::format_css_doc(template_literal, f),
        _ => false,
    }
}

/// Detect Angular `@Component({ template: \`...\`, styles: \`...\` })`.
fn get_angular_component_property<'a>(node: &AstNode<'a, TemplateLiteral<'a>>) -> Option<&'a str> {
    let prop = match node.parent() {
        AstNodes::ObjectProperty(prop) => prop,
        AstNodes::ArrayExpression(arr) => {
            let AstNodes::ObjectProperty(prop) = arr.parent() else {
                return None;
            };
            prop
        }
        _ => return None,
    };

    if prop.computed {
        return None;
    }
    let PropertyKey::StaticIdentifier(key) = &prop.key else {
        return None;
    };

    let AstNodes::ObjectExpression(obj) = prop.parent() else {
        return None;
    };
    let AstNodes::CallExpression(call) = obj.parent() else {
        return None;
    };
    let Expression::Identifier(ident) = &call.callee else {
        return None;
    };
    if ident.name.as_str() != "Component" {
        return None;
    }
    if !matches!(call.parent(), AstNodes::Decorator(_)) {
        return None;
    }

    match key.name.as_str() {
        "template" | "styles" => Some(key.name.as_str()),
        _ => None,
    }
}

// ---

/// Split text on placeholder patterns, returning alternating parts:
/// `[literal, index_str, literal, index_str, ...]`
///
/// Handles both:
/// - CSS: `` `PLACEHOLDER-{N}` ``
/// - HTML: `PRETTIER_HTML_PLACEHOLDER_{N}_{C}_IN_JS`
///
/// The optional `_{digits}` counter group between index and suffix is skipped when present,
/// so the same function works for both formats.
fn split_on_placeholders<'a>(text: &'a str, prefix: &str, suffix: &str) -> Vec<&'a str> {
    let mut result = vec![];
    let mut remaining = text;

    loop {
        let Some(start) = remaining.find(prefix) else {
            result.push(remaining);
            break;
        };

        // Push the literal before the placeholder
        result.push(&remaining[..start]);

        // Skip past the prefix
        let after_prefix = &remaining[start + prefix.len()..];

        // Find the index digits
        let digit_end =
            after_prefix.bytes().position(|b| !b.is_ascii_digit()).unwrap_or(after_prefix.len());

        if digit_end == 0 {
            // No digits found after prefix — not a valid placeholder, treat as literal
            if let Some(last) = result.last_mut() {
                let end = start + prefix.len();
                *last = &remaining[..end];
            }
            remaining = &remaining[start + prefix.len()..];
            continue;
        }

        let digits = &after_prefix[..digit_end];
        let mut after_digits = &after_prefix[digit_end..];

        // Skip optional `_{digits}` (e.g., HTML counter `_0`)
        if let Some(after_underscore) = after_digits.strip_prefix('_') {
            let counter_end = after_underscore
                .bytes()
                .position(|b| !b.is_ascii_digit())
                .unwrap_or(after_underscore.len());
            if counter_end > 0 {
                after_digits = &after_underscore[counter_end..];
            }
        }

        // Check for the suffix
        if let Some(after_suffix) = after_digits.strip_prefix(suffix) {
            // Valid placeholder — push the digit index
            result.push(digits);
            remaining = after_suffix;
        } else {
            // Not a valid placeholder, include in the literal
            let end = start + prefix.len() + digit_end;
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
/// Uses [`literal_line_break`] instead of `hard_line_break()` to avoid adding indentation.
///
/// The external formatter has already computed proper indentation in the text content,
/// so we must not add extra indent from the surrounding `block_indent`.
fn write_text_with_line_breaks<'a>(
    f: &mut JsFormatter<'_, 'a>,
    text: &str,
    allocator: &'a Allocator,
    indent_width: IndentWidth,
) {
    let mut first = true;
    // Splitting on `\n` is safe because `Doc` only contains normalized linebreaks.
    for line in text.split('\n') {
        if !first {
            write!(f, [literal_line_break()]);
        }
        first = false;
        if !line.is_empty() {
            let arena_text = allocator.alloc_str(line);
            let width = TextWidth::from_text(arena_text, indent_width);
            f.write_element(FormatElement::Text { text: arena_text, width });
        }
    }
}

// ---

/// Re-escape template-literal characters (`` ` ``, `${`, `\`) in every `Text` element of an embedded IR.
/// The IR is re-inserted into a JS template literal built from `.cooked` values,
/// so these characters need escaping.
fn escape_template_chars_in_ir<'a>(
    ir: &mut [FormatElement<'a>],
    allocator: &'a Allocator,
    indent_width: IndentWidth,
) {
    map_text_in_ir(ir, indent_width, |s| escape_template_chars(s, allocator));
}

/// Re-escape backticks in `Text` elements of an embedded IR using Prettier's "raw" escape rule.
/// Used by markdown-in-JS, which uses `.raw` quasi values.
fn escape_backticks_raw_in_ir<'a>(
    ir: &mut [FormatElement<'a>],
    allocator: &'a Allocator,
    indent_width: IndentWidth,
) {
    map_text_in_ir(ir, indent_width, |s| escape_backticks_raw(s, allocator));
}

/// Walk an embedded IR (a flat tag stream)
/// and replace each `Text` element whose string the closure rewrites.
/// `None` from the closure leaves the element untouched.
fn map_text_in_ir<'a, F>(ir: &mut [FormatElement<'a>], indent_width: IndentWidth, mut rewrite: F)
where
    F: FnMut(&'a str) -> Option<&'a str>,
{
    for element in ir.iter_mut() {
        if let FormatElement::Text { text, .. } = element
            && let Some(new_text) = rewrite(text)
        {
            let width = TextWidth::from_text(new_text, indent_width);
            *element = FormatElement::Text { text: new_text, width };
        }
    }
}

/// Escape characters that would break template literal syntax.
///
/// Equivalent to Prettier's `uncookTemplateElementValue`:
/// `cookedValue.replaceAll(/([\\`]|\$\{)/gu, String.raw`\$1`);`
///
/// Returns `None` when no escape is needed.
fn escape_template_chars<'a>(s: &'a str, allocator: &'a Allocator) -> Option<&'a str> {
    // All escape targets (`\`, `` ` ``, `${`) are single-byte ASCII;
    // UTF-8 continuation bytes never match, so byte scans/copies are safe.
    let bytes = s.as_bytes();
    let first = bytes.iter().enumerate().position(|(i, &b)| {
        b == b'\\' || b == b'`' || (b == b'$' && bytes.get(i + 1) == Some(&b'{'))
    })?;

    let mut result = ArenaStringBuilder::with_capacity_in(bytes.len(), allocator);
    result.push_str(&s[..first]);

    let mut run_start = first;
    let mut i = first;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' || b == b'`' {
            result.push_str(&s[run_start..i]);
            result.push('\\');
            result.push(b as char);
            i += 1;
            run_start = i;
        } else if b == b'$' && bytes.get(i + 1) == Some(&b'{') {
            result.push_str(&s[run_start..i]);
            result.push_str("\\${");
            i += 2;
            run_start = i;
        } else {
            i += 1;
        }
    }
    result.push_str(&s[run_start..]);

    Some(result.into_str())
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
///
/// Returns `None` when no escape is needed.
fn escape_backticks_raw<'a>(s: &'a str, allocator: &'a Allocator) -> Option<&'a str> {
    // `` ` `` and `\` are ASCII; UTF-8 continuation bytes never match,
    // so the byte walk is safe and avoids per-char decode.
    let bytes = s.as_bytes();
    if !bytes.contains(&b'`') {
        return None;
    }
    let mut result = ArenaStringBuilder::with_capacity_in(bytes.len() + 1, allocator);
    let mut run_start = 0;
    let mut bs_count: usize = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\\' {
            bs_count += 1;
        } else if b == b'`' {
            // Emit the run up to (but not including) the backtick;
            // this already contains the original `bs_count` backslashes.
            // Then emit `bs_count` MORE backslashes to double them,
            // plus the `\` that escapes the backtick itself.
            result.push_str(&s[run_start..i]);
            for _ in 0..bs_count {
                result.push('\\');
            }
            result.push_str("\\`");
            bs_count = 0;
            run_start = i + 1;
        } else {
            bs_count = 0;
        }
    }
    result.push_str(&s[run_start..]);
    Some(result.into_str())
}

/// Count placeholder occurrences matching `{prefix}{digits}(_{digits})?{suffix}`.
///
/// The optional `_{digits}` middle group lets this serve both:
/// - CSS sentinel `` `PLACEHOLDER-{N}` `` (no counter)
/// - HTML sentinel `PRETTIER_HTML_PLACEHOLDER_{N}_{C}_IN_JS` (with counter)
///
/// Mirrors the grammar that [`split_on_placeholders`] uses for substitution.
fn count_placeholders(text: &str, prefix: &str, suffix: &str) -> usize {
    let mut count = 0;
    let mut rest = text;
    while let Some(start) = rest.find(prefix) {
        let after_prefix = &rest[start + prefix.len()..];
        let digit_end =
            after_prefix.bytes().position(|b| !b.is_ascii_digit()).unwrap_or(after_prefix.len());
        if digit_end > 0 {
            let mut after_digits = &after_prefix[digit_end..];
            // Skip optional `_{digits}` counter (HTML).
            if let Some(after_underscore) = after_digits.strip_prefix('_') {
                let counter_end = after_underscore
                    .bytes()
                    .position(|b| !b.is_ascii_digit())
                    .unwrap_or(after_underscore.len());
                if counter_end > 0 {
                    after_digits = &after_underscore[counter_end..];
                }
            }
            if let Some(tail) = after_digits.strip_prefix(suffix) {
                count += 1;
                rest = tail;
                continue;
            }
        }
        rest = &rest[start + prefix.len()..];
    }
    count
}
