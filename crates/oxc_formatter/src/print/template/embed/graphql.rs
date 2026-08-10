use oxc_allocator::{Allocator, ArenaVec};
use oxc_ast::ast::*;
use oxc_formatter_core::{
    FormatElement, IndentWidth, dispatch_fragment_ir,
    format_element::{LineMode, TextWidth},
};

use crate::{
    ast_nodes::AstNode,
    formatter::prelude::*,
    print::template::{
        FormatTemplateExpression, FormatTemplateExpressionOptions, TemplateExpression,
    },
    write,
};

/// Format a GraphQL template literal via the Doc→IR path.
///
/// Handles both no-substitution and `${}` templates uniformly.
/// Called from both:
/// - tagged template (gql`...`)
/// - and function call (`graphql(schema, `...`)`)
pub(super) fn format_graphql_doc<'a>(
    quasi: &AstNode<'a, TemplateLiteral<'a>>,
    f: &mut JsFormatter<'_, 'a>,
) -> bool {
    let quasis = &quasi.quasis;
    let num_quasis = quasis.len();

    // Phase 1: Analyze each quasi
    let mut infos: Vec<QuasiInfo<'a>> = Vec::with_capacity(num_quasis);
    for (idx, quasi_elem) in quasis.iter().enumerate() {
        // Use `.cooked` value instead of `.raw` like Prettier.
        // Bail out if cooked is `None` (e.g. invalid escape sequence)
        let Some(cooked) = quasi_elem.value.cooked.as_ref() else {
            return false;
        };
        let text = cooked.as_str();
        // `.cooked` has normalized line terminators
        let lines: Vec<&str> = text.split('\n').collect();

        // Bail out if interpolation occurs within a GraphQL comment.
        // https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/embed/graphql.js#L37-L40
        // Must use `lines.last()` (from split), not `text.lines().next_back()`.
        // Because `lines()` strips trailing empty lines, causing false positives when text ends with `\n`
        // (e.g. `"\n# comment\n"`).
        if idx != num_quasis - 1
            && let Some(last_line) = lines.last()
            && last_line.contains('#')
        {
            return false;
        }

        let num_lines = lines.len();
        // Detect blank lines around expressions.
        // https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/embed/graphql.js#L26-L31
        let starts_with_blank_line =
            num_lines > 2 && lines[0].trim().is_empty() && lines[1].trim().is_empty();
        let ends_with_blank_line = num_lines > 2
            && lines[num_lines - 1].trim().is_empty()
            && lines[num_lines - 2].trim().is_empty();

        // Check if every line in the text is whitespace-only or a GraphQL comment (`# ...`).
        // https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/embed/graphql.js#L33-L35
        let comments_only = lines.iter().all(|line| {
            let trimmed = line.trim();
            trimmed.is_empty() || trimmed.starts_with('#')
        });

        infos.push(QuasiInfo { text, comments_only, starts_with_blank_line, ends_with_blank_line });
    }

    // Phase 2+3: Build `ir_parts`, one dispatch per non-comment quas.
    // (any failure keeps the whole template verbatim:
    // the quasis are one interleaved template, so degradation is all-or-nothing.
    // Discarding docs already consumed via `into_doc` is safe, see its rustdoc)
    // Comment-only quasis are synthesized locally.
    // The IR is re-inserted into a JS template literal built from `.cooked` values,
    // so template-literal characters (`` ` ``, `${`, `\`) are re-escaped here
    // for both dispatcher returned IRs and manually built comment-only IRs.
    let allocator = f.allocator();
    let indent_width = f.options().indent_width;
    let mut ir_parts: Vec<Option<ArenaVec<'a, FormatElement<'a>>>> = Vec::with_capacity(num_quasis);
    for info in &infos {
        let ir = if info.comments_only {
            build_graphql_comment_ir(info.text, allocator, indent_width)
        } else {
            let Some(ir) = dispatch_fragment_ir(f, "graphql", info.text, None) else {
                return false;
            };
            Some(ir)
        };
        let ir = ir.map(|ir| super::escape_template_chars_in_ir(&ir, f));
        ir_parts.push(ir);
    }

    // Collect expressions via AstNode-aware iterator
    // (`FormatTemplateExpression` needs AstNode-wrapped expressions)
    let expressions: Vec<_> = quasi.expressions().iter().collect();

    // Early return for empty/whitespace-only templates with no expressions.
    // Do not use `block_indent()`, it requires at least one element.
    if expressions.is_empty() && ir_parts.iter().all(Option::is_none) {
        write!(f, ["``"]);
        return true;
    }

    // Phase 4: Write the template structure
    // `["`", indent([hardline, join(hardline, parts)]), hardline, "`"]`
    // https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/embed/graphql.js#L68C10-L68C73
    let format_content = format_once(|f: &mut JsFormatter<'_, 'a>| {
        let mut has_prev_part = false;

        for (idx, mut maybe_ir) in ir_parts.into_iter().enumerate() {
            let is_first = idx == 0;
            let is_last = idx == num_quasis - 1;

            if let Some(ir) = maybe_ir.take() {
                if !is_first && infos[idx].starts_with_blank_line {
                    if has_prev_part {
                        write!(f, [empty_line()]);
                    }
                } else if has_prev_part {
                    write!(f, [hard_line_break()]);
                }
                f.write_elements(ir);
                has_prev_part = true;
            } else if !is_first && !is_last && infos[idx].starts_with_blank_line && has_prev_part {
                write!(f, [empty_line()]);
            }

            if !is_last {
                if infos[idx].ends_with_blank_line && has_prev_part {
                    write!(f, [empty_line()]);
                    has_prev_part = false; // Next part won't add another separator
                }

                if let Some(expr) = expressions.get(idx) {
                    if has_prev_part {
                        write!(f, [hard_line_break()]);
                    }
                    let te = TemplateExpression::Expression(expr);
                    FormatTemplateExpression::new(&te, FormatTemplateExpressionOptions::default())
                        .fmt(f);
                    has_prev_part = true;
                }
            }
        }
    });

    write!(f, ["`", block_indent(&format_content), "`"]);
    true
}

/// Per-quasi metadata extracted during the analysis phase.
struct QuasiInfo<'a> {
    /// The cooked text of the quasi (always `Some` — `None` causes early bail-out).
    text: &'a str,
    /// Whether the quasi contains only whitespace and/or GraphQL comments.
    comments_only: bool,
    /// Blank line at the beginning of this quasi.
    starts_with_blank_line: bool,
    /// Blank line at the end of this quasi.
    ends_with_blank_line: bool,
}

/// Build IR for a comment-only quasi
/// <https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/embed/graphql.js#L71>
///
/// Extracts comment lines, joins with hardline, and preserves blank lines between comment groups.
fn build_graphql_comment_ir<'a>(
    text: &str,
    allocator: &'a Allocator,
    indent_width: IndentWidth,
) -> Option<ArenaVec<'a, FormatElement<'a>>> {
    // This comes from `.cooked`, which has normalized line terminators
    let lines: Vec<&str> = text.split('\n').map(str::trim).collect();
    let mut parts: ArenaVec<'a, FormatElement<'a>> = ArenaVec::new_in(&allocator);
    let mut seen_comment = false;

    for (i, line) in lines.iter().enumerate() {
        if line.is_empty() {
            continue;
        }

        if i > 0 && lines[i - 1].is_empty() && seen_comment {
            // Blank line before this comment group -> emit empty line + text
            parts.push(FormatElement::Line(LineMode::Empty));
            parts.push(FormatElement::ExpandParent);
            let arena_text = allocator.alloc_str(line);
            let width = TextWidth::from_text(arena_text, indent_width);
            parts.push(FormatElement::Text { text: arena_text, width });
        } else {
            if seen_comment {
                parts.push(FormatElement::Line(LineMode::Hard));
                parts.push(FormatElement::ExpandParent);
            }
            let arena_text = allocator.alloc_str(line);
            let width = TextWidth::from_text(arena_text, indent_width);
            parts.push(FormatElement::Text { text: arena_text, width });
        }

        seen_comment = true;
    }

    if parts.is_empty() { None } else { Some(parts) }
}
