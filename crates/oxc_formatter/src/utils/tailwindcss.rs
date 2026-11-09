//! Tailwind CSS class sorting utilities.
//!
//! This module provides utilities for sorting Tailwind CSS classes within:
//! - JSX attributes (`className`, `class`, or custom attributes)
//! - Function calls (`clsx()`, `cn()`, `tw()`, or custom functions)
//! - Template literals with expressions
//!
//! Based on [prettier-plugin-tailwindcss](https://github.com/tailwindlabs/prettier-plugin-tailwindcss).

use unicode_width::UnicodeWidthStr;

use oxc_ast::ast::*;
use oxc_formatter_core::FormatElement;
use oxc_span::GetSpan;

use crate::{
    SortTailwindcssOptions,
    ast_nodes::{AstNode, AstNodes},
    formatter::{TailwindContextEntry, prelude::*},
    write,
};

use super::string::{FormatLiteralStringToken, StringLiteralParentKind};

// ============================================================================
// Detection Functions
// ============================================================================

/// Returns the Tailwind context if we should sort classes in the given string literal or template element.
///
/// Returns `Some(ctx)` when:
/// - We're inside a Tailwind class context (see [`JsFormatContext::tailwind_context`])
/// - The source contains whitespace (no whitespace means only one class, so no need to sort)
pub fn tailwind_context_for_class_list(
    node: &impl GetSpan,
    f: &JsFormatter<'_, '_>,
) -> Option<TailwindContextEntry> {
    f.context()
        .tailwind_context()
        .copied()
        .filter(|_| f.source_text().text_for(node).bytes().any(|b| b.is_ascii_whitespace()))
}

/// Checks if a JSX attribute is a Tailwind class attribute.
///
/// Returns `true` for:
/// - `class` and `className` (default attributes)
/// - Custom attributes specified in `attributes` option
fn is_tailwind_jsx_attribute(
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

/// Returns the Tailwind context to push while printing the value of the JSX attribute `attr_name`.
///
/// Also read by the opening element layout, which must agree with the print on whether the value is collapsed.
pub fn tailwind_context_for_jsx_attribute(
    attr_name: &JSXAttributeName<'_>,
    f: &JsFormatter<'_, '_>,
) -> Option<TailwindContextEntry> {
    f.options()
        .sort_tailwindcss
        .as_ref()
        .filter(|opts| is_tailwind_jsx_attribute(attr_name, opts))
        .map(|opts| TailwindContextEntry::new(opts.preserve_whitespace))
}

/// Whether a template literal printed at this point (under `tag` when tagged) has its line breaks collapsed.
/// - Reads what `TemplateElement` reads: the current Tailwind context, or the one a Tailwind `tag` pushes
pub fn tailwind_collapses_template_line_breaks(
    tag: Option<&Expression<'_>>,
    f: &JsFormatter<'_, '_>,
) -> bool {
    f.context()
        .tailwind_context()
        .copied()
        .or_else(|| tag.and_then(|tag| tailwind_context_for_callee(tag, f)))
        .is_some_and(|ctx| !ctx.preserve_whitespace)
}

/// Returns the Tailwind context to push while printing the arguments of a call to `callee`,
/// or the quasi of a template tagged with it.
///
/// Also read by layout decisions, which must agree with the print on whether the classes are collapsed.
pub fn tailwind_context_for_callee(
    callee: &Expression<'_>,
    f: &JsFormatter<'_, '_>,
) -> Option<TailwindContextEntry> {
    f.options()
        .sort_tailwindcss
        .as_ref()
        .filter(|opts| is_tailwind_function_call(callee, opts))
        .map(|opts| TailwindContextEntry::new(opts.preserve_whitespace))
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
fn is_tailwind_function_call(callee: &Expression<'_>, options: &SortTailwindcssOptions) -> bool {
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

/// Printed width of a class list with collapsed whitespace:
/// trimmed, then split on ASCII whitespace and joined with single spaces, as the sorter does.
/// - Whitespace-only content is printed as a single space
/// - An upper bound: the sorter also drops duplicates of known classes
pub fn collapsed_class_list_width(content: &str) -> usize {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return usize::from(!content.is_empty());
    }
    // Each class plus its separator, minus the last separator
    trimmed.split_ascii_whitespace().map(|class| class.width() + 1).sum::<usize>() - 1
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
    ancestors: impl Iterator<Item = &'b AstNodes<'b, 'a>>,
    f: &JsFormatter<'_, 'a>,
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
        match ancestor {
            AstNodes::BinaryExpression(binary) if binary.operator() == BinaryOperator::Addition => {
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

                // Both flags are one-way latches; no need to continue once both are set.
                if !collapse.start && !collapse.end {
                    break;
                }
            }
            // Transparent nodes: skip through to find outer BinaryExpression(+)
            AstNodes::ConditionalExpression(_)
            | AstNodes::ParenthesizedExpression(_)
            | AstNodes::TSAsExpression(_)
            | AstNodes::TSSatisfiesExpression(_)
            | AstNodes::TSNonNullExpression(_)
            | AstNodes::TSTypeAssertion(_) => {}
            _ => break,
        }
    }

    collapse
}

// ============================================================================
// Write Functions
// ============================================================================

/// Writes a string literal with Tailwind class sorting, and returns its printed width.
///
/// Handles whitespace based on context:
/// - Trims and normalizes whitespace by default
/// - Preserves boundary spaces when required by concat/template context
/// - With `preserve_whitespace`, outputs content unchanged
///
/// The width is an upper bound: the sorter also drops duplicates of known classes,
/// so a layout decided near a threshold can still change on the next pass.
pub fn write_tailwind_string_literal<'a>(
    string_literal: &AstNode<'_, 'a, StringLiteral<'a>>,
    ctx: TailwindContextEntry,
    f: &mut JsFormatter<'_, 'a>,
) -> usize {
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
        let index = f.session().add_tailwind_class(content.to_string());
        f.write_element(FormatElement::TailwindClass(index));
        write!(f, quote);
        return normalized_string.width();
    }

    let trimmed = content.trim();

    // Whitespace-only → normalize to single space
    if trimmed.is_empty() {
        if !content.is_empty() {
            write!(f, text(" "));
        }
        write!(f, quote);
        // Quotes, and the single space the content is normalized to
        return 2 + collapsed_class_list_width(content);
    }

    let (keeps_leading_space, keeps_trailing_space) =
        kept_boundary_spaces(content, string_literal.span, string_literal.ancestors(), f);

    // Leading space
    if keeps_leading_space {
        write!(f, text(" "));
    }

    // Sorted content
    let index = f.session().add_tailwind_class(trimmed.to_string());
    f.write_element(FormatElement::TailwindClass(index));

    // Trailing space
    if keeps_trailing_space {
        write!(f, text(" "));
    }
    write!(f, quote);

    // Quotes, the kept boundary spaces, and the classes
    2 + usize::from(keeps_leading_space)
        + collapsed_class_list_width(trimmed)
        + usize::from(keeps_trailing_space)
}

/// Whether a class list keeps its leading and trailing whitespace, as one space each,
/// where it can't collapse against its surroundings (see [`can_collapse_whitespace`]).
fn kept_boundary_spaces<'a, 'b>(
    content: &str,
    span: Span,
    ancestors: impl Iterator<Item = &'b AstNodes<'b, 'a>>,
    f: &JsFormatter<'_, 'a>,
) -> (bool, bool)
where
    'a: 'b,
{
    let collapse = can_collapse_whitespace(span, ancestors, f);
    (
        content.starts_with(|c: char| c.is_ascii_whitespace()) && !collapse.start,
        content.ends_with(|c: char| c.is_ascii_whitespace()) && !collapse.end,
    )
}

/// Returns the class a string key is printed as when sorted in `ctx`,
/// if that is a single class with no boundary space kept, e.g. `p` for `" p "`.
/// - Its output is known without the sorter, so the key is printed as a plain key with it, not sent to the sorter,
///   and its quotes are decided as Prettier decides them on the plugin's output
/// - Duplicates of a known class (`"flex flex"`) also print one, but only the sorter knows them
pub fn tailwind_single_class_key<'a>(
    key: &StringLiteral<'_>,
    ctx: TailwindContextEntry,
    f: &JsFormatter<'_, 'a>,
) -> Option<&'a str> {
    if ctx.preserve_whitespace {
        return None;
    }
    let text = f.source_text().text_for(key);
    let content = &text[1..text.len() - 1];
    let mut classes = content.trim().split_ascii_whitespace();
    let class = classes.next().filter(|_| classes.next().is_none())?;
    // A key is never a `+` operand, so only a template expression context can keep a boundary space
    let (keeps_leading_space, keeps_trailing_space) =
        kept_boundary_spaces(content, key.span, std::iter::empty(), f);
    (!keeps_leading_space && !keeps_trailing_space).then_some(class)
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
    element: &AstNode<'_, 'a, TemplateElement<'a>>,
    ctx: TailwindContextEntry,
    f: &mut JsFormatter<'_, 'a>,
) {
    // Not the source text, `raw` has its line endings normalized to `\n`
    let content = element.value.raw.as_str();

    // Get quasi position from context (set when the quasi was written in template.rs)
    let is_first = ctx.is_first_quasi;
    let is_last = ctx.is_last_quasi;

    // Split into prefix/sortable/suffix.
    // Classes glued to an adjacent `${...}` expression must stay in place even when preserving whitespace;
    let (prefix, sortable, suffix) = split_template_content(content, is_first, is_last);

    // Write prefix (unsorted class touching previous expression)
    if !prefix.is_empty() {
        write!(f, text(prefix));
    }

    // Write sortable content
    if ctx.preserve_whitespace {
        // Whitespace (including whitespace-only content) round-trips through the sorter unchanged
        let index = f.session().add_tailwind_class(sortable.to_string());
        f.write_element(FormatElement::TailwindClass(index));
    } else if sortable.trim().is_empty() {
        // Whitespace-only → normalize to single space
        if !sortable.is_empty() {
            write!(f, text(" "));
        }
    } else {
        // Check if binary expression context requires preserving boundary whitespace
        let collapse = can_collapse_whitespace_template(element, is_first, is_last, f);
        let has_leading_ws = sortable.starts_with(|c: char| c.is_ascii_whitespace());
        let has_trailing_ws = sortable.ends_with(|c: char| c.is_ascii_whitespace());

        // Leading space: required if not at start of template, or if binary context requires it
        let need_leading = !is_first || !prefix.is_empty() || (has_leading_ws && !collapse.start);
        if need_leading {
            write!(f, text(" "));
        }

        let index = f.session().add_tailwind_class(sortable.trim().to_string());
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
    element: &AstNode<'_, 'a, TemplateElement<'a>>,
    is_first: bool,
    is_last: bool,
    f: &JsFormatter<'_, 'a>,
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
