use oxc_allocator::{Allocator, StringBuilder};
use oxc_ast::ast::*;

use crate::{
    ast_nodes::AstNode,
    format_args,
    formatter::{Formatter, prelude::*},
    write,
};

/// Format a Markdown-in-JS tagged template literal via the Doc→IR path.
///
/// Unescapes backticks in `.raw`, strips common indentation, formats as markdown,
/// then re-escapes backticks and applies indented or dedent-to-root layout.
pub(super) fn try_embed_markdown<'a>(
    tagged: &AstNode<'a, TaggedTemplateExpression<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> bool {
    let raw = tagged.quasi.quasis[0].value.raw.as_str();

    if raw.trim().is_empty() {
        write!(f, ["``"]);
        return true;
    }

    let allocator = f.context().allocator();

    // Phase 1: Unescape backticks (= `raw.replaceAll(/((?:\\\\)*)\\`/g, ...)`)
    // https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/embed/markdown.js#L11-L14
    let text = unescape_backticks(raw, allocator);

    // Phase 2: Detect and strip common indentation
    let indentation = get_indentation(text);
    let has_indent = !indentation.is_empty();
    let text = if has_indent { strip_indentation(text, indentation, allocator) } else { text };

    // Phase 3: Get Doc→IR from external formatter
    let allocator = f.allocator();
    let group_id_builder = f.group_id_builder();
    let Some(Ok(crate::external_formatter::EmbeddedDocResult::SingleDoc(ir))) = f
        .context()
        .external_callbacks()
        .format_embedded_doc(allocator, group_id_builder, "markdown", &[text])
    else {
        return false;
    };

    // Phase 4: Re-escape backticks in the IR (`escapeTemplateCharacters(doc, true)`)
    // This is already handled by  oxfmt `prettier_compat/from_prettier_doc.rs`

    // Phase 5: Layout
    // https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/embed/markdown.js#L24-L29
    let content = format_once(|f| f.write_elements(ir));
    if has_indent {
        write!(f, ["`", indent(&format_args!(soft_line_break(), content)), soft_line_break(), "`"]);
    } else {
        let literalline = format_with(|f| super::write_literalline(f, allocator));
        write!(f, ["`", literalline, dedent_to_root(&content), soft_line_break(), "`"]);
    }

    true
}

// ---

/// Unescape backticks in raw template literal content.
/// Transforms `\`` → `` ` ``, `\\`` → `` \` ``
/// `raw.replaceAll(/((?:\\\\)*)\\`/g, (_, bs) => "\\".repeat(bs.length / 2) + "`")`
/// <https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/embed/markdown.js#L11-L14>
fn unescape_backticks<'a>(raw: &'a str, allocator: &'a Allocator) -> &'a str {
    if !raw.contains('`') {
        return raw;
    }

    let mut result = StringBuilder::with_capacity_in(raw.len(), allocator);
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            // Count consecutive backslashes
            let mut bs_count = 1;
            while chars.peek() == Some(&'\\') {
                bs_count += 1;
                chars.next();
            }
            if chars.peek() == Some(&'`') {
                // Halve the backslashes and unescape the backtick
                for _ in 0..bs_count / 2 {
                    result.push('\\');
                }
                result.push('`');
                chars.next();
            } else {
                // Not followed by backtick, keep backslashes as-is
                for _ in 0..bs_count {
                    result.push('\\');
                }
            }
        } else {
            result.push(c);
        }
    }
    result.into_str()
}

/// Get indentation of the first non-empty line.
/// `str.match(/^([^\S\n]*)\S/m)?.[1] ?? ""`
/// <https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/embed/markdown.js#L32-L35>
fn get_indentation(text: &str) -> &str {
    for line in text.split('\n') {
        let trimmed = line.trim_start_matches(|c: char| c.is_ascii_whitespace() && c != '\n');
        if !trimmed.is_empty() {
            return &line[..line.len() - trimmed.len()];
        }
    }
    ""
}

/// Strip common indentation from all lines.
/// `text.replaceAll(new RegExp(\`^${indentation}\`, "gm"), "")`
/// <https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/embed/markdown.js#L17-L19>
fn strip_indentation<'a>(text: &'a str, indent: &str, allocator: &'a Allocator) -> &'a str {
    let mut result = StringBuilder::with_capacity_in(text.len(), allocator);
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            result.push('\n');
        }
        if let Some(stripped) = line.strip_prefix(indent) {
            result.push_str(stripped);
        } else {
            result.push_str(line);
        }
    }
    result.into_str()
}
