use std::fmt::Write as _;

use oxc_allocator::{Allocator, StringBuilder};
use oxc_ast::Comment;
use oxc_jsdoc::JSDoc;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::ExternalCallbacks;
use crate::options::TrailingCommas;
use crate::options::{JsdocOptions, QuoteStyle};
use crate::{FormatOptions, Formatter, LineWidth, get_parse_options};

use super::{
    mdast_serialize::format_description_mdast,
    normalize::{
        capitalize_first, normalize_markdown_emphasis, normalize_tag_kind, normalize_type,
        normalize_type_preserve_quotes, normalize_type_return, normalize_type_whitespace,
        strip_optional_type_suffix,
    },
    wrap::{tokenize_words, wrap_text},
};

/// The ` * ` prefix used in multiline JSDoc comments (3 chars).
const LINE_PREFIX_LEN: usize = 3;

/// Tags whose descriptions should NOT be capitalized.
/// Includes upstream's `TAGS_PEV_FORMATE_DESCRIPTION` (borrows, default, defaultValue,
/// import, memberof, module, see) plus tags whose content is a name/reference rather
/// than descriptive text (e.g. @function name, @typedef type, @class name).
fn should_skip_capitalize(tag_kind: &str) -> bool {
    matches!(
        tag_kind,
        "augments"
            | "borrows"
            | "callback"
            | "class"
            | "default"
            | "defaultValue"
            | "example"
            | "extends"
            | "function"
            | "import"
            | "link"
            | "memberof"
            | "module"
            | "see"
            | "type"
            | "typedef"
    )
}

/// Tags that use `type_name_comment()` pattern: `@tag {type} name description`
/// Expects canonical (normalized) tag names.
fn is_type_name_comment_tag(tag_kind: &str) -> bool {
    matches!(tag_kind, "param" | "property" | "typedef" | "template")
}

/// Tags that use `type_comment()` pattern: `@tag {type} description`
/// Expects canonical (normalized) tag names.
fn is_type_comment_tag(tag_kind: &str) -> bool {
    matches!(tag_kind, "returns" | "yields" | "throws" | "type" | "satisfies")
}

/// Get the sort priority for a tag kind (lower number = higher priority).
/// Uses only canonical tag names (synonyms resolved by `normalize_tag_kind()`).
/// Weights are upstream values ×2 to handle 39.5 (@this) as integer 79.
fn tag_sort_priority(kind: &str) -> u32 {
    match kind {
        "import" => 0,
        "remarks" => 2,
        "privateRemarks" => 4,
        "providesModule" => 6,
        "module" => 8,
        "license" => 10,
        "flow" => 12,
        "async" => 14,
        "private" => 16,
        "ignore" => 18,
        "memberof" => 20,
        "version" => 22,
        "file" => 24,
        "author" => 26,
        "deprecated" => 28,
        "since" => 30,
        "category" => 32,
        "description" => 34,
        "example" => 36,
        "abstract" => 38,
        "augments" => 40,
        "constant" => 42,
        "default" | "defaultValue" => 44,
        "external" => 48,
        "overload" => 50,
        "fires" => 52,
        "template" | "typeParam" => 54,
        "function" => 58,
        "namespace" => 60,
        "borrows" => 62,
        "class" => 64,
        "extends" => 66,
        "member" => 68,
        "typedef" => 70,
        "type" => 72,
        "satisfies" => 74,
        "property" => 76,
        "callback" => 78,
        "this" => 79,
        "param" => 80,
        "yields" => 82,
        "returns" => 84,
        "throws" => 86,
        "see" => 90,
        "todo" => 92,
        // Unknown tags (upstream "other" = 44, ×2 = 88)
        _ => 88,
    }
}

/// Check if a tag kind is known (has a specific sort priority).
/// Unknown tags skip capitalization, matching upstream's
/// `TAGS_ORDER[tag] === undefined` check in `stringify.js:77`.
fn is_known_tag(kind: &str) -> bool {
    // link/linkcode/linkplain are not in TAGS_ORDER but are special inline tags;
    // for the purposes of capitalization they behave like unknown tags.
    !matches!(tag_sort_priority(kind), 88)
}

/// Check if a tag kind is a group head (starts a new sorting group).
/// Matches prettier-plugin-jsdoc's `TAGS_GROUP_HEAD = [CALLBACK, TYPEDEF]`.
fn is_tags_group_head(kind: &str) -> bool {
    matches!(kind, "callback" | "typedef")
}

/// Reorder @param tags to match the function signature parameter order.
/// Only reorders when:
/// - All @param tags have type annotations (the plugin skips typeless params)
/// - The @param names exactly match the function parameters (same set, different order)
fn reorder_param_tags(
    effective_tags: &mut [(&oxc_jsdoc::parser::JSDocTag<'_>, &str)],
    comment: &Comment,
    source_text: &str,
) {
    // Find consecutive @param tags
    let param_start = effective_tags.iter().position(|(_, kind)| *kind == "param");
    let Some(param_start) = param_start else {
        return;
    };
    let param_end = effective_tags[param_start..]
        .iter()
        .position(|(_, kind)| *kind != "param")
        .map_or(effective_tags.len(), |pos| param_start + pos);

    if param_end - param_start < 2 {
        return;
    }

    // Check that ALL @param tags have type annotations
    let param_names: Vec<&str> = effective_tags[param_start..param_end]
        .iter()
        .filter_map(|(tag, _)| {
            let (type_part, name_part, _) = tag.type_name_comment();
            // Must have both type and name
            type_part?;
            name_part.map(|n| n.parsed())
        })
        .collect();

    if param_names.len() != param_end - param_start {
        return; // Some params lack types or names — don't reorder
    }

    // Extract function parameter names from the source text after the comment
    let fn_params = extract_function_params(comment, source_text);
    if fn_params.is_empty() {
        return;
    }

    // Only reorder if param names exactly match the function params (same set)
    if param_names.len() != fn_params.len() {
        return;
    }
    let mut sorted_doc = param_names.clone();
    sorted_doc.sort_unstable();
    let mut sorted_fn: Vec<&str> = fn_params.iter().map(String::as_str).collect();
    sorted_fn.sort_unstable();
    if sorted_doc != sorted_fn {
        return;
    }

    // Already in order?
    if param_names.iter().zip(fn_params.iter()).all(|(a, b)| *a == b.as_str()) {
        return;
    }

    // Sort @param tags by their position in the function signature
    effective_tags[param_start..param_end].sort_by_key(|(tag, _)| {
        let (_, name_part, _) = tag.type_name_comment();
        let name = name_part.map_or("", |n| n.parsed());
        fn_params.iter().position(|p| p == name).unwrap_or(usize::MAX)
    });
}

/// Extract function parameter names from the source text after the comment.
/// Handles `function name(...)`, `name(...)` methods, `name = (...) =>` arrows.
/// Uses balanced parenthesis matching to handle nested type annotations.
fn extract_function_params(comment: &Comment, source_text: &str) -> Vec<String> {
    let after_start = comment.span.end as usize;
    let after = &source_text[after_start..];

    // Find a function-like construct: look for identifier followed by `(`
    // Skip whitespace, look for `function`, `async`, method names, arrow patterns
    let trimmed = after.trim_start();

    // Find the opening `(` of the parameter list.
    // We look for patterns that indicate a function definition (not a call).
    let paren_pos = find_function_params_start(trimmed);
    let Some(paren_start) = paren_pos else {
        return Vec::new();
    };

    // Find matching closing `)` with balanced parenthesis counting
    let Some(paren_end) = find_matching_paren(trimmed, paren_start) else {
        return Vec::new();
    };

    let params_str = &trimmed[paren_start + 1..paren_end];

    // Parse parameter names, handling TypeScript type annotations
    parse_param_names(params_str)
}

/// Find the start position of function parameter parentheses in the text.
/// Returns the index of `(` in function-like constructs.
fn find_function_params_start(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    // Skip `export`, `async`, `default` keywords
    loop {
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if text[i..].starts_with("export") && i + 6 < len && !bytes[i + 6].is_ascii_alphanumeric() {
            i += 6;
            continue;
        }
        if text[i..].starts_with("async") && i + 5 < len && !bytes[i + 5].is_ascii_alphanumeric() {
            i += 5;
            continue;
        }
        if text[i..].starts_with("default") && i + 7 < len && !bytes[i + 7].is_ascii_alphanumeric()
        {
            i += 7;
            continue;
        }
        break;
    }

    while i < len && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    // `function name(`
    if text[i..].starts_with("function") {
        i += 8;
        // Skip optional `*` for generators
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < len && bytes[i] == b'*' {
            i += 1;
        }
        // Skip function name
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
        {
            i += 1;
        }
        // Skip TypeScript generics `<T>`
        if i < len
            && bytes[i] == b'<'
            && let Some(end) = find_matching_angle(text, i)
        {
            i = end + 1;
        }
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < len && bytes[i] == b'(' {
            return Some(i);
        }
        return None;
    }

    // `const name = (` or `name(` (method)
    if i < len && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' || bytes[i] == b'$') {
        // Skip `const`/`let`/`var` keyword
        if text[i..].starts_with("const ")
            || text[i..].starts_with("let ")
            || text[i..].starts_with("var ")
        {
            while i < len && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
        }

        // Skip identifier
        let id_start = i;
        while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
        {
            i += 1;
        }
        if i == id_start {
            return None;
        }

        // Skip TypeScript generics
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < len
            && bytes[i] == b'<'
            && let Some(end) = find_matching_angle(text, i)
        {
            i = end + 1;
        }

        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }

        // Direct method: `name(`
        if i < len && bytes[i] == b'(' {
            return Some(i);
        }

        // Arrow: `name = (`
        if i < len && bytes[i] == b'=' && i + 1 < len && bytes[i + 1] != b'=' {
            i += 1;
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            // Skip `async`
            if text[i..].starts_with("async")
                && i + 5 < len
                && !bytes[i + 5].is_ascii_alphanumeric()
            {
                i += 5;
                while i < len && bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
            }
            if i < len && bytes[i] == b'(' {
                return Some(i);
            }
        }
    }

    None
}

/// Find matching closing angle bracket `>` for TypeScript generics.
fn find_matching_angle(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'<' => depth += 1,
            b'>' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Find matching closing `)` given position of opening `(`.
fn find_matching_paren(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            b'\'' | b'"' | b'`' => {
                // Skip string literals
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != quote {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Parse parameter names from a function parameter list string.
/// Handles TypeScript type annotations, default values, destructuring, and rest params.
fn parse_param_names(params_str: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut i = 0;
    let bytes = params_str.as_bytes();
    let len = bytes.len();

    while i < len {
        // Skip whitespace
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= len {
            break;
        }

        // Handle destructuring — skip the whole `{...}` or `[...]` structure
        if bytes[i] == b'{' || bytes[i] == b'[' {
            let (open, close) = if bytes[i] == b'{' { (b'{', b'}') } else { (b'[', b']') };
            let mut depth = 0;
            while i < len {
                if bytes[i] == open {
                    depth += 1;
                } else if bytes[i] == close {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                }
                i += 1;
            }
            // Skip type annotation, default value, and comma
            while i < len && bytes[i] != b',' {
                i += 1;
            }
            if i < len {
                i += 1; // skip comma
            }
            continue;
        }

        // Handle rest params: `...name`
        if i + 2 < len && bytes[i] == b'.' && bytes[i + 1] == b'.' && bytes[i + 2] == b'.' {
            i += 3;
        }

        // Extract parameter name
        let name_start = i;
        while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
        {
            i += 1;
        }
        if i > name_start {
            names.push(params_str[name_start..i].to_string());
        }

        // Skip type annotation (`: Type`), which may include nested parens/angles
        while i < len && bytes[i] != b',' {
            match bytes[i] {
                b'(' => {
                    if let Some(end) = find_matching_paren(params_str, i) {
                        i = end + 1;
                    } else {
                        i += 1;
                    }
                }
                b'<' => {
                    if let Some(end) = find_matching_angle(params_str, i) {
                        i = end + 1;
                    } else {
                        i += 1;
                    }
                }
                b'\'' | b'"' => {
                    let quote = bytes[i];
                    i += 1;
                    while i < len && bytes[i] != quote {
                        if bytes[i] == b'\\' {
                            i += 1;
                        }
                        i += 1;
                    }
                    if i < len {
                        i += 1;
                    }
                }
                _ => i += 1,
            }
        }
        if i < len {
            i += 1; // skip comma
        }
    }

    names
}

/// Sort tags by priority within groups.
/// `@typedef` and `@callback` start new groups (TAGS_GROUP_HEAD).
/// Tags within each group are sorted by weight. Groups maintain their relative order.
fn sort_tags_by_groups<'a>(
    tags: &'a [oxc_jsdoc::parser::JSDocTag<'a>],
) -> Vec<&'a oxc_jsdoc::parser::JSDocTag<'a>> {
    if tags.is_empty() {
        return Vec::new();
    }

    // Split into groups at TAGS_GROUP_HEAD boundaries
    let mut groups: Vec<Vec<&oxc_jsdoc::parser::JSDocTag<'a>>> = Vec::new();
    let mut current_group: Vec<&oxc_jsdoc::parser::JSDocTag<'a>> = Vec::new();

    for tag in tags {
        let normalized_kind = normalize_tag_kind(tag.kind.parsed());
        if is_tags_group_head(normalized_kind) && !current_group.is_empty() {
            groups.push(current_group);
            current_group = Vec::new();
        }
        current_group.push(tag);
    }
    if !current_group.is_empty() {
        groups.push(current_group);
    }

    // Sort within each group by weight (stable sort preserves original order for same weight)
    for group in &mut groups {
        group.sort_by_key(|tag| {
            let normalized_kind = normalize_tag_kind(tag.kind.parsed());
            tag_sort_priority(normalized_kind)
        });
    }

    // Flatten groups back into a single list
    groups.into_iter().flatten().collect()
}

/// Check if a tag has meaningful content.
fn tag_has_content(tag: &oxc_jsdoc::parser::JSDocTag<'_>) -> bool {
    let comment = tag.comment().parsed();
    !comment.trim().is_empty()
}

/// Tags that should be removed when they have no content.
/// Matches upstream's `TAGS_DESCRIPTION_NEEDED`.
fn should_remove_empty_tag(kind: &str) -> bool {
    matches!(
        kind,
        "borrows"
            | "category"
            | "description"
            | "example"
            | "import"
            | "privateRemarks"
            | "remarks"
            | "since"
            | "todo"
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
    format_options: &FormatOptions,
    external_callbacks: &ExternalCallbacks,
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

    // Empty JSDoc: no description and no tags
    if description.trim().is_empty() && jsdoc.tags().is_empty() {
        return Some(allocator.alloc_str(""));
    }

    // Width available for content (subtract ` * ` prefix)
    let wrap_width = available_width.saturating_sub(LINE_PREFIX_LEN);

    let mut content_lines: Vec<String> = Vec::new();

    // Format description using mdast parsing (handles heading normalization,
    // emphasis conversion, horizontal rule removal, reference links, nested lists, etc.)
    let desc_trimmed = description.trim();
    if !desc_trimmed.is_empty() {
        let desc_lines =
            format_description_mdast(desc_trimmed, wrap_width, options.capitalize_descriptions);
        content_lines.extend(desc_lines);
    }

    // Sort tags by priority within groups.
    // @typedef and @callback are TAGS_GROUP_HEAD — they start new groups.
    // Tags sort within their group by weight, but groups keep their relative order.
    let tags = jsdoc.tags();
    let sorted_tags = sort_tags_by_groups(tags);

    // Collect effective tags, merging @description into the description area
    let mut effective_tags: Vec<(&oxc_jsdoc::parser::JSDocTag<'_>, &str)> = Vec::new();
    for tag in &sorted_tags {
        let raw_kind = tag.kind.parsed();
        let normalized_kind = normalize_tag_kind(raw_kind);
        if should_remove_empty_tag(normalized_kind) && !tag_has_content(tag) {
            continue;
        }
        // @description tag: merge its content into the main description
        if normalized_kind == "description" {
            let desc_content = tag.comment().parsed();
            let desc_content = desc_content.trim();
            if !desc_content.is_empty() {
                if !content_lines.is_empty() && !content_lines.last().is_some_and(String::is_empty)
                {
                    content_lines.push(String::new());
                }
                let desc_lines = format_description_mdast(
                    desc_content,
                    wrap_width,
                    options.capitalize_descriptions,
                );
                content_lines.extend(desc_lines);
            }
            continue;
        }
        effective_tags.push((tag, normalized_kind));
    }

    // Reorder @param tags to match the function signature order
    reorder_param_tags(&mut effective_tags, comment, source_text);

    // Pre-process @import tags: merge by module, sort, format
    let import_lines = process_import_tags(&effective_tags);
    let has_imports = !import_lines.is_empty();
    let mut imports_emitted = false;

    // Format tags
    let mut prev_normalized_kind: Option<&str> = None;
    for (tag_idx, &(tag, normalized_kind)) in effective_tags.iter().enumerate() {
        // Skip @import tags — they are handled via merged import_lines
        if normalized_kind == "import" {
            if has_imports && !imports_emitted {
                // Emit merged imports at the position of the first @import tag
                if !content_lines.is_empty() && !content_lines.last().is_some_and(String::is_empty)
                {
                    content_lines.push(String::new());
                }
                content_lines.extend(import_lines.clone());
                imports_emitted = true;
                prev_normalized_kind = Some("import");
            }
            continue;
        }

        let is_first_tag = tag_idx == 0 && !imports_emitted;

        let should_capitalize = options.capitalize_descriptions
            && !should_skip_capitalize(normalized_kind)
            && is_known_tag(normalized_kind);

        // Add blank line between description and first tag
        if is_first_tag
            && !content_lines.is_empty()
            && !content_lines.last().is_some_and(String::is_empty)
        {
            content_lines.push(String::new());
        }

        // Add blank lines between tag groups
        if !is_first_tag {
            let should_separate = if prev_normalized_kind.is_some_and(|prev| prev == "example")
                && normalized_kind == "example"
            {
                // Always blank line between consecutive @example tags
                true
            } else if options.separate_tag_groups {
                // Blank line between different tag kinds
                prev_normalized_kind.is_some_and(|prev| prev != normalized_kind)
            } else if options.separate_returns_from_param {
                // Only blank line before @returns/@yields (when coming from @param-like tags)
                matches!(normalized_kind, "returns" | "yields")
                    && prev_normalized_kind
                        .is_some_and(|prev| !matches!(prev, "returns" | "yields"))
            } else {
                // Default: blank line before compound tag groups (@typedef, @callback)
                // when coming from a different tag kind (but not from @import)
                matches!(normalized_kind, "typedef" | "callback")
                    && prev_normalized_kind
                        .is_some_and(|prev| !matches!(prev, "typedef" | "callback" | "import"))
            };

            if should_separate && !content_lines.last().is_some_and(String::is_empty) {
                content_lines.push(String::new());
            }
        }

        prev_normalized_kind = Some(normalized_kind);

        // Track content before formatting this tag
        let lines_before = content_lines.len();

        // Detect if original has no space between tag kind and `{type}`
        // e.g., `@type{import(...)}` vs `@type {import(...)}`
        let has_no_space_before_type = {
            let kind_end = tag.kind.span.end as usize;
            kind_end < source_text.len() && source_text.as_bytes()[kind_end] == b'{'
        };

        let bracket_spacing = options.bracket_spacing;

        if normalized_kind == "example" || normalized_kind == "remarks" {
            format_example_tag(
                normalized_kind,
                tag,
                wrap_width,
                format_options,
                &mut content_lines,
            );
        } else if is_type_name_comment_tag(normalized_kind) {
            format_type_name_comment_tag(
                normalized_kind,
                tag,
                should_capitalize,
                wrap_width,
                has_no_space_before_type,
                bracket_spacing,
                format_options,
                &mut content_lines,
            );
        } else if is_type_comment_tag(normalized_kind) {
            format_type_comment_tag(
                normalized_kind,
                tag,
                should_capitalize,
                wrap_width,
                has_no_space_before_type,
                bracket_spacing,
                format_options,
                &mut content_lines,
            );
        } else {
            format_generic_tag(
                normalized_kind,
                tag,
                should_capitalize,
                wrap_width,
                format_options.quote_style,
                &mut content_lines,
            );
        }

        // If this tag has multi-paragraph content (blank lines within, or is an @example tag
        // with multi-line code) and the next tag is of a different kind, add a trailing
        // blank line for separation.
        let tag_content_has_blank_lines =
            content_lines[lines_before..].iter().any(String::is_empty);
        let tag_content_lines = content_lines.len() - lines_before;
        let is_example_multiline = normalized_kind == "example" && tag_content_lines > 1;
        if (tag_content_has_blank_lines || is_example_multiline)
            && let Some(&(_, next_kind)) = effective_tags.get(tag_idx + 1)
            && next_kind != normalized_kind
            && !content_lines.last().is_some_and(String::is_empty)
        {
            content_lines.push(String::new());
        }
    }

    // Post-process: format code in fenced code blocks and indented code blocks.
    // format_indented_code_blocks must skip lines inside fenced code blocks and
    // @example tags, since those have already been formatted by their respective
    // processors and may contain 4+ space indentation that is NOT a markdown
    // indented code block.
    format_fenced_code_blocks(&mut content_lines, wrap_width, format_options, external_callbacks);
    format_indented_code_blocks(&mut content_lines, wrap_width, format_options);

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

    // Single-line check: convert to single-line if content is a single line.
    // The plugin prefers single-line even if it slightly exceeds printWidth,
    // since the wrapping logic already constrains the content width.
    if options.single_line_when_possible && content_lines.len() == 1 {
        let single = &content_lines[0];
        let formatted = format!("/** {single} */");
        if formatted == content {
            return None;
        }
        return Some(allocator.alloc_str(&formatted));
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

/// Wrap a long type expression across multiple lines at `|` operators.
/// Returns `None` if no wrapping is needed or the type can't be sensibly wrapped.
fn wrap_type_expression(
    tag_prefix: &str,
    type_str: &str,
    name_and_rest: &str,
    wrap_width: usize,
    content_lines: &mut Vec<String>,
) -> bool {
    // Only wrap if the full line exceeds the width
    let full_line = if name_and_rest.is_empty() {
        format!("{tag_prefix} {{{type_str}}}")
    } else {
        format!("{tag_prefix} {{{type_str}}} {name_and_rest}")
    };

    if full_line.len() <= wrap_width {
        return false;
    }

    // Check if the type contains `|` at the top level for union wrapping
    let parts = split_type_at_top_level_pipe(type_str);
    if parts.len() <= 1 {
        // Check for object type `{{ ... }}` wrapping
        if type_str.starts_with('{') && type_str.ends_with('}') {
            return wrap_object_type(
                tag_prefix,
                type_str,
                name_and_rest,
                wrap_width,
                content_lines,
            );
        }
        // Check for generic type `Foo<...>` wrapping at top-level angle bracket
        if let Some(wrapped) = wrap_generic_type(tag_prefix, type_str, name_and_rest, content_lines)
        {
            return wrapped;
        }
        return false;
    }

    // Wrap union type at `|` operators
    let indent = "  ";
    let first_part = parts[0].trim();
    content_lines.push(format!("{tag_prefix} {{{first_part}"));

    for (i, part) in parts.iter().enumerate().skip(1) {
        let part = part.trim();
        if i == parts.len() - 1 {
            // Last part: close the braces, include name on same line if present
            if name_and_rest.is_empty() {
                content_lines.push(format!("{indent}| {part}}}"));
            } else {
                content_lines.push(format!("{indent}| {part}}} {name_and_rest}"));
            }
        } else {
            content_lines.push(format!("{indent}| {part}"));
        }
    }

    true
}

/// Wrap an object type literal across multiple lines.
fn wrap_object_type(
    tag_prefix: &str,
    type_str: &str,
    name_and_rest: &str,
    _wrap_width: usize,
    content_lines: &mut Vec<String>,
) -> bool {
    // type_str is like `{ userId: string; title: string; ... }`
    let inner = type_str[1..type_str.len() - 1].trim();
    if inner.is_empty() {
        return false;
    }

    // Split at `;` or `,` while respecting nested brackets
    let fields = split_object_fields(inner);
    if fields.len() <= 1 {
        return false;
    }

    let indent = "  ";
    // First line: tag + opening brace
    content_lines.push(format!("{tag_prefix} {{{{"));

    // Each field on its own line, always using semicolons (matching TS convention)
    for field in &fields {
        let field = field.trim();
        if field.is_empty() {
            continue;
        }
        // Normalize delimiter: strip trailing `,` or `;` and always use `;`
        let field = field.strip_suffix(',').or_else(|| field.strip_suffix(';')).unwrap_or(field);
        // Normalize field value: `*` → `any`
        let field = normalize_object_field(field);
        // Check if the field value is a nested object that should be expanded
        if let Some((key, nested_inner)) = extract_nested_object(&field) {
            let nested_fields = split_object_fields(nested_inner);
            if nested_fields.len() > 1 {
                // Recursively format nested object
                let nested_indent = format!("{indent}  ");
                content_lines.push(format!("{indent}{key}: {{"));
                for nf in &nested_fields {
                    let nf = nf.trim();
                    if nf.is_empty() {
                        continue;
                    }
                    let nf = nf.strip_suffix(',').or_else(|| nf.strip_suffix(';')).unwrap_or(nf);
                    let nf = normalize_object_field(nf);
                    content_lines.push(format!("{nested_indent}{}", field_with_semicolon(&nf)));
                }
                content_lines.push(format!("{indent}}};"));
                continue;
            }
        }
        content_lines.push(format!("{indent}{}", field_with_semicolon(&field)));
    }

    // Closing brace + name
    if name_and_rest.is_empty() {
        content_lines.push("}}".to_string());
    } else {
        content_lines.push(format!("}}}} {name_and_rest}"));
    }

    true
}

/// Append a semicolon to a field, inserting it before any `// comment` suffix.
/// `"foo: string // comment"` → `"foo: string; // comment"`
/// `"foo: string"` → `"foo: string;"`
fn field_with_semicolon(field: &str) -> String {
    // Find `//` that's not inside brackets or quotes
    if let Some(comment_pos) = find_line_comment(field) {
        let before = field[..comment_pos].trim_end();
        let comment = &field[comment_pos..];
        format!("{before}; {comment}")
    } else {
        format!("{field};")
    }
}

/// Find position of a `// ` line comment in a field string, skipping nested contexts.
fn find_line_comment(field: &str) -> Option<usize> {
    let bytes = field.as_bytes();
    let len = bytes.len();
    let mut depth = 0i32;
    let mut i = 0;
    while i < len {
        match bytes[i] {
            b'(' | b'<' | b'[' | b'{' => depth += 1,
            b')' | b'>' | b']' | b'}' => depth = depth.saturating_sub(1),
            b'"' | b'\'' => {
                let q = bytes[i];
                i += 1;
                while i < len && bytes[i] != q {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'/' if depth == 0 && i + 1 < len && bytes[i + 1] == b'/' => {
                return Some(i);
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Normalize a single object field's value: `*` → `any`, whitespace cleanup.
fn normalize_object_field(field: &str) -> String {
    if let Some(colon_pos) = find_field_colon(field) {
        let key = field[..colon_pos].trim();
        let value = field[colon_pos + 1..].trim();
        let normalized_value =
            if value == "*" { "any".to_string() } else { normalize_type_whitespace(value) };
        // Preserve inline comments
        format!("{key}: {normalized_value}")
    } else {
        field.to_string()
    }
}

/// Find the position of the `:` that separates a field name from its value.
/// Skips `:` inside nested brackets and quoted strings.
fn find_field_colon(field: &str) -> Option<usize> {
    let mut depth = 0i32;
    let bytes = field.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' | b'<' | b'[' | b'{' => depth += 1,
            b')' | b'>' | b']' | b'}' => depth -= 1,
            b':' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Check if a field value is a nested object `{ ... }` and extract the key and inner content.
fn extract_nested_object(field: &str) -> Option<(&str, &str)> {
    let colon_pos = find_field_colon(field)?;
    let key = field[..colon_pos].trim();
    let value = field[colon_pos + 1..].trim();
    if value.starts_with('{') && value.ends_with('}') {
        let inner = value[1..value.len() - 1].trim();
        Some((key, inner))
    } else {
        None
    }
}

/// Wrap a generic type `Foo<Bar>` across multiple lines at the top-level angle bracket.
/// Only wraps if the inner content is long enough to justify multi-line formatting.
/// Expected output format:
/// ```text
/// @returns {import("axios").AxiosResponse<
///   import("../types").ResellerUserIntroduced[]
/// >}
/// ```
fn wrap_generic_type(
    tag_prefix: &str,
    type_str: &str,
    name_and_rest: &str,
    content_lines: &mut Vec<String>,
) -> Option<bool> {
    // Find the first top-level `<` (depth 0)
    let mut depth = 0i32;
    let mut angle_pos = None;
    for (i, ch) in type_str.char_indices() {
        match ch {
            '<' if depth == 0 => {
                angle_pos = Some(i);
                break;
            }
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            _ => {}
        }
    }

    let angle_pos = angle_pos?;

    // The type must end with `>` for this wrapping to apply
    if !type_str.ends_with('>') {
        return None;
    }

    let prefix_part = &type_str[..=angle_pos]; // includes the `<`
    let inner = type_str[angle_pos + 1..type_str.len() - 1].trim(); // content between < and >

    if inner.is_empty() {
        return None;
    }

    // Only wrap if the inner content is substantial enough to justify wrapping.
    // Short inner types like `number` shouldn't trigger wrapping.
    if inner.len() < 20 {
        return None;
    }

    let indent = "  ";
    // First line: tag + opening part including <
    content_lines.push(format!("{tag_prefix} {{{prefix_part}"));
    // Inner content with 2-space indent
    content_lines.push(format!("{indent}{inner}"));
    // Closing >} with optional name
    if name_and_rest.is_empty() {
        content_lines.push(">}".to_string());
    } else {
        content_lines.push(format!(">}} {name_and_rest}"));
    }

    Some(true)
}

/// Split a type string at top-level `|` operators (not inside `<>`, `()`, `{}`, `[]`).
fn split_type_at_top_level_pipe(type_str: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;

    for (i, ch) in type_str.char_indices() {
        match ch {
            '(' | '<' | '[' | '{' => depth += 1,
            ')' | '>' | ']' | '}' => depth -= 1,
            '|' if depth == 0 => {
                parts.push(&type_str[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&type_str[start..]);
    parts
}

/// Split an object type's inner content at `;` or `,` delimiters, respecting nesting.
/// Also handles `// ...` line comments by preserving them with the preceding field.
fn split_object_fields(inner: &str) -> Vec<String> {
    let mut fields: Vec<String> = Vec::new();
    let bytes = inner.as_bytes();
    let len = bytes.len();
    let mut depth = 0i32;
    let mut start = 0;
    let mut i = 0;
    // Pending inline comment to attach to the preceding field
    let mut pending_comment: Option<String> = None;

    while i < len {
        match bytes[i] {
            b'(' | b'<' | b'[' | b'{' => {
                depth += 1;
                i += 1;
            }
            b')' | b'>' | b']' | b'}' => {
                depth -= 1;
                i += 1;
            }
            b'/' if depth == 0 && i + 1 < len && bytes[i + 1] == b'/' => {
                // Line comment: capture text from `//` to end of line
                let comment_start = i;
                while i < len && bytes[i] != b'\n' {
                    i += 1;
                }
                pending_comment = Some(inner[comment_start..i].trim().to_string());
                if i < len {
                    i += 1; // skip newline
                }
                // Skip past the comment so it's not included in the next field
                start = i;
            }
            b';' | b',' if depth == 0 => {
                let field = inner[start..i].trim().to_string();
                if !field.is_empty() {
                    // Attach any pending inline comment to the previous field
                    if let Some(comment) = pending_comment.take()
                        && let Some(last) = fields.last_mut()
                    {
                        last.push(' ');
                        last.push_str(&comment);
                    }
                    fields.push(field);
                } else if let Some(comment) = pending_comment.take() {
                    // Field text was empty (comment was between two delimiters)
                    // Attach to previous field if available
                    if let Some(last) = fields.last_mut() {
                        last.push(' ');
                        last.push_str(&comment);
                    }
                }
                start = i + 1;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    // Attach any trailing pending comment
    if let Some(comment) = pending_comment.take()
        && let Some(last) = fields.last_mut()
    {
        last.push(' ');
        last.push_str(&comment);
    }

    let last = inner[start..].trim();
    if !last.is_empty() {
        fields.push(last.to_string());
    }
    fields
}

/// Format a `@default` / `@defaultValue` value.
/// Handles JSON-like formatting: spaces after `:` and `,`, inside `{}`.
/// Converts quotes based on the `quote_style` option.
/// Non-JSON values (code, plain text) are returned as-is.
fn format_default_value(value: &str, quote_style: QuoteStyle) -> String {
    let trimmed = value.trim();
    // Detect if value looks like JSON/object/array literal
    let first_char = trimmed.chars().next().unwrap_or(' ');
    if !matches!(first_char, '{' | '[' | '"' | '\'') {
        // Doesn't start with JSON-like syntax; return unchanged
        return trimmed.to_string();
    }

    // Determine target and source quote characters based on quote style.
    let (target_quote, other_quote) = match quote_style {
        QuoteStyle::Double => ('"', '\''),
        QuoteStyle::Single => ('\'', '"'),
    };

    // Format JSON-like values: normalize spacing around `:`, `,`, `{`, `}`, `[`
    // and convert quotes based on the quote_style option.
    let mut result = String::with_capacity(trimmed.len() + 16);
    let chars: Vec<char> = trimmed.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_target_quote = false;
    let mut in_other_quote = false;

    while i < len {
        let ch = chars[i];

        if in_target_quote {
            result.push(ch);
            if ch == target_quote && (i == 0 || chars[i - 1] != '\\') {
                in_target_quote = false;
            }
            i += 1;
            continue;
        }

        if in_other_quote {
            if ch == other_quote && (i == 0 || chars[i - 1] != '\\') {
                result.push(target_quote); // Close with target quote
                in_other_quote = false;
            } else {
                result.push(ch);
            }
            i += 1;
            continue;
        }

        match ch {
            c if c == target_quote => {
                result.push(target_quote);
                in_target_quote = true;
            }
            c if c == other_quote => {
                result.push(target_quote); // Open with target quote
                in_other_quote = true;
            }
            ':' => {
                result.push(':');
                // Add space after `:` if not already there
                if i + 1 < len && chars[i + 1] != ' ' {
                    result.push(' ');
                }
            }
            ',' => {
                result.push(',');
                // Add space after `,` if not already there
                if i + 1 < len && chars[i + 1] != ' ' {
                    result.push(' ');
                }
            }
            '{' => {
                result.push('{');
                // Add space after `{` if next char is not `}` and not already a space
                if i + 1 < len && chars[i + 1] != '}' && chars[i + 1] != ' ' {
                    result.push(' ');
                }
            }
            '}' => {
                // Add space before `}` if previous char is not `{` and not already a space
                if !result.is_empty() {
                    let last = result.chars().last().unwrap_or(' ');
                    if last != '{' && last != ' ' {
                        result.push(' ');
                    }
                }
                result.push('}');
            }
            '[' => {
                result.push('[');
                // Add space after `[` if next char is `]` (empty array special case: `[ ]`)
                if i + 1 < len && chars[i + 1] == ']' {
                    result.push(' ');
                }
            }
            _ => {
                result.push(ch);
            }
        }
        i += 1;
    }
    result
}

/// Strip an existing "Default is `...`" or "Default is ..." suffix from a description.
/// The plugin always recomputes this from the `[name=value]` syntax.
fn strip_default_is_suffix(desc: &str) -> String {
    // Look for "Default is " (case insensitive matching for "default is")
    if let Some(pos) = desc.find("Default is ") {
        let before = desc[..pos].trim_end();
        // Remove trailing period before "Default is"
        let before = before.strip_suffix('.').unwrap_or(before);
        before.trim_end().to_string()
    } else {
        desc.to_string()
    }
}

/// Post-process content lines to format code inside fenced code blocks with language tags.
/// Map fenced code block language tags to external formatter language identifiers.
/// Returns `None` if the language should be handled by the native JS/TS formatter.
fn fenced_lang_to_external_language(lang: &str) -> Option<&'static str> {
    match lang {
        "css" | "scss" | "less" => Some("tagged-css"),
        "html" => Some("tagged-html"),
        "graphql" | "gql" => Some("tagged-graphql"),
        "markdown" | "md" | "mdx" => Some("tagged-markdown"),
        _ => None,
    }
}

/// Returns `true` if the fenced code block language is JS/TS/JSX/TSX.
fn is_js_ts_lang(lang: &str) -> bool {
    matches!(lang, "js" | "javascript" | "jsx" | "ts" | "typescript" | "tsx")
}

/// Finds fenced code blocks (```lang ... ```) and reformats them.
/// JS/TS/JSX/TSX code is formatted natively; other supported languages
/// (CSS, HTML, GraphQL, Markdown) are delegated to the external formatter
/// (Prettier) via the `external_callbacks`.
fn format_fenced_code_blocks(
    content_lines: &mut Vec<String>,
    wrap_width: usize,
    format_options: &FormatOptions,
    external_callbacks: &ExternalCallbacks,
) {
    let mut i = 0;
    while i < content_lines.len() {
        let line = &content_lines[i];
        // Look for opening code fence with a language tag
        if line.starts_with("```") && line.len() > 3 {
            let lang = line[3..].trim().to_string();

            // Normalize spacing: ``` js → ```js
            if content_lines[i][3..].starts_with(' ') && !lang.is_empty() {
                content_lines[i] = format!("```{lang}");
            }

            let is_js = is_js_ts_lang(&lang);
            let external_lang = fenced_lang_to_external_language(&lang);

            if !is_js && external_lang.is_none() {
                i += 1;
                continue;
            }

            // Find closing code fence
            let start = i + 1;
            let end = content_lines[start..].iter().position(|l| l == "```").map(|pos| start + pos);

            let Some(end_idx) = end else {
                i += 1;
                continue;
            };

            // Extract code content
            let code: String = content_lines[start..end_idx]
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join("\n");

            // Try to format: native for JS/TS, external for other languages.
            // The upstream plugin reduces print width by 4 for code inside fenced blocks.
            let code_width = wrap_width.saturating_sub(4);
            let formatted = if is_js {
                format_embedded_js(&code, code_width, format_options)
            } else if let Some(ext_lang) = external_lang {
                format_external_language(&code, ext_lang, code_width, external_callbacks)
            } else {
                None
            };

            if let Some(formatted) = formatted {
                // Replace the code lines with formatted output
                let new_lines: Vec<String> = formatted.lines().map(String::from).collect();
                // Remove old code lines and insert new ones
                let range = start..end_idx;
                content_lines.splice(range, new_lines.clone());
                // Adjust index past the new content + closing fence
                i = start + new_lines.len() + 1;
            } else {
                i = end_idx + 1;
            }
        } else {
            i += 1;
        }
    }
}

/// Format code using the external formatter (Prettier) for non-JS/TS languages.
/// Returns `Some(formatted)` on success, `None` if no callback is available or formatting fails.
fn format_external_language(
    code: &str,
    language: &str,
    _wrap_width: usize,
    external_callbacks: &ExternalCallbacks,
) -> Option<String> {
    let result = external_callbacks.format_embedded(language, code)?;
    match result {
        Ok(formatted) => Some(formatted.trim_end().to_string()),
        Err(_) => None,
    }
}

/// Count unescaped backticks on a line and update template literal depth.
/// Returns the new depth after processing the line.
fn update_template_depth(line: &str, mut depth: u32) -> u32 {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2; // skip escaped character
            continue;
        }
        if bytes[i] == b'`' {
            if depth == 0 {
                depth += 1;
            } else {
                depth -= 1;
            }
        }
        i += 1;
    }
    depth
}

/// Post-process content lines to format indented code blocks (4-space indented).
/// These are blocks of consecutive lines starting with 4+ spaces, typically
/// between blank lines in description text (markdown indented code blocks).
///
/// Skips lines inside fenced code blocks (``` ... ```) and @example tag
/// content, since those regions may contain 4+ space indentation from the
/// formatter's own output that should not be re-processed.
fn format_indented_code_blocks(
    content_lines: &mut Vec<String>,
    wrap_width: usize,
    format_options: &FormatOptions,
) {
    let mut i = 0;
    let mut in_fenced_block = false;
    let mut in_example_tag = false;
    while i < content_lines.len() {
        let line = &content_lines[i];

        // Track fenced code blocks so we don't re-format code inside them
        if line.starts_with("```") {
            in_fenced_block = !in_fenced_block;
            i += 1;
            continue;
        }
        if in_fenced_block {
            i += 1;
            continue;
        }

        // Track @example tag regions: starts with "@example" line,
        // continues through indented content until a blank line or new tag
        if line.starts_with('@') {
            in_example_tag = line.starts_with("@example");
            i += 1;
            continue;
        }
        if in_example_tag {
            // @example content continues until a blank line or a non-indented line
            if line.is_empty() {
                in_example_tag = false;
            }
            i += 1;
            continue;
        }

        if content_lines[i].starts_with("    ") {
            // Found start of indented code block
            let start = i;
            while i < content_lines.len()
                && (content_lines[i].starts_with("    ") || content_lines[i].is_empty())
            {
                i += 1;
            }
            // Don't include trailing empty lines as part of the code block
            while i > start && content_lines[i - 1].is_empty() {
                i -= 1;
            }
            let end = i;

            if start >= end {
                continue;
            }

            // Skip blocks that look like nested list items (not code)
            let looks_like_list = content_lines[start..end].iter().any(|l| {
                let trimmed = l.trim();
                !trimmed.is_empty()
                    && (trimmed.starts_with("- ")
                        || trimmed.starts_with("* ")
                        || trimmed.starts_with("+ ")
                        || trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
                            && (trimmed.contains(". ") || trimmed.contains("- ")))
            });
            if looks_like_list {
                continue;
            }

            // Extract code content (strip 4-space prefix)
            let code: String = content_lines[start..end]
                .iter()
                .map(|l| l.strip_prefix("    ").unwrap_or(l.as_str()))
                .collect::<Vec<_>>()
                .join("\n");

            // Try to format (effective width = wrap_width - 4 for the indent)
            let effective_width = wrap_width.saturating_sub(4);
            if let Some(formatted) = format_embedded_js(&code, effective_width, format_options) {
                let new_lines: Vec<String> =
                    formatted.lines().map(|l| format!("    {l}")).collect();
                let range = start..end;
                let new_len = new_lines.len();
                content_lines.splice(range, new_lines);
                i = start + new_len;
            }
        } else {
            i += 1;
        }
    }
}

/// Try to format JS/TS/JSX code using the formatter.
/// Returns `Some(formatted)` on success, `None` if parsing fails.
/// The `print_width` is the available width for the formatted code.
/// Uses the parent `format_options` to ensure consistent formatting behavior.
fn format_embedded_js(
    code: &str,
    print_width: usize,
    format_options: &FormatOptions,
) -> Option<String> {
    let line_width = LineWidth::try_from(u16::try_from(print_width).unwrap_or(80)).unwrap();

    // Build options from parent, overriding line_width and disabling JSDoc
    // to prevent recursive formatting
    let make_options = || FormatOptions { line_width, jsdoc: None, ..format_options.clone() };

    // Try to parse and format with the given source type
    let try_format = |code: &str, source_type: SourceType| -> Option<String> {
        let allocator = Allocator::default();
        let ret =
            Parser::new(&allocator, code, source_type).with_options(get_parse_options()).parse();
        if ret.panicked || !ret.errors.is_empty() {
            return None;
        }
        let formatted = Formatter::new(&allocator, make_options()).build(&ret.program);
        Some(formatted.trim_end().to_string())
    };

    // Try JSX first (most @example code in React projects uses JSX),
    // then TSX (for TypeScript code with JSX).
    if let Some(result) = try_format(code, SourceType::jsx()) {
        return Some(result);
    }
    if let Some(result) = try_format(code, SourceType::tsx()) {
        return Some(result);
    }

    // If direct parsing fails, try wrapping in expression context
    // to handle object literals like `{ "key": value }` that parse as blocks
    let trimmed = code.trim();
    if trimmed.starts_with('{') {
        let wrapped = format!("({trimmed})");

        let try_format_obj = |code: &str, source_type: SourceType| -> Option<String> {
            let allocator = Allocator::default();
            let ret = Parser::new(&allocator, code, source_type)
                .with_options(get_parse_options())
                .parse();
            if ret.panicked || !ret.errors.is_empty() {
                return None;
            }
            // Use TrailingCommas::None for object literals since JSON-like code
            // shouldn't have trailing commas
            let options = FormatOptions { trailing_commas: TrailingCommas::None, ..make_options() };
            let formatted = Formatter::new(&allocator, options).build(&ret.program);
            let formatted = formatted.trim_end();
            // Remove the wrapping parens and trailing semicolon
            if let Some(inner) = formatted.strip_prefix('(')
                && let Some(inner) = inner.strip_suffix(");")
            {
                return Some(inner.to_string());
            }
            Some(formatted.to_string())
        };

        if let Some(result) = try_format_obj(&wrapped, SourceType::jsx()) {
            return Some(result);
        }
        if let Some(result) = try_format_obj(&wrapped, SourceType::tsx()) {
            return Some(result);
        }
    }
    None
}

/// Format a JSDoc type expression using the formatter (simulating upstream's `formatType()`).
///
/// Wraps the type as `type __t = {type_str};`, parses as TSX, formats, then extracts
/// the formatted type. Handles `...Type` rest params by formatting the inner type
/// separately. Returns `None` on parse/format failure.
fn format_type_via_formatter(type_str: &str, format_options: &FormatOptions) -> Option<String> {
    if type_str.is_empty() {
        return None;
    }

    // Handle rest/spread prefix: convert `...Type` to `(Type)[]`, format, then strip
    // the trailing `[]` and prepend `...`. Same approach as upstream's `formatType()`.
    if let Some(rest) = type_str.strip_prefix("...") {
        let rest = rest.trim_start();
        if rest.is_empty() {
            return None;
        }
        let wrapped = format!("({rest})[]");
        let formatted = format_type_via_formatter(&wrapped, format_options)?;
        let inner = formatted.strip_suffix("[]")?;
        return Some(format!("...{inner}"));
    }

    let input = format!("type __t = {type_str};");

    let allocator = Allocator::default();
    let line_width = LineWidth::try_from(80u16).unwrap();
    let options = FormatOptions { line_width, jsdoc: None, ..format_options.clone() };

    let ret = Parser::new(&allocator, &input, SourceType::tsx())
        .with_options(get_parse_options())
        .parse();
    if ret.panicked || !ret.errors.is_empty() {
        return None;
    }

    let formatted = Formatter::new(&allocator, options).build(&ret.program);
    let formatted = formatted.trim_end();

    // Strip the `type __t = ` prefix and trailing `;`
    let result = formatted.strip_prefix("type __t = ")?.strip_suffix(';')?.trim();

    Some(result.to_string())
}

/// Format example code content with 2-space base indent.
/// Tries to format the code as JS/JSX first; falls back to pass-through on parse failure.
fn format_example_code(
    code: &str,
    wrap_width: usize,
    format_options: &FormatOptions,
    content_lines: &mut Vec<String>,
) {
    if code.is_empty() {
        return;
    }

    // Check for fenced code blocks (```lang ... ```). Triple backticks are
    // actually valid JavaScript (template literal expressions), so
    // `format_embedded_js` would parse them as JS and produce wrong output.
    // Handle fenced blocks by stripping the markers, formatting just the
    // inner code, and re-adding the fences with proper indentation.
    let code_lines: Vec<&str> = code.lines().collect();
    if code_lines.len() >= 2
        && code_lines[0].starts_with("```")
        && code_lines.last().is_some_and(|l| l.trim() == "```")
    {
        format_example_fenced_block(&code_lines, wrap_width, format_options, content_lines);
        return;
    }

    // Try formatting the code. The effective print width for @example code is
    // wrap_width - 2 (for the 2-space indent within the comment).
    let effective_width = wrap_width.saturating_sub(2);
    if let Some(formatted) = format_embedded_js(code, effective_width, format_options) {
        // Add 2-space indent to code structure lines, but NOT to template literal
        // content. The formatter preserves template literal content verbatim, so
        // adding indent to those lines would shift them incorrectly.
        let mut template_depth: u32 = 0;
        for line in formatted.lines() {
            if line.is_empty() {
                content_lines.push(String::new());
            } else if template_depth == 0 {
                content_lines.push(format!("  {line}"));
            } else {
                content_lines.push(line.to_string());
            }
            // Count unescaped backticks to track template literal depth
            template_depth = update_template_depth(line, template_depth);
        }
        return;
    }

    // Fallback: pass through with 2-space indent
    for line in code.lines() {
        let line_trimmed = line.trim();
        if line_trimmed.is_empty() {
            content_lines.push(String::new());
        } else {
            content_lines.push(format!("  {line_trimmed}"));
        }
    }
}

/// Handle fenced code blocks inside @example tags.
/// Strips the ``` markers, formats the inner code, and re-adds fences
/// with proper 2-space indentation.
fn format_example_fenced_block(
    code_lines: &[&str],
    wrap_width: usize,
    format_options: &FormatOptions,
    content_lines: &mut Vec<String>,
) {
    let lang_line = code_lines[0];
    let inner_code: String = code_lines[1..code_lines.len() - 1].to_vec().join("\n");
    let effective_width = wrap_width.saturating_sub(2);

    // Add opening fence with indent
    content_lines.push(format!("  {lang_line}"));

    if !inner_code.is_empty() {
        let lang = lang_line[3..].trim();
        if is_js_ts_lang(lang) {
            if let Some(formatted) =
                format_embedded_js(&inner_code, effective_width, format_options)
            {
                let mut template_depth: u32 = 0;
                for line in formatted.lines() {
                    if line.is_empty() {
                        content_lines.push(String::new());
                    } else if template_depth == 0 {
                        content_lines.push(format!("  {line}"));
                    } else {
                        content_lines.push(line.to_string());
                    }
                    template_depth = update_template_depth(line, template_depth);
                }
            } else {
                // Fallback for unparseable inner code
                for line in inner_code.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        content_lines.push(String::new());
                    } else {
                        content_lines.push(format!("  {trimmed}"));
                    }
                }
            }
        } else {
            // Non-JS/TS fenced code: preserve with 2-space indent
            for line in inner_code.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    content_lines.push(String::new());
                } else {
                    content_lines.push(format!("  {trimmed}"));
                }
            }
        }
    }

    // Add closing fence with indent
    content_lines.push(format!("  {}", code_lines.last().unwrap().trim()));
}

fn format_example_tag(
    normalized_kind: &str,
    tag: &oxc_jsdoc::parser::JSDocTag<'_>,
    wrap_width: usize,
    format_options: &FormatOptions,
    content_lines: &mut Vec<String>,
) {
    let comment_part = tag.comment();
    let raw_text = comment_part.parsed_preserving_whitespace();
    let trimmed = raw_text.trim();

    // Check for <caption>...</caption> at the start — keep inline with @example
    if let Some(rest) = trimmed.strip_prefix("<caption>")
        && let Some(end_pos) = rest.find("</caption>")
    {
        let caption = &rest[..end_pos];
        let after_caption = rest[end_pos + "</caption>".len()..].trim();
        content_lines.push(format!("@{normalized_kind} <caption>{caption}</caption>"));
        format_example_code(after_caption, wrap_width, format_options, content_lines);
        return;
    }

    content_lines.push(format!("@{normalized_kind}"));
    format_example_code(trimmed, wrap_width, format_options, content_lines);
}

fn format_type_name_comment_tag(
    normalized_kind: &str,
    tag: &oxc_jsdoc::parser::JSDocTag<'_>,
    should_capitalize: bool,
    wrap_width: usize,
    has_no_space_before_type: bool,
    bracket_spacing: bool,
    format_options: &FormatOptions,
    content_lines: &mut Vec<String>,
) {
    let (type_part, name_part, comment_part) = tag.type_name_comment();

    let tag_prefix = format!("@{normalized_kind}");
    let mut is_type_optional = false;
    let mut normalized_type_str = String::new();

    // When original has no space before `{type}` (e.g., `@typedef{import(...)}`),
    // preserve original quotes — the plugin treats this as a raw type annotation.
    let preserve_quotes = has_no_space_before_type;

    if let Some(tp) = &type_part {
        let raw_type = tp.parsed();
        if !raw_type.is_empty() {
            let (type_to_normalize, type_optional) = strip_optional_type_suffix(raw_type);
            is_type_optional = type_optional;
            normalized_type_str = if preserve_quotes {
                normalize_type_preserve_quotes(type_to_normalize)
            } else {
                normalize_type(type_to_normalize)
            };
            // Try formatting via the formatter (simulates upstream's formatType())
            if !preserve_quotes
                && let Some(formatted) =
                    format_type_via_formatter(&normalized_type_str, format_options)
            {
                normalized_type_str = formatted;
            }
        }
    }

    // Build name string and extract default value
    let mut name_str = String::new();
    let mut default_value: Option<String> = None;
    if let Some(np) = &name_part {
        let name_raw = np.raw();
        if is_type_optional && !name_raw.starts_with('[') {
            name_str = format!("[{name_raw}]");
        } else if name_raw.starts_with('[') && name_raw.ends_with(']') {
            if let Some(eq_pos) = name_raw.find('=') {
                let name_part_inner = &name_raw[1..eq_pos];
                let val = name_raw[eq_pos + 1..name_raw.len() - 1].trim();
                if val.is_empty() {
                    name_str = format!("[{name_part_inner}]");
                } else {
                    default_value = Some(val.to_string());
                    name_str = format!("[{name_part_inner}={val}]");
                }
            } else {
                name_str = name_raw.to_string();
            }
        } else {
            name_str = name_raw.to_string();
        }
    }

    // Build the full tag line
    let mut tag_line = tag_prefix.clone();
    if !normalized_type_str.is_empty() {
        let preserve_no_space = has_no_space_before_type && !normalized_type_str.starts_with('{');
        let space = if preserve_no_space { "" } else { " " };
        let (ob, cb) = if bracket_spacing { ("{ ", " }") } else { ("{", "}") };
        write!(tag_line, "{space}{ob}{normalized_type_str}{cb}").unwrap();
    }
    if !name_str.is_empty() {
        write!(tag_line, " {name_str}").unwrap();
    }

    let desc_raw = comment_part.parsed_preserving_whitespace();
    let desc_raw = desc_raw.trim();
    let desc_normalized = normalize_markdown_emphasis(desc_raw);
    let desc_raw = desc_normalized.trim();

    // Strip existing "Default is ..." from description when we have an actual default value
    let desc_raw = if default_value.is_some() {
        strip_default_is_suffix(desc_raw)
    } else {
        desc_raw.to_string()
    };
    let desc_raw = desc_raw.trim();

    if desc_raw.is_empty() && default_value.is_none() {
        // Try type wrapping if line is too long
        if tag_line.len() > wrap_width
            && !normalized_type_str.is_empty()
            && wrap_type_expression(
                &tag_prefix,
                &normalized_type_str,
                &name_str,
                wrap_width,
                content_lines,
            )
        {
            return;
        }
        content_lines.push(tag_line);
        return;
    }

    // Extract first text line from description (before any structural content)
    let desc_lines_raw: Vec<&str> = desc_raw.lines().collect();
    let first_text_line = desc_lines_raw.first().map_or("", |s| s.trim());

    // If the description starts with a code fence, output the tag line alone
    // and treat the entire description as structural content with a blank line separator
    if first_text_line.starts_with("```") {
        content_lines.push(tag_line);
        content_lines.push(String::new());
        let indent = if matches!(normalized_kind, "typedef" | "callback") { "" } else { "  " };
        let indent_width = wrap_width.saturating_sub(indent.len());
        let mut desc_lines = Vec::new();
        wrap_text(desc_raw, indent_width, &mut desc_lines);
        // Skip leading blank line from wrap_text since we already added one
        let start = usize::from(desc_lines.first().is_some_and(String::is_empty));
        for line in &desc_lines[start..] {
            if line.is_empty() {
                content_lines.push(String::new());
            } else {
                content_lines.push(format!("{indent}{line}"));
            }
        }
        return;
    }

    // Check if first line starts with a dash
    let (has_dash, first_text) = if let Some(rest) = first_text_line.strip_prefix("- ") {
        (true, rest)
    } else if first_text_line == "-" {
        (true, "")
    } else {
        (false, first_text_line)
    };

    let first_text =
        if should_capitalize { capitalize_first(first_text) } else { first_text.to_string() };

    // Build the default value suffix
    let default_suffix = default_value.as_ref().map(|dv| format!("Default is `{dv}`"));

    if first_text.is_empty() && default_suffix.is_none() && desc_lines_raw.len() <= 1 {
        content_lines.push(tag_line);
        return;
    }

    // Build the separator between tag+name and description
    let separator = if has_dash { " - " } else { " " };

    // Check if the description has extra content beyond the first text line
    // (subsequent lines with text, tables, code blocks, etc.)
    // Strip the common leading whitespace from continuation lines — this is
    // just the original JSDoc formatting indent, not semantic content.
    let remaining_desc = if desc_lines_raw.len() > 1 {
        let rest_lines: Vec<&str> = desc_lines_raw[1..].iter().map(|s| s.trim()).collect();
        rest_lines.join("\n")
    } else {
        String::new()
    };
    let has_remaining = !remaining_desc.trim().is_empty();

    // Check if everything fits on one line
    let one_liner = if has_remaining {
        format!("{tag_line}{separator}{first_text}")
    } else if let Some(ref ds) = default_suffix {
        if first_text.is_empty() {
            format!("{tag_line}{separator}{ds}")
        } else {
            let mut d = first_text.clone();
            let last_char = d.chars().last().unwrap_or(' ');
            if !matches!(last_char, '.' | '!' | '?') {
                d.push('.');
            }
            format!("{tag_line}{separator}{d} {ds}")
        }
    } else {
        format!("{tag_line}{separator}{first_text}")
    };

    if !has_remaining && one_liner.len() <= wrap_width {
        content_lines.push(one_liner);
    } else {
        // Multi-line: wrap first text line with tag line
        let first_line_prefix = format!("{tag_line}{separator}");
        let first_line_content_width = wrap_width.saturating_sub(first_line_prefix.len());

        let words: Vec<&str> = tokenize_words(&first_text);
        let mut first_line = String::new();
        let mut remaining_start = 0;

        for (i, word) in words.iter().enumerate() {
            if first_line.is_empty() {
                if word.len() <= first_line_content_width {
                    first_line.push_str(word);
                    remaining_start = i + 1;
                } else {
                    break;
                }
            } else if first_line.len() + 1 + word.len() <= first_line_content_width {
                first_line.push(' ');
                first_line.push_str(word);
                remaining_start = i + 1;
            } else {
                break;
            }
        }

        if first_line.is_empty() {
            content_lines.push(tag_line);
        } else {
            content_lines.push(format!("{first_line_prefix}{first_line}"));
        }

        // @typedef/@callback descriptions use no indent (plugin passes no beginningSpace).
        // @param/@property and other tags use 2-space continuation indent.
        let indent = if matches!(normalized_kind, "typedef" | "callback") { "" } else { "  " };
        let indent_width = wrap_width.saturating_sub(indent.len());

        // Remaining words from first text line
        let mut remaining_first_text = String::new();
        if remaining_start < words.len() {
            remaining_first_text = words[remaining_start..].join(" ");
        }

        // Combine remaining first text with remaining description lines
        // to preserve structural content (tables, code blocks, etc.)
        let full_remaining = if !remaining_first_text.is_empty() && has_remaining {
            format!("{remaining_first_text}\n{remaining_desc}")
        } else if !remaining_first_text.is_empty() {
            remaining_first_text
        } else {
            remaining_desc
        };

        if !full_remaining.trim().is_empty() {
            let mut desc_lines = Vec::new();
            wrap_text(full_remaining.trim(), indent_width, &mut desc_lines);

            // In markdown, a list or table after a paragraph needs a blank line separator.
            // The plugin's markdown AST processing (remark) handles this naturally.
            // We detect when the first wrapped content is a list item or table row
            // and insert a blank line between the tag's first text line and the
            // structured content.
            let first_desc_is_structural = desc_lines.first().is_some_and(|first| {
                let t = first.trim();
                t.starts_with("- ")
                    || t.starts_with("* ")
                    || t.starts_with("+ ")
                    || t.starts_with('|')
                    || (t.len() > 2
                        && t.chars().next().is_some_and(|c| c.is_ascii_digit())
                        && t.contains(". "))
            });
            if first_desc_is_structural && !first_line.is_empty() {
                content_lines.push(String::new());
            }

            for dl in desc_lines {
                if dl.is_empty() {
                    content_lines.push(String::new());
                } else {
                    content_lines.push(format!("{indent}{dl}"));
                }
            }
        }

        // Add default value as a separate paragraph with blank line
        if let Some(ref ds) = default_suffix
            && !first_text.is_empty()
        {
            content_lines.push(String::new());
            content_lines.push(format!("{indent}{ds}"));
        }
    }
}

fn format_type_comment_tag(
    normalized_kind: &str,
    tag: &oxc_jsdoc::parser::JSDocTag<'_>,
    should_capitalize: bool,
    wrap_width: usize,
    has_no_space_before_type: bool,
    bracket_spacing: bool,
    format_options: &FormatOptions,
    content_lines: &mut Vec<String>,
) {
    let (type_part, comment_part) = tag.type_comment();

    let tag_prefix = format!("@{normalized_kind}");
    let mut normalized_type_str = String::new();
    let mut tag_line = tag_prefix.clone();

    // For @type/@satisfies, the plugin keeps types mostly as-is (no quote conversion).
    // For @returns/@yields/etc., it runs Prettier's TS parser on the type.
    let preserve_quotes = matches!(normalized_kind, "type" | "satisfies");

    if let Some(tp) = &type_part {
        let raw_type = tp.parsed();
        if !raw_type.is_empty() {
            normalized_type_str = if preserve_quotes {
                normalize_type_preserve_quotes(raw_type)
            } else {
                normalize_type_return(raw_type)
            };
            // Try formatting via the formatter (simulates upstream's formatType())
            if !preserve_quotes
                && let Some(formatted) =
                    format_type_via_formatter(&normalized_type_str, format_options)
            {
                normalized_type_str = formatted;
            }
            // Preserve no-space only when the type isn't an object literal
            // (object types start with `{`, making `@type{{` → should be `@type {{`)
            let preserve_no_space =
                has_no_space_before_type && !normalized_type_str.starts_with('{');
            let space = if preserve_no_space { "" } else { " " };
            let (ob, cb) = if bracket_spacing { ("{ ", " }") } else { ("{", "}") };
            write!(tag_line, "{space}{ob}{normalized_type_str}{cb}").unwrap();
        }
    }

    let desc_text = comment_part.parsed();
    let desc_text = normalize_markdown_emphasis(desc_text.trim());
    let desc_text = desc_text.trim();

    if desc_text.is_empty() {
        // Try type wrapping if line is too long
        if tag_line.len() > wrap_width
            && !normalized_type_str.is_empty()
            && wrap_type_expression(
                &tag_prefix,
                &normalized_type_str,
                "",
                wrap_width,
                content_lines,
            )
        {
            return;
        }
        content_lines.push(tag_line);
        return;
    }

    let desc_text =
        if should_capitalize { capitalize_first(desc_text) } else { desc_text.to_string() };

    let one_liner = format!("{tag_line} {desc_text}");
    if one_liner.len() <= wrap_width {
        content_lines.push(one_liner);
    } else if !normalized_type_str.is_empty()
        && tag_line.len() > wrap_width
        && wrap_type_expression(&tag_prefix, &normalized_type_str, "", wrap_width, content_lines)
    {
        // Type was wrapped. Add description as continuation.
        let indent = "  ";
        let indent_width = wrap_width.saturating_sub(indent.len());
        let mut desc_lines = Vec::new();
        wrap_text(&desc_text, indent_width, &mut desc_lines);
        for dl in desc_lines {
            content_lines.push(format!("{indent}{dl}"));
        }
    } else {
        // Regular word-wrapping of description
        let first_line_prefix = format!("{tag_line} ");
        let first_line_content_width = wrap_width.saturating_sub(first_line_prefix.len());
        let words: Vec<&str> = tokenize_words(&desc_text);
        let mut first_line = String::new();
        let mut remaining_start = 0;

        for (i, word) in words.iter().enumerate() {
            if first_line.is_empty() {
                if word.len() <= first_line_content_width {
                    first_line.push_str(word);
                    remaining_start = i + 1;
                } else {
                    break;
                }
            } else if first_line.len() + 1 + word.len() <= first_line_content_width {
                first_line.push(' ');
                first_line.push_str(word);
                remaining_start = i + 1;
            } else {
                break;
            }
        }

        if first_line.is_empty() {
            content_lines.push(tag_line);
        } else {
            content_lines.push(format!("{first_line_prefix}{first_line}"));
        }

        let indent = "  ";
        let indent_width = wrap_width.saturating_sub(indent.len());
        if remaining_start < words.len() {
            let remaining: String = words[remaining_start..].join(" ");
            let mut desc_lines = Vec::new();
            wrap_text(&remaining, indent_width, &mut desc_lines);
            for dl in desc_lines {
                content_lines.push(format!("{indent}{dl}"));
            }
        }
    }
}

fn format_generic_tag(
    normalized_kind: &str,
    tag: &oxc_jsdoc::parser::JSDocTag<'_>,
    should_capitalize: bool,
    wrap_width: usize,
    quote_style: QuoteStyle,
    content_lines: &mut Vec<String>,
) {
    let tag_line = format!("@{normalized_kind}");
    let desc_text = tag.comment().parsed();
    let desc_text = normalize_markdown_emphasis(desc_text.trim());
    let desc_text = desc_text.trim();

    if desc_text.is_empty() {
        content_lines.push(tag_line);
        return;
    }

    // For @default/@defaultValue, format JSON-like values
    let desc_text = if matches!(normalized_kind, "default" | "defaultValue") {
        format_default_value(desc_text, quote_style)
    } else if should_capitalize {
        capitalize_first(desc_text)
    } else {
        desc_text.to_string()
    };

    let one_liner = format!("{tag_line} {desc_text}");
    if one_liner.len() <= wrap_width {
        content_lines.push(one_liner);
    } else {
        // Try to fit some description on the first line
        let first_line_prefix = format!("{tag_line} ");
        let first_line_content_width = wrap_width.saturating_sub(first_line_prefix.len());
        let words: Vec<&str> = tokenize_words(&desc_text);
        let mut first_line = String::new();
        let mut remaining_start = 0;

        for (i, word) in words.iter().enumerate() {
            if first_line.is_empty() {
                if word.len() <= first_line_content_width {
                    first_line.push_str(word);
                    remaining_start = i + 1;
                } else {
                    break;
                }
            } else if first_line.len() + 1 + word.len() <= first_line_content_width {
                first_line.push(' ');
                first_line.push_str(word);
                remaining_start = i + 1;
            } else {
                break;
            }
        }

        if first_line.is_empty() {
            content_lines.push(tag_line);
        } else {
            content_lines.push(format!("{first_line_prefix}{first_line}"));
        }

        let indent = "  ";
        let indent_width = wrap_width.saturating_sub(indent.len());
        if remaining_start < words.len() {
            let remaining: String = words[remaining_start..].join(" ");
            let mut desc_lines = Vec::new();
            wrap_text(&remaining, indent_width, &mut desc_lines);
            for dl in desc_lines {
                content_lines.push(format!("{indent}{dl}"));
            }
        }
    }
}

// ──────────────────────────────────────────────────
// @import tag processing
// ──────────────────────────────────────────────────

/// A parsed `@import` tag.
#[derive(Clone)]
struct ImportInfo {
    default_import: Option<String>,
    named_imports: Vec<String>,
    module_path: String,
}

/// Parse an `@import` tag's comment text into its components.
///
/// Handles these forms:
/// - `Default, {Named1, Named2} from "module"`
/// - `{Named1} from 'module'`
/// - `Default from "module"`
fn parse_import_tag(comment_text: &str) -> Option<ImportInfo> {
    // Normalize: join lines, collapse whitespace
    let text: String = comment_text.lines().map(str::trim).collect::<Vec<_>>().join(" ");
    let text = text.trim();

    // Find "from" keyword followed by a quoted string
    let from_idx = text.rfind(" from ")?;
    let specifier = text[..from_idx].trim();
    let module_part = text[from_idx + 6..].trim();

    // Extract module path (strip quotes)
    let module_path =
        module_part.trim_start_matches(['"', '\'']).trim_end_matches(['"', '\'']).to_string();

    if module_path.is_empty() {
        return None;
    }

    // Parse specifier: "Default, {Named1, Named2}", "{Named1}", or "Default"
    let (default_import, named_imports) = if let Some(brace_start) = specifier.find('{') {
        let brace_end = specifier.rfind('}')?;
        let default_part = specifier[..brace_start].trim().trim_end_matches(',').trim();
        let named_part = &specifier[brace_start + 1..brace_end];

        let default_import =
            if default_part.is_empty() { None } else { Some(default_part.to_string()) };

        let named_imports: Vec<String> = named_part
            .split(',')
            .map(|s| {
                // Normalize whitespace: "B  as  B1" → "B as B1"
                s.split_whitespace().collect::<Vec<_>>().join(" ")
            })
            .filter(|s| !s.is_empty())
            .collect();

        (default_import, named_imports)
    } else {
        // No braces — just a default import
        let name = specifier.split_whitespace().collect::<Vec<_>>().join(" ");
        (Some(name), Vec::new())
    };

    Some(ImportInfo { default_import, named_imports, module_path })
}

/// Get the sort key for a named import specifier.
/// `"B as B1"` → `"B"`, `"B2"` → `"B2"`.
fn import_specifier_sort_key(specifier: &str) -> &str {
    if let Some(idx) = specifier.find(" as ") { specifier[..idx].trim() } else { specifier.trim() }
}

/// Merge `@import` tags that share the same module path.
/// Returns merged imports sorted by module path (third-party before relative).
fn merge_and_sort_imports(imports: Vec<ImportInfo>) -> Vec<ImportInfo> {
    if imports.is_empty() {
        return imports;
    }

    // Group by module path (preserving insertion order)
    let mut groups: Vec<ImportInfo> = Vec::new();

    for import in imports {
        if let Some(existing) = groups.iter_mut().find(|g| g.module_path == import.module_path) {
            // Merge: take last default import, combine named imports
            if import.default_import.is_some() {
                existing.default_import = import.default_import;
            }
            for named in import.named_imports {
                // Deduplicate by original import name
                let key = import_specifier_sort_key(&named);
                let already_exists =
                    existing.named_imports.iter().any(|n| import_specifier_sort_key(n) == key);
                if !already_exists {
                    existing.named_imports.push(named);
                }
            }
        } else {
            groups.push(import);
        }
    }

    // Sort named imports within each group by original import name
    for import in &mut groups {
        import
            .named_imports
            .sort_by(|a, b| import_specifier_sort_key(a).cmp(import_specifier_sort_key(b)));
    }

    // Sort groups: third-party (no ./ or ../) before relative, then alphabetically
    groups.sort_by(|a, b| {
        let a_relative = a.module_path.starts_with('.');
        let b_relative = b.module_path.starts_with('.');
        match (a_relative, b_relative) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => a.module_path.cmp(&b.module_path),
        }
    });

    groups
}

/// Format a single merged `@import` tag into output lines.
fn format_import_lines(import: &ImportInfo, content_lines: &mut Vec<String>) {
    let module = &import.module_path;

    match (&import.default_import, import.named_imports.len()) {
        (Some(default), 0) => {
            content_lines.push(format!("@import {default} from \"{module}\""));
        }
        (None, 1) => {
            let named = &import.named_imports[0];
            content_lines.push(format!("@import {{{named}}} from \"{module}\""));
        }
        (Some(default), 1) => {
            let named = &import.named_imports[0];
            content_lines.push(format!("@import {default}, {{{named}}} from \"{module}\""));
        }
        (None, n) if n >= 2 => {
            content_lines.push("@import {".to_string());
            for (i, named) in import.named_imports.iter().enumerate() {
                let comma = if i < import.named_imports.len() - 1 { "," } else { "" };
                content_lines.push(format!("  {named}{comma}"));
            }
            content_lines.push(format!("}} from \"{module}\""));
        }
        (Some(default), n) if n >= 2 => {
            content_lines.push(format!("@import {default}, {{"));
            for (i, named) in import.named_imports.iter().enumerate() {
                let comma = if i < import.named_imports.len() - 1 { "," } else { "" };
                content_lines.push(format!("  {named}{comma}"));
            }
            content_lines.push(format!("}} from \"{module}\""));
        }
        _ => {}
    }
}

/// Process all `@import` tags: parse, merge by module, sort, and format.
/// Returns formatted lines ready to be inserted into the comment.
fn process_import_tags(tags: &[(&oxc_jsdoc::parser::JSDocTag<'_>, &str)]) -> Vec<String> {
    let mut imports = Vec::new();

    for &(tag, kind) in tags {
        if kind != "import" {
            continue;
        }
        let comment = tag.comment().parsed();
        if let Some(info) = parse_import_tag(&comment) {
            imports.push(info);
        }
    }

    let merged = merge_and_sort_imports(imports);

    let mut lines = Vec::new();
    for import in &merged {
        format_import_lines(import, &mut lines);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_sort_priority_canonical_names() {
        // Canonical names should have specific priorities
        assert_eq!(tag_sort_priority("import"), 0);
        assert_eq!(tag_sort_priority("param"), 80);
        assert_eq!(tag_sort_priority("returns"), 84);
        assert_eq!(tag_sort_priority("this"), 79); // 39.5 × 2
        assert_eq!(tag_sort_priority("see"), 90);
        assert_eq!(tag_sort_priority("todo"), 92);
    }

    #[test]
    fn test_tag_sort_priority_unknown_tags() {
        // Unknown/custom tags get the "other" weight
        assert_eq!(tag_sort_priority("custom"), 88);
        assert_eq!(tag_sort_priority("override"), 88);
        assert_eq!(tag_sort_priority("internal"), 88);
        assert_eq!(tag_sort_priority("link"), 88);
    }

    #[test]
    fn test_tag_sort_priority_no_synonyms() {
        // Synonyms should NOT appear — they must be normalized first
        assert_eq!(tag_sort_priority("return"), 88); // not "returns"
        assert_eq!(tag_sort_priority("arg"), 88); // not "param"
        assert_eq!(tag_sort_priority("yield"), 88); // not "yields"
        assert_eq!(tag_sort_priority("constructor"), 88); // not "class"
    }

    #[test]
    fn test_is_known_tag() {
        assert!(is_known_tag("param"));
        assert!(is_known_tag("returns"));
        assert!(is_known_tag("typedef"));
        assert!(is_known_tag("this"));
        assert!(!is_known_tag("custom"));
        assert!(!is_known_tag("override"));
        assert!(!is_known_tag("link"));
    }

    #[test]
    fn test_should_skip_capitalize() {
        // Tags in TAGS_PEV_FORMATE_DESCRIPTION
        assert!(should_skip_capitalize("borrows"));
        assert!(should_skip_capitalize("default"));
        assert!(should_skip_capitalize("defaultValue"));
        assert!(should_skip_capitalize("import"));
        assert!(should_skip_capitalize("memberof"));
        assert!(should_skip_capitalize("module"));
        assert!(should_skip_capitalize("see"));

        // Tags whose content is a name/reference
        assert!(should_skip_capitalize("function"));
        assert!(should_skip_capitalize("typedef"));
        assert!(should_skip_capitalize("class"));
        assert!(should_skip_capitalize("callback"));

        // Tags that should capitalize
        assert!(!should_skip_capitalize("param"));
        assert!(!should_skip_capitalize("returns"));
        assert!(!should_skip_capitalize("deprecated"));
    }

    #[test]
    fn test_should_remove_empty_tag() {
        // Upstream's TAGS_DESCRIPTION_NEEDED
        assert!(should_remove_empty_tag("borrows"));
        assert!(should_remove_empty_tag("category"));
        assert!(should_remove_empty_tag("description"));
        assert!(should_remove_empty_tag("example"));
        assert!(should_remove_empty_tag("import"));
        assert!(should_remove_empty_tag("privateRemarks"));
        assert!(should_remove_empty_tag("remarks"));
        assert!(should_remove_empty_tag("since"));
        assert!(should_remove_empty_tag("todo"));

        // Tags that should NOT be removed when empty
        assert!(!should_remove_empty_tag("param"));
        assert!(!should_remove_empty_tag("returns"));
        assert!(!should_remove_empty_tag("deprecated"));
        assert!(!should_remove_empty_tag("abstract"));
    }

    fn fmt_type(type_str: &str) -> Option<String> {
        format_type_via_formatter(type_str, &FormatOptions::default())
    }

    #[test]
    fn test_format_type_via_formatter() {
        assert_eq!(fmt_type("string"), Some("string".to_string()));
        assert_eq!(fmt_type("number"), Some("number".to_string()));
        assert_eq!(fmt_type("string | number"), Some("string | number".to_string()));
        assert_eq!(fmt_type(""), None);
    }

    #[test]
    fn test_format_type_via_formatter_rest() {
        assert_eq!(fmt_type("...any"), Some("...any".to_string()));
        assert_eq!(fmt_type("...number"), Some("...number".to_string()));
        assert_eq!(fmt_type("...(string | number)"), Some("...(string | number)".to_string()));
    }
}
