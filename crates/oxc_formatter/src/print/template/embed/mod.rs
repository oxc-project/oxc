mod graphql;

use oxc_allocator::{Allocator, StringBuilder};
use oxc_ast::ast::*;
use oxc_syntax::line_terminator::LineTerminatorSplitter;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::{Formatter, prelude::*},
    write,
};

/// Try to format a tagged template with the embedded formatter if supported.
/// Returns `true` if formatting was performed, `false` if not applicable.
pub(super) fn try_format_embedded_template<'a>(
    tagged: &AstNode<'a, TaggedTemplateExpression<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> bool {
    match get_tag_name(&tagged.tag) {
        Some("css" | "styled") => try_embed_css(tagged, f),
        Some("gql" | "graphql") => graphql::format_graphql_doc(tagged.quasi(), f),
        Some("html") => try_embed_html(tagged, f),
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

/// Try to format a template literal inside css prop or styled-jsx with the embedded formatter.
/// Returns `true` if formatting was attempted, `false` if not applicable.
pub(super) fn try_format_css_template<'a>(
    template_literal: &AstNode<'a, TemplateLiteral<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> bool {
    // TODO: Support expressions in the css-in-js
    if !template_literal.is_no_substitution_template() {
        return false;
    }

    if !is_in_css_jsx(template_literal) {
        return false;
    }

    let template_content = template_literal.quasis()[0].value.raw.as_str();
    format_embedded_template(f, "styled-jsx", template_content)
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
    // TODO: Support expressions in the css-in-js and html-in-js
    // Also need to split html or css path with using `.raw` for css, `.cooked` for html
    if !template_literal.is_no_substitution_template() {
        return false;
    }

    let Some(language) = get_angular_component_language(template_literal) else {
        return false;
    };

    let template_content = template_literal.quasis()[0].value.raw.as_str();
    format_embedded_template(f, language, template_content)
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

fn try_embed_css<'a>(
    tagged: &AstNode<'a, TaggedTemplateExpression<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> bool {
    // TODO: Remove this check and use placeholder approach for expressions
    if !tagged.quasi.is_no_substitution_template() {
        return false;
    }
    let template_content = tagged.quasi.quasis[0].value.raw.as_str();
    format_embedded_template(f, "tagged-css", template_content)
}

fn try_embed_html<'a>(
    tagged: &AstNode<'a, TaggedTemplateExpression<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> bool {
    // TODO: Remove this check and use placeholder approach for expressions
    if !tagged.quasi.is_no_substitution_template() {
        return false;
    }
    let template_content = tagged.quasi.quasis[0].value.raw.as_str();
    format_embedded_template(f, "tagged-html", template_content)
}

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
