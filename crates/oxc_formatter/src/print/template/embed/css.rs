use oxc_allocator::ArenaStringBuilder;
use oxc_ast::ast::*;
use oxc_formatter_core::IndentWidth;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::AstNode,
    format_args,
    formatter::{FormatElement, format_element::TextWidth, prelude::*},
    write,
};

// css-in-js interpolation marker `` `PLACEHOLDER-N` ``.
// Backtick is invalid SCSS (the variant the dispatcher formats as),
// so the marker is unmistakably out-of-band.
// NOTE: Keep these in sync with `oxc_formatter_css`'s.
// Or pass it from the dispatcher?
const PLACEHOLDER_PREFIX: &str = "`PLACEHOLDER-";
const PLACEHOLDER_SUFFIX: &str = "`";

/// Re-emit a (already arena-backed) text slice as a `Text` element.
/// No-op for an empty slice.
fn write_text_piece<'a>(text: &'a str, indent_width: IndentWidth, f: &mut JsFormatter<'_, 'a>) {
    if text.is_empty() {
        return;
    }
    let width = TextWidth::from_text(text, indent_width);
    f.write_element(FormatElement::Text { text, width });
}

/// Format a CSS-in-JS template literal via the Doc→IR path with placeholder replacement.
///
/// Joins quasis with special placeholder markers, formats as SCSS,
/// then replaces placeholder occurrences in the resulting IR with `${expr}` Docs.
pub(super) fn format_css_doc<'a>(
    quasi: &AstNode<'a, TemplateLiteral<'a>>,
    f: &mut JsFormatter<'_, 'a>,
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
        let Some(Ok(mut result)) = f.context().external_callbacks().dispatch_embedded(
            allocator,
            group_id_builder,
            "css",
            &[raw],
        ) else {
            return false;
        };
        result.remap_tailwind_into(f.context_mut());
        let Some(ir) = result.docs.into_iter().next() else {
            return false;
        };

        write!(f, ["`", block_indent(&format_once(|f| f.write_elements(ir))), "`"]);
        return true;
    }

    // Phase 1: Build joined text
    // quasis[0].raw + "`PLACEHOLDER-0`" + quasis[1].raw + ...
    let allocator = f.allocator();
    let joined = {
        let mut sb = ArenaStringBuilder::new_in(allocator);
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

    // Phase 2: Format via the dispatcher (IR path)
    let allocator = f.allocator();
    let group_id_builder = f.group_id_builder();
    let Some(Ok(mut result)) = f.context().external_callbacks().dispatch_embedded(
        allocator,
        group_id_builder,
        "css",
        &[joined],
    ) else {
        return false;
    };
    result.remap_tailwind_into(f.context_mut());
    let Some(ir) = result.docs.into_iter().next() else {
        return false;
    };

    // Verify all placeholders survived SCSS formatting.
    // Some edge cases (e.g. `/* prettier-ignore */` before a placeholder without semicolon)
    // cause SCSS to drop placeholders.
    // In that case, fall back to regular template formatting
    // (same behavior as Prettier's `replacePlaceholders()` returning `null`).
    // https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/embed/css.js#L42
    //
    // Surviving placeholders come in two forms:
    // - typed `EmbedPlaceholder` markers (the main path)
    // - sentinels that stayed inside verbatim `Text` because a lexical context
    //   (string / `url()`) keeps them opaque to the CSS lexer
    // The Phase 3 substitution below handles both kinds, so we count both here.
    let placeholder_count: usize = ir
        .iter()
        .map(|el| match el {
            FormatElement::EmbedPlaceholder(_) => 1,
            FormatElement::Text { text, .. } => {
                super::count_placeholders(text, PLACEHOLDER_PREFIX, PLACEHOLDER_SUFFIX)
            }
            _ => 0,
        })
        .sum();
    if placeholder_count != expressions.len() {
        return false;
    }

    // Phase 3: Replace each `${exprN}` placeholder with the formatted expression.
    // Two kinds survive SCSS formatting:
    // - the typed `EmbedPlaceholder(N)` marker (the main path) -> a breakable group
    // - a `` `PLACEHOLDER-N` `` sentinel still embedded in a `Text` run
    //   because a string / `url()` keeps it opaque to the CSS lexer -> inline.
    let format_content = format_once(move |f: &mut JsFormatter<'_, 'a>| {
        let indent_width = f.options().indent_width;
        for element in ir {
            match element {
                FormatElement::EmbedPlaceholder(index) => {
                    let Some(&expr) = expressions.get(index as usize) else {
                        continue;
                    };
                    // Prettier's `printTemplateExpression()` adds indent+softline when:
                    // - the original source has newlines in the interpolation
                    // - AND the expression is a comment-bearing node or Identifier/etc
                    // For CSS embed, the relevant case is comments inside `${...}`.
                    let has_newline = f.source_text().has_line_terminator_before(expr.span().start)
                        || f.source_text().has_line_terminator_after(expr.span().end);
                    let has_comment = has_newline && {
                        let comments = f.context().comments();
                        let leading = comments.comments_before(expr.span().start);
                        // Scan from the expression's END so a `}` inside its own source
                        // (string, object, nested template) doesn't cut the lookup short.
                        let trailing = comments.comments_before_character(expr.span().end, b'}');
                        !leading.is_empty() || !trailing.is_empty()
                    };

                    let format_expr = format_with(|f| {
                        if has_comment {
                            write!(
                                f,
                                [
                                    indent(&format_args!(
                                        soft_line_break(),
                                        expr,
                                        line_suffix_boundary()
                                    )),
                                    soft_line_break()
                                ]
                            );
                        } else {
                            write!(f, [expr, line_suffix_boundary()]);
                        }
                    });
                    write!(f, [group(&format_args!("${", format_expr, "}"))]);
                }
                // A sentinel inside a string / `url()` is always inline `${expr}`.
                // Same scan as html.rs: `split_on_placeholders` yields alternating
                // [literal, index, literal, ...] (invalid matches fold into the literal),
                // so even parts are text and odd parts are a placeholder.
                FormatElement::Text { text, .. } if text.contains(PLACEHOLDER_PREFIX) => {
                    let parts =
                        super::split_on_placeholders(text, PLACEHOLDER_PREFIX, PLACEHOLDER_SUFFIX);
                    for (i, part) in parts.iter().enumerate() {
                        if i % 2 == 0 {
                            write_text_piece(part, indent_width, f);
                        } else if let Ok(idx) = part.parse::<usize>()
                            && let Some(&expr) = expressions.get(idx)
                        {
                            write!(f, ["${", expr, "}"]);
                        }
                    }
                }
                _ => f.write_element(element),
            }
        }
    });

    write!(f, ["`", block_indent(&format_content), "`"]);
    true
}
