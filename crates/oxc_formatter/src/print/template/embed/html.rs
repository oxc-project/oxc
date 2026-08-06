use cow_utils::CowUtils;

use oxc_allocator::ArenaStringBuilder;
use oxc_ast::ast::*;
use oxc_formatter_core::{DispatchOutcome, DispatchRequest, FormatElement, InputKind};
use oxc_syntax::line_terminator::LineTerminatorSplitter;

use crate::{
    ast_nodes::AstNode,
    external_formatter::HtmlEmbedMeta,
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

        let Ok(DispatchOutcome::Formatted(mut result)) = f.session().dispatch(DispatchRequest {
            language: embedded_language,
            texts: &[cooked],
            input_kind: InputKind::Fragment,
            parent_context: None,
        }) else {
            return false;
        };
        let Some(html_has_multiple_root_elements) = result
            .meta
            .as_ref()
            .and_then(|meta| meta.downcast_ref::<HtmlEmbedMeta>())
            .map(|meta| meta.has_multiple_root_elements)
        else {
            return false;
        };
        // Remap is a no-op today (the Prettier Doc path never carries classes),
        // but the boundary contract is "merge at every embed site".
        // A Rust HTML formatter collecting `class` attributes will rely on this.
        result.remap_tailwind_into(f.context_mut());
        let Some(mut ir) = result.docs.into_iter().next() else {
            return false;
        };

        // Re-escape template chars in `Text` runs:
        // the IR is reinserted into a JS template literal built from `.cooked` values.
        let allocator = f.allocator();
        super::escape_template_chars_in_ir(&mut ir, allocator, f.options().indent_width);

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
    let Ok(DispatchOutcome::Formatted(mut result)) = f.session().dispatch(DispatchRequest {
        language: embedded_language,
        texts: &[joined],
        input_kind: InputKind::Fragment,
        parent_context: None,
    }) else {
        // NOTE: If this html-in-js part contains `<script>` (= js-in-html-in-js),
        // returned Prettier's `Doc` output may contain `conditionalGroup`.
        // But currently, `oxfmt/prettier_compat/from_prettier_doc.rs` does not support this.
        // So the dispatch will return `Err`.
        //
        // In Prettier, `conditionalGroup` is only used by JS and YAML formatting.
        // And we want to format JS by `oxc_formatter` via oxfmt-plugin,
        // so for now, fall back to string-based rather than give up entirely.
        // Of course, there will be formatting differences.
        // Support `conditionalGroup` and convert to our `BestFitting` may be possible,
        // but it also requires placeholder replacement, which is non-trivial.
        //
        // NOTE: `Ok(PreserveOriginal)` lands here too and gets the same recovery attempt
        // (today it only arises when the dispatcher is absent,
        // in which case the string callback is absent under the same gate
        // and recovery degrades to the plain template path anyway).
        // Splitting the match is string-channel-migration work,
        // out of scope while html stays on the Prettier string path.
        return format_js_in_html_as_fallback(joined, &expressions, f);
    };

    let Some(html_has_multiple_root_elements) = result
        .meta
        .as_ref()
        .and_then(|meta| meta.downcast_ref::<HtmlEmbedMeta>())
        .map(|meta| meta.has_multiple_root_elements)
    else {
        return false;
    };
    // See the Phase 0 note: remap is no-op today, load-bearing once `oxc_formatter_html` lands
    result.remap_tailwind_into(f.context_mut());
    let Some(mut ir) = result.docs.into_iter().next() else {
        return false;
    };

    // Re-escape template chars in `Text` runs before counting / substituting:
    // the IR is reinserted into a JS template literal built from `.cooked` values.
    let indent_width = f.options().indent_width;
    super::escape_template_chars_in_ir(&mut ir, allocator, indent_width);

    // Verify all placeholders survived HTML formatting
    let placeholder_count: usize = ir
        .iter()
        .map(|el| match el {
            FormatElement::Text { text, .. } => {
                super::count_placeholders(text, PLACEHOLDER_PREFIX, PLACEHOLDER_SUFFIX)
            }
            _ => 0,
        })
        .sum();
    if placeholder_count != expressions.len() {
        return false;
    }

    // Phase 3: Replace placeholders in IR with expressions
    let format_content = format_once(move |f: &mut JsFormatter<'_, 'a>| {
        for element in ir {
            match &element {
                FormatElement::Text { text, .. } if text.contains(PLACEHOLDER_PREFIX) => {
                    let parts =
                        super::split_on_placeholders(text, PLACEHOLDER_PREFIX, PLACEHOLDER_SUFFIX);
                    for (i, part) in parts.iter().enumerate() {
                        if i.is_multiple_of(2) {
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
                            // Prettier prints embedded `${expr}` with `printEmbeddedTemplateExpressions()`:
                            // the plain-template expression logic minus source-indentation preservation.
                            // Default options (zero indention) are exactly that (same as graphql.rs).
                            let te = TemplateExpression::Expression(expr);
                            FormatTemplateExpression::new(
                                &te,
                                FormatTemplateExpressionOptions::default(),
                            )
                            .fmt(f);
                        }
                    }
                }
                _ => f.write_element(element),
            }
        }
    });

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

/// Fallback formatting for JS-in-HTML-in-JS cases where
/// Prettier's HTML formatting returns unsupported IR (e.g. with `conditionalGroup`).
fn format_js_in_html_as_fallback<'a>(
    joined: &str,
    expressions: &[&AstNode<'a, Expression<'a>>],
    f: &mut JsFormatter<'_, 'a>,
) -> bool {
    let Some(Ok(formatted)) = f.context().external_callbacks().format_embedded("html", joined)
    else {
        return false;
    };

    // Replace placeholders with `${source_text}` from the original expressions
    let mut result = formatted;
    for (idx, expr) in expressions.iter().enumerate() {
        let placeholder = format!("{PLACEHOLDER_PREFIX}{idx}_{COUNTER}{PLACEHOLDER_SUFFIX}");
        if !result.contains(&placeholder) {
            return false;
        }
        let source = f.source_text().text_for(expr);
        result = result.cow_replace(&placeholder, &format!("${{{source}}}")).into_owned();
    }

    // Write line by line, same as `format_embedded_template()`
    let format_content = format_with(|f: &mut JsFormatter<'_, 'a>| {
        let content = f.allocator().alloc_str(&result);
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
