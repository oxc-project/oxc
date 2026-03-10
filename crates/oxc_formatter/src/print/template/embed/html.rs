use oxc_allocator::StringBuilder;
use oxc_ast::ast::*;

use crate::{
    ast_nodes::AstNode,
    external_formatter::EmbeddedDocResult,
    format_args,
    formatter::{FormatElement, Formatter, prelude::*},
    write,
};

// Prettier uses `PRETTIER_HTML_PLACEHOLDER_{index}_{counter}_IN_JS` as placeholders.
// We use a fixed counter of 0 since we don't have nested embeds.
const PLACEHOLDER_PREFIX: &str = "PRETTIER_HTML_PLACEHOLDER_";
const PLACEHOLDER_SUFFIX: &str = "_IN_JS";
const COUNTER: &str = "0";

/// Format an HTML-in-JS template literal via the Doc->IR path with placeholder replacement.
///
/// Uses `.cooked` values (unlike CSS which uses `.raw`), joins quasis with
/// `PRETTIER_HTML_PLACEHOLDER_{N}_0_IN_JS` markers, formats as HTML,
/// then replaces placeholder occurrences in the resulting IR with `${expr}` Docs.
pub(super) fn format_html_doc<'a>(
    quasi: &AstNode<'a, TemplateLiteral<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> bool {
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

        let allocator = f.allocator();
        let group_id_builder = f.group_id_builder();
        let Some(Ok(EmbeddedDocResult::DocWithPlaceholders { ir, top_level_count, .. })) = f
            .context()
            .external_callbacks()
            .format_embedded_doc(allocator, group_id_builder, "tagged-html", &[cooked])
        else {
            return false;
        };

        let content = format_once(|f| f.write_elements(ir));
        let ws_ignore = f.options().html_whitespace_sensitivity_ignore;
        write_html_template(
            f,
            &content,
            has_leading_ws,
            has_trailing_ws,
            top_level_count,
            ws_ignore,
        );
        return true;
    }

    // Phase 1: Build joined text using .cooked with HTML placeholders
    // quasis[0].cooked + "PRETTIER_HTML_PLACEHOLDER_0_0_IN_JS" + quasis[1].cooked + ...
    let allocator = f.allocator();
    let joined = {
        let mut sb = StringBuilder::new_in(allocator);
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

    // Phase 2: Format via the Doc->IR path
    let allocator = f.allocator();
    let group_id_builder = f.group_id_builder();
    let Some(Ok(EmbeddedDocResult::DocWithPlaceholders { ir, placeholder_count, top_level_count })) =
        f.context().external_callbacks().format_embedded_doc(
            allocator,
            group_id_builder,
            "tagged-html",
            &[joined],
        )
    else {
        return false;
    };

    // Verify all placeholders survived HTML formatting.
    if placeholder_count != expressions.len() {
        return false;
    }

    // Phase 3: Replace placeholders in IR with expressions
    let indent_width = f.options().indent_width;
    let format_content = format_once(move |f: &mut Formatter<'_, 'a>| {
        for element in ir {
            match &element {
                FormatElement::Text { text, .. } if text.contains(PLACEHOLDER_PREFIX) => {
                    let parts = split_on_placeholders(text);
                    for (i, part) in parts.iter().enumerate() {
                        if i % 2 == 0 {
                            if !part.is_empty() {
                                super::write_text_with_line_breaks(
                                    f,
                                    part,
                                    allocator,
                                    indent_width,
                                );
                            }
                        } else if let Some(idx) = part.parse::<usize>().ok()
                            && let Some(expr) = expressions.get(idx)
                        {
                            write!(
                                f,
                                [group(&format_args!("${", expr, line_suffix_boundary(), "}"))]
                            );
                        }
                    }
                }
                _ => {
                    f.write_element(element);
                }
            }
        }
    });

    let ws_ignore = f.options().html_whitespace_sensitivity_ignore;
    write_html_template(
        f,
        &format_content,
        has_leading_ws,
        has_trailing_ws,
        top_level_count,
        ws_ignore,
    );
    true
}

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
///     → `topLevelCount > 1` wraps with `indent`, `topLevelCount <= 1` does not
fn write_html_template<'a>(
    f: &mut Formatter<'_, 'a>,
    content: &impl Format<'a>,
    has_leading_ws: bool,
    has_trailing_ws: bool,
    top_level_count: Option<usize>,
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
        let use_indent = top_level_count.is_none_or(|c| c > 1);
        if use_indent {
            write!(f, [group(&format_args!("`", leading, indent(&group(content)), trailing, "`"))]);
        } else {
            write!(f, [group(&format_args!("`", leading, group(content), trailing, "`"))]);
        }
    }
}

// ---

/// Split text on `PRETTIER_HTML_PLACEHOLDER_N_C_IN_JS` patterns.
///
/// Returns alternating parts: `[literal, index_str, literal, index_str, ...]`
fn split_on_placeholders(text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut remaining = text;

    loop {
        let Some(start) = remaining.find(PLACEHOLDER_PREFIX) else {
            result.push(remaining);
            break;
        };

        // Push the literal before the placeholder
        result.push(&remaining[..start]);

        // Skip past the prefix
        let after_prefix = &remaining[start + PLACEHOLDER_PREFIX.len()..];

        // Find the index digits
        let digit_end =
            after_prefix.bytes().position(|b| !b.is_ascii_digit()).unwrap_or(after_prefix.len());

        if digit_end == 0 {
            // No digits found after prefix - not a valid placeholder, treat as literal
            if let Some(last) = result.last_mut() {
                let end = start + PLACEHOLDER_PREFIX.len();
                *last = &remaining[..end];
            }
            remaining = &remaining[start + PLACEHOLDER_PREFIX.len()..];
            continue;
        }

        let digits = &after_prefix[..digit_end];
        let after_digits = &after_prefix[digit_end..];

        // Check for `_{counter}_IN_JS` suffix
        if let Some(after_underscore) = after_digits.strip_prefix('_') {
            let counter_end = after_underscore
                .bytes()
                .position(|b| !b.is_ascii_digit())
                .unwrap_or(after_underscore.len());
            if counter_end > 0 {
                let after_counter = &after_underscore[counter_end..];
                if let Some(after_suffix) = after_counter.strip_prefix(PLACEHOLDER_SUFFIX) {
                    // Valid placeholder - push the digit index
                    result.push(digits);
                    remaining = after_suffix;
                    continue;
                }
            }
        }

        // Not a valid placeholder, include in the literal
        let end = start + PLACEHOLDER_PREFIX.len() + digit_end;
        if let Some(last) = result.last_mut() {
            *last = &remaining[..end];
        }
        remaining = &remaining[end..];
    }

    result
}
