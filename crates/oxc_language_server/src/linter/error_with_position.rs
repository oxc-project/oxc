use std::str::FromStr;

use tower_lsp_server::lsp_types::{
    self, CodeDescription, Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity,
    NumberOrString, Position, Range, Uri,
};

use oxc_data_structures::rope::{Rope, get_line_column};
use oxc_diagnostics::{OxcCode, Severity};
use oxc_linter::{Fix, Message, PossibleFixes};

#[derive(Debug, Clone, Default)]
pub struct DiagnosticReport {
    pub diagnostic: Diagnostic,
    pub fixed_content: PossibleFixContent,
}

#[derive(Debug, Clone)]
pub struct FixedContent {
    pub message: Option<String>,
    pub code: String,
    pub range: Range,
}

#[derive(Debug, Clone, Default)]
pub enum PossibleFixContent {
    #[default]
    None,
    Single(FixedContent),
    Multiple(Vec<FixedContent>),
}

// clippy: the source field is checked and assumed to be less than 4GB, and
// we assume that the fix offset will not exceed 2GB in either direction
#[expect(clippy::cast_possible_truncation)]
pub fn message_to_lsp_diagnostic(
    message: &Message,
    uri: &Uri,
    source_text: &str,
    rope: &Rope,
) -> DiagnosticReport {
    let severity = match message.error.severity {
        Severity::Error => Some(lsp_types::DiagnosticSeverity::ERROR),
        _ => Some(lsp_types::DiagnosticSeverity::WARNING),
    };

    let related_information = message.error.labels.as_ref().map(|spans| {
        spans
            .iter()
            .map(|span| {
                let offset = span.offset() as u32;
                let start_position = offset_to_position(rope, offset, source_text);
                let end_position =
                    offset_to_position(rope, offset + span.len() as u32, source_text);

                lsp_types::DiagnosticRelatedInformation {
                    location: lsp_types::Location {
                        uri: uri.clone(),
                        range: lsp_types::Range::new(start_position, end_position),
                    },
                    message: span
                        .label()
                        .map_or_else(String::new, std::string::ToString::to_string),
                }
            })
            .collect()
    });

    let start_position = offset_to_position(rope, message.span.start, source_text);
    let end_position = offset_to_position(rope, message.span.end, source_text);
    let range = Range::new(start_position, end_position);

    let code = message.error.code.to_string();
    let code_description = message
        .error
        .url
        .as_ref()
        .map(|url| CodeDescription { href: Uri::from_str(url).ok().unwrap() });

    let diagnostic_message = match &message.error.help {
        Some(help) => {
            let main_msg = &message.error.message;
            let mut msg = String::with_capacity(main_msg.len() + help.len() + 7);
            msg.push_str(main_msg);
            msg.push_str("\nhelp: ");
            msg.push_str(help);
            msg
        }
        None => message.error.message.to_string(),
    };

    let diagnostic = Diagnostic {
        range,
        severity,
        code: Some(NumberOrString::String(code)),
        message: diagnostic_message,
        source: Some("oxc".into()),
        code_description,
        related_information,
        tags: None,
        data: None,
    };

    // Convert PossibleFixes directly to PossibleFixContent
    let fixed_content = match &message.fixes {
        PossibleFixes::None => PossibleFixContent::None,
        PossibleFixes::Single(fix) => {
            PossibleFixContent::Single(fix_to_fixed_content(fix, rope, source_text))
        }
        PossibleFixes::Multiple(fixes) => PossibleFixContent::Multiple(
            fixes.iter().map(|fix| fix_to_fixed_content(fix, rope, source_text)).collect(),
        ),
    };

    // Add ignore fixes
    let error_offset = message.span.start;
    let section_offset = message.section_offset;

    // If the error is exactly at the section offset and has 0 span length, it means that the file is the problem
    // and attaching a ignore comment would not ignore the error.
    // This is because the ignore comment would need to be placed before the error offset, which is not possible.
    if error_offset == section_offset && message.span.end == section_offset {
        return DiagnosticReport { diagnostic, fixed_content };
    }

    let fixed_content = add_ignore_fixes(
        fixed_content,
        &message.error.code,
        error_offset,
        section_offset,
        rope,
        source_text,
    );

    DiagnosticReport { diagnostic, fixed_content }
}

fn fix_to_fixed_content(fix: &Fix, rope: &Rope, source_text: &str) -> FixedContent {
    let start_position = offset_to_position(rope, fix.span.start, source_text);
    let end_position = offset_to_position(rope, fix.span.end, source_text);

    FixedContent {
        message: fix.message.as_ref().map(std::string::ToString::to_string),
        code: fix.content.to_string(),
        range: Range::new(start_position, end_position),
    }
}

pub fn generate_inverted_diagnostics(
    diagnostics: &[DiagnosticReport],
    uri: &Uri,
) -> Vec<DiagnosticReport> {
    let mut inverted_diagnostics = vec![];
    for d in diagnostics {
        let Some(related_info) = &d.diagnostic.related_information else {
            continue;
        };
        let related_information = Some(vec![DiagnosticRelatedInformation {
            location: lsp_types::Location { uri: uri.clone(), range: d.diagnostic.range },
            message: "original diagnostic".to_string(),
        }]);
        for r in related_info {
            if r.location.range == d.diagnostic.range {
                continue;
            }
            // If there is no message content for this span, then don't produce an additional diagnostic
            // which also has no content. This prevents issues where editors expect diagnostics to have messages.
            if r.message.is_empty() {
                continue;
            }
            inverted_diagnostics.push(DiagnosticReport {
                diagnostic: Diagnostic {
                    range: r.location.range,
                    severity: Some(DiagnosticSeverity::HINT),
                    code: None,
                    message: r.message.clone(),
                    source: d.diagnostic.source.clone(),
                    code_description: None,
                    related_information: related_information.clone(),
                    tags: None,
                    data: None,
                },
                fixed_content: PossibleFixContent::None,
            });
        }
    }
    inverted_diagnostics
}

pub fn offset_to_position(rope: &Rope, offset: u32, source_text: &str) -> Position {
    let (line, column) = get_line_column(rope, offset, source_text);
    Position::new(line, column)
}

/// Add "ignore this line" and "ignore this rule" fixes to the existing fixes.
/// These fixes will be added to the end of the existing fixes.
/// If the existing fixes already contain an "remove unused disable directive" fix,
/// then no ignore fixes will be added.
fn add_ignore_fixes(
    fixes: PossibleFixContent,
    code: &OxcCode,
    error_offset: u32,
    section_offset: u32,
    rope: &Rope,
    source_text: &str,
) -> PossibleFixContent {
    // do not append ignore code actions when the error is the ignore action
    if matches!(fixes, PossibleFixContent::Single(ref fix) if fix.message.as_ref().is_some_and(|message| message.starts_with("remove unused disable directive")))
    {
        return fixes;
    }

    let mut new_fixes: Vec<FixedContent> = vec![];
    if let PossibleFixContent::Single(fix) = fixes {
        new_fixes.push(fix);
    } else if let PossibleFixContent::Multiple(existing_fixes) = fixes {
        new_fixes.extend(existing_fixes);
    }

    if let Some(rule_name) = code.number.as_ref() {
        // TODO: doesn't support disabling multiple rules by name for a given line.
        new_fixes.push(disable_for_this_line(
            rule_name,
            error_offset,
            section_offset,
            rope,
            source_text,
        ));
        new_fixes.push(disable_for_this_section(rule_name, section_offset, rope, source_text));
    }

    if new_fixes.is_empty() {
        PossibleFixContent::None
    } else if new_fixes.len() == 1 {
        PossibleFixContent::Single(new_fixes.remove(0))
    } else {
        PossibleFixContent::Multiple(new_fixes)
    }
}

fn disable_for_this_line(
    rule_name: &str,
    error_offset: u32,
    section_offset: u32,
    rope: &Rope,
    source_text: &str,
) -> FixedContent {
    let bytes = source_text.as_bytes();
    // Find the line break before the error
    let mut line_break_offset = error_offset;
    for byte in bytes[section_offset as usize..error_offset as usize].iter().rev() {
        if *byte == b'\n' || *byte == b'\r' {
            break;
        }
        line_break_offset -= 1;
    }

    // For framework files, ensure we don't go before the section start
    if section_offset > 0 && line_break_offset < section_offset {
        line_break_offset = section_offset;
    }

    let (content_prefix, insert_offset) =
        get_section_insert_position(section_offset, line_break_offset, bytes);

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

    let position = offset_to_position(rope, insert_offset, source_text);
    FixedContent {
        message: Some(format!("Disable {rule_name} for this line")),
        code: format!(
            "{content_prefix}{whitespace_string}// oxlint-disable-next-line {rule_name}\n"
        ),
        range: Range::new(position, position),
    }
}

fn disable_for_this_section(
    rule_name: &str,
    section_offset: u32,
    rope: &Rope,
    source_text: &str,
) -> FixedContent {
    let comment = format!("// oxlint-disable {rule_name}\n");

    let (content_prefix, insert_offset) =
        get_section_insert_position(section_offset, section_offset, source_text.as_bytes());

    let content = format!("{content_prefix}{comment}");
    let position = offset_to_position(rope, insert_offset, source_text);

    FixedContent {
        message: Some(format!("Disable {rule_name} for this file")),
        code: content,
        range: Range::new(position, position),
    }
}

/// Get the insert position and content prefix for section-based insertions.
///
/// For framework files (section_offset > 0), this handles proper line break detection.
/// For regular JS files (section_offset == 0), it returns the offset as-is.
///
/// Returns (content_prefix, insert_offset) where:
/// - content_prefix: "\n" if we need to add a line break, "" otherwise
/// - insert_offset: the byte offset where the content should be inserted
fn get_section_insert_position(
    section_offset: u32,
    target_offset: u32,
    bytes: &[u8],
) -> (&'static str, u32) {
    if section_offset == 0 {
        // Regular JS files - insert at target offset
        ("", target_offset)
    } else if target_offset == section_offset {
        // Framework files - check for line breaks at section_offset
        let current = bytes.get(section_offset as usize);
        let next = bytes.get((section_offset + 1) as usize);

        match (current, next) {
            (Some(b'\n'), _) => {
                // LF at offset, insert after it
                ("", section_offset + 1)
            }
            (Some(b'\r'), Some(b'\n')) => {
                // CRLF at offset, insert after both
                ("", section_offset + 2)
            }
            _ => {
                // Not at line start, prepend newline
                ("\n", section_offset)
            }
        }
    } else {
        // Framework files where target_offset != section_offset (line was found)
        ("", target_offset)
    }
}

#[cfg(test)]
mod test {
    use oxc_data_structures::rope::Rope;

    use super::offset_to_position;

    #[test]
    fn single_line() {
        let source = "foo.bar!;";
        assert_position(source, 0, (0, 0));
        assert_position(source, 4, (0, 4));
        assert_position(source, 9, (0, 9));
    }

    #[test]
    fn multi_line() {
        let source = "console.log(\n  foo.bar!\n);";
        assert_position(source, 0, (0, 0));
        assert_position(source, 12, (0, 12));
        assert_position(source, 13, (1, 0));
        assert_position(source, 23, (1, 10));
        assert_position(source, 24, (2, 0));
        assert_position(source, 26, (2, 2));
    }

    #[test]
    fn multi_byte() {
        let source = "let foo = \n  '👍';";
        assert_position(source, 10, (0, 10));
        assert_position(source, 11, (1, 0));
        assert_position(source, 14, (1, 3));
        assert_position(source, 18, (1, 5));
        assert_position(source, 19, (1, 6));
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn out_of_bounds() {
        offset_to_position(&Rope::from_str("foo"), 100, "foo");
    }

    #[test]
    fn disable_for_section_js_file() {
        let source = "console.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_section("no-console", 0, &rope, source);

        assert_eq!(fix.code, "// oxlint-disable no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_section_after_lf() {
        let source = "<script>\nconsole.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_section("no-console", 8, &rope, source);

        assert_eq!(fix.code, "// oxlint-disable no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_section_after_crlf() {
        let source = "<script>\r\nconsole.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_section("no-console", 8, &rope, source);

        assert_eq!(fix.code, "// oxlint-disable no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_section_mid_line() {
        let source = "const x = 5;";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_section("no-unused-vars", 6, &rope, source);

        assert_eq!(fix.code, "\n// oxlint-disable no-unused-vars\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 6);
    }

    #[test]
    fn disable_for_this_line_single_line() {
        let source = "console.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 0, 0, &rope, source);

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_with_spaces() {
        let source = "  console.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 10, 0, &rope, source);

        assert_eq!(fix.code, "  // oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_with_tabs() {
        let source = "\t\tconsole.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 10, 0, &rope, source);

        assert_eq!(fix.code, "\t\t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_mixed_tabs_spaces() {
        let source = "\t  \tconsole.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 12, 0, &rope, source);

        assert_eq!(fix.code, "\t  \t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_multiline_with_tabs() {
        let source = "function test() {\n\tconsole.log('hello');\n}";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 27, 0, &rope, source);

        assert_eq!(fix.code, "\t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_multiline_with_spaces() {
        let source = "function test() {\n    console.log('hello');\n}";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 30, 0, &rope, source);

        assert_eq!(fix.code, "    // oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_complex_indentation() {
        let source = "function test() {\n\t  \t  console.log('hello');\n}";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 33, 0, &rope, source);

        assert_eq!(fix.code, "\t  \t  // oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_no_indentation() {
        let source = "function test() {\nconsole.log('hello');\n}";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 26, 0, &rope, source);

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_crlf_with_tabs() {
        let source = "function test() {\r\n\tconsole.log('hello');\r\n}";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 28, 0, &rope, source);

        assert_eq!(fix.code, "\t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_deeply_nested() {
        let source = "if (true) {\n\t\tif (nested) {\n\t\t\tconsole.log('deep');\n\t\t}\n}";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 40, 0, &rope, source);

        assert_eq!(fix.code, "\t\t\t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 2);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_at_start_of_file() {
        let source = "console.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 0, 0, &rope, source);

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_whitespace_only_continuous() {
        // Test that only continuous whitespace from line start is captured
        let source = "function test() {\n  \tcode  \there\n}";
        let rope = Rope::from_str(source);
        // Error at position of 'code' (after "  \t")
        let fix = super::disable_for_this_line("no-console", 21, 0, &rope, source);

        // Should only capture "  \t" at the beginning, not the spaces around "here"
        assert_eq!(fix.code, "  \t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_with_section_offset() {
        // Test framework file with section offset (like Vue/Svelte)
        let source = "<script>\nconsole.log('hello');\n</script>";
        let rope = Rope::from_str(source);
        let section_offset = 8; // At the \n after "<script>"
        let error_offset = 17; // At 'console'
        let fix =
            super::disable_for_this_line("no-console", error_offset, section_offset, &rope, source);

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_section_offset_mid_line() {
        // Test framework file where section starts mid-line
        let source = "<script>console.log('hello');\n</script>";
        let rope = Rope::from_str(source);
        let section_offset = 8; // After "<script>"
        let error_offset = 16; // At 'console'
        let fix =
            super::disable_for_this_line("no-console", error_offset, section_offset, &rope, source);

        assert_eq!(fix.code, "\n// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 8);
    }

    #[test]
    fn disable_for_this_line_section_offset_with_indentation() {
        // Test framework file with indented code
        let source = "<template>\n</template>\n<script>\n  console.log('hello');\n</script>";
        let rope = Rope::from_str(source);
        let section_offset = 31; // At \n after "<script>"
        let error_offset = 36; // At 'console' (after "  ")
        let fix =
            super::disable_for_this_line("no-console", error_offset, section_offset, &rope, source);

        assert_eq!(fix.code, "  // oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 3);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_section_offset_start() {
        // Test framework file where error is exactly at section offset
        let source = "<script>\nconsole.log('hello');\n</script>";
        let rope = Rope::from_str(source);
        let section_offset = 8; // At the \n after "<script>"
        let error_offset = 8; // Error exactly at section offset
        let fix =
            super::disable_for_this_line("no-console", error_offset, section_offset, &rope, source);

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    fn assert_position(source: &str, offset: u32, expected: (u32, u32)) {
        let position = offset_to_position(&Rope::from_str(source), offset, source);
        assert_eq!(position.line, expected.0);
        assert_eq!(position.character, expected.1);
    }
}
