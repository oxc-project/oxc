use oxc_allocator::{ArenaStringBuilder, ArenaVec};
use oxc_ast::ast::*;
use oxc_formatter_core::{
    DispatchRequest, DispatchResponse, FormatElement, IndentWidth, InputKind,
    format_element::{LineMode, TextWidth},
};

use crate::{
    ast_nodes::AstNode,
    embed_context::HtmlEmbedMeta,
    format_args,
    formatter::prelude::*,
    print::template::{
        FormatTemplateExpression, FormatTemplateExpressionOptions, TemplateExpression,
    },
    write,
};

// Prettier uses `PRETTIER_HTML_PLACEHOLDER_{index}_{counter}_IN_JS` as placeholders.
// We use a fixed counter of 0 since we don't have nested embeds.
const PLACEHOLDER_PREFIX: &str = "PRETTIER_HTML_PLACEHOLDER_";
const PLACEHOLDER_SUFFIX: &str = "_IN_JS";
const COUNTER: &str = "0";

/// Format an HTML(Angular)-in-JS template literal via the Doc->IR path with placeholder replacement.
///
/// Uses `.cooked` values (unlike CSS which uses `.raw`), joins quasis with
/// `PRETTIER_HTML_PLACEHOLDER_{N}_0_IN_JS` markers, formats via the given `embedded_language`,
/// then replaces placeholder occurrences in the resulting IR with `${expr}` Docs.
///
/// Supports both html-in-js and angular-in-js (`@Component({ template })`).
pub(super) fn format_html_doc<'a>(
    quasi: &AstNode<'a, TemplateLiteral<'a>>,
    f: &mut JsFormatter<'_, 'a>,
    is_angular: bool,
) -> bool {
    let embedded_language = if is_angular { "angular" } else { "html" };
    let quasis = &quasi.quasis;
    let expressions: Vec<_> = quasi.expressions().iter().collect();

    // Phase 0: No expressions
    if expressions.is_empty() {
        let Some(cooked) = quasis[0].value.cooked.as_ref() else {
            return false;
        };
        let cooked = cooked.as_str();

        if cooked.trim().is_empty() {
            write!(f, ["``"]);
            return true;
        }

        let has_leading_ws = cooked.starts_with(|c: char| c.is_ascii_whitespace());
        let has_trailing_ws = cooked.ends_with(|c: char| c.is_ascii_whitespace());

        let Ok(DispatchResponse::Formatted(result)) = f.session().dispatch(DispatchRequest {
            language: embedded_language,
            text: cooked,
            input_kind: InputKind::Fragment,
            parent_context: None,
        }) else {
            return false;
        };
        let Some(html_has_multiple_root_elements) = result
            .child_context
            .as_ref()
            .and_then(|context| context.downcast_ref::<HtmlEmbedMeta>())
            .map(|meta| meta.has_multiple_root_elements)
        else {
            return false;
        };
        // Remap is a no-op today (the Prettier Doc path never carries classes),
        // but the boundary contract is "merge at every embed site".
        // A Rust HTML formatter collecting `class` attributes will rely on this.
        let ir = result.into_doc(f.context_mut());

        // Re-escape template chars in `Text` runs:
        // the IR is reinserted into a JS template literal built from `.cooked` values.
        let ir = super::escape_template_chars_in_ir(&ir, f);

        let content = format_once(|f| f.write_elements(ir));
        let ws_ignore = f.options().html_whitespace_sensitivity_ignore;
        write_html_template(
            f,
            &content,
            has_leading_ws,
            has_trailing_ws,
            html_has_multiple_root_elements.unwrap_or(true),
            ws_ignore,
        );
        return true;
    }

    // Phase 1: Build joined text using .cooked with HTML placeholders
    // quasis[0].cooked + "PRETTIER_HTML_PLACEHOLDER_0_0_IN_JS" + quasis[1].cooked + ...
    let allocator = f.allocator();
    let joined = {
        let mut sb = ArenaStringBuilder::new_in(allocator);
        for (idx, quasi_elem) in quasis.iter().enumerate() {
            if idx > 0 {
                sb.push_str(PLACEHOLDER_PREFIX);
                let _ = std::fmt::Write::write_fmt(&mut sb, std::format_args!("{}", idx - 1));
                sb.push('_');
                sb.push_str(COUNTER);
                sb.push_str(PLACEHOLDER_SUFFIX);
            }
            let Some(cooked) = quasi_elem.value.cooked.as_ref() else {
                return false;
            };
            sb.push_str(cooked.as_str());
        }
        sb.into_str()
    };

    let has_leading_ws = joined.starts_with(|c: char| c.is_ascii_whitespace());
    let has_trailing_ws = joined.ends_with(|c: char| c.is_ascii_whitespace());

    // Phase 2: Format via the dispatcher (IR path)
    let Ok(DispatchResponse::Formatted(result)) = f.session().dispatch(DispatchRequest {
        language: embedded_language,
        text: joined,
        input_kind: InputKind::Fragment,
        parent_context: None,
    }) else {
        return false;
    };

    let Some(html_has_multiple_root_elements) = result
        .child_context
        .as_ref()
        .and_then(|context| context.downcast_ref::<HtmlEmbedMeta>())
        .map(|meta| meta.has_multiple_root_elements)
    else {
        return false;
    };
    // See the Phase 0 note: remap is no-op today, load-bearing once `oxc_formatter_html` lands
    let ir = result.into_doc(f.context_mut());

    // Validate before formatting any expression.
    // Formatting consumes comment state,
    // so discovering an unusable placeholder layout afterwards would make the verbatim fallback unsafe.
    let mut next_placeholder = 0;
    if !placeholders_are_sequential(&ir, &mut next_placeholder)
        || next_placeholder != expressions.len()
    {
        return false;
    }

    // Format every expression exactly once.
    // These elements are cloned into all `BestFitting` variants.
    let mut formatted_expressions = Vec::with_capacity(expressions.len());
    for expr in expressions {
        let te = TemplateExpression::Expression(expr);
        let element = f
            .intern(&FormatTemplateExpression::new(&te, FormatTemplateExpressionOptions::default()))
            .expect("a template expression always emits non-empty IR");
        formatted_expressions.push(element);
    }

    // Phase 3: Rebuild the IR in one pass, re-escaping template chars in `Text` runs
    // (the IR is reinserted into a JS template literal built from `.cooked` values)
    // and substituting placeholders inside every `BestFitting` variant.
    // Escaping cannot alter placeholders: the sentinel is ASCII word characters only.
    let indent_width = f.options().indent_width;
    let ir = super::map_text_in_ir(&ir, f, &mut |text, out| {
        let escaped = super::escape_template_chars(text, allocator);
        let text = escaped.unwrap_or(text);
        if text.contains(PLACEHOLDER_PREFIX) {
            let parts = super::split_on_placeholders(text, PLACEHOLDER_PREFIX, PLACEHOLDER_SUFFIX);
            if parts.len() > 1 {
                for (index, part) in parts.iter().enumerate() {
                    if index.is_multiple_of(2) {
                        push_text_with_line_breaks(out, part, indent_width);
                    } else {
                        let expression = part
                            .parse::<usize>()
                            .ok()
                            .and_then(|index| formatted_expressions.get(index))
                            .expect(
                                "placeholder indices were validated before expression formatting",
                            );
                        out.push(expression.clone());
                    }
                }
                return true;
            }
        }
        // No placeholder to substitute; push the escaped text if escaping changed it
        let Some(text) = escaped else { return false };
        out.push(FormatElement::Text { text, width: TextWidth::from_text(text, indent_width) });
        true
    });
    let format_content = format_once(move |f: &mut JsFormatter<'_, 'a>| f.write_elements(ir));

    let ws_ignore = f.options().html_whitespace_sensitivity_ignore;
    write_html_template(
        f,
        &format_content,
        has_leading_ws,
        has_trailing_ws,
        html_has_multiple_root_elements.unwrap_or(true),
        ws_ignore,
    );
    true
}

// ---

/// Write the HTML template with appropriate wrapping based on whitespace and top-level count.
///
/// Prettier's wrapping logic:
/// - `htmlWhitespaceSensitivity: "ignore"`:
///   Always `group(["`", indent([hardline, group(content)]), hardline, "`"])`
/// - `htmlWhitespaceSensitivity: "css"` (default) or `"strict"`:
///   - Both leading+trailing whitespace: `group(["`", indent([line, group(content)]), line, "`"])`
///     → `line` becomes a space in flat mode, newline when expanded
///   - Otherwise: `group(["`", leadingWS?, maybeIndent(group(content)), trailingWS?, "`"])`
///     → content hugs the backtick directly
///     → multiple root elements wraps with `indent`, single does not
fn write_html_template<'a>(
    f: &mut JsFormatter<'_, 'a>,
    content: &impl Format<'a, JsFormatContext<'a>>,
    has_leading_ws: bool,
    has_trailing_ws: bool,
    has_multiple_root_elements: bool,
    ws_ignore: bool,
) {
    if ws_ignore {
        // group(["`", indent([hardline, group(content)]), hardline, "`"])
        write!(
            f,
            [group(&format_args!(
                "`",
                indent(&format_args!(hard_line_break(), group(content))),
                hard_line_break(),
                "`"
            ))]
        );
    } else if has_leading_ws && has_trailing_ws {
        // group(["`", indent([line, group(content)]), line, "`"])
        // `soft_line_break_or_space` = Prettier's `line`: space in flat mode, newline when expanded
        write!(
            f,
            [group(&format_args!(
                "`",
                indent(&format_args!(soft_line_break_or_space(), group(content))),
                soft_line_break_or_space(),
                "`"
            ))]
        );
    } else {
        // group(["`", leadingWS?, maybeIndent(group(content)), trailingWS?, "`"])
        let leading = if has_leading_ws { " " } else { "" };
        let trailing = if has_trailing_ws { " " } else { "" };
        if has_multiple_root_elements {
            write!(f, [group(&format_args!("`", leading, indent(&group(content)), trailing, "`"))]);
        } else {
            write!(f, [group(&format_args!("`", leading, group(content), trailing, "`"))]);
        }
    }
}

/// Check that placeholders appear in index order starting at `*next`, one occurrence each.
///
/// `BestFitting` alternatives describe the same logical content,
/// so every variant must contain the same index sequence and contributes it only once to its parent sequence.
/// On success `*next` is one past the last index seen; the caller checks it against the expression count.
fn placeholders_are_sequential(ir: &[FormatElement<'_>], next: &mut usize) -> bool {
    for element in ir {
        match element {
            FormatElement::Text { text, .. } => {
                if !text.contains(PLACEHOLDER_PREFIX) {
                    continue;
                }
                let parts =
                    super::split_on_placeholders(text, PLACEHOLDER_PREFIX, PLACEHOLDER_SUFFIX);
                for part in parts.iter().skip(1).step_by(2) {
                    if !part.parse::<usize>().is_ok_and(|index| index == *next) {
                        return false;
                    }
                    *next += 1;
                }
            }
            FormatElement::BestFitting(best_fitting) => {
                let start = *next;
                let mut variants = best_fitting.variants().iter();
                let Some(first) = variants.next() else { return false };
                if !placeholders_are_sequential(first, next) {
                    return false;
                }
                for variant in variants {
                    let mut variant_next = start;
                    if !placeholders_are_sequential(variant, &mut variant_next)
                        || variant_next != *next
                    {
                        return false;
                    }
                }
            }
            FormatElement::Interned(interned) if !placeholders_are_sequential(interned, next) => {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Emit text with newlines converted to literal line breaks (`replaceEndOfLine()` equivalent).
///
/// Uses [`LineMode::Literal`] instead of a hard line break to avoid adding indentation:
/// the returned HTML Doc already carries its indentation in the text content,
/// so the surrounding `block_indent` must not add more.
fn push_text_with_line_breaks<'a>(
    out: &mut ArenaVec<'a, FormatElement<'a>>,
    text: &'a str,
    indent_width: IndentWidth,
) {
    let mut first = true;
    // Splitting on `\n` is safe because `Doc` only contains normalized linebreaks
    for line in text.split('\n') {
        if !first {
            out.push(FormatElement::Line(LineMode::Literal));
        }
        first = false;
        if !line.is_empty() {
            out.push(FormatElement::Text {
                text: line,
                width: TextWidth::from_text(line, indent_width),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::{Allocator, ArenaVec};
    use oxc_formatter_core::{
        BestFittingElement, FormatElement, IndentWidth, format_element::TextWidth,
    };

    use super::placeholders_are_sequential;

    fn text(text: &'static str) -> FormatElement<'static> {
        FormatElement::Text { text, width: TextWidth::from_text(text, IndentWidth::default()) }
    }

    fn best_fitting<'a>(
        allocator: &'a Allocator,
        first: FormatElement<'a>,
        second: FormatElement<'a>,
    ) -> FormatElement<'a> {
        let first = ArenaVec::from_array_in([first], &allocator).into_arena_slice();
        let second = ArenaVec::from_array_in([second], &allocator).into_arena_slice();
        let variants = ArenaVec::from_array_in([first as &[_], second as &[_]], &allocator);
        // SAFETY: The helper always constructs exactly two variants
        FormatElement::BestFitting(unsafe { BestFittingElement::from_vec_unchecked(variants) })
    }

    #[test]
    fn placeholder_indices_count_best_fitting_variants_once() {
        let allocator = Allocator::default();
        let element = best_fitting(
            &allocator,
            text("aPRETTIER_HTML_PLACEHOLDER_0_0_IN_JSbPRETTIER_HTML_PLACEHOLDER_1_0_IN_JSc"),
            text("PRETTIER_HTML_PLACEHOLDER_0_0_IN_JS PRETTIER_HTML_PLACEHOLDER_1_0_IN_JS"),
        );

        let mut next = 0;
        assert!(placeholders_are_sequential(&[element], &mut next));
        assert_eq!(next, 2);
    }

    #[test]
    fn placeholder_indices_reject_different_best_fitting_variants() {
        let allocator = Allocator::default();
        let element = best_fitting(
            &allocator,
            text("PRETTIER_HTML_PLACEHOLDER_0_0_IN_JS"),
            text("PRETTIER_HTML_PLACEHOLDER_1_0_IN_JS"),
        );

        let mut next = 0;
        assert!(!placeholders_are_sequential(&[element], &mut next));
    }
}
