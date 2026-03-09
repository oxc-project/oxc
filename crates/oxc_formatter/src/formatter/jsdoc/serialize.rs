use std::borrow::Cow;

use oxc_allocator::Allocator;
use oxc_ast::Comment;
use oxc_jsdoc::JSDoc;
use oxc_span::Span;

use crate::ExternalCallbacks;
use crate::FormatOptions;
use crate::formatter::Formatter;
use crate::formatter::prelude::*;
use crate::options::{JsdocOptions, QuoteStyle};
use crate::write;

use super::{
    imports::process_import_tags, line_buffer::LineBuffer,
    mdast_serialize::format_description_mdast, normalize::normalize_tag_kind,
    param_order::reorder_param_tags,
};

/// Result of formatting a JSDoc comment, ready to be emitted as IR.
pub enum FormattedJsdoc<'a> {
    /// Empty JSDoc (should be removed) — matches current `Some("")` behavior.
    Empty,
    /// Single-line: inner content only (e.g. "Description here").
    SingleLine(&'a str),
    /// Multi-line: \n-separated content lines (no `/** */` wrapper, no ` * ` prefixes).
    MultiLine(&'a str),
}

impl<'a> Format<'a> for FormattedJsdoc<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        match self {
            FormattedJsdoc::Empty => {}
            FormattedJsdoc::SingleLine(content) => {
                write!(f, [token("/**"), " ", text(content), " ", token("*/")]);
            }
            FormattedJsdoc::MultiLine(content_str) => {
                write!(f, [token("/**")]);
                for line in content_str.split('\n') {
                    if line.is_empty() {
                        write!(f, [hard_line_break(), " ", token("*")]);
                    } else {
                        write!(f, [hard_line_break(), " ", token("*"), " ", text(line)]);
                    }
                }
                write!(f, [hard_line_break(), " ", token("*/")]);
            }
        }
    }
}

/// The ` * ` prefix used in multiline JSDoc comments (3 chars).
const LINE_PREFIX_LEN: usize = 3;

/// Holds the shared per-comment state for JSDoc formatting,
/// reducing parameter passing across formatting functions.
///
/// Uses two lifetimes: `'a` for the allocator (tied to output strings)
/// and `'o` for options/callbacks (only need to live as long as the formatter).
pub(super) struct JsdocFormatter<'a, 'o> {
    pub(super) options: &'o JsdocOptions,
    pub(super) format_options: &'o FormatOptions,
    /// FormatOptions for type formatting — copies only scalar fields, setting
    /// Vec-containing options (sort_imports, sort_tailwindcss, jsdoc) to None.
    pub(super) type_format_options: FormatOptions,
    pub(super) external_callbacks: &'o ExternalCallbacks,
    pub(super) allocator: &'a Allocator,
    pub(super) wrap_width: usize,
    pub(super) content_lines: LineBuffer,
}

impl<'a, 'o> JsdocFormatter<'a, 'o> {
    fn new(
        options: &'o JsdocOptions,
        format_options: &'o FormatOptions,
        external_callbacks: &'o ExternalCallbacks,
        allocator: &'a Allocator,
        available_width: usize,
    ) -> Self {
        let wrap_width = available_width.saturating_sub(LINE_PREFIX_LEN);
        Self {
            options,
            format_options,
            type_format_options: FormatOptions {
                sort_imports: None,
                sort_tailwindcss: None,
                jsdoc: None,
                ..*format_options
            },
            external_callbacks,
            allocator,
            wrap_width,
            content_lines: LineBuffer::new(),
        }
    }

    /// Format a JSDoc comment. Returns `Some(formatted)` if the comment was modified,
    /// `None` if no changes are needed.
    fn format(mut self, comment: &Comment, source_text: &str) -> Option<FormattedJsdoc<'a>> {
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
            return Some(FormattedJsdoc::Empty);
        }

        // Sort tags by priority within groups.
        // @typedef and @callback are TAGS_GROUP_HEAD — they start new groups.
        // Tags sort within their group by weight, but groups keep their relative order.
        let tags = jsdoc.tags();
        let sorted_tags = sort_tags_by_groups(tags);

        // Merge all description sources: header description + @description tags
        // (upstream always merges these, regardless of the description_tag option)
        let desc_trimmed = description.trim();
        let mut merged_desc: Option<String> =
            if desc_trimmed.is_empty() { None } else { Some(desc_trimmed.to_string()) };
        // Collect effective tags, absorbing @description tag content.
        // Track whether any @import tags exist to skip import processing.
        let mut effective_tags: Vec<(&oxc_jsdoc::parser::JSDocTag<'_>, &str)> =
            Vec::with_capacity(sorted_tags.len());
        let mut has_import_tags = false;
        for (tag, normalized_kind) in &sorted_tags {
            if should_remove_empty_tag(normalized_kind) && !tag_has_content(tag) {
                continue;
            }
            if *normalized_kind == "description" {
                let desc_content = tag.comment().parsed();
                let desc_content = desc_content.trim();
                if !desc_content.is_empty() {
                    let desc = merged_desc.get_or_insert_with(String::new);
                    if !desc.is_empty() {
                        desc.push_str("\n\n");
                    }
                    desc.push_str(desc_content);
                }
                continue;
            }
            if *normalized_kind == "import" {
                has_import_tags = true;
            }
            effective_tags.push((tag, normalized_kind));
        }

        // Format and emit the merged description
        if let Some(merged_desc) = &merged_desc {
            let desc = format_description_mdast(
                merged_desc,
                self.wrap_width,
                0,
                self.options.capitalize_descriptions,
                Some(self.format_options),
                Some(self.external_callbacks),
                Some(self.allocator),
            );
            if self.options.description_tag {
                // Emit as @description tag
                let first_line = format!("@description {desc}");
                self.content_lines.push(first_line);
            } else {
                self.content_lines.push(desc);
            }
        }

        // Reorder @param tags to match the function signature order
        reorder_param_tags(&mut effective_tags, comment, source_text);

        // Pre-process @import tags: merge by module, sort, format.
        // Skip entirely when no @import tags exist (common case) to avoid allocation.
        let (mut import_lines, parsed_import_indices) = if has_import_tags {
            let (lines, indices) = process_import_tags(&effective_tags);
            (Some(lines), indices)
        } else {
            (None, smallvec::SmallVec::new())
        };
        let has_imports = import_lines.as_ref().is_some_and(|l| !l.is_empty());
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
                    if !self.content_lines.is_empty() && !self.content_lines.last_is_empty() {
                        self.content_lines.push_empty();
                    }
                    let import_str = import_lines.take().unwrap().into_string();
                    self.content_lines.push(import_str);
                    imports_emitted = true;
                    prev_normalized_kind = Some("import");
                }
                continue;
            }

            let is_first_tag = !first_non_import_tag_emitted && !imports_emitted;

            let should_capitalize = self.options.capitalize_descriptions
                && !should_skip_capitalize(normalized_kind)
                && is_known_tag(normalized_kind);

            // Add blank line between description and first tag
            if is_first_tag && !self.content_lines.is_empty() && !self.content_lines.last_is_empty()
            {
                self.content_lines.push_empty();
            }

            // Add blank lines between tag groups
            if !is_first_tag {
                let should_separate = if prev_normalized_kind.is_some_and(|prev| prev == "example")
                    && normalized_kind == "example"
                {
                    // Always blank line between consecutive @example tags
                    true
                } else if self.options.separate_tag_groups {
                    // Blank line between different tag kinds
                    prev_normalized_kind.is_some_and(|prev| prev != normalized_kind)
                } else if self.options.separate_returns_from_param {
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

                if should_separate && !self.content_lines.last_is_empty() {
                    self.content_lines.push_empty();
                }
            }

            first_non_import_tag_emitted = true;
            prev_normalized_kind = Some(normalized_kind);

            // Track content before formatting this tag
            let lines_before = self.content_lines.byte_len();

            // Detect if original has no space between tag kind and `{type}`
            // e.g., `@type{import(...)}` vs `@type {import(...)}`
            let has_no_space_before_type = {
                let kind_end = tag.kind.span.end as usize;
                kind_end < source_text.len() && source_text.as_bytes()[kind_end] == b'{'
            };

            if normalized_kind == "example" || normalized_kind == "remarks" {
                self.format_example_tag(normalized_kind, tag);
            } else if is_type_name_comment_tag(normalized_kind) {
                self.format_type_name_comment_tag(
                    normalized_kind,
                    tag,
                    should_capitalize,
                    has_no_space_before_type,
                );
            } else if is_type_comment_tag(normalized_kind) {
                self.format_type_comment_tag(
                    normalized_kind,
                    tag,
                    should_capitalize,
                    has_no_space_before_type,
                );
            } else {
                self.format_generic_tag(normalized_kind, tag, should_capitalize);
            }

            // If this tag has multi-paragraph content (blank lines within, or is an @example tag
            // with multi-line code) and the next tag is of a different kind, add a trailing
            // blank line for separation.
            let tag_content_has_blank_lines = self.content_lines.has_blank_line_since(lines_before);
            // line_count_since counts \n separators added since the snapshot; ≥2 means multi-line.
            let tag_newline_count = self.content_lines.line_count_since(lines_before);
            let is_example_multiline = normalized_kind == "example" && tag_newline_count > 1;
            if (tag_content_has_blank_lines || is_example_multiline)
                && let Some(&(_, next_kind)) = effective_tags.get(tag_idx + 1)
                && next_kind != normalized_kind
                && !self.content_lines.last_is_empty()
            {
                self.content_lines.push_empty();
            }
        }

        // Get the full content as a single string and iterate lines,
        // trimming leading and trailing blank lines.
        let content_str = self.content_lines.into_string();
        let content_str = content_str.trim_end_matches('\n');
        let mut iter = content_str.split('\n').skip_while(|l| l.is_empty());

        let Some(first) = iter.next() else {
            return Some(FormattedJsdoc::Empty);
        };

        // Single-line check: convert to single-line if content is a single line.
        // The plugin prefers single-line even if it slightly exceeds printWidth,
        // since the wrapping logic already constrains the content width.
        let second = iter.next();
        let use_single_line = match self.options.comment_line_strategy {
            crate::options::CommentLineStrategy::SingleLine => second.is_none(),
            crate::options::CommentLineStrategy::Multiline => false,
            crate::options::CommentLineStrategy::Keep => {
                // Preserve original: only use single-line if original was single-line
                second.is_none() && !content.contains('\n')
            }
        };
        if use_single_line {
            // Build temp string for comparison without arena-allocating
            let mut tmp = String::with_capacity(4 + first.len() + 3);
            tmp.push_str("/** ");
            tmp.push_str(first);
            tmp.push_str(" */");
            if tmp == content {
                return None;
            }
            let alloc_first = self.allocator.alloc_str(first);
            return Some(FormattedJsdoc::SingleLine(alloc_first));
        }

        // Build temp string with `/** ... */` wrapper for comparison against original
        let capacity =
            content_str.len() + content_str.bytes().filter(|&b| b == b'\n').count() * 4 + 10;
        let mut tmp = String::with_capacity(capacity);
        tmp.push_str("/**");

        for line in std::iter::once(first).chain(second).chain(iter) {
            tmp.push('\n');
            if line.is_empty() {
                tmp.push_str(" *");
            } else {
                tmp.push_str(" * ");
                tmp.push_str(line);
            }
        }
        tmp.push('\n');
        tmp.push_str(" */");

        // Compare with original — if unchanged, return None
        if tmp == content {
            return None;
        }

        // Arena-allocate only the inner content (without /** */ wrapper)
        let alloc_content = self.allocator.alloc_str(content_str);
        Some(FormattedJsdoc::MultiLine(alloc_content))
    }

    /// Push a (possibly multi-line) description into `content_lines` as a single string,
    /// prepending `indent` to each non-empty line. When indent is empty, moves `desc` directly.
    pub(super) fn push_indented_desc(&mut self, indent: &str, mut desc: String) {
        if desc.is_empty() {
            return;
        }
        if indent.is_empty() {
            self.content_lines.push(desc);
            return;
        }
        if !desc.contains('\n') {
            desc.insert_str(0, indent);
            self.content_lines.push(desc);
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
        self.content_lines.push(s);
    }

    /// Wrap a long type expression across multiple lines at `|` operators.
    /// Returns `true` if wrapping was performed.
    pub(super) fn wrap_type_expression(
        &mut self,
        tag_prefix: &str,
        type_str: &str,
        name_and_rest: &str,
    ) -> bool {
        // Only wrap if the full line exceeds the width
        let full_len = tag_prefix.len()
            + 2 // " {"
            + type_str.len()
            + if name_and_rest.is_empty() { 1 } else { 2 + name_and_rest.len() }; // "}" or "} name"
        if full_len <= self.wrap_width {
            return false;
        }

        // Check if the type contains `|` at the top level for union wrapping
        let parts = split_type_at_top_level_pipe(type_str);
        if parts.len() <= 1 {
            // Check for generic type `Foo<...>` wrapping at top-level angle bracket
            if let Some(wrapped) =
                wrap_generic_type(tag_prefix, type_str, name_and_rest, &mut self.content_lines)
            {
                return wrapped;
            }
            return false;
        }

        // Wrap union type at `|` operators
        let first_part = parts[0].trim();
        {
            let s = self.content_lines.begin_line();
            s.push_str(tag_prefix);
            s.push_str(" {");
            s.push_str(first_part);
        }

        for (i, part) in parts.iter().enumerate().skip(1) {
            let part = part.trim();
            let s = self.content_lines.begin_line();
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

    /// Whether bracket spacing is enabled.
    pub(super) fn bracket_spacing(&self) -> bool {
        self.options.bracket_spacing
    }

    /// The configured quote style.
    pub(super) fn quote_style(&self) -> QuoteStyle {
        self.format_options.quote_style
    }
}

/// Trim trailing whitespace from an owned `String` in place, avoiding a reallocation.
pub(super) fn truncate_trim_end(s: &mut String) {
    let trimmed_len = s.trim_end().len();
    s.truncate(trimmed_len);
}

/// Join an iterator of string slices with a separator, avoiding an intermediate `Vec`.
/// Uses `size_hint()` for a rough capacity estimate to reduce reallocations.
pub(super) fn join_iter<'a>(iter: impl Iterator<Item = &'a str>, sep: &str) -> String {
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
pub(super) fn is_type_name_comment_tag(tag_kind: &str) -> bool {
    matches!(tag_kind, "param" | "property" | "typedef" | "template")
}

/// Tags that use `type_comment()` pattern: `@tag {type} description`
/// Expects canonical (normalized) tag names.
pub(super) fn is_type_comment_tag(tag_kind: &str) -> bool {
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
pub(super) fn is_named_generic_tag(kind: &str) -> bool {
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
        // Skip sort if already in priority order (common case: well-formatted comments)
        if !sorted.windows(2).all(|w| tag_sort_priority(w[0].1) <= tag_sort_priority(w[1].1)) {
            sorted.sort_by_key(|(_, kind)| tag_sort_priority(kind));
        }
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
pub(super) fn format_default_value(value: &str, quote_style: QuoteStyle) -> Cow<'_, str> {
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
pub(super) fn strip_default_is_suffix(desc: &str) -> Cow<'_, str> {
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

/// Format a JSDoc comment. Returns `Some(formatted)` if the comment was modified,
/// `None` if no changes are needed.
///
/// The returned `FormattedJsdoc` implements `Format` and emits the `/** ... */`
/// wrapper directly as IR tokens.
pub fn format_jsdoc_comment<'a>(
    comment: &Comment,
    options: &JsdocOptions,
    source_text: &str,
    available_width: usize,
    f: &Formatter<'_, 'a>,
) -> Option<FormattedJsdoc<'a>> {
    let fmt = JsdocFormatter::new(
        options,
        f.options(),
        f.context().external_callbacks(),
        f.allocator(),
        available_width,
    );
    fmt.format(comment, source_text)
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
        use crate::formatter::jsdoc::embedded::format_type_via_formatter;
        let allocator = oxc_allocator::Allocator::default();
        format_type_via_formatter(type_str, &FormatOptions::default(), &allocator)
    }

    #[test]
    fn test_format_type_via_formatter() {
        // Simple types return None (no formatting needed — fast path)
        assert_eq!(fmt_type("string"), None);
        assert_eq!(fmt_type("number"), None);
        // Types with operators go through the formatter but return None when unchanged
        assert_eq!(fmt_type("string | number"), None);
        // Types that would actually change
        assert_eq!(fmt_type("string|number"), Some("string | number".to_string()));
        assert_eq!(fmt_type(""), None);
    }

    #[test]
    fn test_format_type_via_formatter_rest() {
        assert_eq!(fmt_type("...any"), Some("...any".to_string()));
        assert_eq!(fmt_type("...number"), Some("...number".to_string()));
        assert_eq!(fmt_type("...(string | number)"), Some("...(string | number)".to_string()));
    }
}
