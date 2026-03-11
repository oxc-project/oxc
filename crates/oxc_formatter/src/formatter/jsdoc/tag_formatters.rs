use std::borrow::Cow;

use super::{
    embedded::{
        format_embedded_js, format_type_via_formatter, is_js_ts_lang, update_template_depth,
    },
    normalize::{
        capitalize_first, normalize_markdown_emphasis, normalize_type,
        normalize_type_preserve_quotes, normalize_type_return, strip_optional_type_suffix,
    },
    serialize::{
        JsdocFormatter, format_default_value, is_named_generic_tag, join_iter,
        should_skip_description_formatting, strip_default_is_suffix,
    },
    wrap::{str_width, wrap_text},
};

impl JsdocFormatter<'_, '_> {
    pub(super) fn format_example_tag(
        &mut self,
        normalized_kind: &str,
        tag: &oxc_jsdoc::parser::JSDocTag<'_>,
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
                let s = self.content_lines.begin_line();
                s.push('@');
                s.push_str(normalized_kind);
                s.push_str(" <caption>");
                s.push_str(caption);
                s.push_str("</caption>");
            }
            self.format_example_code(after_caption);
            return;
        }

        {
            let s = self.content_lines.begin_line();
            s.push('@');
            s.push_str(normalized_kind);
        }
        self.format_example_code(trimmed);
    }

    /// Format example code content with 2-space base indent.
    /// Tries to format the code as JS/JSX first; falls back to pass-through on parse failure.
    fn format_example_code(&mut self, code: &str) {
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
                self.format_example_fenced_block(first_line, inner_code, closing_fence);
                return;
            } else if rest.trim() == "```" {
                // Only two lines: opening + closing fence, no inner code
                self.format_example_fenced_block(first_line, "", rest.trim());
                return;
            }
        }

        // Try formatting the code. The effective print width for @example code is
        // wrap_width - 2 (for the 2-space indent within the comment).
        let effective_width = self.wrap_width.saturating_sub(2);
        if let Some(formatted) =
            format_embedded_js(code, effective_width, self.format_options, self.allocator)
        {
            // Add 2-space indent to code structure lines, but NOT to template literal
            // content. The formatter preserves template literal content verbatim, so
            // adding indent to those lines would shift them incorrectly.
            let mut template_depth: u32 = 0;
            for line in formatted.lines() {
                if line.is_empty() {
                    self.content_lines.push_empty();
                } else if template_depth == 0 {
                    {
                        let s = self.content_lines.begin_line();
                        s.push_str("  ");
                        s.push_str(line);
                    }
                } else {
                    self.content_lines.push(line);
                }
                // Count unescaped backticks to track template literal depth
                template_depth = update_template_depth(line, template_depth);
            }
            return;
        }

        // Fallback: pass through with 2-space indent
        for line in code.lines() {
            let line_content =
                if self.options.keep_unparsable_example_indent { line } else { line.trim() };
            if line_content.is_empty() {
                self.content_lines.push_empty();
            } else {
                {
                    let s = self.content_lines.begin_line();
                    s.push_str("  ");
                    s.push_str(line_content);
                }
            }
        }
    }

    /// Handle fenced code blocks inside @example tags.
    /// Strips the ``` markers, formats the inner code, and re-adds fences
    /// with proper 2-space indentation.
    fn format_example_fenced_block(
        &mut self,
        lang_line: &str,
        inner_code: &str,
        closing_fence: &str,
    ) {
        let effective_width = self.wrap_width.saturating_sub(2);

        // Add opening fence with indent
        {
            let s = self.content_lines.begin_line();
            s.push_str("  ");
            s.push_str(lang_line);
        }

        if !inner_code.is_empty() {
            let lang = lang_line[3..].trim();
            if is_js_ts_lang(lang) {
                if let Some(formatted) = format_embedded_js(
                    inner_code,
                    effective_width,
                    self.format_options,
                    self.allocator,
                ) {
                    let mut template_depth: u32 = 0;
                    for line in formatted.lines() {
                        if line.is_empty() {
                            self.content_lines.push_empty();
                        } else if template_depth == 0 {
                            {
                                let s = self.content_lines.begin_line();
                                s.push_str("  ");
                                s.push_str(line);
                            }
                        } else {
                            self.content_lines.push(line);
                        }
                        template_depth = update_template_depth(line, template_depth);
                    }
                } else {
                    // Fallback for unparsable inner code
                    for line in inner_code.lines() {
                        let content = if self.options.keep_unparsable_example_indent {
                            line
                        } else {
                            line.trim()
                        };
                        if content.is_empty() {
                            self.content_lines.push_empty();
                        } else {
                            {
                                let s = self.content_lines.begin_line();
                                s.push_str("  ");
                                s.push_str(content);
                            }
                        }
                    }
                }
            } else {
                // Non-JS/TS fenced code: preserve with 2-space indent
                for line in inner_code.lines() {
                    let content = if self.options.keep_unparsable_example_indent {
                        line
                    } else {
                        line.trim()
                    };
                    if content.is_empty() {
                        self.content_lines.push_empty();
                    } else {
                        {
                            let s = self.content_lines.begin_line();
                            s.push_str("  ");
                            s.push_str(content);
                        }
                    }
                }
            }
        }

        // Add closing fence with indent
        {
            let s = self.content_lines.begin_line();
            s.push_str("  ");
            s.push_str(closing_fence);
        }
    }

    pub(super) fn format_type_name_comment_tag(
        &mut self,
        normalized_kind: &str,
        tag: &oxc_jsdoc::parser::JSDocTag<'_>,
        should_capitalize: bool,
        has_no_space_before_type: bool,
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
                    && let Some(formatted) = format_type_via_formatter(
                        &normalized_type_str,
                        &self.type_format_options,
                        self.allocator,
                    )
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
                name_str = self.allocator.alloc_concat_strs_array(["[", name_raw, "]"]);
            } else if name_raw.starts_with('[') && name_raw.ends_with(']') {
                if let Some(eq_pos) = name_raw.find('=') {
                    let name_part_inner = &name_raw[1..eq_pos];
                    let val = name_raw[eq_pos + 1..name_raw.len() - 1].trim();
                    if val.is_empty() {
                        name_str =
                            self.allocator.alloc_concat_strs_array(["[", name_part_inner, "]"]);
                    } else {
                        default_value = Some(val);
                        name_str = self.allocator.alloc_concat_strs_array([
                            "[",
                            name_part_inner,
                            "=",
                            val,
                            "]",
                        ]);
                    }
                } else {
                    name_str = name_raw;
                }
            } else {
                name_str = name_raw;
            }
        }

        // Build the full tag line (tag_line already contains "@{normalized_kind}")
        let bracket_spacing = self.bracket_spacing();
        if !normalized_type_str.is_empty() {
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
        if !name_str.is_empty() {
            tag_line.push(' ');
            tag_line.push_str(name_str);
        }

        let desc_raw = comment_part.parsed_preserving_whitespace();
        let desc_raw = desc_raw.trim();
        let desc_normalized = normalize_markdown_emphasis(desc_raw);
        let desc_raw = desc_normalized.trim();

        // Strip existing "Default is ..." from description when we have an actual default value
        // and `add_default_to_description` is enabled (we'll re-append a normalized version)
        let desc_raw = if default_value.is_some() && self.options.add_default_to_description {
            strip_default_is_suffix(desc_raw)
        } else {
            Cow::Borrowed(desc_raw)
        };
        let desc_raw = desc_raw.trim();

        if desc_raw.is_empty() && default_value.is_none() {
            // Try type wrapping if line is too long
            if str_width(&tag_line) > self.wrap_width
                && !normalized_type_str.is_empty()
                && self.wrap_type_expression(
                    &tag_line[..tag_prefix_len],
                    &normalized_type_str,
                    name_str,
                )
            {
                return;
            }
            self.content_lines.push(tag_line);
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
            self.content_lines.push(tag_line);
            self.content_lines.push_empty();
            let indent = if matches!(normalized_kind, "typedef" | "callback") { "" } else { "  " };
            let indent_width = self.wrap_width.saturating_sub(str_width(indent));
            let mut desc = wrap_text(
                desc_raw,
                indent_width,
                0,
                false,
                Some(self.format_options),
                Some(self.external_callbacks),
                Some(self.allocator),
            );
            // Skip leading blank line from wrap_text since we already added one
            if desc.starts_with('\n') {
                desc.remove(0);
            }
            self.push_indented_desc(indent, desc);
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

        let first_text: Cow<'_, str> = if should_capitalize {
            capitalize_first(first_text)
        } else {
            Cow::Borrowed(first_text)
        };

        // When add_default_to_description is false, don't append "Default is ..." to description
        let default_value_for_desc =
            if self.options.add_default_to_description { default_value } else { None };

        // Default suffix length: "Default is `" (12) + value + "`" (1) = 13 + dv.len()
        let default_suffix_len: Option<usize> = default_value_for_desc.map(|dv| 13 + dv.len());

        if first_text.is_empty() && default_suffix_len.is_none() && rest_of_desc.is_none() {
            self.content_lines.push(tag_line);
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
        let prefix_len = str_width(&tag_line) + str_width(separator);
        let one_liner_len = if has_remaining {
            prefix_len + str_width(&first_text)
        } else if let Some(ds_len) = default_suffix_len {
            if first_text.is_empty() {
                prefix_len + ds_len
            } else {
                // +2 for ". " or " " before default suffix
                prefix_len + str_width(&first_text) + 2 + ds_len
            }
        } else {
            prefix_len + str_width(&first_text)
        };

        if !has_remaining && one_liner_len <= self.wrap_width {
            // Fits on one line — write directly into LineBuffer
            let s = self.content_lines.begin_line();
            s.push_str(&tag_line);
            s.push_str(separator);
            if let Some(dv) = default_value_for_desc {
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
            } else if self.options.description_with_dot {
                let dotted = super::normalize::append_trailing_dot(&first_text);
                s.push_str(&dotted);
            } else {
                s.push_str(&first_text);
            }
        } else {
            // Multi-line: pass full description through wrap_text with tag_string_length.
            // This matches upstream's approach of passing the entire description through
            // formatDescription with a tagStringLength parameter that controls first-line offset.
            let indent = if matches!(normalized_kind, "typedef" | "callback") { "" } else { "  " };
            let indent_width = self.wrap_width.saturating_sub(str_width(indent));

            // Build full description text (first line + remaining)
            let full_desc = if has_remaining {
                let mut s = String::with_capacity(first_text.len() + 1 + remaining_desc.len());
                s.push_str(&first_text);
                s.push('\n');
                s.push_str(&remaining_desc);
                s
            } else {
                String::from(first_text.as_ref())
            };

            let tag_str_len = prefix_len.saturating_sub(str_width(indent));

            // Upstream: tagString.length + firstWord.length > printWidth → new line
            let first_word_w = full_desc.split_whitespace().next().map_or(0, str_width);
            if prefix_len + first_word_w >= self.wrap_width {
                // Tag prefix + first word don't fit → description starts on new line
                self.content_lines.push(tag_line);
                let desc = wrap_text(
                    &full_desc,
                    indent_width,
                    0,
                    false,
                    Some(self.format_options),
                    Some(self.external_callbacks),
                    Some(self.allocator),
                );
                self.push_indented_desc(indent, desc);
            } else {
                // Append description inline with tag_string_length offset
                let desc = wrap_text(
                    &full_desc,
                    indent_width,
                    tag_str_len,
                    false,
                    Some(self.format_options),
                    Some(self.external_callbacks),
                    Some(self.allocator),
                );
                let mut iter = desc.split('\n');
                if let Some(first) = iter.next() {
                    let s = self.content_lines.begin_line();
                    s.push_str(&tag_line);
                    s.push_str(separator);
                    s.push_str(first);
                }
                for line in iter {
                    if line.is_empty() {
                        self.content_lines.push_empty();
                    } else {
                        let s = self.content_lines.begin_line();
                        s.push_str(indent);
                        s.push_str(line);
                    }
                }
            }

            // Add default value as a separate paragraph
            if let Some(dv) = default_value_for_desc {
                // Add blank line separator when there's preceding description text
                if !first_text.is_empty() {
                    self.content_lines.push_empty();
                }
                let s = self.content_lines.begin_line();
                s.push_str(indent);
                s.push_str("Default is `");
                s.push_str(dv);
                s.push('`');
            }
        }
    }

    pub(super) fn format_type_comment_tag(
        &mut self,
        normalized_kind: &str,
        tag: &oxc_jsdoc::parser::JSDocTag<'_>,
        should_capitalize: bool,
        has_no_space_before_type: bool,
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

        let bracket_spacing = self.bracket_spacing();
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
                    && let Some(formatted) = format_type_via_formatter(
                        &normalized_type_str,
                        &self.type_format_options,
                        self.allocator,
                    )
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
            if str_width(&tag_line) > self.wrap_width
                && !normalized_type_str.is_empty()
                && self.wrap_type_expression(&tag_line[..tag_prefix_len], &normalized_type_str, "")
            {
                return;
            }
            self.content_lines.push(tag_line);
            return;
        }

        let desc_text: Cow<'_, str> =
            if should_capitalize { capitalize_first(desc_text) } else { Cow::Borrowed(desc_text) };

        let prefix_len = str_width(&tag_line) + 1; // tag_line + " "
        let one_liner_len = prefix_len + str_width(&desc_text);
        if one_liner_len <= self.wrap_width {
            let s = self.content_lines.begin_line();
            s.push_str(&tag_line);
            s.push(' ');
            s.push_str(&desc_text);
        } else if !normalized_type_str.is_empty()
            && str_width(&tag_line) > self.wrap_width
            && self.wrap_type_expression(&tag_line[..tag_prefix_len], &normalized_type_str, "")
        {
            // Type was wrapped. Add description as continuation.
            let indent = "  ";
            let indent_width = self.wrap_width.saturating_sub(str_width(indent));
            let desc = wrap_text(
                &desc_text,
                indent_width,
                0,
                false,
                Some(self.format_options),
                Some(self.external_callbacks),
                Some(self.allocator),
            );
            self.push_indented_desc(indent, desc);
        } else {
            // Pass description through wrap_text with tag_string_length offset
            let indent = "  ";
            let indent_width = self.wrap_width.saturating_sub(str_width(indent));
            let tag_str_len = prefix_len.saturating_sub(str_width(indent));

            let first_word_w = desc_text.split_whitespace().next().map_or(0, str_width);
            if prefix_len + first_word_w >= self.wrap_width {
                self.content_lines.push(tag_line);
                let desc = wrap_text(
                    &desc_text,
                    indent_width,
                    0,
                    false,
                    Some(self.format_options),
                    Some(self.external_callbacks),
                    Some(self.allocator),
                );
                self.push_indented_desc(indent, desc);
            } else {
                let desc = wrap_text(
                    &desc_text,
                    indent_width,
                    tag_str_len,
                    false,
                    Some(self.format_options),
                    Some(self.external_callbacks),
                    Some(self.allocator),
                );
                let mut iter = desc.split('\n');
                if let Some(first) = iter.next() {
                    let s = self.content_lines.begin_line();
                    s.push_str(&tag_line);
                    s.push(' ');
                    s.push_str(first);
                }
                for line in iter {
                    if line.is_empty() {
                        self.content_lines.push_empty();
                    } else {
                        let s = self.content_lines.begin_line();
                        s.push_str(indent);
                        s.push_str(line);
                    }
                }
            }
        }
    }

    pub(super) fn format_generic_tag(
        &mut self,
        normalized_kind: &str,
        tag: &oxc_jsdoc::parser::JSDocTag<'_>,
        should_capitalize: bool,
    ) {
        let mut tag_line = String::with_capacity(normalized_kind.len() + 1);
        tag_line.push('@');
        tag_line.push_str(normalized_kind);

        // Check if there's a blank line between the tag and description using
        // whitespace-preserving parse. This detects patterns like:
        //   @internal
        //
        //   Some description
        let raw_ws = tag.comment().parsed_preserving_whitespace();
        let has_leading_blank_line = {
            let trimmed_start = raw_ws.trim_start_matches(' ');
            trimmed_start.starts_with("\n\n") || trimmed_start.starts_with("\n \n")
        };

        let desc_text = tag.comment().parsed();
        let desc_text = normalize_markdown_emphasis(desc_text.trim());
        let desc_text = desc_text.trim();

        if desc_text.is_empty() {
            self.content_lines.push(tag_line);
            return;
        }

        let quote_style = self.quote_style();

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

        // If there was a blank line between the tag and the description,
        // preserve the separation: output the tag alone, a blank line, then
        // the description as a separate paragraph.
        if has_leading_blank_line {
            self.content_lines.push(tag_line);
            self.content_lines.push_empty();
            let indent = "  ";
            let indent_width = self.wrap_width.saturating_sub(str_width(indent));
            let mut desc = wrap_text(
                &desc_text,
                indent_width,
                0,
                false,
                Some(self.format_options),
                Some(self.external_callbacks),
                Some(self.allocator),
            );
            // Skip leading blank line from wrap_text since we already added one
            if desc.starts_with('\n') {
                desc.remove(0);
            }
            self.push_indented_desc(indent, desc);
            return;
        }

        let prefix_len = str_width(&tag_line) + 1; // tag_line + " "
        let skip_wrapping = should_skip_description_formatting(normalized_kind);
        if prefix_len + str_width(&desc_text) <= self.wrap_width || skip_wrapping {
            // Fits on one line, or tag skips description formatting (no wrapping).
            // Tags in TAGS_PEV_FORMATE_DESCRIPTION (e.g. @see) keep their description
            // on one line regardless of length.
            let s = self.content_lines.begin_line();
            s.push_str(&tag_line);
            s.push(' ');
            s.push_str(&desc_text);
        } else {
            // Pass description through wrap_text with tag_string_length offset
            let indent = "  ";
            let indent_width = self.wrap_width.saturating_sub(str_width(indent));
            let tag_str_len = prefix_len.saturating_sub(str_width(indent));

            let first_word_w = desc_text.split_whitespace().next().map_or(0, str_width);
            if prefix_len + first_word_w >= self.wrap_width {
                self.content_lines.push(tag_line);
                let desc = wrap_text(
                    &desc_text,
                    indent_width,
                    0,
                    false,
                    Some(self.format_options),
                    Some(self.external_callbacks),
                    Some(self.allocator),
                );
                self.push_indented_desc(indent, desc);
            } else {
                let desc = wrap_text(
                    &desc_text,
                    indent_width,
                    tag_str_len,
                    false,
                    Some(self.format_options),
                    Some(self.external_callbacks),
                    Some(self.allocator),
                );
                let mut iter = desc.split('\n');
                if let Some(first) = iter.next() {
                    let s = self.content_lines.begin_line();
                    s.push_str(&tag_line);
                    s.push(' ');
                    s.push_str(first);
                }
                for line in iter {
                    if line.is_empty() {
                        self.content_lines.push_empty();
                    } else {
                        let s = self.content_lines.begin_line();
                        s.push_str(indent);
                        s.push_str(line);
                    }
                }
            }
        }
    }
}
