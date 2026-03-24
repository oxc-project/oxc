mod css;
mod graphql;
mod html;

use oxc_allocator::{Allocator, StringBuilder};
use oxc_ast::ast::*;
use oxc_syntax::line_terminator::LineTerminatorSplitter;

use crate::{
    IndentWidth,
    ast_nodes::{AstNode, AstNodes},
    formatter::{FormatElement, Formatter, format_element::TextWidth, prelude::*},
    write,
};

/// Try to format a tagged template with the embedded formatter if supported.
/// Returns `true` if formatting was performed, `false` if not applicable.
pub(super) fn try_format_embedded_template<'a>(
    tagged: &AstNode<'a, TaggedTemplateExpression<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> bool {
    match get_tag_name(&tagged.tag) {
        Some("css" | "styled") => css::format_css_doc(tagged.quasi(), f),
        Some("gql" | "graphql") => graphql::format_graphql_doc(tagged.quasi(), f),
        Some("html") => html::format_html_doc(tagged.quasi(), f, "tagged-html"),
        Some("md" | "markdown") => try_embed_markdown(tagged, f),
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
    f: &mut Formatter<'_, 'a>,
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
    f: &mut Formatter<'_, 'a>,
) -> bool {
    let Some(language) = get_language_comment(template, f) else {
        return false;
    };
    match language {
        "html" => html::format_html_doc(template, f, "tagged-html"),
        "graphql" => graphql::format_graphql_doc(template, f),
        _ => false,
    }
}

/// Check if the template literal has a leading block comment that specifies an embedded language.
///
/// Returns the language name if found.
/// The comment must be:
/// - a block comment
/// - with exactly one space on either side
fn get_language_comment<'a>(
    template: &AstNode<'a, TemplateLiteral<'a>>,
    f: &Formatter<'_, 'a>,
) -> Option<&'static str> {
    // By the time `TemplateLiteral::write()` runs, parent nodes have already printed
    // leading comments via the cursor-based system. So `/* HTML */` is the last printed comment.
    let comment = f.context().comments().printed_comments().last()?;
    if !comment.is_block() || comment.span.end > template.span.start {
        return None;
    }

    // Ensure there's nothing but whitespace between the comment and the template literal.
    // This prevents matching `const html /* HTML */ = \`...\`` where `=` is between them.
    if !f
        .source_text()
        .all_bytes_match(comment.span.end, template.span.start, |b| b.is_ascii_whitespace())
    {
        return None;
    }

    let text = f.source_text().text_for(&comment.content_span());
    match text {
        " HTML " => Some("html"),
        " GraphQL " => Some("graphql"),
        _ => None,
    }
}

/// Try to format a template literal inside css prop or styled-jsx with the embedded formatter.
/// Returns `true` if formatting was attempted, `false` if not applicable.
pub(super) fn try_format_css_template<'a>(
    template_literal: &AstNode<'a, TemplateLiteral<'a>>,
    f: &mut Formatter<'_, 'a>,
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
    f: &mut Formatter<'_, 'a>,
) -> bool {
    match get_angular_component_language(template_literal) {
        Some("angular-template") => html::format_html_doc(template_literal, f, "angular-template"),
        Some("angular-styles") => css::format_css_doc(template_literal, f),
        _ => false,
    }
}

/// Detect Angular `@Component({ template: \`...\`, styles: \`...\` })`.
fn get_angular_component_language(node: &AstNode<'_, TemplateLiteral<'_>>) -> Option<&'static str> {
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
        "template" => Some("angular-template"),
        "styles" => Some("angular-styles"),
        _ => None,
    }
}

// ---

fn try_embed_markdown<'a>(
    tagged: &AstNode<'a, TaggedTemplateExpression<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> bool {
    // Markdown never supports expressions (Prettier doesn't either)
    if !tagged.quasi.is_no_substitution_template() {
        return false;
    }
    let template_content = tagged.quasi.quasis[0].value.raw.as_str();
    format_embedded_template(f, "tagged-markdown", template_content)
}

// ---

/// Format embedded language content inside a template literal using the string path.
///
/// This is the shared formatting logic for no-substitution templates:
/// dedent → external formatter (Prettier) → reconstruct template structure.
fn format_embedded_template<'a>(
    f: &mut Formatter<'_, 'a>,
    language: &str,
    template_content: &str,
) -> bool {
    if template_content.trim().is_empty() {
        write!(f, ["``"]);
        return true;
    }

    let template_content = dedent(template_content, f.context().allocator());

    let Some(Ok(formatted)) =
        f.context().external_callbacks().format_embedded(language, template_content)
    else {
        return false;
    };

    let format_content = format_with(|f: &mut Formatter<'_, 'a>| {
        let content = f.context().allocator().alloc_str(&formatted);
        for line in LineTerminatorSplitter::new(content) {
            if line.is_empty() {
                write!(f, [empty_line()]);
            } else {
                write!(f, [text(line), hard_line_break()]);
            }
        }
    });

    // NOTE: This path always returns the formatted string with each line indented,
    // regardless of the length of the content, which may not be compatible with Prettier in some cases.
    // If we use `Doc` like in the gql-in-js path, it would behave aligned with Prettier.
    write!(f, ["`", block_indent(&format_content), "`"]);
    true
}

/// Strip the common leading indentation from all non-empty lines in `text`.
/// The `text` here is taken from `.raw`, so only `\n` is used as the line terminator.
fn dedent<'a>(text: &'a str, allocator: &'a Allocator) -> &'a str {
    let min_indent = text
        .split('\n')
        .filter(|line| !line.trim_ascii_start().is_empty())
        .map(|line| line.bytes().take_while(u8::is_ascii_whitespace).count())
        .min()
        .unwrap_or(0);

    if min_indent == 0 {
        return text;
    }

    let mut result = StringBuilder::with_capacity_in(text.len(), allocator);
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            result.push('\n');
        }
        let strip = line.bytes().take_while(u8::is_ascii_whitespace).count().min(min_indent);
        result.push_str(&line[strip..]);
    }

    result.into_str()
}

/// Split text on placeholder patterns, returning alternating parts:
/// `[literal, index_str, literal, index_str, ...]`
///
/// Handles both:
/// - CSS: `@prettier-placeholder-{N}-id`
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
/// Uses `Text("\n") + ExpandParent` (= `literalline()`)
/// instead of `hard_line_break()` to avoid adding indentation.
///
/// The external formatter has already computed proper indentation in the text content,
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
