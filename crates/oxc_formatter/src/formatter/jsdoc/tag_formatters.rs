use std::borrow::Cow;

use cow_utils::CowUtils;

use super::{
    embedded::{
        format_embedded_js, format_type_via_formatter, is_js_ts_lang, update_template_depth,
    },
    normalize::{
        capitalize_first, normalize_markdown_emphasis, normalize_type,
        normalize_type_preserve_quotes, normalize_type_return, strip_jsdoc_stars_preserve_newlines,
        strip_optional_type_suffix,
    },
    serialize::{
        JsdocFormatter, format_default_value, is_known_tag, is_named_generic_tag,
        should_preserve_description_verbatim, should_skip_description_formatting,
        strip_default_is_suffix,
    },
    wrap::{str_width, wrap_text},
};

/// Replace standalone JSDoc `*` type syntax with `any` in a multiline type string.
///
/// After JSDoc `*` line prefixes have been stripped (via `strip_jsdoc_stars_preserve_newlines`),
/// remaining `*` characters represent the JSDoc "any" type (e.g., `"profileImageLink": *,`).
/// This replaces them so the TS parser can handle the type expression.
/// Matches the upstream behavior of `type.replace(/\*/g, " any ")` in `convertToModernType()`.
fn replace_jsdoc_star_type(s: &str) -> Cow<'_, str> {
    s.cow_replace('*', " any ")
}

/// Strip the minimum common leading whitespace from all non-empty lines,
/// preserving relative indentation. `base_indent` is the indent of context
/// not included in `text` (e.g. 0 when there's inline text on the tag line).
/// The minimum indent is clamped to at most `base_indent`, so that relative
/// indentation (e.g. 4-space code blocks) is preserved.
fn dedent_lines(text: &str, base_indent: usize) -> String {
    let min_indent = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .chain(std::iter::once(base_indent))
        .min()
        .unwrap_or(0);

    let mut result = String::with_capacity(text.len());
    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        if line.trim().is_empty() {
            // Keep empty lines empty
        } else if min_indent > 0 && line.len() >= min_indent {
            result.push_str(&line[min_indent..]);
        } else if min_indent == 0 {
            // No common indent to strip — preserve original whitespace
            result.push_str(line);
        } else {
            result.push_str(line.trim_start());
        }
    }
    result
}

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

    /// Format example code content with continuation indent.
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

        let indent = self.code_indent();

        // Try formatting the code. The effective print width for @example code is
        // wrap_width minus the code indent width.
        let effective_width = self.wrap_width.saturating_sub(self.code_indent_width());
        if let Some(formatted) =
            format_embedded_js(code, effective_width, self.format_options, self.allocator).filter(
                |f| {
                    // Reject pseudo-code that parses as valid JS but produces
                    // structurally different output (e.g., `{undefined}('popup', 'options')`
                    // parsed as block statement + expression). If the line count
                    // more than doubles, the code was likely misinterpreted.
                    let input_lines = code.lines().count();
                    let output_lines = f.lines().count();
                    output_lines <= input_lines * 2 + 1
                },
            )
        {
            // Add continuation indent to code structure lines, but NOT to template literal
            // content. The formatter preserves template literal content verbatim, so
            // adding indent to those lines would shift them incorrectly.
            let mut template_depth: u32 = 0;
            for line in formatted.lines() {
                if line.is_empty() {
                    self.content_lines.push_empty();
                } else if template_depth == 0 {
                    {
                        let s = self.content_lines.begin_line();
                        s.push_str(indent);
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

        // Fallback: pass through with continuation indent
        for line in code.lines() {
            let line_content =
                if self.options.keep_unparsable_example_indent { line } else { line.trim() };
            if line_content.is_empty() {
                self.content_lines.push_empty();
            } else {
                {
                    let s = self.content_lines.begin_line();
                    s.push_str(indent);
                    s.push_str(line_content);
                }
            }
        }
    }

    /// Handle fenced code blocks inside @example tags.
    /// Strips the ``` markers, formats the inner code, and re-adds fences
    /// with proper continuation indentation.
    fn format_example_fenced_block(
        &mut self,
        lang_line: &str,
        inner_code: &str,
        closing_fence: &str,
    ) {
        let indent = self.code_indent();
        let effective_width = self.wrap_width.saturating_sub(self.code_indent_width());

        // Add opening fence with indent
        {
            let s = self.content_lines.begin_line();
            s.push_str(indent);
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
                                s.push_str(indent);
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
                                s.push_str(indent);
                                s.push_str(content);
                            }
                        }
                    }
                }
            } else {
                // Non-JS/TS fenced code: preserve with continuation indent
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
                            s.push_str(indent);
                            s.push_str(content);
                        }
                    }
                }
            }
        }

        // Add closing fence with indent
        {
            let s = self.content_lines.begin_line();
            s.push_str(indent);
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
            // Check multiline on raw type BEFORE trim() strips leading newlines.
            // `parsed()` calls `.trim()` which collapses `{\n\tfoo\n}` to `foo`.
            // Using raw() preserves the original structure for this check.
            let raw_inner = &tp.raw()[1..tp.raw().len() - 1];
            let raw_was_multiline = raw_inner.contains('\n');
            if !raw_type.is_empty() {
                let (type_to_normalize, type_optional) = strip_optional_type_suffix(raw_type);
                is_type_optional = type_optional;
                normalized_type_str = if preserve_quotes {
                    normalize_type_preserve_quotes(type_to_normalize)
                } else {
                    normalize_type(type_to_normalize)
                };
                // Try formatting via the formatter (simulates upstream's formatType()).
                // For multi-line types, pass a version with newlines preserved so the
                // TS formatter receives multi-line input and maintains line structure.
                // This matches upstream where comment-parser strips `*` prefixes but
                // preserves newlines before passing to formatType().
                if !preserve_quotes {
                    let was_multiline = raw_was_multiline || type_to_normalize.contains('\n');
                    // Use the untrimmed raw_inner for star-stripping when the type was
                    // multiline — parsed()/trim() may have collapsed the newlines.
                    let multiline_source = if raw_was_multiline && !type_to_normalize.contains('\n')
                    {
                        raw_inner
                    } else {
                        type_to_normalize
                    };
                    // For multiline types, build a formatter input that preserves
                    // newlines but converts JSDoc-specific syntax (`*` → `any`) so
                    // the TS parser can handle it. For single-line types, use the
                    // normalized string directly.
                    let formatter_input: Cow<'_, str> = if was_multiline {
                        // Strip JSDoc `*` line prefixes while preserving newlines,
                        // then replace standalone `*` type syntax with `any`.
                        let star_stripped = strip_jsdoc_stars_preserve_newlines(multiline_source);
                        Cow::Owned(replace_jsdoc_star_type(&star_stripped).into_owned())
                    } else {
                        Cow::Borrowed(normalized_type_str.as_ref())
                    };
                    if let Some(formatted) = format_type_via_formatter(
                        &formatter_input,
                        &self.type_format_options,
                        self.allocator,
                    ) {
                        // If the formatter collapsed a multi-line type to single
                        // line, verify it fits within the available width. The tag
                        // line will be `@kind {type} name`, so check whether the
                        // collapsed type + tag prefix exceeds wrap_width. If so,
                        // keep the multi-line version to prevent overlong lines
                        // that merge with subsequent tags during wrapping.
                        if was_multiline
                            && !formatted.contains('\n')
                            && formatter_input.contains('\n')
                        {
                            // Estimate: @kind + space + { + type + } = tag_prefix_len + 3 + type.len()
                            let estimated_width = tag_prefix_len + 3 + formatted.len();
                            if estimated_width > self.wrap_width {
                                // Use the star-stripped multiline version
                                normalized_type_str = Cow::Owned(formatter_input.into_owned());
                            } else {
                                normalized_type_str = Cow::Owned(formatted);
                            }
                        } else {
                            normalized_type_str = Cow::Owned(formatted);
                        }
                    } else if was_multiline {
                        // Formatter failed (parse error) but type was multi-line;
                        // use the star-stripped version with newlines preserved.
                        // For types that are only "multiline" due to JSDoc formatting
                        // (e.g., `() => a.b` on its own line), the normalized
                        // single-line version is already correct.
                        if formatter_input.contains('\n') {
                            normalized_type_str = Cow::Owned(formatter_input.into_owned());
                        }
                        // Otherwise keep normalized_type_str (single-line, already correct)
                    }
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

        // Multi-line type: push each line of the tag separately, then handle
        // name + description on subsequent lines.
        if tag_line.contains('\n') {
            let desc_raw = comment_part.parsed_preserving_whitespace();
            let desc_raw = desc_raw.trim();
            let desc_normalized = normalize_markdown_emphasis(desc_raw);
            let desc_raw = desc_normalized.trim();

            let mut lines_iter = tag_line.split('\n');
            if let Some(first) = lines_iter.next() {
                self.content_lines.push(first);
            }
            for line in lines_iter {
                self.content_lines.push(line);
            }
            if !desc_raw.is_empty() {
                let indent = self.continuation_indent();
                let indent_width = self.wrap_width.saturating_sub(self.continuation_indent_width());
                let desc = wrap_text(
                    desc_raw,
                    indent_width,
                    0,
                    false,
                    Some(self.format_options),
                    Some(self.external_callbacks),
                    Some(self.allocator),
                );
                self.push_indented_desc(indent, desc);
            }
            return;
        }

        let desc_ws_raw = comment_part.parsed_preserving_whitespace();
        // Detect blank line between tag+type+name and description
        let has_desc_blank_line = {
            let t = desc_ws_raw.trim_start_matches(' ');
            t.starts_with("\n\n") || t.starts_with("\n \n")
        };
        let desc_raw = desc_ws_raw.trim();
        let desc_normalized = normalize_markdown_emphasis(desc_raw);
        let desc_raw = desc_normalized.trim();

        // Strip existing "Default is ..." from description when we have an actual default value
        // and `add_default_to_description` is enabled (we'll re-append a normalized version).
        // Skip for @template — its bracket default `[T=Value]` is syntactic, not a description default.
        let desc_raw = if default_value.is_some()
            && self.options.add_default_to_description
            && normalized_kind != "template"
        {
            strip_default_is_suffix(desc_raw)
        } else {
            Cow::Borrowed(desc_raw)
        };
        let desc_raw = desc_raw.trim();

        if desc_raw.is_empty() && default_value.is_none() {
            self.content_lines.push(tag_line);
            return;
        }

        // If there's a blank line between tag+type+name and description,
        // output them separately with a blank line in between.
        if has_desc_blank_line && !desc_raw.is_empty() {
            self.content_lines.push(tag_line);
            self.content_lines.push_empty();
            let indent = self.continuation_indent();
            let indent_width = self.wrap_width.saturating_sub(self.continuation_indent_width());
            let desc = wrap_text(
                desc_raw,
                indent_width,
                0,
                false,
                Some(self.format_options),
                Some(self.external_callbacks),
                Some(self.allocator),
            );
            self.push_indented_desc(indent, desc);
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
            let indent = self.continuation_indent();
            let indent_width = self.wrap_width.saturating_sub(self.continuation_indent_width());
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

        // When add_default_to_description is false, don't append "Default is ..." to description.
        // Also skip for @template (bracket defaults are syntactic, not description-level).
        let default_value_for_desc =
            if self.options.add_default_to_description && normalized_kind != "template" {
                default_value
            } else {
                None
            };

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
        // Strip the common leading whitespace from continuation lines while
        // preserving relative indentation (e.g. 4-space indented code blocks).
        // When first_text is non-empty, the base indent is 0 (inline with tag),
        // so continuation line indentation is preserved relative to that.
        let remaining_desc = if let Some(rest) = rest_of_desc {
            let base = if first_text_line.is_empty() { usize::MAX } else { 0 };
            dedent_lines(rest, base)
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
            let indent = self.continuation_indent();
            let indent_width = self.wrap_width.saturating_sub(self.continuation_indent_width());

            // When description is just a dash with text on the next line
            // (e.g., @param {Type} name -\n  description), output the tag + dash
            // on the first line and description on the next line with continuation indent.
            // Without this, the leading \n in full_desc causes a spurious blank line.
            if has_dash && first_text.is_empty() && has_remaining {
                let s = self.content_lines.begin_line();
                s.push_str(&tag_line);
                s.push_str(" -");
                let desc = wrap_text(
                    &remaining_desc,
                    indent_width,
                    0,
                    false,
                    Some(self.format_options),
                    Some(self.external_callbacks),
                    Some(self.allocator),
                );
                self.push_indented_desc(indent, desc);
                return;
            }

            // Build full description text (first line + remaining), appending default suffix
            // inline so it wraps naturally with the description text.
            let full_desc = {
                let mut s = if has_remaining {
                    let mut s = String::with_capacity(first_text.len() + 1 + remaining_desc.len());
                    s.push_str(&first_text);
                    s.push('\n');
                    s.push_str(&remaining_desc);
                    s
                } else {
                    String::from(first_text.as_ref())
                };
                if let Some(dv) = default_value_for_desc {
                    let last = s.as_bytes().last().copied().unwrap_or(b' ');
                    if matches!(last, b'.' | b'!' | b'?') {
                        s.push(' ');
                    } else {
                        s.push_str(". ");
                    }
                    s.push_str("Default is `");
                    s.push_str(dv);
                    s.push('`');
                }
                s
            };

            let tag_str_len = prefix_len.saturating_sub(if indent.is_empty() {
                0
            } else {
                self.continuation_indent_width()
            });

            // Upstream: tagString.length + firstWord.length > printWidth → new line
            let first_word_w = full_desc.split_whitespace().next().map_or(0, str_width);
            if prefix_len + first_word_w > self.wrap_width {
                // Tag prefix + first word don't fit → description starts on new line
                let mut line = tag_line;
                if has_dash {
                    line.push_str(" -");
                }
                self.content_lines.push(line);
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

            // Default value is now appended inline in full_desc above
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
        #[expect(unused_assignments)]
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
                if !skip_formatter {
                    let was_multiline = raw_type.contains('\n');
                    let formatter_input = if was_multiline {
                        Cow::Owned(strip_jsdoc_stars_preserve_newlines(raw_type))
                    } else {
                        Cow::Borrowed(normalized_type_str.as_ref())
                    };
                    if let Some(formatted) = format_type_via_formatter(
                        &formatter_input,
                        &self.type_format_options,
                        self.allocator,
                    ) {
                        if was_multiline && !formatted.contains('\n') {
                            normalized_type_str =
                                Cow::Owned(strip_jsdoc_stars_preserve_newlines(raw_type));
                        } else {
                            normalized_type_str = Cow::Owned(formatted);
                        }
                    } else if was_multiline {
                        normalized_type_str =
                            Cow::Owned(strip_jsdoc_stars_preserve_newlines(raw_type));
                    }
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
            self.content_lines.push(tag_line);
            return;
        }

        let desc_text: Cow<'_, str> =
            if should_capitalize { capitalize_first(desc_text) } else { Cow::Borrowed(desc_text) };

        // When the type is multi-line, push each type line, then handle the
        // description. Single-token descriptions (no spaces) are kept inline
        // on the last type line — matching upstream where comment-parser treats
        // short trailing text as a "name" field that stays on the tag line.
        // Multi-word descriptions go to a new indented line.
        if tag_line.contains('\n') {
            let last_line_width = tag_line.rsplit('\n').next().map_or(0, str_width);
            let desc_is_single_token = !desc_text.contains(' ');
            let desc_fits_on_last_line = desc_is_single_token
                && last_line_width + 1 + str_width(&desc_text) <= self.wrap_width;

            if desc_fits_on_last_line {
                // Append single-token description to the last line of the tag
                let mut lines: Vec<&str> = tag_line.split('\n').collect();
                if let Some(last) = lines.last_mut() {
                    let combined = self.allocator.alloc_concat_strs_array([last, " ", &desc_text]);
                    for line in &lines[..lines.len() - 1] {
                        self.content_lines.push(*line);
                    }
                    self.content_lines.push(combined);
                }
            } else {
                let indent = self.continuation_indent();
                let indent_width = self.wrap_width.saturating_sub(self.continuation_indent_width());
                let mut lines_iter = tag_line.split('\n');
                if let Some(first) = lines_iter.next() {
                    self.content_lines.push(first);
                }
                for line in lines_iter {
                    self.content_lines.push(line);
                }
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
            }
            return;
        }

        // Strip dash separator (e.g., `- description`) to prevent the mdast
        // parser from treating it as a markdown list marker. Re-add as part of
        // the separator, matching format_type_name_comment_tag's approach.
        let (has_dash, desc_text_no_dash): (bool, Cow<'_, str>) =
            if let Some(rest) = desc_text.strip_prefix("- ") {
                (true, Cow::Borrowed(rest))
            } else if *desc_text == *"-" {
                (true, Cow::Borrowed(""))
            } else {
                (false, Cow::Borrowed(&*desc_text))
            };
        let separator = if has_dash { " - " } else { " " };
        let sep_len = separator.len();

        let prefix_len = str_width(&tag_line) + sep_len;
        let one_liner_len = prefix_len + str_width(&desc_text_no_dash);
        if one_liner_len <= self.wrap_width {
            let s = self.content_lines.begin_line();
            s.push_str(&tag_line);
            s.push_str(separator);
            s.push_str(&desc_text_no_dash);
        } else if desc_text_no_dash.is_empty() {
            // Only a dash, no description text
            let s = self.content_lines.begin_line();
            s.push_str(&tag_line);
            s.push_str(separator);
        } else {
            // Pass description through wrap_text with tag_string_length offset
            let indent = self.continuation_indent();
            let indent_width = self.wrap_width.saturating_sub(if indent.is_empty() {
                0
            } else {
                self.continuation_indent_width()
            });
            let tag_str_len = prefix_len.saturating_sub(if indent.is_empty() {
                0
            } else {
                self.continuation_indent_width()
            });

            let first_word_w = desc_text_no_dash.split_whitespace().next().map_or(0, str_width);
            if prefix_len + first_word_w > self.wrap_width {
                self.content_lines.push(tag_line);
                let desc = wrap_text(
                    &desc_text_no_dash,
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
                    &desc_text_no_dash,
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
        // For unknown tags, check if description starts on a new line
        // (upstream preserves original line structure for unknown tags)
        let desc_starts_on_new_line = {
            let trimmed_start = raw_ws.trim_start_matches(' ');
            trimmed_start.starts_with('\n')
        };

        let desc_text = tag.comment().parsed();
        let desc_text = normalize_markdown_emphasis(desc_text.trim());
        let desc_text = desc_text.trim();
        // Whitespace-preserving version for skip-formatting tags: preserves blank
        // lines within the description that parsed() strips. Used when
        // skip_wrapping is true and the content has embedded newlines.
        let raw_ws_trimmed = raw_ws.trim();
        let raw_ws_normalized = normalize_markdown_emphasis(raw_ws_trimmed);
        let raw_ws_desc = raw_ws_normalized.trim();

        if desc_text.is_empty() {
            self.content_lines.push(tag_line);
            return;
        }

        let quote_style = self.quote_style();

        // For @default/@defaultValue, format JSON-like values (single-line only;
        // multi-line values preserve internal indentation as-is)
        let desc_text: Cow<'_, str> = if matches!(normalized_kind, "default" | "defaultValue") {
            if raw_ws_desc.contains('\n') {
                Cow::Borrowed(desc_text)
            } else {
                format_default_value(desc_text, quote_style)
            }
        } else if should_capitalize
            && is_named_generic_tag(normalized_kind)
            && !has_leading_blank_line
        {
            // Named tags: first word is the "name" (don't capitalize), rest is description.
            // Upstream comment-parser separates name/description; we do it inline.
            // When the name starts with `{`, skip past the balanced closing `}` to find
            // the real split point — the type expression may contain whitespace
            // (e.g., `{CustomEvent<{ id: string }>}`).
            let name_end = if desc_text.starts_with('{') {
                find_balanced_brace_end(desc_text)
            } else {
                desc_text.find(|c: char| c.is_ascii_whitespace())
            };
            if let Some(name_end_idx) = name_end {
                let name_part = &desc_text[..name_end_idx];
                let desc_part = desc_text[name_end_idx..].trim_start();
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
            capitalize_first_skip_type(desc_text)
        } else {
            Cow::Borrowed(desc_text)
        };

        // If there was a blank line between the tag and the description,
        // preserve the separation: output the tag alone, a blank line, then
        // the description as a continuation-indented paragraph.
        if has_leading_blank_line {
            self.content_lines.push(tag_line);
            self.content_lines.push_empty();
            let skip_fmt = should_skip_description_formatting(normalized_kind);
            if skip_fmt && raw_ws_desc.contains('\n') {
                // Multi-line skip-formatting: preserve raw line structure exactly.
                // wrap_text would collapse lines via markdown parsing, so output directly.
                for line in raw_ws_desc.split('\n') {
                    if line.trim().is_empty() {
                        self.content_lines.push_empty();
                    } else {
                        self.content_lines.push(line);
                    }
                }
            } else if skip_fmt {
                // Single-line skip-formatting: wrap at full width, no continuation indent.
                let mut desc = wrap_text(
                    raw_ws_desc,
                    self.wrap_width,
                    0,
                    false,
                    Some(self.format_options),
                    Some(self.external_callbacks),
                    Some(self.allocator),
                );
                if desc.starts_with('\n') {
                    desc.remove(0);
                }
                self.push_indented_desc("", desc);
            } else {
                let indent = self.continuation_indent();
                let indent_width = self.wrap_width.saturating_sub(self.continuation_indent_width());
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
            }
            return;
        }

        // @remarks and @privateRemarks always force description to next line
        // (matching upstream stringify.ts:151)
        if matches!(normalized_kind, "remarks" | "privateRemarks") {
            self.content_lines.push(tag_line);
            let indent = self.continuation_indent();
            let indent_width = self.wrap_width.saturating_sub(self.continuation_indent_width());
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
            return;
        }

        // Named generic tags with description on new line: preserve the line break.
        // e.g., @categoryDescription Component\n  Description text...
        // Upstream stringify.ts:173 preserves `descriptionString.startsWith("\n")`.
        if is_named_generic_tag(normalized_kind)
            && desc_starts_on_new_line
            && !should_skip_description_formatting(normalized_kind)
        {
            if let Some(space_idx) = desc_text.find(|c: char| c.is_ascii_whitespace()) {
                let name_part = &desc_text[..space_idx];
                let desc_part = desc_text[space_idx..].trim_start();
                if !desc_part.is_empty() {
                    let s = self.content_lines.begin_line();
                    s.push_str(&tag_line);
                    s.push(' ');
                    s.push_str(name_part);
                    let indent = self.continuation_indent();
                    let indent_width =
                        self.wrap_width.saturating_sub(self.continuation_indent_width());
                    let desc = wrap_text(
                        desc_part,
                        indent_width,
                        0,
                        false,
                        Some(self.format_options),
                        Some(self.external_callbacks),
                        Some(self.allocator),
                    );
                    self.push_indented_desc(indent, desc);
                    return;
                }
            }
            // No description part — just tag + name on one line
            let s = self.content_lines.begin_line();
            s.push_str(&tag_line);
            s.push(' ');
            s.push_str(&desc_text);
            return;
        }

        let prefix_len = str_width(&tag_line) + 1; // tag_line + " "
        let is_unknown = !is_known_tag(normalized_kind);
        // Skip wrapping for TAGS_PEV_FORMATE_DESCRIPTION (@see, @borrows, etc.)
        // AND for unknown/custom tags (not in TAGS_ORDER) — upstream preserves
        // their description as-is at stringify.ts:132-136.
        let skip_wrapping = should_skip_description_formatting(normalized_kind) || is_unknown;

        // Unknown tags and skip-wrapping tags (TAGS_PEV_FORMATE_DESCRIPTION):
        // if description was originally on a new line, keep it there.
        // Upstream preserves original line structure at stringify.ts:132-136,173.
        let skip_formatting = should_skip_description_formatting(normalized_kind);
        if (is_unknown || skip_formatting) && desc_starts_on_new_line {
            self.content_lines.push(tag_line);
            // Use raw_ws_desc which preserves internal indentation (e.g., in @default
            // code blocks). parsed() strips leading whitespace from each line.
            for line in raw_ws_desc.split('\n') {
                if line.trim().is_empty() {
                    self.content_lines.push_empty();
                } else {
                    self.content_lines.push(line);
                }
            }
            return;
        }

        let fits_on_one_line = prefix_len + str_width(&desc_text) <= self.wrap_width;
        if fits_on_one_line || skip_wrapping {
            // Fits on one line, or tag skips description formatting (no wrapping).
            // Tags in TAGS_PEV_FORMATE_DESCRIPTION (e.g. @see) and unknown tags
            // keep their description on one line regardless of length.
            // When desc has embedded newlines (e.g. @module Name\n\nMore text),
            // split by lines to preserve blank lines in the output.
            // Use raw_ws_desc which preserves blank lines that parsed() strips.
            if skip_wrapping && raw_ws_desc.contains('\n') {
                let mut lines = raw_ws_desc.split('\n');
                if let Some(first) = lines.next() {
                    let s = self.content_lines.begin_line();
                    s.push_str(&tag_line);
                    s.push(' ');
                    s.push_str(first);
                }
                for line in lines {
                    if line.is_empty() {
                        self.content_lines.push_empty();
                    } else {
                        self.content_lines.push(line);
                    }
                }
            } else if skip_wrapping && !fits_on_one_line {
                // Upstream (stringify.ts:131-136) uses the description as-is
                // for TAGS_PEV_FORMATE_DESCRIPTION (no wrapping). But our
                // `should_skip_description_formatting()` includes extra tags
                // like @deprecated that upstream DOES wrap. Only truly skip
                // wrapping for tags in upstream's list: @default, @defaultValue,
                // @borrows, @import, @memberof, @module, @see.
                if should_preserve_description_verbatim(normalized_kind) || is_unknown {
                    let s = self.content_lines.begin_line();
                    s.push_str(&tag_line);
                    s.push(' ');
                    s.push_str(raw_ws_desc);
                } else {
                    // Tags like @deprecated that skip capitalization but still
                    // wrap at printWidth.
                    let indent = self.continuation_indent();
                    let indent_width = self.wrap_width.saturating_sub(if indent.is_empty() {
                        0
                    } else {
                        self.continuation_indent_width()
                    });
                    let tag_str_len = prefix_len.saturating_sub(if indent.is_empty() {
                        0
                    } else {
                        self.continuation_indent_width()
                    });
                    let desc = wrap_text(
                        raw_ws_desc,
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
            } else {
                let s = self.content_lines.begin_line();
                s.push_str(&tag_line);
                s.push(' ');
                s.push_str(&desc_text);
            }
        } else {
            // Pass description through wrap_text with tag_string_length offset
            let indent = self.continuation_indent();
            let indent_width = self.wrap_width.saturating_sub(if indent.is_empty() {
                0
            } else {
                self.continuation_indent_width()
            });
            let tag_str_len = prefix_len.saturating_sub(if indent.is_empty() {
                0
            } else {
                self.continuation_indent_width()
            });

            let first_word_w = desc_text.split_whitespace().next().map_or(0, str_width);
            if prefix_len + first_word_w > self.wrap_width {
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

/// Find the index just past the balanced closing `}` for a string that starts with `{`.
/// Returns the index after the matching `}`, accounting for nested braces.
/// Returns `None` if no balanced closing brace is found.
fn find_balanced_brace_end(s: &str) -> Option<usize> {
    debug_assert!(s.starts_with('{'));
    let mut depth: u32 = 0;
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1); // index past the closing `}`
                }
            }
            _ => {}
        }
    }
    None
}

/// Like `capitalize_first`, but skips over a leading type expression `{...}` if present.
/// The type expression is preserved as-is and only the description after it is capitalized.
fn capitalize_first_skip_type(s: &str) -> Cow<'_, str> {
    if !s.starts_with('{') {
        return capitalize_first(s);
    }
    if let Some(end) = find_balanced_brace_end(s) {
        let type_part = &s[..end];
        let rest = s[end..].trim_start();
        if rest.is_empty() {
            return Cow::Borrowed(s);
        }
        let capitalized = capitalize_first(rest);
        if matches!(capitalized, Cow::Borrowed(_)) {
            // Nothing changed — return original
            return Cow::Borrowed(s);
        }
        let mut result = String::with_capacity(type_part.len() + 1 + capitalized.len());
        result.push_str(type_part);
        result.push(' ');
        result.push_str(&capitalized);
        Cow::Owned(result)
    } else {
        // No balanced brace — fall back to normal capitalize
        capitalize_first(s)
    }
}
