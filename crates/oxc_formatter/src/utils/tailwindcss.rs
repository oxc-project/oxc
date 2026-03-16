//! Tailwind CSS class sorting utilities.
//!
//! This module provides utilities for sorting Tailwind CSS classes within:
//! - JSX attributes (`className`, `class`, or custom attributes)
//! - Function calls (`clsx()`, `cn()`, `tw()`, or custom functions)
//! - Template literals with expressions
//!
//! Based on [prettier-plugin-tailwindcss](https://github.com/tailwindlabs/prettier-plugin-tailwindcss).

use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    Buffer, SortTailwindcssOptions,
    ast_nodes::{AstNode, AstNodes},
    formatter::{FormatElement, Formatter, TailwindContextEntry, prelude::*},
    write,
};

use super::string::{FormatLiteralStringToken, StringLiteralParentKind};

// ============================================================================
// Detection Functions
// ============================================================================

/// Returns the Tailwind context if we should sort classes in the given string literal.
///
/// Returns `Some(ctx)` when:
/// - We're inside a Tailwind function call context
/// - The context is not disabled (e.g., not inside a nested non-Tailwind call)
/// - The string contains whitespace (indicating multiple classes to sort)
pub fn tailwind_context_for_string_literal<'a>(
    string: &AstNode<'a, StringLiteral<'a>>,
    f: &Formatter<'_, 'a>,
) -> Option<TailwindContextEntry> {
    f.context().tailwind_context().copied().filter(|ctx| {
        let text = f.source_text().text_for(string);

        if ctx.disabled {
            return false;
        }

        text.as_bytes().iter().any(|&b| b.is_ascii_whitespace())
    })
}

/// Checks if a JSX attribute is a Tailwind class attribute.
///
/// Returns `true` for:
/// - `class` and `className` (default attributes)
/// - Custom attributes specified in `attributes` option
pub fn is_tailwind_jsx_attribute(
    attr_name: &JSXAttributeName<'_>,
    options: &SortTailwindcssOptions,
) -> bool {
    let JSXAttributeName::Identifier(ident) = attr_name else {
        return false;
    };
    let name = ident.name.as_str();

    // Default: `class` and `className`
    if name == "class" || name == "className" {
        return true;
    }

    // Custom attributes from options
    options.attributes.iter().any(|a| a == name)
}

/// Checks if a callee expression is a Tailwind function.
///
/// Traverses through `CallExpression` and `MemberExpression` nodes to find the
/// root identifier, then checks if it matches any function in `functions`.
///
/// This matches patterns like:
/// - `clsx(...)` - direct call
/// - `clsx.foo(...)` - member expression
/// - `obj.clsx(...)` - object member
/// - `foo().clsx(...)` - chained calls
///
/// Based on [prettier-plugin-tailwindcss's `isSortableExpression`](https://github.com/tailwindlabs/prettier-plugin-tailwindcss/blob/28beb4e008b913414562addec4abb8ab261f3828/src/index.ts#L584-L605).
pub fn is_tailwind_function_call(
    callee: &Expression<'_>,
    options: &SortTailwindcssOptions,
) -> bool {
    if options.functions.is_empty() {
        return false;
    }

    // Traverse property accesses and function calls to find the leading identifier
    let mut node = callee;

    loop {
        match node {
            Expression::CallExpression(call) => {
                node = &call.callee;
            }
            Expression::StaticMemberExpression(member) => {
                node = &member.object;
            }
            Expression::ComputedMemberExpression(member) => {
                node = &member.object;
            }
            Expression::Identifier(ident) => {
                return options.functions.iter().any(|f| f == ident.name.as_str());
            }
            _ => return false,
        }
    }
}

// ============================================================================
// Whitespace Collapse Logic
// ============================================================================

/// Controls whether whitespace can be trimmed at string boundaries.
///
/// When `start` or `end` is `false`, a single space must be preserved
/// at that boundary to maintain proper class separation.
///
/// Based on [prettier-plugin-tailwindcss's `canCollapseWhitespaceIn`](https://github.com/tailwindlabs/prettier-plugin-tailwindcss/blob/28beb4e008b913414562addec4abb8ab261f3828/src/index.ts#L607-L648).
#[derive(Debug, Clone, Copy, Default)]
pub struct CollapseWhitespace {
    /// `true` = can trim leading whitespace, `false` = preserve one space
    pub start: bool,
    /// `true` = can trim trailing whitespace, `false` = preserve one space
    pub end: bool,
}

impl CollapseWhitespace {
    fn new() -> Self {
        Self { start: true, end: true }
    }
}

/// Determines whitespace collapse rules for a string/template literal based on context.
///
/// # Rules
///
/// 1. **Template expression context** (`${...}`):
///    - If quasi before doesn't end with whitespace → preserve leading space
///    - If quasi after doesn't start with whitespace → preserve trailing space
///
/// 2. **Binary concat context** (`a + "..." + b`):
///    - On left side of `+` → preserve trailing space (need separation from `+ right`)
///    - On right side of `+` → preserve leading space (need separation from `left +`)
///
/// # Examples
///
/// ```text
/// // Template expression - quasi "header" has no trailing whitespace
/// `header${x ? " active" : ""}`
/// //           ^ preserve leading space
///
/// // Binary concat - string in middle needs both spaces preserved
/// className={a + " p-4 " + b}
/// //             ^     ^ preserve both
///
/// // Binary concat - template on right side only
/// a + ` flex p-4`     // leading space preserved (from `a +`)
///     ^
///
/// // Binary concat - template on left side only
/// `flex p-4 ` + b     // trailing space preserved (for `+ b`)
///          ^
/// ```
pub fn can_collapse_whitespace<'a, 'b>(
    span: Span,
    ancestors: impl Iterator<Item = &'b AstNodes<'a>>,
    f: &Formatter<'_, 'a>,
) -> CollapseWhitespace
where
    'a: 'b,
{
    let mut collapse = CollapseWhitespace::new();

    // 1. Check template expression context (O(1) via context stack)
    if let Some(ctx) = f.context().tailwind_context()
        && ctx.in_template_expression
    {
        if !ctx.quasi_before_has_trailing_ws {
            collapse.start = false;
        }
        if !ctx.quasi_after_has_leading_ws {
            collapse.end = false;
        }
    }

    // 2. Check binary concat context (walk parent chain)
    for ancestor in ancestors {
        let AstNodes::BinaryExpression(binary) = ancestor else {
            break;
        };

        if binary.operator() != BinaryOperator::Addition {
            break;
        }

        let left = binary.left().span();
        let right = binary.right().span();

        // Left operand needs trailing space for separation from `+ right`
        if left.contains_inclusive(span) {
            collapse.end = false;
        }
        // Right operand needs leading space for separation from `left +`
        if right.contains_inclusive(span) {
            collapse.start = false;
        }
    }

    collapse
}

// ============================================================================
// Write Functions
// ============================================================================

/// Writes a string literal with Tailwind class sorting.
///
/// Handles whitespace based on context:
/// - Trims and normalizes whitespace by default
/// - Preserves boundary spaces when required by concat/template context
/// - With `preserve_whitespace`, outputs content unchanged
pub fn write_tailwind_string_literal<'a>(
    string_literal: &AstNode<'a, StringLiteral<'a>>,
    ctx: TailwindContextEntry,
    f: &mut Formatter<'_, 'a>,
) {
    debug_assert!(
        !string_literal.value.is_empty(),
        "Empty string literals should be skipped for Tailwind sorting"
    );

    let normalized_string = FormatLiteralStringToken::new(
        f.source_text().text_for(&string_literal),
        // `className="string"`
        //            ^^^^^^^^
        matches!(string_literal.parent(), AstNodes::JSXAttribute(_)),
        StringLiteralParentKind::Expression,
    )
    .clean_text(f);

    let quote = normalized_string.as_bytes()[0];
    let quote = match quote {
        b'\'' => "\'",
        b'"' => "\"",
        _ => unreachable!("Unexpected quote character in string literal"),
    };

    write!(f, quote);

    // At least three characters: opening quote, content, closing quote
    let content = &normalized_string[1..normalized_string.len() - 1];

    if ctx.preserve_whitespace {
        let index = f.context_mut().add_tailwind_class(content.to_string());
        f.write_element(FormatElement::TailwindClass(index));
        write!(f, quote);
        return;
    }

    let trimmed = content.trim();

    // Whitespace-only → normalize to single space
    if trimmed.is_empty() {
        if !content.is_empty() {
            write!(f, text(" "));
        }
        write!(f, quote);
        return;
    }

    let collapse = can_collapse_whitespace(string_literal.span, string_literal.ancestors(), f);
    let has_leading_ws = content.starts_with(|c: char| c.is_ascii_whitespace());
    let has_trailing_ws = content.ends_with(|c: char| c.is_ascii_whitespace());

    // Leading space
    if has_leading_ws && !collapse.start {
        write!(f, text(" "));
    }

    // Sorted content
    let index = f.context_mut().add_tailwind_class(trimmed.to_string());
    f.write_element(FormatElement::TailwindClass(index));

    // Trailing space
    if has_trailing_ws && !collapse.end {
        write!(f, text(" "));
    }
    write!(f, quote);
}

/// Writes a template element (quasi) with Tailwind class sorting.
///
/// Template elements need special handling because classes can "touch"
/// adjacent expressions without whitespace separation:
///
/// ```text
/// `${variant}items-center p-4`
/// //         ^^^^^^^^^^^^ "items-center" touches ${variant}, not sorted
/// //                      ^^^ "p-4" is separated by space, will be sorted
/// ```
///
/// # Content Structure
///
/// Content is split into three parts:
/// - **prefix**: Class touching previous expression (not sorted)
/// - **sortable**: Classes separated by whitespace (sorted)
/// - **suffix**: Class touching next expression (not sorted)
///
/// Based on [prettier-plugin-tailwindcss](https://github.com/tailwindlabs/prettier-plugin-tailwindcss/blob/28beb4e008b913414562addec4abb8ab261f3828/src/index.ts#L511-L566).
pub fn write_tailwind_template_element<'a>(
    element: &AstNode<'a, TemplateElement<'a>>,
    ctx: TailwindContextEntry,
    f: &mut Formatter<'_, 'a>,
) {
    let content = f.source_text().text_for(element);

    if ctx.preserve_whitespace {
        let index = f.context_mut().add_tailwind_class(content.to_string());
        f.write_element(FormatElement::TailwindClass(index));
        return;
    }

    // Get quasi position from context (set when the quasi was written in template.rs)
    let is_first = ctx.is_first_quasi;
    let is_last = ctx.is_last_quasi;

    // Check if binary expression context requires preserving boundary whitespace
    let collapse = can_collapse_whitespace_template(element, is_first, is_last, f);

    // Split into prefix/sortable/suffix
    let (prefix, sortable, suffix) = split_template_content(content, is_first, is_last);

    // Write prefix (unsorted class touching previous expression)
    if !prefix.is_empty() {
        write!(f, text(prefix));
    }

    // Write sortable content
    let trimmed = sortable.trim();

    if trimmed.is_empty() {
        // Whitespace-only → normalize to single space
        if !sortable.is_empty() {
            write!(f, text(" "));
        }
    } else {
        let has_leading_ws = sortable.starts_with(|c: char| c.is_ascii_whitespace());
        let has_trailing_ws = sortable.ends_with(|c: char| c.is_ascii_whitespace());

        // Leading space: required if not at start of template, or if binary context requires it
        let need_leading = !is_first || !prefix.is_empty() || (has_leading_ws && !collapse.start);
        if need_leading {
            write!(f, text(" "));
        }

        let index = f.context_mut().add_tailwind_class(trimmed.to_string());
        f.write_element(FormatElement::TailwindClass(index));

        // Trailing space: required if not at end of template, or if binary context requires it
        let need_trailing = !is_last || !suffix.is_empty() || (has_trailing_ws && !collapse.end);
        if need_trailing {
            write!(f, text(" "));
        }
    }

    // Write suffix (unsorted class touching next expression)
    if !suffix.is_empty() {
        write!(f, text(suffix));
    }
}

/// Determines whitespace collapse rules for a template element based on binary expression context.
fn can_collapse_whitespace_template<'a>(
    element: &AstNode<'a, TemplateElement<'a>>,
    is_first: bool,
    is_last: bool,
    f: &Formatter<'_, 'a>,
) -> CollapseWhitespace {
    // Only first/last quasis can be affected by binary expression context
    if !is_first && !is_last {
        return CollapseWhitespace::new();
    }

    can_collapse_whitespace(element.span, element.ancestors().skip(1), f)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Splits template content into (prefix, sortable, suffix).
///
/// - **prefix**: First class if it touches previous expression (no leading whitespace)
/// - **suffix**: Last class if it touches next expression (no trailing whitespace)
/// - **sortable**: Everything in between
///
/// # Examples
///
/// ```text
/// // Input: "items-center p-4 m-2", is_first=false, is_last=true
/// // "items-center" touches prev expr → prefix
/// // "p-4 m-2" will be sorted → sortable
/// // No suffix (is_last=true)
/// Result: ("items-center", " p-4 m-2", "")
///
/// // Input: " flex p-4", is_first=true, is_last=true
/// // Leading space → no prefix
/// // Everything is sortable
/// Result: ("", " flex p-4", "")
/// ```
fn split_template_content(content: &str, is_first: bool, is_last: bool) -> (&str, &str, &str) {
    let has_leading_ws = content.starts_with(|c: char| c.is_ascii_whitespace());
    let has_trailing_ws = content.ends_with(|c: char| c.is_ascii_whitespace());

    // Determine what to ignore (not sort)
    let has_prefix = !is_first && !has_leading_ws;
    let has_suffix = !is_last && !has_trailing_ws;

    // Find split points
    let prefix_end =
        if has_prefix { content.find(|c: char| c.is_ascii_whitespace()) } else { None };
    let suffix_start = if has_suffix {
        content.rfind(|c: char| c.is_ascii_whitespace()).map(|i| i + 1)
    } else {
        None
    };

    match (prefix_end, suffix_start) {
        // Both prefix and suffix
        (Some(pe), Some(ss)) if pe < ss => (&content[..pe], &content[pe..ss], &content[ss..]),
        // Only prefix (suffix_start overlaps or doesn't exist)
        (Some(pe), _) => (&content[..pe], &content[pe..], ""),
        // Only suffix
        (None, Some(ss)) => ("", &content[..ss], &content[ss..]),
        // Neither
        (None, None) => ("", content, ""),
    }
}
