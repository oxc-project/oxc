use std::borrow::Cow;

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
    line_buffer::LineBuffer,
    mdast_serialize::format_description_mdast,
    normalize::{
        capitalize_first, normalize_markdown_emphasis, normalize_tag_kind, normalize_type,
        normalize_type_preserve_quotes, normalize_type_return, strip_optional_type_suffix,
    },
    wrap::{tokenize_words, wrap_text},
};

/// The ` * ` prefix used in multiline JSDoc comments (3 chars).
const LINE_PREFIX_LEN: usize = 3;

/// Trim trailing whitespace from an owned `String` in place, avoiding a reallocation.
fn truncate_trim_end(s: &mut String) {
    let trimmed_len = s.trim_end().len();
    s.truncate(trimmed_len);
}

/// Push a (possibly multi-line) description into `content_lines` as a single string,
/// prepending `indent` to each non-empty line. When indent is empty, moves `desc` directly.
fn push_indented_desc(content_lines: &mut LineBuffer, indent: &str, mut desc: String) {
    if desc.is_empty() {
        return;
    }
    if indent.is_empty() {
        content_lines.push(desc);
        return;
    }
    if !desc.contains('\n') {
        desc.insert_str(0, indent);
        content_lines.push(desc);
        return;
    }
    // One allocation, one forward pass using `find` (SIMD-accelerated in std).
    let mut s = String::with_capacity(desc.len() + indent.len() * 4);
    let mut rest = desc.as_str();
    while let Some(nl) = rest.find('\n') {
        // Skip indent for empty lines (nl == 0) — blank lines in JSDoc body
        // should not have leading spaces.
        if nl > 0 {
            s.push_str(indent);
        }
        s.push_str(&rest[..=nl]);
        rest = &rest[nl + 1..];
    }
    if !rest.is_empty() {
        s.push_str(indent);
        s.push_str(rest);
    }
    content_lines.push(s);
}

/// Join an iterator of string slices with a separator, avoiding an intermediate `Vec`.
/// Uses `size_hint()` for a rough capacity estimate to reduce reallocations.
fn join_iter<'a>(iter: impl Iterator<Item = &'a str>, sep: &str) -> String {
    let mut iter = iter;
    let (lower, _) = iter.size_hint();
    let mut result = String::with_capacity(lower.saturating_mul(20));
    if let Some(first) = iter.next() {
        result.push_str(first);
        for item in iter {
            result.push_str(sep);
            result.push_str(item);
        }
    }
    result
}

/// Tags whose descriptions should NOT be capitalized.
/// Matches upstream's `TAGS_PEV_FORMAT_DESCRIPTION` exactly:
/// borrows, default, defaultValue, import, memberof, module, see.
fn should_skip_capitalize(tag_kind: &str) -> bool {
    matches!(
        tag_kind,
        "borrows" | "default" | "defaultValue" | "import" | "memberof" | "module" | "see"
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
    matches!(tag_kind, "returns" | "yields" | "throws" | "type" | "satisfies" | "this" | "extends")
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
        "default" => 44,
        "defaultValue" => 46,
        "external" => 48,
        "overload" => 50,
        "fires" => 52,
        "template" => 54,
        "typeParam" => 56,
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

/// Check if a tag kind is a group condition (enables group splitting).
/// Matches prettier-plugin-jsdoc's `TAGS_GROUP_CONDITION`.
fn is_tags_group_condition(kind: &str) -> bool {
    matches!(
        kind,
        "callback"
            | "typedef"
            | "type"
            | "property"
            | "param"
            | "returns"
            | "this"
            | "yields"
            | "throws"
    )
}

/// Check if a tag that goes through `format_generic_tag` has a "name" field
/// in upstream's comment-parser (i.e., is NOT in `TAGS_NAMELESS`).
/// For these tags, the first word of the comment is the name and should NOT
/// be capitalized — only the description after the name should be.
///
/// This only lists tags that are routed to `format_generic_tag` (i.e., not
/// handled by type_name_comment, type_comment, or example/remarks formatters).
fn is_named_generic_tag(kind: &str) -> bool {
    matches!(
        kind,
        "abstract"
            | "async"
            | "augments"
            | "author"
            | "callback"
            | "class"
            | "constant"
            | "external"
            | "fires"
            | "flow"
            | "function"
            | "ignore"
            | "member"
            | "memberof"
            | "private"
            | "see"
            | "version"
            | "typeParam"
    )
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

    let param_tags = &effective_tags[param_start..param_end];

    // Parse type_name_comment() once per tag, cache the results.
    // Each call does O(n) brace-counting, and we'd otherwise call it 4x per tag.
    let parsed: Vec<_> = param_tags
        .iter()
        .map(|(tag, _)| {
            let (type_part, name_part, _) = tag.type_name_comment();
            (type_part.is_some(), name_part.map(|n| n.parsed()))
        })
        .collect();

    // Check that ALL @param tags have type annotations and names
    if !parsed.iter().all(|(has_type, name)| *has_type && name.is_some()) {
        return;
    }

    // Extract the cached names (we verified all are Some above)
    let names: Vec<&str> = parsed.iter().map(|(_, name)| name.unwrap_or("")).collect();

    // Extract function parameter names from the source text after the comment
    let fn_params = extract_function_params(comment, source_text);
    if fn_params.len() != names.len() {
        return;
    }

    // Already in order?
    if names.iter().zip(fn_params.iter()).all(|(name, p)| *name == *p) {
        return;
    }

    // Check same set of names (lengths already verified equal, param lists are small)
    if !names.iter().all(|name| fn_params.contains(name)) {
        return;
    }

    // Sort @param tags by their position in the function signature.
    // Use sort_by_cached_key to call the key function once per element.
    effective_tags[param_start..param_end].sort_by_cached_key(|(tag, _)| {
        let (_, name_part, _) = tag.type_name_comment();
        let name = name_part.map_or("", |n| n.parsed());
        fn_params.iter().position(|p| *p == name).unwrap_or(usize::MAX)
    });
}

/// Extract function parameter names from the source text after the comment.
/// Handles `function name(...)`, `name(...)` methods, `name = (...) =>` arrows.
/// Uses balanced parenthesis matching to handle nested type annotations.
fn extract_function_params<'a>(comment: &Comment, source_text: &'a str) -> Vec<&'a str> {
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
        let is_id_continue = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'$';
        if text[i..].starts_with("export") && i + 6 < len && !is_id_continue(bytes[i + 6]) {
            i += 6;
            continue;
        }
        if text[i..].starts_with("async") && i + 5 < len && !is_id_continue(bytes[i + 5]) {
            i += 5;
            continue;
        }
        if text[i..].starts_with("default") && i + 7 < len && !is_id_continue(bytes[i + 7]) {
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
            let is_id_continue = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'$';
            if text[i..].starts_with("async") && i + 5 < len && !is_id_continue(bytes[i + 5]) {
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
fn parse_param_names(params_str: &str) -> Vec<&str> {
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
            // Skip type annotation, default value, and comma (bracket-aware)
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
            names.push(&params_str[name_start..i]);
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
/// Returns tuples of `(tag, normalized_kind)` so callers don't need to recompute the kind.
fn sort_tags_by_groups<'a>(
    tags: &'a [oxc_jsdoc::parser::JSDocTag<'a>],
) -> Vec<(&'a oxc_jsdoc::parser::JSDocTag<'a>, &'a str)> {
    if tags.is_empty() {
        return Vec::new();
    }

    // Quick scan: check if any group split is needed (no allocation).
    let mut needs_split = false;
    let mut seen_condition = false;
    for tag in tags {
        let kind = normalize_tag_kind(tag.kind.parsed());
        if is_tags_group_condition(kind) {
            seen_condition = true;
        }
        if is_tags_group_head(kind) && seen_condition {
            needs_split = true;
            break;
        }
    }

    // normalize_tag_kind is a cheap string match, so calling it again below is fine.
    let normalize =
        |tag: &'a oxc_jsdoc::parser::JSDocTag<'a>| (tag, normalize_tag_kind(tag.kind.parsed()));

    if !needs_split {
        // Single group — sort directly by priority.
        let mut sorted: Vec<_> = tags.iter().map(normalize).collect();
        sorted.sort_by_key(|(_, kind)| tag_sort_priority(kind));
        return sorted;
    }

    // Multi-group path: build groups directly from tags, no intermediate Vec.
    let mut groups: Vec<Vec<(&oxc_jsdoc::parser::JSDocTag<'a>, &'a str)>> = Vec::new();
    let mut current_group: Vec<(&oxc_jsdoc::parser::JSDocTag<'a>, &'a str)> = Vec::new();
    let mut can_group_next_tags = false;

    for tag in tags {
        let kind = normalize_tag_kind(tag.kind.parsed());
        if is_tags_group_head(kind) && can_group_next_tags && !current_group.is_empty() {
            groups.push(current_group);
            current_group = Vec::new();
            can_group_next_tags = false;
        }
        if is_tags_group_condition(kind) {
            can_group_next_tags = true;
        }
        current_group.push((tag, kind));
    }
    if !current_group.is_empty() {
        groups.push(current_group);
    }

    // Sort within each group, then flatten.
    for group in &mut groups {
        group.sort_by_key(|(_, kind)| tag_sort_priority(kind));
    }
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

    // Extract inner content (between `/**` and `*/`)
    let content_span = comment.content_span();
    // content_span strips `/*` and `*/`; bump start by 1 to also skip the extra `*` in `/**`
    let jsdoc_span = Span::new(content_span.start + 1, content_span.end);
    let inner = jsdoc_span.source_text(source_text);
    let jsdoc = JSDoc::new(inner, jsdoc_span);

    let comment_part = jsdoc.comment();
    let description = comment_part.parsed_preserving_whitespace();

    // Empty JSDoc: no description and no tags
    if description.trim().is_empty() && jsdoc.tags().is_empty() {
        return Some(allocator.alloc_str(""));
    }

    // Width available for content (subtract ` * ` prefix)
    let wrap_width = available_width.saturating_sub(LINE_PREFIX_LEN);

    // Pre-build format options for type formatting (jsdoc: None prevents recursion).
    // This is cloned once here instead of per-tag in format_type_via_formatter.
    let type_format_options = FormatOptions { jsdoc: None, ..format_options.clone() };

    let mut content_lines = LineBuffer::new();

    // Format description using mdast parsing (handles heading normalization,
    // emphasis conversion, horizontal rule removal, reference links, nested lists, etc.)
    let desc_trimmed = description.trim();
    if !desc_trimmed.is_empty() {
        let desc = format_description_mdast(
            desc_trimmed,
            wrap_width,
            options.capitalize_descriptions,
            Some(format_options),
            Some(external_callbacks),
            Some(allocator),
        );
        content_lines.push(desc);
    }

    // Sort tags by priority within groups.
    // @typedef and @callback are TAGS_GROUP_HEAD — they start new groups.
    // Tags sort within their group by weight, but groups keep their relative order.
    let tags = jsdoc.tags();
    let sorted_tags = sort_tags_by_groups(tags);

    // Collect effective tags, merging @description into the description area
    let mut effective_tags: Vec<(&oxc_jsdoc::parser::JSDocTag<'_>, &str)> = Vec::new();
    for (tag, normalized_kind) in &sorted_tags {
        if should_remove_empty_tag(normalized_kind) && !tag_has_content(tag) {
            continue;
        }
        // @description tag: merge its content into the main description
        if *normalized_kind == "description" {
            let desc_content = tag.comment().parsed();
            let desc_content = desc_content.trim();
            if !desc_content.is_empty() {
                if !content_lines.is_empty() && !content_lines.last_is_empty() {
                    content_lines.push_empty();
                }
                let desc = format_description_mdast(
                    desc_content,
                    wrap_width,
                    options.capitalize_descriptions,
                    Some(format_options),
                    Some(external_callbacks),
                    Some(allocator),
                );
                content_lines.push(desc);
            }
            continue;
        }
        effective_tags.push((tag, normalized_kind));
    }

    // Reorder @param tags to match the function signature order
    reorder_param_tags(&mut effective_tags, comment, source_text);

    // Pre-process @import tags: merge by module, sort, format
    let (mut import_lines, parsed_import_indices) = process_import_tags(&effective_tags);
    let has_imports = !import_lines.is_empty();
    let mut imports_emitted = false;

    // Format tags
    let mut prev_normalized_kind: Option<&str> = None;
    let mut first_non_import_tag_emitted = false;
    for (tag_idx, &(tag, normalized_kind)) in effective_tags.iter().enumerate() {
        // Skip successfully parsed @import tags — they are handled via merged import_lines.
        // Unparsable @import tags fall through to format_generic_tag().
        if parsed_import_indices.contains(&tag_idx) {
            if has_imports && !imports_emitted {
                // Emit merged imports at the position of the first @import tag
                if !content_lines.is_empty() && !content_lines.last_is_empty() {
                    content_lines.push_empty();
                }
                let import_str =
                    std::mem::replace(&mut import_lines, LineBuffer::new()).into_string();
                content_lines.push(import_str);
                imports_emitted = true;
                prev_normalized_kind = Some("import");
            }
            continue;
        }

        let is_first_tag = !first_non_import_tag_emitted && !imports_emitted;

        let should_capitalize = options.capitalize_descriptions
            && !should_skip_capitalize(normalized_kind)
            && is_known_tag(normalized_kind);

        // Add blank line between description and first tag
        if is_first_tag && !content_lines.is_empty() && !content_lines.last_is_empty() {
            content_lines.push_empty();
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

            if should_separate && !content_lines.last_is_empty() {
                content_lines.push_empty();
            }
        }

        first_non_import_tag_emitted = true;
        prev_normalized_kind = Some(normalized_kind);

        // Track content before formatting this tag
        let lines_before = content_lines.byte_len();

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
                allocator,
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
                &type_format_options,
                external_callbacks,
                allocator,
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
                &type_format_options,
                external_callbacks,
                allocator,
                &mut content_lines,
            );
        } else {
            format_generic_tag(
                normalized_kind,
                tag,
                should_capitalize,
                wrap_width,
                format_options.quote_style,
                format_options,
                external_callbacks,
                allocator,
                &mut content_lines,
            );
        }

        // If this tag has multi-paragraph content (blank lines within, or is an @example tag
        // with multi-line code) and the next tag is of a different kind, add a trailing
        // blank line for separation.
        let tag_content_has_blank_lines = content_lines.has_blank_line_since(lines_before);
        // line_count_since counts \n separators added since the snapshot; ≥2 means multi-line.
        let tag_newline_count = content_lines.line_count_since(lines_before);
        let is_example_multiline = normalized_kind == "example" && tag_newline_count > 1;
        if (tag_content_has_blank_lines || is_example_multiline)
            && let Some(&(_, next_kind)) = effective_tags.get(tag_idx + 1)
            && next_kind != normalized_kind
            && !content_lines.last_is_empty()
        {
            content_lines.push_empty();
        }
    }

    // Get the full content as a single string and iterate lines,
    // trimming leading and trailing blank lines.
    let content_str = content_lines.into_string();
    let content_str = content_str.trim_end_matches('\n');
    let mut iter = content_str.split('\n').skip_while(|l| l.is_empty());

    let Some(first) = iter.next() else {
        return Some(allocator.alloc_str(""));
    };

    // Single-line check: convert to single-line if content is a single line.
    // The plugin prefers single-line even if it slightly exceeds printWidth,
    // since the wrapping logic already constrains the content width.
    let second = iter.next();
    if options.single_line_when_possible && second.is_none() {
        let formatted = allocator.alloc_concat_strs_array(["/** ", first, " */"]);
        if formatted == content {
            return None;
        }
        return Some(formatted);
    }

    // Build multiline comment
    let capacity = content_str.len() + content_str.bytes().filter(|&b| b == b'\n').count() * 4 + 10;
    let mut builder = StringBuilder::with_capacity_in(capacity, allocator);
    builder.push_str("/**");

    for line in std::iter::once(first).chain(second).chain(iter) {
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
    content_lines: &mut LineBuffer,
) -> bool {
    // Only wrap if the full line exceeds the width
    let full_len = tag_prefix.len()
        + 2 // " {"
        + type_str.len()
        + if name_and_rest.is_empty() { 1 } else { 2 + name_and_rest.len() }; // "}" or "} name"
    if full_len <= wrap_width {
        return false;
    }

    // Check if the type contains `|` at the top level for union wrapping
    let parts = split_type_at_top_level_pipe(type_str);
    if parts.len() <= 1 {
        // Check for generic type `Foo<...>` wrapping at top-level angle bracket
        if let Some(wrapped) = wrap_generic_type(tag_prefix, type_str, name_and_rest, content_lines)
        {
            return wrapped;
        }
        return false;
    }

    // Wrap union type at `|` operators
    let first_part = parts[0].trim();
    {
        let s = content_lines.begin_line();
        s.push_str(tag_prefix);
        s.push_str(" {");
        s.push_str(first_part);
    }

    for (i, part) in parts.iter().enumerate().skip(1) {
        let part = part.trim();
        let s = content_lines.begin_line();
        s.push_str("  | ");
        s.push_str(part);
        if i == parts.len() - 1 {
            s.push('}');
            if !name_and_rest.is_empty() {
                s.push(' ');
                s.push_str(name_and_rest);
            }
        }
    }

    true
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
    content_lines: &mut LineBuffer,
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

    // First line: tag + opening part including <
    {
        let s = content_lines.begin_line();
        s.push_str(tag_prefix);
        s.push_str(" {");
        s.push_str(prefix_part);
    }
    // Inner content with 2-space indent
    {
        let s = content_lines.begin_line();
        s.push_str("  ");
        s.push_str(inner);
    }
    // Closing >} with optional name
    if name_and_rest.is_empty() {
        content_lines.push(">}");
    } else {
        let s = content_lines.begin_line();
        s.push_str(">} ");
        s.push_str(name_and_rest);
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

/// Format a `@default` / `@defaultValue` value.
/// Handles JSON-like formatting: spaces after `:` and `,`, inside `{}`.
/// Converts quotes based on the `quote_style` option.
/// Non-JSON values (code, plain text) are returned as-is.
fn format_default_value(value: &str, quote_style: QuoteStyle) -> Cow<'_, str> {
    let trimmed = value.trim();
    // Detect if value looks like JSON/object/array literal
    let first_byte = trimmed.as_bytes().first().copied().unwrap_or(b' ');
    if !matches!(first_byte, b'{' | b'[' | b'"' | b'\'') {
        // Doesn't start with JSON-like syntax; return unchanged
        return Cow::Borrowed(trimmed);
    }

    // Determine target and source quote characters based on quote style.
    let (target_quote, other_quote) = match quote_style {
        QuoteStyle::Double => (b'"', b'\''),
        QuoteStyle::Single => (b'\'', b'"'),
    };

    // Format JSON-like values: normalize spacing around `:`, `,`, `{`, `}`, `[`
    // and convert quotes based on the quote_style option.
    let bytes = trimmed.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len + 16);
    let mut i = 0;
    let mut in_target_quote = false;
    let mut in_other_quote = false;

    while i < len {
        let b = bytes[i];

        if in_target_quote {
            if b.is_ascii() {
                result.push(b as char);
            } else {
                let ch = trimmed[i..].chars().next().unwrap();
                result.push(ch);
                i += ch.len_utf8();
                continue;
            }
            if b == target_quote && (i == 0 || bytes[i - 1] != b'\\') {
                in_target_quote = false;
            }
            i += 1;
            continue;
        }

        if in_other_quote {
            if b == other_quote && (i == 0 || bytes[i - 1] != b'\\') {
                result.push(target_quote as char); // Close with target quote
                in_other_quote = false;
            } else if b.is_ascii() {
                result.push(b as char);
            } else {
                let ch = trimmed[i..].chars().next().unwrap();
                result.push(ch);
                i += ch.len_utf8();
                continue;
            }
            i += 1;
            continue;
        }

        match b {
            _ if b == target_quote => {
                result.push(target_quote as char);
                in_target_quote = true;
                i += 1;
            }
            _ if b == other_quote => {
                result.push(target_quote as char); // Open with target quote
                in_other_quote = true;
                i += 1;
            }
            b':' => {
                result.push(':');
                // Add space after `:` if not already there
                if i + 1 < len && bytes[i + 1] != b' ' {
                    result.push(' ');
                }
                i += 1;
            }
            b',' => {
                result.push(',');
                // Add space after `,` if not already there
                if i + 1 < len && bytes[i + 1] != b' ' {
                    result.push(' ');
                }
                i += 1;
            }
            b'{' => {
                result.push('{');
                // Add space after `{` if next char is not `}` and not already a space
                if i + 1 < len && bytes[i + 1] != b'}' && bytes[i + 1] != b' ' {
                    result.push(' ');
                }
                i += 1;
            }
            b'}' => {
                // Add space before `}` if previous char is not `{` and not already a space
                if !result.is_empty() {
                    let last = result.as_bytes().last().copied().unwrap_or(b' ');
                    if last != b'{' && last != b' ' {
                        result.push(' ');
                    }
                }
                result.push('}');
                i += 1;
            }
            b'[' => {
                result.push('[');
                // Add space after `[` if next char is `]` (empty array special case: `[ ]`)
                if i + 1 < len && bytes[i + 1] == b']' {
                    result.push(' ');
                }
                i += 1;
            }
            _ if b.is_ascii() => {
                result.push(b as char);
                i += 1;
            }
            _ => {
                let ch = trimmed[i..].chars().next().unwrap();
                result.push(ch);
                i += ch.len_utf8();
            }
        }
    }
    Cow::Owned(result)
}

/// Strip an existing "Default is `...`" or "Default is ..." suffix from a description.
/// The plugin always recomputes this from the `[name=value]` syntax.
fn strip_default_is_suffix(desc: &str) -> Cow<'_, str> {
    // Look for "Default is " (case insensitive matching for "default is")
    if let Some(pos) = desc.find("Default is ") {
        let before = desc[..pos].trim_end();
        // Remove trailing period before "Default is"
        let before = before.strip_suffix('.').unwrap_or(before);
        Cow::Borrowed(before.trim_end())
    } else {
        Cow::Borrowed(desc)
    }
}

/// Map fenced code block language tags to external formatter language identifiers.
/// Returns `None` if the language should be handled by the native JS/TS formatter.
pub(super) fn fenced_lang_to_external_language(lang: &str) -> Option<&'static str> {
    match lang {
        "css" | "scss" | "less" => Some("tagged-css"),
        "html" => Some("tagged-html"),
        "graphql" | "gql" => Some("tagged-graphql"),
        "markdown" | "md" | "mdx" => Some("tagged-markdown"),
        "yaml" | "yml" => Some("tagged-yaml"),
        _ => None,
    }
}

/// Returns `true` if the fenced code block language is JS/TS/JSX/TSX.
pub(super) fn is_js_ts_lang(lang: &str) -> bool {
    matches!(lang, "js" | "javascript" | "jsx" | "ts" | "typescript" | "tsx")
}

/// Format code using the external formatter (Prettier) for non-JS/TS languages.
/// Returns `Some(formatted)` on success, `None` if no callback is available or formatting fails.
pub(super) fn format_external_language(
    code: &str,
    language: &str,
    _wrap_width: usize,
    external_callbacks: &ExternalCallbacks,
) -> Option<String> {
    let result = external_callbacks.format_embedded(language, code)?;
    match result {
        Ok(mut formatted) => {
            truncate_trim_end(&mut formatted);
            Some(formatted)
        }
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

/// Try to format JS/TS/JSX code using the formatter.
/// Returns `Some(formatted)` on success, `None` if parsing fails.
/// The `print_width` is the available width for the formatted code.
/// Uses the parent `format_options` to ensure consistent formatting behavior.
pub(super) fn format_embedded_js(
    code: &str,
    print_width: usize,
    format_options: &FormatOptions,
    allocator: &Allocator,
) -> Option<String> {
    let width = u16::try_from(print_width).unwrap_or(80).clamp(1, 320);
    let line_width = LineWidth::try_from(width).unwrap();

    // Clone once upfront — subsequent clones of base_options are cheap since
    // the Vec fields (sort_imports, sort_tailwindcss) are already owned.
    let base_options = FormatOptions { line_width, jsdoc: None, ..format_options.clone() };

    // Try to parse and format with the given source type
    let try_format = |code: &str, source_type: SourceType| -> Option<String> {
        let ret =
            Parser::new(allocator, code, source_type).with_options(get_parse_options()).parse();
        if ret.panicked || !ret.errors.is_empty() {
            return None;
        }
        let mut formatted = Formatter::new(allocator, base_options.clone()).build(&ret.program);
        truncate_trim_end(&mut formatted);
        Some(formatted)
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
        let wrapped = allocator.alloc_concat_strs_array(["(", trimmed, ")"]);
        // Use TrailingCommas::None for object literals since JSON-like code
        // shouldn't have trailing commas
        let obj_options =
            FormatOptions { trailing_commas: TrailingCommas::None, ..base_options.clone() };

        let try_format_obj = |code: &str, source_type: SourceType| -> Option<String> {
            let ret =
                Parser::new(allocator, code, source_type).with_options(get_parse_options()).parse();
            if ret.panicked || !ret.errors.is_empty() {
                return None;
            }
            let formatted = Formatter::new(allocator, obj_options.clone()).build(&ret.program);
            let formatted = formatted.trim_end();
            // Remove the wrapping parens and trailing semicolon
            if let Some(inner) = formatted.strip_prefix('(')
                && let Some(inner) = inner.strip_suffix(");")
            {
                return Some(String::from(inner));
            }
            Some(String::from(formatted))
        };

        if let Some(result) = try_format_obj(wrapped, SourceType::jsx()) {
            return Some(result);
        }
        if let Some(result) = try_format_obj(wrapped, SourceType::tsx()) {
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
/// Format a type expression through the formatter.
/// `type_options` must already have `jsdoc: None` to prevent recursive formatting.
fn format_type_via_formatter(
    type_str: &str,
    type_options: &FormatOptions,
    allocator: &Allocator,
) -> Option<String> {
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
        let wrapped = allocator.alloc_concat_strs_array(["(", rest, ")[]"]);
        let formatted = format_type_via_formatter(wrapped, type_options, allocator)?;
        let inner = formatted.strip_suffix("[]")?;
        let mut result = String::with_capacity(inner.len() + 3);
        result.push_str("...");
        result.push_str(inner);
        return Some(result);
    }

    // Fast path: skip the expensive parse+format cycle for types that the TS
    // formatter won't change. Types without union/intersection operators, object
    // literals, function arrows, or parenthesized expressions are already in
    // their final form after normalize_type().
    if !needs_formatter_pass(type_str) {
        return None;
    }

    let input = allocator.alloc_concat_strs_array(["type __t = ", type_str, ";"]);

    let ret =
        Parser::new(allocator, input, SourceType::tsx()).with_options(get_parse_options()).parse();
    if ret.panicked || !ret.errors.is_empty() {
        return None;
    }

    let formatted = Formatter::new(allocator, type_options.clone()).build(&ret.program);
    let formatted = formatted.trim_end();

    // Strip the `type __t = ` prefix (11 chars) using slice, matching upstream's
    // `pretty.slice(TYPE_START.length)` approach. This handles both same-line and
    // wrapped output (e.g. `type __t =\n  | ...`).
    let result = formatted.get("type __t = ".len()..)?;

    // Upstream cleanup: strip leading whitespace, trailing `;` and newlines,
    // leading `|`, then trim.
    let result = result.trim_start();
    let result = result.trim_end_matches([';', '\n']);
    let result = result.strip_prefix('|').unwrap_or(result);
    let result = result.trim();

    if result.is_empty() {
        return None;
    }

    Some(String::from(result))
}

/// Check if a type expression needs to go through the TS formatter.
///
/// Returns `false` for types that are already in their final form after
/// `normalize_type()` — simple identifiers, dotted names, array shorthand (`T[]`),
/// and generic types without complex structure.
///
/// The TS formatter can only change types that contain:
/// - `|` or `&`: union/intersection types may need wrapping
/// - `{` or `}`: object literal types need spacing
/// - `(` or `)`: parenthesized/function types need formatting
/// - `=>`: function type arrows
/// - Newlines: multi-line types need reformatting
fn needs_formatter_pass(type_str: &str) -> bool {
    for &b in type_str.as_bytes() {
        match b {
            b'|' | b'&' | b'{' | b'}' | b'(' | b')' | b'\n' => return true,
            _ => {}
        }
    }
    type_str.contains("=>")
}

/// Format example code content with 2-space base indent.
/// Tries to format the code as JS/JSX first; falls back to pass-through on parse failure.
fn format_example_code(
    code: &str,
    wrap_width: usize,
    format_options: &FormatOptions,
    allocator: &Allocator,
    content_lines: &mut LineBuffer,
) {
    if code.is_empty() {
        return;
    }

    // Check for fenced code blocks (```lang ... ```). Triple backticks are
    // actually valid JavaScript (template literal expressions), so
    // `format_embedded_js` would parse them as JS and produce wrong output.
    // Handle fenced blocks by stripping the markers, formatting just the
    // inner code, and re-adding the fences with proper indentation.
    if let Some((first_line, rest)) = code.split_once('\n')
        && first_line.starts_with("```")
    {
        if let Some(closing_pos) = rest.rfind("\n```") {
            let inner_code = &rest[..closing_pos];
            let closing_fence = rest[closing_pos + 1..].trim();
            format_example_fenced_block(
                first_line,
                inner_code,
                closing_fence,
                wrap_width,
                format_options,
                allocator,
                content_lines,
            );
            return;
        } else if rest.trim() == "```" {
            // Only two lines: opening + closing fence, no inner code
            format_example_fenced_block(
                first_line,
                "",
                rest.trim(),
                wrap_width,
                format_options,
                allocator,
                content_lines,
            );
            return;
        }
    }

    // Try formatting the code. The effective print width for @example code is
    // wrap_width - 2 (for the 2-space indent within the comment).
    let effective_width = wrap_width.saturating_sub(2);
    if let Some(formatted) = format_embedded_js(code, effective_width, format_options, allocator) {
        // Add 2-space indent to code structure lines, but NOT to template literal
        // content. The formatter preserves template literal content verbatim, so
        // adding indent to those lines would shift them incorrectly.
        let mut template_depth: u32 = 0;
        for line in formatted.lines() {
            if line.is_empty() {
                content_lines.push_empty();
            } else if template_depth == 0 {
                {
                    let s = content_lines.begin_line();
                    s.push_str("  ");
                    s.push_str(line);
                }
            } else {
                content_lines.push(line);
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
            content_lines.push_empty();
        } else {
            {
                let s = content_lines.begin_line();
                s.push_str("  ");
                s.push_str(line_trimmed);
            }
        }
    }
}

/// Handle fenced code blocks inside @example tags.
/// Strips the ``` markers, formats the inner code, and re-adds fences
/// with proper 2-space indentation.
fn format_example_fenced_block(
    lang_line: &str,
    inner_code: &str,
    closing_fence: &str,
    wrap_width: usize,
    format_options: &FormatOptions,
    allocator: &Allocator,
    content_lines: &mut LineBuffer,
) {
    let effective_width = wrap_width.saturating_sub(2);

    // Add opening fence with indent
    {
        let s = content_lines.begin_line();
        s.push_str("  ");
        s.push_str(lang_line);
    }

    if !inner_code.is_empty() {
        let lang = lang_line[3..].trim();
        if is_js_ts_lang(lang) {
            if let Some(formatted) =
                format_embedded_js(inner_code, effective_width, format_options, allocator)
            {
                let mut template_depth: u32 = 0;
                for line in formatted.lines() {
                    if line.is_empty() {
                        content_lines.push_empty();
                    } else if template_depth == 0 {
                        {
                            let s = content_lines.begin_line();
                            s.push_str("  ");
                            s.push_str(line);
                        }
                    } else {
                        content_lines.push(line);
                    }
                    template_depth = update_template_depth(line, template_depth);
                }
            } else {
                // Fallback for unparsable inner code
                for line in inner_code.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        content_lines.push_empty();
                    } else {
                        {
                            let s = content_lines.begin_line();
                            s.push_str("  ");
                            s.push_str(trimmed);
                        }
                    }
                }
            }
        } else {
            // Non-JS/TS fenced code: preserve with 2-space indent
            for line in inner_code.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    content_lines.push_empty();
                } else {
                    {
                        let s = content_lines.begin_line();
                        s.push_str("  ");
                        s.push_str(trimmed);
                    }
                }
            }
        }
    }

    // Add closing fence with indent
    {
        let s = content_lines.begin_line();
        s.push_str("  ");
        s.push_str(closing_fence);
    }
}

fn format_example_tag(
    normalized_kind: &str,
    tag: &oxc_jsdoc::parser::JSDocTag<'_>,
    wrap_width: usize,
    format_options: &FormatOptions,
    allocator: &Allocator,
    content_lines: &mut LineBuffer,
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
        {
            let s = content_lines.begin_line();
            s.push('@');
            s.push_str(normalized_kind);
            s.push_str(" <caption>");
            s.push_str(caption);
            s.push_str("</caption>");
        }
        format_example_code(after_caption, wrap_width, format_options, allocator, content_lines);
        return;
    }

    {
        let s = content_lines.begin_line();
        s.push('@');
        s.push_str(normalized_kind);
    }
    format_example_code(trimmed, wrap_width, format_options, allocator, content_lines);
}

/// Join a slice of words with spaces, pre-allocating capacity.
fn join_words(words: &[&str]) -> String {
    if words.is_empty() {
        return String::new();
    }
    let cap: usize = words.iter().map(|w| w.len()).sum::<usize>() + words.len() - 1;
    let mut s = String::with_capacity(cap);
    for (i, w) in words.iter().enumerate() {
        if i > 0 {
            s.push(' ');
        }
        s.push_str(w);
    }
    s
}

fn format_type_name_comment_tag(
    normalized_kind: &str,
    tag: &oxc_jsdoc::parser::JSDocTag<'_>,
    should_capitalize: bool,
    wrap_width: usize,
    has_no_space_before_type: bool,
    bracket_spacing: bool,
    format_options: &FormatOptions,
    type_format_options: &FormatOptions,
    external_callbacks: &ExternalCallbacks,
    allocator: &Allocator,
    content_lines: &mut LineBuffer,
) {
    let (type_part, name_part, comment_part) = tag.type_name_comment();

    let tag_prefix_len = 1 + normalized_kind.len();
    let mut tag_line = String::with_capacity(tag_prefix_len + 32);
    tag_line.push('@');
    tag_line.push_str(normalized_kind);
    let mut is_type_optional = false;
    let mut normalized_type_str: Cow<'_, str> = Cow::Borrowed("");

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
                    format_type_via_formatter(&normalized_type_str, type_format_options, allocator)
            {
                normalized_type_str = Cow::Owned(formatted);
            }
        }
    }

    // Build name string and extract default value
    let mut name_str: &str = "";
    let mut default_value: Option<&str> = None;
    if let Some(np) = &name_part {
        let name_raw = np.raw();
        if is_type_optional && !name_raw.starts_with('[') {
            name_str = allocator.alloc_concat_strs_array(["[", name_raw, "]"]);
        } else if name_raw.starts_with('[') && name_raw.ends_with(']') {
            if let Some(eq_pos) = name_raw.find('=') {
                let name_part_inner = &name_raw[1..eq_pos];
                let val = name_raw[eq_pos + 1..name_raw.len() - 1].trim();
                if val.is_empty() {
                    name_str = allocator.alloc_concat_strs_array(["[", name_part_inner, "]"]);
                } else {
                    default_value = Some(val);
                    name_str =
                        allocator.alloc_concat_strs_array(["[", name_part_inner, "=", val, "]"]);
                }
            } else {
                name_str = name_raw;
            }
        } else {
            name_str = name_raw;
        }
    }

    // Build the full tag line (tag_line already contains "@{normalized_kind}")
    if !normalized_type_str.is_empty() {
        let preserve_no_space = has_no_space_before_type && !normalized_type_str.starts_with('{');
        if !preserve_no_space {
            tag_line.push(' ');
        }
        if bracket_spacing {
            tag_line.push_str("{ ");
        } else {
            tag_line.push('{');
        }
        tag_line.push_str(&normalized_type_str);
        if bracket_spacing {
            tag_line.push_str(" }");
        } else {
            tag_line.push('}');
        }
    }
    if !name_str.is_empty() {
        tag_line.push(' ');
        tag_line.push_str(name_str);
    }

    let desc_raw = comment_part.parsed_preserving_whitespace();
    let desc_raw = desc_raw.trim();
    let desc_normalized = normalize_markdown_emphasis(desc_raw);
    let desc_raw = desc_normalized.trim();

    // Strip existing "Default is ..." from description when we have an actual default value
    let desc_raw = if default_value.is_some() {
        strip_default_is_suffix(desc_raw)
    } else {
        Cow::Borrowed(desc_raw)
    };
    let desc_raw = desc_raw.trim();

    if desc_raw.is_empty() && default_value.is_none() {
        // Try type wrapping if line is too long
        if tag_line.len() > wrap_width
            && !normalized_type_str.is_empty()
            && wrap_type_expression(
                &tag_line[..tag_prefix_len],
                &normalized_type_str,
                name_str,
                wrap_width,
                content_lines,
            )
        {
            return;
        }
        content_lines.push(tag_line);
        return;
    }

    // Split description into first line and rest (avoids collecting all lines)
    let (first_text_line, rest_of_desc) = match desc_raw.split_once('\n') {
        Some((first, rest)) => (first.trim(), Some(rest)),
        None => (desc_raw.trim(), None),
    };

    // If the description starts with a code fence, output the tag line alone
    // and treat the entire description as structural content with a blank line separator
    if first_text_line.starts_with("```") {
        content_lines.push(tag_line);
        content_lines.push_empty();
        let indent = if matches!(normalized_kind, "typedef" | "callback") { "" } else { "  " };
        let indent_width = wrap_width.saturating_sub(indent.len());
        let mut desc = wrap_text(
            desc_raw,
            indent_width,
            Some(format_options),
            Some(external_callbacks),
            Some(allocator),
        );
        // Skip leading blank line from wrap_text since we already added one
        if desc.starts_with('\n') {
            desc.remove(0);
        }
        push_indented_desc(content_lines, indent, desc);
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

    let first_text: Cow<'_, str> =
        if should_capitalize { capitalize_first(first_text) } else { Cow::Borrowed(first_text) };

    // Default suffix length: "Default is `" (12) + value + "`" (1) = 13 + dv.len()
    let default_suffix_len: Option<usize> = default_value.map(|dv| 13 + dv.len());

    if first_text.is_empty() && default_suffix_len.is_none() && rest_of_desc.is_none() {
        content_lines.push(tag_line);
        return;
    }

    // Build the separator between tag+name and description
    let separator = if has_dash { " - " } else { " " };

    // Check if the description has extra content beyond the first text line
    // (subsequent lines with text, tables, code blocks, etc.)
    // Strip the common leading whitespace from continuation lines — this is
    // just the original JSDoc formatting indent, not semantic content.
    let remaining_desc = if let Some(rest) = rest_of_desc {
        join_iter(rest.lines().map(str::trim), "\n")
    } else {
        String::new()
    };
    let has_remaining = !remaining_desc.trim().is_empty();

    // Compute one-liner length without allocating
    let prefix_len = tag_line.len() + separator.len();
    let one_liner_len = if has_remaining {
        prefix_len + first_text.len()
    } else if let Some(ds_len) = default_suffix_len {
        if first_text.is_empty() {
            prefix_len + ds_len
        } else {
            // +2 for ". " or " " before default suffix
            prefix_len + first_text.len() + 2 + ds_len
        }
    } else {
        prefix_len + first_text.len()
    };

    if !has_remaining && one_liner_len <= wrap_width {
        // Fits on one line — write directly into LineBuffer
        let s = content_lines.begin_line();
        s.push_str(&tag_line);
        s.push_str(separator);
        if let Some(dv) = default_value {
            if first_text.is_empty() {
                s.push_str("Default is `");
                s.push_str(dv);
                s.push('`');
            } else {
                s.push_str(&first_text);
                let last_char = first_text.as_bytes().last().copied().unwrap_or(b' ');
                if matches!(last_char, b'.' | b'!' | b'?') {
                    s.push(' ');
                } else {
                    s.push_str(". ");
                }
                s.push_str("Default is `");
                s.push_str(dv);
                s.push('`');
            }
        } else {
            s.push_str(&first_text);
        }
    } else {
        // Multi-line: wrap first text line with tag line
        let first_line_content_width = wrap_width.saturating_sub(prefix_len);

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
            let s = content_lines.begin_line();
            s.push_str(&tag_line);
            s.push_str(separator);
            s.push_str(&first_line);
        }

        // @typedef/@callback descriptions use no indent (plugin passes no beginningSpace).
        // @param/@property and other tags use 2-space continuation indent.
        let indent = if matches!(normalized_kind, "typedef" | "callback") { "" } else { "  " };
        let indent_width = wrap_width.saturating_sub(indent.len());

        // Remaining words from first text line
        let mut remaining_first_text = String::new();
        if remaining_start < words.len() {
            remaining_first_text = join_words(&words[remaining_start..]);
        }

        // Combine remaining first text with remaining description lines
        // to preserve structural content (tables, code blocks, etc.)
        let full_remaining = if !remaining_first_text.is_empty() && has_remaining {
            let mut s =
                String::with_capacity(remaining_first_text.len() + 1 + remaining_desc.len());
            s.push_str(&remaining_first_text);
            s.push('\n');
            s.push_str(&remaining_desc);
            s
        } else if !remaining_first_text.is_empty() {
            remaining_first_text
        } else {
            remaining_desc
        };

        let full_remaining = full_remaining.trim();
        if !full_remaining.is_empty() {
            let desc = wrap_text(
                full_remaining,
                indent_width,
                Some(format_options),
                Some(external_callbacks),
                Some(allocator),
            );

            // In markdown, a list or table after a paragraph needs a blank line separator.
            // The plugin's markdown AST processing (remark) handles this naturally.
            // We detect when the first wrapped content is a list item or table row
            // and insert a blank line between the tag's first text line and the
            // structured content.
            let first_desc_is_structural = desc.split('\n').next().is_some_and(|first| {
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
                content_lines.push_empty();
            }

            push_indented_desc(content_lines, indent, desc);
        }

        // Add default value as a separate paragraph with blank line
        if let Some(dv) = default_value
            && !first_text.is_empty()
        {
            content_lines.push_empty();
            let s = content_lines.begin_line();
            s.push_str(indent);
            s.push_str("Default is `");
            s.push_str(dv);
            s.push('`');
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
    type_format_options: &FormatOptions,
    external_callbacks: &ExternalCallbacks,
    allocator: &Allocator,
    content_lines: &mut LineBuffer,
) {
    let (type_part, comment_part) = tag.type_comment();

    let tag_prefix_len = 1 + normalized_kind.len();
    let mut normalized_type_str: Cow<'_, str> = Cow::Borrowed("");
    let mut tag_line = String::with_capacity(tag_prefix_len + 32);
    tag_line.push('@');
    tag_line.push_str(normalized_kind);

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
            // Try formatting via the formatter (simulates upstream's formatType()).
            // For @type/@satisfies with no-space-before-type and non-object types,
            // skip to preserve quotes (e.g. @type{import('...')} stays unchanged).
            // Object types (starting with `{`) always get formatted.
            let skip_formatter = preserve_quotes
                && has_no_space_before_type
                && !normalized_type_str.starts_with('{');
            if !skip_formatter
                && let Some(formatted) =
                    format_type_via_formatter(&normalized_type_str, type_format_options, allocator)
            {
                normalized_type_str = Cow::Owned(formatted);
            }
            // Preserve no-space only when the type isn't an object literal
            // (object types start with `{`, making `@type{{` → should be `@type {{`)
            let preserve_no_space =
                has_no_space_before_type && !normalized_type_str.starts_with('{');
            if !preserve_no_space {
                tag_line.push(' ');
            }
            if bracket_spacing {
                tag_line.push_str("{ ");
            } else {
                tag_line.push('{');
            }
            tag_line.push_str(&normalized_type_str);
            if bracket_spacing {
                tag_line.push_str(" }");
            } else {
                tag_line.push('}');
            }
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
                &tag_line[..tag_prefix_len],
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

    let desc_text: Cow<'_, str> =
        if should_capitalize { capitalize_first(desc_text) } else { Cow::Borrowed(desc_text) };

    let prefix_len = tag_line.len() + 1; // tag_line + " "
    let one_liner_len = prefix_len + desc_text.len();
    if one_liner_len <= wrap_width {
        let s = content_lines.begin_line();
        s.push_str(&tag_line);
        s.push(' ');
        s.push_str(&desc_text);
    } else if !normalized_type_str.is_empty()
        && tag_line.len() > wrap_width
        && wrap_type_expression(
            &tag_line[..tag_prefix_len],
            &normalized_type_str,
            "",
            wrap_width,
            content_lines,
        )
    {
        // Type was wrapped. Add description as continuation.
        let indent = "  ";
        let indent_width = wrap_width.saturating_sub(indent.len());
        let desc = wrap_text(
            &desc_text,
            indent_width,
            Some(format_options),
            Some(external_callbacks),
            Some(allocator),
        );
        push_indented_desc(content_lines, indent, desc);
    } else {
        // Regular word-wrapping of description
        let first_line_content_width = wrap_width.saturating_sub(prefix_len);
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
            let s = content_lines.begin_line();
            s.push_str(&tag_line);
            s.push(' ');
            s.push_str(&first_line);
        }

        let indent = "  ";
        let indent_width = wrap_width.saturating_sub(indent.len());
        if remaining_start < words.len() {
            let remaining = join_words(&words[remaining_start..]);
            let desc = wrap_text(
                &remaining,
                indent_width,
                Some(format_options),
                Some(external_callbacks),
                Some(allocator),
            );
            push_indented_desc(content_lines, indent, desc);
        }
    }
}

fn format_generic_tag(
    normalized_kind: &str,
    tag: &oxc_jsdoc::parser::JSDocTag<'_>,
    should_capitalize: bool,
    wrap_width: usize,
    quote_style: QuoteStyle,
    format_options: &FormatOptions,
    external_callbacks: &ExternalCallbacks,
    allocator: &Allocator,
    content_lines: &mut LineBuffer,
) {
    let mut tag_line = String::with_capacity(normalized_kind.len() + 1);
    tag_line.push('@');
    tag_line.push_str(normalized_kind);
    let desc_text = tag.comment().parsed();
    let desc_text = normalize_markdown_emphasis(desc_text.trim());
    let desc_text = desc_text.trim();

    if desc_text.is_empty() {
        content_lines.push(tag_line);
        return;
    }

    // For @default/@defaultValue, format JSON-like values
    let desc_text: Cow<'_, str> = if matches!(normalized_kind, "default" | "defaultValue") {
        format_default_value(desc_text, quote_style)
    } else if should_capitalize && is_named_generic_tag(normalized_kind) {
        // Named tags: first word is the "name" (don't capitalize), rest is description.
        // Upstream comment-parser separates name/description; we do it inline.
        if let Some(space_idx) = desc_text.find(|c: char| c.is_ascii_whitespace()) {
            let name_part = &desc_text[..space_idx];
            let desc_part = desc_text[space_idx..].trim_start();
            if desc_part.is_empty() {
                Cow::Borrowed(desc_text)
            } else {
                let capitalized = capitalize_first(desc_part);
                let mut s = String::with_capacity(name_part.len() + 1 + capitalized.len());
                s.push_str(name_part);
                s.push(' ');
                s.push_str(&capitalized);
                Cow::Owned(s)
            }
        } else {
            // Only a name, no description — no capitalization needed
            Cow::Borrowed(desc_text)
        }
    } else if should_capitalize {
        capitalize_first(desc_text)
    } else {
        Cow::Borrowed(desc_text)
    };

    let prefix_len = tag_line.len() + 1; // tag_line + " "
    if prefix_len + desc_text.len() <= wrap_width {
        let s = content_lines.begin_line();
        s.push_str(&tag_line);
        s.push(' ');
        s.push_str(&desc_text);
    } else {
        // Try to fit some description on the first line
        let first_line_content_width = wrap_width.saturating_sub(prefix_len);
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
            let s = content_lines.begin_line();
            s.push_str(&tag_line);
            s.push(' ');
            s.push_str(&first_line);
        }

        let indent = "  ";
        let indent_width = wrap_width.saturating_sub(indent.len());
        if remaining_start < words.len() {
            let remaining = join_words(&words[remaining_start..]);
            let desc = wrap_text(
                &remaining,
                indent_width,
                Some(format_options),
                Some(external_callbacks),
                Some(allocator),
            );
            push_indented_desc(content_lines, indent, desc);
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
    let text: String = join_iter(comment_text.lines().map(str::trim), " ");
    let text = text.trim();

    // Find "from" keyword followed by a quoted string
    let from_idx = text.rfind(" from ")?;
    let specifier = text[..from_idx].trim();
    let module_part = text[from_idx + 6..].trim();

    // Extract module path (strip matching quotes)
    let quote = match module_part.as_bytes().first() {
        Some(b'"' | b'\'') => module_part.as_bytes()[0] as char,
        _ => return None,
    };
    let module_path = module_part.strip_prefix(quote)?.strip_suffix(quote)?;
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
                join_iter(s.split_whitespace(), " ")
            })
            .filter(|s| !s.is_empty())
            .collect();

        (default_import, named_imports)
    } else {
        // No braces — just a default import
        let name = join_iter(specifier.split_whitespace(), " ");
        (Some(name), Vec::new())
    };

    Some(ImportInfo { default_import, named_imports, module_path: module_path.to_string() })
}

/// Get the sort key for a named import specifier (sort by alias).
/// `"B as B1"` → `"B1"`, `"B2"` → `"B2"`.
fn import_specifier_sort_key(specifier: &str) -> &str {
    if let Some(idx) = specifier.find(" as ") {
        specifier[idx + 4..].trim()
    } else {
        specifier.trim()
    }
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
fn format_import_lines(import: &ImportInfo, content_lines: &mut LineBuffer) {
    let module = &import.module_path;

    match (&import.default_import, import.named_imports.len()) {
        (Some(default), 0) => {
            let s = content_lines.begin_line();
            s.push_str("@import ");
            s.push_str(default);
            s.push_str(" from \"");
            s.push_str(module);
            s.push('"');
        }
        (None, 1) => {
            let s = content_lines.begin_line();
            s.push_str("@import {");
            s.push_str(&import.named_imports[0]);
            s.push_str("} from \"");
            s.push_str(module);
            s.push('"');
        }
        (Some(default), 1) => {
            let s = content_lines.begin_line();
            s.push_str("@import ");
            s.push_str(default);
            s.push_str(", {");
            s.push_str(&import.named_imports[0]);
            s.push_str("} from \"");
            s.push_str(module);
            s.push('"');
        }
        (None, n) if n >= 2 => {
            content_lines.push("@import {");
            for (i, named) in import.named_imports.iter().enumerate() {
                let s = content_lines.begin_line();
                s.push_str("  ");
                s.push_str(named);
                if i < import.named_imports.len() - 1 {
                    s.push(',');
                }
            }
            let s = content_lines.begin_line();
            s.push_str("} from \"");
            s.push_str(module);
            s.push('"');
        }
        (Some(default), n) if n >= 2 => {
            let s = content_lines.begin_line();
            s.push_str("@import ");
            s.push_str(default);
            s.push_str(", {");
            for (i, named) in import.named_imports.iter().enumerate() {
                let s = content_lines.begin_line();
                s.push_str("  ");
                s.push_str(named);
                if i < import.named_imports.len() - 1 {
                    s.push(',');
                }
            }
            let s = content_lines.begin_line();
            s.push_str("} from \"");
            s.push_str(module);
            s.push('"');
        }
        _ => {}
    }
}

/// Process all `@import` tags: parse, merge by module, sort, and format.
/// Returns formatted lines ready to be inserted into the comment, plus
/// the set of tag indices that were successfully parsed (so unparsable
/// `@import` tags can fall through to `format_generic_tag()`).
fn process_import_tags(
    tags: &[(&oxc_jsdoc::parser::JSDocTag<'_>, &str)],
) -> (LineBuffer, smallvec::SmallVec<[usize; 4]>) {
    let mut imports = Vec::new();
    let mut parsed_indices = smallvec::SmallVec::<[usize; 4]>::new();

    for (idx, &(tag, kind)) in tags.iter().enumerate() {
        if kind != "import" {
            continue;
        }
        let comment = tag.comment().parsed();
        if let Some(info) = parse_import_tag(&comment) {
            imports.push(info);
            parsed_indices.push(idx);
        }
    }

    let merged = merge_and_sort_imports(imports);

    let mut lines = LineBuffer::new();
    for import in &merged {
        format_import_lines(import, &mut lines);
    }
    (lines, parsed_indices)
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
        // Tags in TAGS_PEV_FORMAT_DESCRIPTION
        assert!(should_skip_capitalize("borrows"));
        assert!(should_skip_capitalize("default"));
        assert!(should_skip_capitalize("defaultValue"));
        assert!(should_skip_capitalize("import"));
        assert!(should_skip_capitalize("memberof"));
        assert!(should_skip_capitalize("module"));
        assert!(should_skip_capitalize("see"));

        // Tags that SHOULD capitalize (not in TAGS_PEV_FORMAT_DESCRIPTION)
        assert!(!should_skip_capitalize("param"));
        assert!(!should_skip_capitalize("returns"));
        assert!(!should_skip_capitalize("deprecated"));
        assert!(!should_skip_capitalize("function"));
        assert!(!should_skip_capitalize("typedef"));
        assert!(!should_skip_capitalize("class"));
        assert!(!should_skip_capitalize("callback"));
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
        let allocator = Allocator::default();
        format_type_via_formatter(type_str, &FormatOptions::default(), &allocator)
    }

    #[test]
    fn test_format_type_via_formatter() {
        // Simple types return None (no formatting needed — fast path)
        assert_eq!(fmt_type("string"), None);
        assert_eq!(fmt_type("number"), None);
        // Types with operators go through the formatter
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
