use oxc_allocator::{Allocator, StringBuilder};
use oxc_ast::ast::*;
use oxc_syntax::line_terminator::LineTerminatorSplitter;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::{Formatter, prelude::*},
    write,
};

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

/// Format embedded language content (CSS, GraphQL, etc.)
/// inside a template literal using an external formatter (Prettier).
///
/// NOTE: Unlike Prettier, which formats embedded languages in-process via its document IR
/// (e.g. `textToDoc()` → `indent([hardline, doc])`),
/// we communicate with the external formatter over a plain text interface.
///
/// This means we must:
/// - Dedent the inherited JS/TS indentation before sending
/// - Reconstruct the template structure (`block_indent()`) from the formatted text
///
/// If `format_embedded()` could return `FormatElement` (IR) directly,
/// most of work in this function would be unnecessary.
fn format_embedded_template<'a>(
    f: &mut Formatter<'_, 'a>,
    language: &str,
    template_content: &str,
) -> bool {
    // Whitespace-only templates become empty backticks.
    // Regular template literals would preserve them as-is.
    if template_content.trim().is_empty() {
        write!(f, ["``"]);
        return true;
    }

    // Strip inherited indentation.
    // So the external formatter receives clean embedded language content.
    // Otherwise, indentation may be duplicated on each formatting pass.
    let template_content = dedent(template_content, f.context().allocator());

    let Some(Ok(formatted)) =
        f.context().external_callbacks().format_embedded(language, template_content)
    else {
        return false;
    };

    // Format with proper template literal structure:
    // - Opening backtick
    // - Hard line break (newline after backtick)
    // - Indented content (each line will be indented)
    // - Hard line break (newline before closing backtick)
    // - Closing backtick
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

    write!(f, ["`", block_indent(&format_content), "`"]);

    true
}

/// Strip the common leading indentation from all non-empty lines in `text`.
/// Returns the original `text` unchanged if there is no common indentation.
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

/// Try to format a tagged template with the embedded formatter if supported.
/// Returns `true` if formatting was performed, `false` if not applicable.
pub(in super::super) fn try_format_embedded_template<'a>(
    tagged: &AstNode<'a, TaggedTemplateExpression<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> bool {
    let quasi = &tagged.quasi;
    // TODO: Support expressions in the template
    if !quasi.is_no_substitution_template() {
        return false;
    }

    let language = match get_tag_name(&tagged.tag) {
        Some("css" | "styled") => "tagged-css",
        Some("gql" | "graphql") => "tagged-graphql",
        Some("html") => "tagged-html",
        Some("md" | "markdown") => "tagged-markdown",
        _ => return false,
    };

    let template_content = quasi.quasis[0].value.raw.as_str();

    format_embedded_template(f, language, template_content)
}

/// Check if the template literal is inside a `css` prop or `<style jsx>` element.
///
/// ```jsx
/// <div css={`color: red;`} />
/// <style jsx>{`div { color: red; }`}</style>
/// ```
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

/// Try to format a template literal inside css prop or styled-jsx with the embedded formatter.
/// Returns `true` if formatting was attempted, `false` if not applicable.
pub(in super::super) fn try_format_css_template<'a>(
    template_literal: &AstNode<'a, TemplateLiteral<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> bool {
    // TODO: Support expressions in the template
    if !template_literal.is_no_substitution_template() {
        return false;
    }

    if !is_in_css_jsx(template_literal) {
        return false;
    }

    let quasi = template_literal.quasis();
    let template_content = quasi[0].value.raw.as_str();

    format_embedded_template(f, "styled-jsx", template_content)
}

/// Try to format a template literal inside Angular @Component's template/styles property.
/// Returns `true` if formatting was performed, `false` if not applicable.
pub(in super::super) fn try_format_angular_component<'a>(
    template_literal: &AstNode<'a, TemplateLiteral<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> bool {
    // TODO: Support expressions in the template
    if !template_literal.is_no_substitution_template() {
        return false;
    }

    // Check if inside `@Component` decorator's `template/styles` property
    let Some(language) = get_angular_component_language(template_literal) else {
        return false;
    };

    let quasi = template_literal.quasis();
    let template_content = quasi[0].value.raw.as_str();

    format_embedded_template(f, language, template_content)
}

/// Check if this template literal is one of:
/// ```ts
/// @Component({
///   template: `...`,
///   styles: `...`,
///   // or styles: [`...`]
/// })
/// ```
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

    // Skip computed properties
    if prop.computed {
        return None;
    }
    let PropertyKey::StaticIdentifier(key) = &prop.key else {
        return None;
    };

    // Check parent chain: ObjectExpression -> CallExpression(Component) -> Decorator
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

    let language = match key.name.as_str() {
        "template" => "angular-template",
        "styles" => "angular-styles",
        _ => return None,
    };
    Some(language)
}
