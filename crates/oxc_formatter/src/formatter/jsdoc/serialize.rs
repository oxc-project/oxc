use std::fmt::Write as _;

use oxc_allocator::{Allocator, StringBuilder};
use oxc_ast::Comment;
use oxc_jsdoc::JSDoc;
use oxc_span::Span;

use crate::options::JsdocOptions;

use super::{
    normalize::{capitalize_first, normalize_tag_kind, normalize_type_whitespace},
    wrap::wrap_text,
};

/// The ` * ` prefix used in multiline JSDoc comments (3 chars).
const LINE_PREFIX_LEN: usize = 3;

/// Tags whose descriptions should NOT be capitalized (contain code literals).
fn should_skip_capitalize(tag_kind: &str) -> bool {
    matches!(tag_kind, "default" | "defaultValue" | "example" | "see" | "link")
}

/// Tags that use `type_name_comment()` pattern: `@tag {type} name description`
fn is_type_name_comment_tag(tag_kind: &str) -> bool {
    matches!(tag_kind, "param" | "property" | "typedef" | "template" | "arg" | "argument" | "prop")
}

/// Tags that use `type_comment()` pattern: `@tag {type} description`
fn is_type_comment_tag(tag_kind: &str) -> bool {
    matches!(
        tag_kind,
        "returns" | "yields" | "throws" | "type" | "satisfies" | "return" | "yield" | "exception"
    )
}

/// Format a JSDoc comment. Returns `Some(formatted)` if the comment was modified,
/// `None` if no changes are needed.
///
/// The returned string is the full comment content (e.g. `/** ... */`), or empty
/// string to signal the comment should be removed (empty JSDoc).
pub fn format_jsdoc_comment<'a>(
    comment: &Comment,
    options: &JsdocOptions,
    source_text: &str,
    allocator: &'a Allocator,
    available_width: usize,
) -> Option<&'a str> {
    let content = &source_text[comment.span.start as usize..comment.span.end as usize];

    // Must be at least `/** */` (5 chars)
    if content.len() < 5 {
        return None;
    }

    // Extract inner content (between `/**` and `*/`)
    let inner = &content[3..content.len() - 2];
    let jsdoc = JSDoc::new(inner, Span::new(comment.span.start + 3, comment.span.end - 2));

    let comment_part = jsdoc.comment();
    let description = comment_part.parsed_preserving_whitespace();
    let tags = jsdoc.tags();

    // Empty JSDoc: no description and no tags
    if description.trim().is_empty() && tags.is_empty() {
        return Some(allocator.alloc_str(""));
    }

    // Width available for content (subtract ` * ` prefix)
    let wrap_width = available_width.saturating_sub(LINE_PREFIX_LEN);

    let mut content_lines: Vec<String> = Vec::new();

    // Format description (preserving paragraph structure)
    let desc_trimmed = description.trim();
    if !desc_trimmed.is_empty() {
        let mut desc = desc_trimmed.to_string();
        if options.capitalize_descriptions {
            desc = capitalize_first(&desc);
        }
        wrap_text(&desc, wrap_width, &mut content_lines);
    }

    // Format tags
    let mut prev_tag_kind: Option<&str> = None;
    for tag in tags {
        let raw_kind = tag.kind.parsed();
        let normalized_kind = normalize_tag_kind(raw_kind);
        let should_capitalize =
            options.capitalize_descriptions && !should_skip_capitalize(normalized_kind);

        // Add blank line before tags, but not between consecutive tags of the same kind
        if !content_lines.is_empty()
            && !content_lines.last().is_some_and(String::is_empty)
            && prev_tag_kind != Some(normalized_kind)
        {
            content_lines.push(String::new());
        }

        if normalized_kind == "example" {
            format_example_tag(normalized_kind, tag, &mut content_lines);
        } else if is_type_name_comment_tag(raw_kind) {
            format_type_name_comment_tag(
                normalized_kind,
                tag,
                should_capitalize,
                wrap_width,
                &mut content_lines,
            );
        } else if is_type_comment_tag(raw_kind) {
            format_type_comment_tag(
                normalized_kind,
                tag,
                should_capitalize,
                wrap_width,
                &mut content_lines,
            );
        } else {
            format_generic_tag(
                normalized_kind,
                tag,
                should_capitalize,
                wrap_width,
                &mut content_lines,
            );
        }

        prev_tag_kind = Some(normalized_kind);
    }

    // Remove trailing empty lines
    while content_lines.last().is_some_and(String::is_empty) {
        content_lines.pop();
    }

    // Remove leading empty lines
    while content_lines.first().is_some_and(String::is_empty) {
        content_lines.remove(0);
    }

    if content_lines.is_empty() {
        return Some(allocator.alloc_str(""));
    }

    // Single-line check
    if options.single_line_when_possible
        && content_lines.len() == 1
        && content_lines[0].starts_with('@')
    {
        let single = &content_lines[0];
        // `/** ` (4) + content + ` */` (3) = 7 extra chars
        if single.len() + 7 <= available_width {
            let formatted = format!("/** {single} */");
            if formatted == content {
                return None;
            }
            return Some(allocator.alloc_str(&formatted));
        }
    }

    // Build multiline comment
    let mut builder = StringBuilder::with_capacity_in(
        content_lines.iter().map(|l| l.len() + 4).sum::<usize>() + 10,
        allocator,
    );
    builder.push_str("/**");

    for line in &content_lines {
        builder.push('\n');
        if line.is_empty() {
            builder.push_str(" *");
        } else {
            builder.push_str(" * ");
            builder.push_str(line);
        }
    }
    builder.push('\n');
    builder.push_str(" */");

    let result = builder.into_str();

    // Compare with original
    if result == content {
        return None;
    }

    Some(result)
}

fn format_example_tag(
    normalized_kind: &str,
    tag: &oxc_jsdoc::parser::JSDocTag<'_>,
    content_lines: &mut Vec<String>,
) {
    content_lines.push(format!("@{normalized_kind}"));
    let comment_part = tag.comment();
    let raw_text = comment_part.parsed_preserving_whitespace();
    let trimmed = raw_text.trim();
    if !trimmed.is_empty() {
        for line in trimmed.lines() {
            content_lines.push(line.to_string());
        }
    }
}

fn format_type_name_comment_tag(
    normalized_kind: &str,
    tag: &oxc_jsdoc::parser::JSDocTag<'_>,
    should_capitalize: bool,
    wrap_width: usize,
    content_lines: &mut Vec<String>,
) {
    let (type_part, name_part, comment_part) = tag.type_name_comment();

    let mut tag_line = format!("@{normalized_kind}");

    if let Some(tp) = &type_part {
        let raw_type = tp.parsed();
        let normalized_type = normalize_type_whitespace(raw_type);
        write!(tag_line, " {{{normalized_type}}}").unwrap();
    }

    if let Some(np) = &name_part {
        write!(tag_line, " {}", np.raw()).unwrap();
    }

    let desc_text = comment_part.parsed();
    let desc_text = desc_text.trim();

    if desc_text.is_empty() {
        content_lines.push(tag_line);
        return;
    }

    // Strip leading dash prefix (e.g. `- description` -> `description`)
    let desc_text = desc_text.strip_prefix("- ").unwrap_or(desc_text);
    let desc_text =
        if should_capitalize { capitalize_first(desc_text) } else { desc_text.to_string() };

    // Check if it fits on one line
    let one_liner = format!("{tag_line} - {desc_text}");
    if one_liner.len() <= wrap_width {
        content_lines.push(one_liner);
    } else {
        // Put tag + type + name on first line, wrap description
        content_lines.push(format!("{tag_line} -"));
        let indent = "  ";
        let indent_width = wrap_width.saturating_sub(indent.len());
        let mut desc_lines = Vec::new();
        wrap_text(&desc_text, indent_width, &mut desc_lines);
        for dl in desc_lines {
            content_lines.push(format!("{indent}{dl}"));
        }
    }
}

fn format_type_comment_tag(
    normalized_kind: &str,
    tag: &oxc_jsdoc::parser::JSDocTag<'_>,
    should_capitalize: bool,
    wrap_width: usize,
    content_lines: &mut Vec<String>,
) {
    let (type_part, comment_part) = tag.type_comment();

    let mut tag_line = format!("@{normalized_kind}");

    if let Some(tp) = &type_part {
        let raw_type = tp.parsed();
        let normalized_type = normalize_type_whitespace(raw_type);
        write!(tag_line, " {{{normalized_type}}}").unwrap();
    }

    let desc_text = comment_part.parsed();
    let desc_text = desc_text.trim();

    if desc_text.is_empty() {
        content_lines.push(tag_line);
        return;
    }

    let desc_text =
        if should_capitalize { capitalize_first(desc_text) } else { desc_text.to_string() };

    let one_liner = format!("{tag_line} {desc_text}");
    if one_liner.len() <= wrap_width {
        content_lines.push(one_liner);
    } else {
        content_lines.push(tag_line);
        let mut desc_lines = Vec::new();
        wrap_text(&desc_text, wrap_width, &mut desc_lines);
        content_lines.extend(desc_lines);
    }
}

fn format_generic_tag(
    normalized_kind: &str,
    tag: &oxc_jsdoc::parser::JSDocTag<'_>,
    should_capitalize: bool,
    wrap_width: usize,
    content_lines: &mut Vec<String>,
) {
    let tag_line = format!("@{normalized_kind}");
    let desc_text = tag.comment().parsed();
    let desc_text = desc_text.trim();

    if desc_text.is_empty() {
        content_lines.push(tag_line);
        return;
    }

    let desc_text =
        if should_capitalize { capitalize_first(desc_text) } else { desc_text.to_string() };

    let one_liner = format!("{tag_line} {desc_text}");
    if one_liner.len() <= wrap_width {
        content_lines.push(one_liner);
    } else {
        content_lines.push(tag_line);
        let mut desc_lines = Vec::new();
        wrap_text(&desc_text, wrap_width, &mut desc_lines);
        content_lines.extend(desc_lines);
    }
}
