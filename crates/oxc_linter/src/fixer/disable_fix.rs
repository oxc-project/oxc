use oxc_span::Span;

use crate::{Fix, FixKind, Message, oxc_code_short_canonical_name};
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisableDirective {
    NextLine,
    Line,
    Section,
}

impl Message {
    pub(crate) fn add_ignore_fix(&mut self, section_offset: u32, section_source_text: &str) {
        // If the error is exactly at the section offset and has 0 span length, it means that the file is the problem
        // and attaching a ignore comment would not ignore the error.
        // This is because the ignore comment would need to be placed before the error offset, which is not possible.
        if self.span.start == 0 && self.span.end == 0 {
            return;
        }

        let start = self.span.start as usize;
        let end = self.span.end as usize;

        if start > end
            || end > section_source_text.len()
            || !section_source_text.is_char_boundary(start)
            || !section_source_text.is_char_boundary(end)
        {
            return;
        }

        let Some(rule_name) = oxc_code_short_canonical_name(&self.error.code) else { return };

        self.fixes.extend_fix(vec![
            disable_for_this_line(&rule_name, self.span.start, section_offset, section_source_text),
            disable_for_this_section(&rule_name, section_offset, section_source_text),
        ]);
    }
}

fn disable_for_this_line(
    rule_name: &str,
    error_offset: u32,
    section_offset: u32,
    section_source_text: &str,
) -> Fix {
    let bytes = section_source_text.as_bytes();
    let message = format!("Disable {rule_name} for this line");

    // Reuse an inline disable-line comment on the same line when present.
    if let Some(existing_comment_end) = get_inline_disable_line_comment_end(error_offset, bytes) {
        return Fix {
            message: Some(Cow::Owned(message)),
            content: Cow::Owned(format!(" {rule_name}")),
            span: Span::empty(existing_comment_end),
            kind: FixKind::IgnoreFix,
        };
    }

    // Find the line break before the error
    let mut line_break_offset = error_offset;
    for byte in bytes[..error_offset as usize].iter().rev() {
        if *byte == b'\n' || *byte == b'\r' {
            break;
        }
        line_break_offset -= 1;
    }

    let (content_prefix, insert_offset) =
        get_section_insert_position(section_offset, line_break_offset, bytes);

    // Reuse an existing disable-next-line comment when present by appending the rule.
    if let Some(existing_comment_end) =
        get_existing_disable_comment_end(insert_offset, DisableDirective::NextLine, bytes)
    {
        return Fix {
            message: Some(Cow::Owned(message)),
            content: Cow::Owned(format!(" {rule_name}")),
            span: Span::empty(existing_comment_end),
            kind: FixKind::IgnoreFix,
        };
    }

    // Preserve leading indentation from the target line for newly inserted comments.
    let whitespace_range = {
        let start = insert_offset as usize;
        let end = error_offset as usize;

        // make sure that end is at least start to avoid panic
        let end = end.max(start);
        let slice = &bytes[start..end];
        let whitespace_len = slice.iter().take_while(|c| matches!(c, b' ' | b'\t')).count();
        &slice[..whitespace_len]
    };
    let whitespace_string = String::from_utf8_lossy(whitespace_range);

    Fix {
        message: Some(Cow::Owned(message)),
        content: Cow::Owned(format!(
            "{content_prefix}{whitespace_string}// oxlint-disable-next-line {rule_name}\n"
        )),
        span: Span::empty(insert_offset),
        kind: FixKind::IgnoreFix,
    }
}

fn disable_for_this_section(
    rule_name: &str,
    section_offset: u32,
    section_source_text: &str,
) -> Fix {
    let bytes = section_source_text.as_bytes();
    let message = format!("Disable {rule_name} for this whole file");

    let (content_prefix, insert_offset) = get_section_insert_position(section_offset, 0, bytes);

    // Reuse an existing section disable comment when present by appending the rule.
    if let Some(existing_comment_end) =
        get_existing_disable_comment_end(insert_offset, DisableDirective::Section, bytes)
    {
        return Fix {
            message: Some(Cow::Owned(message)),
            content: Cow::Owned(format!(" {rule_name}")),
            span: Span::empty(existing_comment_end),
            kind: FixKind::IgnoreFix,
        };
    }

    let content = format!("{content_prefix}// oxlint-disable {rule_name}\n");

    Fix {
        message: Some(Cow::Owned(message)),
        content: Cow::Owned(content),
        span: Span::empty(insert_offset),
        kind: FixKind::IgnoreFix,
    }
}

/// Get the insert position and content prefix for section-based insertions.
///
/// The section source is already sliced at section offset, so offset 0 is the section start.
/// This handles section-start line break detection and shebang lines at the section start.
///
/// Returns (content_prefix, insert_offset) where:
/// - content_prefix: "\n" if we need to add a line break, "" otherwise
/// - insert_offset: the byte offset where the content should be inserted
#[expect(clippy::cast_possible_truncation)]
fn get_section_insert_position(
    section_offset: u32,
    target_offset: u32,
    bytes: &[u8],
) -> (&'static str, u32) {
    if target_offset == 0 {
        if bytes.starts_with(b"#!") {
            // Shebang present, insert after the first line.
            let mut shebang_end = 0;
            for (i, &byte) in bytes.iter().enumerate() {
                if byte == b'\n' {
                    shebang_end = i + 1;
                    break;
                }
            }
            return ("", shebang_end as u32);
        }

        if section_offset == 0 {
            // Full file starts at offset 0, insert before first byte with no extra newline.
            return ("", 0);
        }

        // Section starts at a line break, insert after it.
        if bytes.first() == Some(&b'\n') {
            return ("", 1);
        }
        if bytes.first() == Some(&b'\r') && bytes.get(1) == Some(&b'\n') {
            return ("", 2);
        }

        // Section starts in the middle of a line, prepend a newline.
        ("\n", 0)
    } else {
        // Insertion point was derived from a line start in the section slice.
        ("", target_offset)
    }
}

#[expect(clippy::cast_possible_truncation)]
fn get_inline_disable_line_comment_end(error_offset: u32, bytes: &[u8]) -> Option<u32> {
    let error_offset = error_offset as usize;
    if error_offset > bytes.len() {
        return None;
    }

    let mut line_end = error_offset;
    while line_end < bytes.len() && !matches!(bytes[line_end], b'\n' | b'\r') {
        line_end += 1;
    }

    let comment_start = bytes[error_offset..line_end].windows(2).position(|w| w == b"//")?;
    let comment_offset = error_offset + comment_start;

    get_disable_comment_end_at_comment_start(comment_offset, DisableDirective::Line, bytes)
        .map(|offset| offset as u32)
}

fn get_disable_comment_end_at_line_start(
    line_start: usize,
    directive: DisableDirective,
    bytes: &[u8],
) -> Option<usize> {
    if line_start > bytes.len() {
        return None;
    }

    if line_start > 0 && !matches!(bytes[line_start - 1], b'\n' | b'\r') {
        return None;
    }

    get_disable_comment_end_at_comment_start(line_start, directive, bytes)
}

#[expect(clippy::cast_possible_truncation)]
fn get_existing_disable_comment_end(
    insert_offset: u32,
    directive: DisableDirective,
    bytes: &[u8],
) -> Option<u32> {
    let insert_offset = insert_offset as usize;

    if insert_offset > bytes.len() {
        return None;
    }

    // First check the insertion line itself (e.g. section offsets that already point at a comment).
    if let Some(line_end) = get_disable_comment_end_at_line_start(insert_offset, directive, bytes) {
        return Some(line_end as u32);
    }

    if insert_offset == 0 {
        return None;
    }

    // We only merge when insertion happens at the start of a line.
    if !matches!(bytes[insert_offset - 1], b'\n' | b'\r') {
        return None;
    }

    // Then check the line immediately above the insertion point.
    let mut line_end = insert_offset;
    while line_end > 0 && matches!(bytes[line_end - 1], b'\n' | b'\r') {
        line_end -= 1;
    }

    if line_end == 0 {
        return None;
    }

    let mut line_start = line_end;
    while line_start > 0 && !matches!(bytes[line_start - 1], b'\n' | b'\r') {
        line_start -= 1;
    }

    get_disable_comment_end_at_line_start(line_start, directive, bytes)
        .map(|line_end| line_end as u32)
}

fn get_disable_comment_end_at_comment_start(
    comment_start: usize,
    directive: DisableDirective,
    bytes: &[u8],
) -> Option<usize> {
    if comment_start > bytes.len() {
        return None;
    }

    let mut line_end = comment_start;
    while line_end < bytes.len() && !matches!(bytes[line_end], b'\n' | b'\r') {
        line_end += 1;
    }

    // Parse a single-line comment in place and ensure it starts with the expected directive.
    let line = &bytes[comment_start..line_end];
    let mut idx = 0;

    while idx < line.len() && matches!(line[idx], b' ' | b'\t') {
        idx += 1;
    }

    if !line[idx..].starts_with(b"//") {
        return None;
    }
    idx += 2;

    while idx < line.len() && matches!(line[idx], b' ' | b'\t') {
        idx += 1;
    }

    let matched_directive_len = match directive {
        DisableDirective::NextLine => {
            if line[idx..].starts_with(b"oxlint-disable-next-line") {
                Some(b"oxlint-disable-next-line".len())
            } else if line[idx..].starts_with(b"eslint-disable-next-line") {
                Some(b"eslint-disable-next-line".len())
            } else {
                None
            }
        }
        DisableDirective::Line => {
            if line[idx..].starts_with(b"oxlint-disable-line") {
                Some(b"oxlint-disable-line".len())
            } else if line[idx..].starts_with(b"eslint-disable-line") {
                Some(b"eslint-disable-line".len())
            } else {
                None
            }
        }
        DisableDirective::Section => {
            if line[idx..].starts_with(b"oxlint-disable") {
                Some(b"oxlint-disable".len())
            } else if line[idx..].starts_with(b"eslint-disable") {
                Some(b"eslint-disable".len())
            } else {
                None
            }
        }
    }?;
    idx += matched_directive_len;

    // Avoid matching prefixes like "oxlint-disable-next-line-foo".
    if idx < line.len() && !matches!(line[idx], b' ' | b'\t') {
        return None;
    }

    // Match the same description forms as `DisableDirectivesBuilder::get_rule_names`:
    // - `--` anywhere
    // - a single `-` surrounded by whitespace
    let merge_end = find_description_start_offset(&line[idx..])
        .map_or(line_end, |pos| comment_start + idx + pos);

    Some(merge_end)
}

fn find_description_start_offset(text: &[u8]) -> Option<usize> {
    let mut previous = None;

    for (index, &ch) in text.iter().enumerate() {
        if ch != b'-' {
            previous = Some(ch);
            continue;
        }

        let next = text.get(index + 1).copied();
        let is_description_start = next.is_some_and(|c| {
            c == b'-'
                || (previous.is_some_and(|p: u8| p.is_ascii_whitespace())
                    && c.is_ascii_whitespace())
        });

        if is_description_start {
            return Some(index);
        }

        previous = Some(ch);
    }

    None
}

#[cfg(test)]
#[expect(clippy::cast_possible_truncation)]
mod tests {
    use oxc_diagnostics::OxcDiagnostic;
    use oxc_span::Span;

    use crate::{Message, PossibleFixes};

    fn message_with_span(span: Span) -> Message {
        Message::new(
            OxcDiagnostic::error("test diagnostic")
                .with_label(span)
                .with_error_code("test-plugin", "test-rule"),
            PossibleFixes::None,
        )
    }

    #[test]
    fn ignore_fix_is_not_created_for_out_of_bounds_span() {
        let mut message = message_with_span(Span::new(1, 2));

        message.add_ignore_fix(0, "");

        assert!(message.fixes.is_empty());
    }

    #[test]
    fn ignore_fix_is_not_created_for_non_utf8_boundary_span() {
        let mut message = message_with_span(Span::new(1, 2));

        message.add_ignore_fix(0, "\u{e9}");

        assert!(message.fixes.is_empty());
    }

    #[test]
    fn disable_for_section_js_file() {
        let source = "console.log('hello');";
        let fix = super::disable_for_this_section("no-console", 0, source);

        assert_eq!(fix.content, "// oxlint-disable no-console\n");
        assert_eq!(fix.span, Span::empty(0));
    }

    #[test]
    fn disable_for_section_after_lf() {
        let source = "<script>\nconsole.log('hello');";
        let section_offset = 8;
        let section_source_text = &source[8..];
        let fix =
            super::disable_for_this_section("no-console", section_offset, section_source_text);

        assert_eq!(fix.content, "// oxlint-disable no-console\n");
        assert_eq!(fix.span, Span::empty(1));
    }

    #[test]
    fn disable_for_section_after_crlf() {
        let source = "<script>\r\nconsole.log('hello');";
        let section_offset = 8;
        let section_source_text = &source[8..];
        let fix =
            super::disable_for_this_section("no-console", section_offset, section_source_text);

        assert_eq!(fix.content, "// oxlint-disable no-console\n");
        assert_eq!(fix.span, Span::empty(2));
    }

    #[test]
    fn disable_for_section_with_shebang() {
        let source = "#!/usr/bin/env node\nconsole.log('hello');";
        let fix = super::disable_for_this_section("no-console", 0, source);

        assert_eq!(fix.content, "// oxlint-disable no-console\n");
        assert_eq!(fix.span, Span::empty(20));
    }

    #[test]
    fn disable_for_section_with_shebang_crlf() {
        let source = "#!/usr/bin/env node\r\nconsole.log('hello');";
        let fix = super::disable_for_this_section("no-console", 0, source);

        assert_eq!(fix.content, "// oxlint-disable no-console\n");
        assert_eq!(fix.span, Span::empty(21));
    }

    #[test]
    fn disable_for_section_mid_line() {
        let source = "const x = 5;";
        let section_offset = 6;
        let section_source_text = &source[6..];
        let fix =
            super::disable_for_this_section("no-unused-vars", section_offset, section_source_text);

        assert_eq!(fix.content, "\n// oxlint-disable no-unused-vars\n");
        assert_eq!(fix.span, Span::empty(0));
    }

    #[test]
    fn disable_for_section_vue_script_block_after_template() {
        let source =
            "<template>\n  <div />\n</template>\n<script>\nconsole.log('hello');\n</script>";
        let section_offset = source.find("<script>").unwrap() as u32 + "<script>".len() as u32;
        let section_source_text = &source[section_offset as usize..];
        let fix =
            super::disable_for_this_section("no-console", section_offset, section_source_text);

        assert_eq!(fix.content, "// oxlint-disable no-console\n");
        assert_eq!(fix.span, Span::empty(1));
    }

    #[test]
    fn disable_for_section_vue_script_block_after_template_crlf() {
        let source = "<template>\r\n  <div />\r\n</template>\r\n<script>\r\nconsole.log('hello');\r\n</script>";
        let section_offset = source.find("<script>").unwrap() as u32 + "<script>".len() as u32;
        let section_source_text = &source[section_offset as usize..];
        let fix =
            super::disable_for_this_section("no-console", section_offset, section_source_text);

        assert_eq!(fix.content, "// oxlint-disable no-console\n");
        assert_eq!(fix.span, Span::empty(2));
    }

    #[test]
    fn disable_for_section_vue_script_setup_mid_line() {
        let source = "<template><div /></template>\n<script setup>const x = 1;\n</script>";
        let section_offset = source.find("const x").unwrap() as u32;
        let section_source_text = &source[section_offset as usize..];
        let fix =
            super::disable_for_this_section("no-unused-vars", section_offset, section_source_text);

        assert_eq!(fix.content, "\n// oxlint-disable no-unused-vars\n");
        assert_eq!(fix.span, Span::empty(0));
    }

    #[test]
    fn disable_for_section_vue_script_block_merges_existing_ignore_line() {
        let existing = "// oxlint-disable no-alert";
        let source = format!(
            "<template>\n</template>\n<script>\n{existing}\nconsole.log('hello');\n</script>"
        );
        let section_offset = source.find(existing).unwrap() as u32;
        let section_source_text = &source[section_offset as usize..];

        let fix =
            super::disable_for_this_section("no-console", section_offset, section_source_text);

        assert_eq!(fix.content, " no-console");
        assert_eq!(fix.span, Span::empty(26));
    }

    #[test]
    fn disable_for_this_line_single_line() {
        let source = "console.log('hello');";
        let fix = super::disable_for_this_line("no-console", 0, 0, source);

        assert_eq!(fix.content, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(0));
    }

    #[test]
    fn disable_for_this_line_with_spaces() {
        let source = "  console.log('hello');";
        let fix = super::disable_for_this_line("no-console", 10, 0, source);

        assert_eq!(fix.content, "  // oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(0));
    }

    #[test]
    fn disable_for_this_line_with_tabs() {
        let source = "\t\tconsole.log('hello');";
        let fix = super::disable_for_this_line("no-console", 10, 0, source);

        assert_eq!(fix.content, "\t\t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(0));
    }

    #[test]
    fn disable_for_this_line_mixed_tabs_spaces() {
        let source = "\t  \tconsole.log('hello');";
        let fix = super::disable_for_this_line("no-console", 12, 0, source);

        assert_eq!(fix.content, "\t  \t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(0));
    }

    #[test]
    fn disable_for_this_line_multiline_with_tabs() {
        let source = "function test() {\n\tconsole.log('hello');\n}";
        let fix = super::disable_for_this_line("no-console", 27, 0, source);

        assert_eq!(fix.content, "\t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(18));
    }

    #[test]
    fn disable_for_this_line_multiline_with_spaces() {
        let source = "function test() {\n    console.log('hello');\n}";
        let fix = super::disable_for_this_line("no-console", 30, 0, source);

        assert_eq!(fix.content, "    // oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(18));
    }

    #[test]
    fn disable_for_this_line_complex_indentation() {
        let source = "function test() {\n\t  \t  console.log('hello');\n}";
        let fix = super::disable_for_this_line("no-console", 33, 0, source);

        assert_eq!(fix.content, "\t  \t  // oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(18));
    }

    #[test]
    fn disable_for_this_line_no_indentation() {
        let source = "function test() {\nconsole.log('hello');\n}";
        let fix = super::disable_for_this_line("no-console", 26, 0, source);

        assert_eq!(fix.content, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(18));
    }

    #[test]
    fn disable_for_this_line_crlf_with_tabs() {
        let source = "function test() {\r\n\tconsole.log('hello');\r\n}";
        let fix = super::disable_for_this_line("no-console", 28, 0, source);

        assert_eq!(fix.content, "\t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(19));
    }

    #[test]
    fn disable_for_this_line_deeply_nested() {
        let source = "if (true) {\n\t\tif (nested) {\n\t\t\tconsole.log('deep');\n\t\t}\n}";
        let fix = super::disable_for_this_line("no-console", 40, 0, source);

        assert_eq!(fix.content, "\t\t\t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(28));
    }

    #[test]
    fn disable_for_this_line_at_start_of_file() {
        let source = "console.log('hello');";
        let fix = super::disable_for_this_line("no-console", 0, 0, source);

        assert_eq!(fix.content, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(0));
    }

    #[test]
    fn disable_for_this_line_whitespace_only_continuous() {
        // Test that only continuous whitespace from line start is captured
        let source = "function test() {\n  \tcode  \there\n}";
        // Error at position of 'code' (after "  \t")
        let fix = super::disable_for_this_line("no-console", 21, 0, source);

        // Should only capture "  \t" at the beginning, not the spaces around "here"
        assert_eq!(fix.content, "  \t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(18));
    }

    #[test]
    fn disable_for_this_line_with_section_offset() {
        // Test framework file with section offset (like Vue/Svelte)
        let source = "<script>\nconsole.log('hello');\n</script>";
        let section_offset = 8; // At the \n after "<script>"
        let error_offset = 17; // At 'console'
        let section_source_text = &source[section_offset as usize..];
        let error_offset_in_section = error_offset - section_offset;
        let fix = super::disable_for_this_line(
            "no-console",
            error_offset_in_section,
            section_offset,
            section_source_text,
        );

        assert_eq!(fix.content, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(1));
    }

    #[test]
    fn disable_for_this_line_section_offset_mid_line() {
        // Test framework file where section starts mid-line
        let source = "<script>console.log('hello');\n</script>";
        let section_offset = 8; // After "<script>"
        let error_offset = 16; // At 'console'
        let section_source_text = &source[section_offset as usize..];
        let error_offset_in_section = error_offset - section_offset;
        let fix = super::disable_for_this_line(
            "no-console",
            error_offset_in_section,
            section_offset,
            section_source_text,
        );

        assert_eq!(fix.content, "\n// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(0));
    }

    #[test]
    fn disable_for_this_line_section_offset_with_indentation() {
        // Test framework file with indented code
        let source = "<template>\n</template>\n<script>\n  console.log('hello');\n</script>";
        let section_offset = 31; // At \n after "<script>"
        let error_offset = 36; // At 'console' (after "  ")
        let section_source_text = &source[section_offset as usize..];
        let error_offset_in_section = error_offset - section_offset;
        let fix = super::disable_for_this_line(
            "no-console",
            error_offset_in_section,
            section_offset,
            section_source_text,
        );

        assert_eq!(fix.content, "  // oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(1));
    }

    #[test]
    fn disable_for_this_line_section_offset_start() {
        // Test framework file where error is exactly at section offset
        let source = "<script>\nconsole.log('hello');\n</script>";
        let section_offset = 8; // At the \n after "<script>"
        let error_offset = 8; // Error exactly at section offset
        let section_source_text = &source[section_offset as usize..];
        let error_offset_in_section = error_offset - section_offset;
        let fix = super::disable_for_this_line(
            "no-console",
            error_offset_in_section,
            section_offset,
            section_source_text,
        );

        assert_eq!(fix.content, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(1));
    }

    #[test]
    fn disable_for_this_line_with_shebang() {
        let source = "#!/usr/bin/env node\nconsole.log('hello');";
        let fix = super::disable_for_this_line("no-console", 0, 0, source);

        assert_eq!(fix.content, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(20));
    }

    #[test]
    fn disable_for_this_line_with_shebang_crlf() {
        let source = "#!/usr/bin/env node\r\nconsole.log('hello');";
        let fix = super::disable_for_this_line("no-console", 0, 0, source);

        assert_eq!(fix.content, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.span, Span::empty(21));
    }

    #[test]
    fn disable_for_this_line_merges_with_existing_ignore_comment_above() {
        let existing = "// oxlint-disable-next-line no-alert";
        let source = format!("{existing}\nconsole.log('hello');");
        let error_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, &source);

        assert_eq!(fix.content, " no-console");
        assert_eq!(fix.span, Span::empty(36));
    }

    #[test]
    fn disable_for_this_line_merges_with_inline_disable_line_comment() {
        let existing = "// oxlint-disable-line no-alert";
        let source = format!("console.log('hello'); {existing}");
        let error_offset = source.find("console").unwrap() as u32;
        let insert_offset = source.find(existing).unwrap() as u32 + existing.len() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, &source);

        assert_eq!(fix.content, " no-console");
        assert_eq!(fix.span, Span::empty(insert_offset));
    }

    #[test]
    fn disable_for_this_line_merges_inline_disable_line_before_description() {
        let existing = "// oxlint-disable-line no-alert -- reason";
        let source = format!("console.log('hello'); {existing}");
        let error_offset = source.find("console").unwrap() as u32;
        let insert_offset = source.find("--").unwrap() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, &source);

        assert_eq!(fix.content, " no-console");
        assert_eq!(fix.span, Span::empty(insert_offset));
    }

    #[test]
    fn disable_for_this_line_merges_before_description_suffix() {
        let existing = "// oxlint-disable-next-line no-alert -- description";
        let source = format!("{existing}\nconsole.log('hello');");
        let error_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, &source);

        assert_eq!(fix.content, " no-console");
        assert_eq!(fix.span, Span::empty(37));
    }

    #[test]
    fn disable_for_this_line_merges_before_single_dash_description_suffix() {
        let existing = "// oxlint-disable-next-line no-alert\t-\treason";
        let source = format!("{existing}\nconsole.log('hello');");
        let error_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, &source);

        assert_eq!(fix.content, " no-console");
        assert_eq!(fix.span, Span::empty(37));
    }

    #[test]
    fn disable_for_this_line_merges_before_double_dash_without_leading_space() {
        let existing = "// oxlint-disable-next-line no-alert-- reason";
        let source = format!("{existing}\nconsole.log('hello');");
        let error_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, &source);

        assert_eq!(fix.content, " no-console");
        assert_eq!(fix.span, Span::empty(36));
    }

    #[test]
    fn disable_for_this_line_merges_with_eslint_disable_comment_above() {
        let existing = "// eslint-disable-next-line no-alert";
        let source = format!("{existing}\nconsole.log('hello');");
        let error_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, &source);

        assert_eq!(fix.content, " no-console");
        assert_eq!(fix.span, Span::empty(36));
    }

    #[test]
    fn disable_for_this_section_merges_with_existing_ignore_comment_above() {
        let existing = "// oxlint-disable no-alert";
        let source = format!("{existing}\nconsole.log('hello');");
        let section_offset = source.find(existing).unwrap() as u32;
        let section_source_text = &source[section_offset as usize..];

        let fix =
            super::disable_for_this_section("no-console", section_offset, section_source_text);

        assert_eq!(fix.content, " no-console");
        assert_eq!(fix.span, Span::empty(26));
    }

    #[test]
    fn disable_for_this_section_merges_with_eslint_disable_comment_above() {
        let existing = "// eslint-disable no-alert";
        let source = format!("{existing}\nconsole.log('hello');");
        let section_offset = source.find(existing).unwrap() as u32;
        let section_source_text = &source[section_offset as usize..];

        let fix =
            super::disable_for_this_section("no-console", section_offset, section_source_text);

        assert_eq!(fix.content, " no-console");
        assert_eq!(fix.span, Span::empty(26));
    }

    #[test]
    fn disable_for_this_line_does_not_merge_with_non_disable_comment_above() {
        let source = "// this is not a disable comment\nconsole.log('hello');";
        let error_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, source);

        assert_eq!(fix.content, "// oxlint-disable-next-line no-console\n");
    }

    #[test]
    fn disable_for_this_line_does_not_merge_with_lookalike_comment_above() {
        let source = "// oxlint-disable-next-line-foo no-alert\nconsole.log('hello');";
        let error_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, source);

        assert_eq!(fix.content, "// oxlint-disable-next-line no-console\n");
    }

    #[test]
    fn disable_for_this_section_does_not_merge_with_non_disable_comment_above() {
        let source = "// tslint:disable no-alert\nconsole.log('hello');";
        let section_offset = source.find("console").unwrap() as u32;
        let section_source_text = &source[section_offset as usize..];

        let fix =
            super::disable_for_this_section("no-console", section_offset, section_source_text);

        assert_eq!(fix.content, "\n// oxlint-disable no-console\n");
    }
}
