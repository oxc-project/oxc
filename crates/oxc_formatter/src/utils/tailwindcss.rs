use oxc_ast::ast::*;

use crate::{
    Buffer, TailwindcssOptions,
    ast_nodes::{AstNode, AstNodes},
    formatter::{FormatElement, Formatter, prelude::*},
    write,
};

/// Check if a JSX attribute is a tailwind class attribute (class/className or custom tailwindAttributes)
pub fn is_tailwind_jsx_attribute(
    attr_name: &JSXAttributeName<'_>,
    tailwind_options: &TailwindcssOptions,
) -> bool {
    let JSXAttributeName::Identifier(ident) = attr_name else {
        return false;
    };
    let name = ident.name.as_str();

    // Default attributes: `class` and `className`
    let is_default_attr = name == "class" || name == "className";

    if is_default_attr {
        return true;
    }

    // Custom attributes from `tailwindAttributes` option
    tailwind_options
        .tailwind_attributes
        .as_ref()
        .is_some_and(|attrs| attrs.iter().any(|a| a == name))
}

/// Check if a callee expression is a tailwind function (e.g., `clsx`, `cn`, `tw`)
pub fn is_tailwind_function_call(
    callee: &Expression<'_>,
    tailwind_options: &TailwindcssOptions,
) -> bool {
    let Some(functions) = &tailwind_options.tailwind_functions else {
        return false;
    };

    let Expression::Identifier(ident) = callee else {
        return false;
    };

    functions.iter().any(|f| f == ident.name.as_str())
}

/// Writes a template element with Tailwind CSS class sorting support.
///
/// Implements ignoreFirst/ignoreLast/collapseWhitespace logic:
/// - ignoreFirst: class touching previous expression (no leading whitespace) is preserved
/// - ignoreLast: class touching next expression (no trailing whitespace) is preserved
/// - collapseWhitespace: multiple spaces normalized to single space
///
/// Based on <https://github.com/tailwindlabs/prettier-plugin-tailwindcss/blob/28beb4e008b913414562addec4abb8ab261f3828/src/index.ts#L511-L566>
pub fn write_tailwind_template_element<'a>(
    element: &AstNode<'a, TemplateElement<'a>>,
    preserve_whitespace: bool,
    f: &mut Formatter<'_, 'a>,
) {
    let content = f.source_text().text_for(element);

    if preserve_whitespace {
        let index = f.context_mut().add_tailwind_class(content.to_string());
        f.write_element(FormatElement::TailwindClass(index));
        return;
    }

    let (quasi_index, expressions_count) = get_template_position(element).unwrap_or((0, 0));
    let is_first_quasi = quasi_index == 0;
    let is_last_quasi = quasi_index >= expressions_count;

    // Determine which boundary classes to ignore (classes touching expressions)
    let ignore_first = !is_first_quasi && !content.starts_with(|c: char| c.is_ascii_whitespace());
    let ignore_last = !is_last_quasi && !content.ends_with(|c: char| c.is_ascii_whitespace());

    // Find whitespace positions for splitting
    let first_ws = ignore_first.then(|| content.find(|c: char| c.is_ascii_whitespace())).flatten();
    let last_ws = ignore_last.then(|| content.rfind(|c: char| c.is_ascii_whitespace())).flatten();

    // Split into: prefix (ignored) | sortable | suffix (ignored)
    let (prefix, sortable, suffix) = match (first_ws, last_ws) {
        (Some(f), Some(l)) if f < l => (&content[..f], &content[f..=l], &content[l + 1..]),
        (Some(f), _) => (&content[..f], &content[f..], ""),
        (None, Some(l)) => ("", &content[..=l], &content[l + 1..]),
        (None, None) => ("", content, ""),
    };

    let has_prefix = !prefix.is_empty();
    let has_suffix = !suffix.is_empty();

    // Write prefix (class attached to previous expression)
    if has_prefix {
        write!(f, text(prefix));
    }

    // Write sortable content with normalized whitespace
    let trimmed = sortable.trim();
    if trimmed.is_empty() {
        // Whitespace-only: normalize to single space
        if !sortable.is_empty() {
            write!(f, text(" "));
        }
    } else {
        // Leading space: after expression or after ignored prefix
        if !is_first_quasi || has_prefix {
            write!(f, text(" "));
        }

        let index = f.context_mut().add_tailwind_class(trimmed.to_string());
        f.write_element(FormatElement::TailwindClass(index));

        // Trailing space: before expression or before ignored suffix
        if !is_last_quasi || has_suffix {
            write!(f, text(" "));
        }
    }

    // Write suffix (class attached to next expression)
    if has_suffix {
        write!(f, text(suffix));
    }
}

pub fn write_tailwind_string_literal<'a>(
    string_literal: &AstNode<'a, StringLiteral<'a>>,
    f: &mut Formatter<'_, 'a>,
) {
    let content = f.source_text().text_for(string_literal);
    let is_direct_child = matches!(string_literal.parent, AstNodes::JSXAttribute(_));

    // For nested string literals (not direct JSXAttribute children), preserve whitespace
    // because the sorter will trim it otherwise
    if is_direct_child {
        // Direct attribute value - sorter handles everything
        let index = f.context_mut().add_tailwind_class(content.to_string());
        f.write_element(FormatElement::TailwindClass(index));
    } else {
        // Nested string literal - preserve leading/trailing whitespace
        let leading_ws: String = content.chars().take_while(char::is_ascii_whitespace).collect();
        let trailing_ws: String =
            content.chars().rev().take_while(char::is_ascii_whitespace).collect();
        let trimmed = content.trim();

        // Write leading whitespace
        if !leading_ws.is_empty() {
            let ws = f.context().allocator().alloc_str(&leading_ws);
            write!(f, text(ws));
        }

        // Sort the trimmed content (if any)
        if !trimmed.is_empty() {
            let index = f.context_mut().add_tailwind_class(trimmed.to_string());
            f.write_element(FormatElement::TailwindClass(index));
        }

        // Write trailing whitespace
        if !trailing_ws.is_empty() {
            let ws = f.context().allocator().alloc_str(&trailing_ws);
            write!(f, text(ws));
        }
    }
}

/// Returns (quasi_index, expressions_count) for a template element within its parent template literal.
fn get_template_position(element: &AstNode<'_, TemplateElement<'_>>) -> Option<(usize, usize)> {
    match element.parent {
        AstNodes::TemplateLiteral(tl) => {
            let expressions_count = tl.expressions.len();
            // Find our index by comparing spans
            let quasi_index = tl.quasis.iter().position(|q| q.span == element.span).unwrap_or(0);
            Some((quasi_index, expressions_count))
        }
        AstNodes::TSTemplateLiteralType(tl) => {
            let expressions_count = tl.types.len();
            let quasi_index = tl.quasis.iter().position(|q| q.span == element.span).unwrap_or(0);
            Some((quasi_index, expressions_count))
        }
        _ => None,
    }
}
