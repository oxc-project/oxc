use std::borrow::Cow;

use tower_lsp_server::ls_types::{
    self, CodeDescription, Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity,
    NumberOrString, Position, Range, Uri,
};

use oxc_data_structures::rope::{Rope, get_line_column};
use oxc_diagnostics::{OxcCode, Severity};
use oxc_linter::{DisableDirectives, Fix, Message, PossibleFixes, RuleCommentType};

#[derive(Debug, Clone, Default)]
pub struct DiagnosticReport {
    pub diagnostic: Diagnostic,
    pub code_action: Option<LinterCodeAction>,
}

#[derive(Debug, Clone, Default)]
pub struct LinterCodeAction {
    pub range: Range,
    pub fixed_content: Vec<FixedContent>,
}

#[derive(Debug, Clone, Default)]
pub struct FixedContent {
    pub message: String,
    pub code: String,
    pub range: Range,
}

// clippy: the source field is checked and assumed to be less than 4GB, and
// we assume that the fix offset will not exceed 2GB in either direction
#[expect(clippy::cast_possible_truncation)]
pub fn message_to_lsp_diagnostic(
    mut message: Message,
    uri: &Uri,
    source_text: &str,
    rope: &Rope,
    directives: Option<&DisableDirectives>,
) -> DiagnosticReport {
    let severity = match message.error.severity {
        Severity::Error => Some(ls_types::DiagnosticSeverity::ERROR),
        _ => Some(ls_types::DiagnosticSeverity::WARNING),
    };

    let related_information = message.error.labels.as_ref().map(|spans| {
        spans
            .iter()
            .map(|span| {
                let offset = span.offset() as u32;
                let start_position = offset_to_position(rope, offset, source_text);
                let end_position =
                    offset_to_position(rope, offset + span.len() as u32, source_text);

                ls_types::DiagnosticRelatedInformation {
                    location: ls_types::Location {
                        uri: uri.clone(),
                        range: ls_types::Range::new(start_position, end_position),
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
        .and_then(|url| url.parse().ok())
        .map(|href| CodeDescription { href });

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

    // 1) Use `fixed_content.message` if it exists
    // 2) Try to parse the report diagnostic message
    // 3) Fallback to "Fix this problem"
    let alternative_fix_title: Cow<'static, str> =
        if let Some(code) = diagnostic_message.split(':').next() {
            format!("Fix this {code} problem").into()
        } else {
            std::borrow::Cow::Borrowed("Fix this problem")
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

    let mut fixed_content = vec![];
    // Convert PossibleFixes directly to PossibleFixContent
    match &mut message.fixes {
        PossibleFixes::None => {}
        PossibleFixes::Single(fix) => {
            if fix.message.is_none() {
                fix.message = Some(alternative_fix_title);
            }
            fixed_content.push(fix_to_fixed_content(fix, rope, source_text));
        }
        PossibleFixes::Multiple(fixes) => {
            fixed_content.extend(fixes.iter_mut().map(|fix| {
                if fix.message.is_none() {
                    fix.message = Some(alternative_fix_title.clone());
                }
                fix_to_fixed_content(fix, rope, source_text)
            }));
        }
    }

    // Add ignore fixes
    let error_offset = message.span.start;
    let section_offset = message.section_offset;

    // If the error is exactly at the section offset and has 0 span length, it means that the file is the problem
    // and attaching a ignore comment would not ignore the error.
    // This is because the ignore comment would need to be placed before the error offset, which is not possible.
    if error_offset == section_offset && message.span.end == section_offset {
        return DiagnosticReport {
            diagnostic,
            code_action: Some(LinterCodeAction { range, fixed_content }),
        };
    }

    add_ignore_fixes(
        &mut fixed_content,
        &message.error.code,
        error_offset,
        section_offset,
        rope,
        source_text,
        directives,
    );

    let code_action = if fixed_content.is_empty() {
        None
    } else {
        Some(LinterCodeAction { range, fixed_content })
    };

    DiagnosticReport { diagnostic, code_action }
}

fn fix_to_fixed_content(fix: &Fix, rope: &Rope, source_text: &str) -> FixedContent {
    let start_position = offset_to_position(rope, fix.span.start, source_text);
    let end_position = offset_to_position(rope, fix.span.end, source_text);

    debug_assert!(
        fix.message.is_some(),
        "Fix message should be present. `message_to_lsp_diagnostic` should modify fixes to include messages."
    );

    FixedContent {
        message: fix.message.as_ref().map(std::string::ToString::to_string).unwrap_or_default(),
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
            location: ls_types::Location { uri: uri.clone(), range: d.diagnostic.range },
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
                code_action: None,
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
    fixes: &mut Vec<FixedContent>,
    code: &OxcCode,
    error_offset: u32,
    section_offset: u32,
    rope: &Rope,
    source_text: &str,
    directives: Option<&DisableDirectives>,
) {
    // do not append ignore code actions when the error is the ignore action
    if fixes.len() == 1 && fixes[0].message.starts_with("remove unused disable directive") {
        return;
    }

    if let Some(rule_name) = code.number.as_ref() {
        fixes.push(disable_for_this_line(
            rule_name,
            error_offset,
            section_offset,
            rope,
            source_text,
            directives,
        ));
        fixes.push(disable_for_this_section(rule_name, section_offset, rope, source_text));
    }
}

fn disable_for_this_line(
    rule_name: &str,
    error_offset: u32,
    section_offset: u32,
    rope: &Rope,
    source_text: &str,
    directives: Option<&DisableDirectives>,
) -> FixedContent {
    if let Some(directives) = directives
        && let Some(existing_comment) =
            directives.find_disable_next_line_comment_for_position(error_offset)
    {
        return append_rule_to_existing_comment(rule_name, existing_comment, rope, source_text);
    }

    create_new_disable_comment(rule_name, error_offset, section_offset, rope, source_text)
}

/// Append a rule to an existing disable-next-line comment, or return a no-op if the rule already exists.
fn append_rule_to_existing_comment(
    rule_name: &str,
    existing_comment: &oxc_linter::DisableRuleComment,
    rope: &Rope,
    source_text: &str,
) -> FixedContent {
    let comment_span = existing_comment.span;
    let comment_text = &source_text[comment_span.start as usize..comment_span.end as usize];

    // Get the existing rules from the comment
    let existing_rules: Vec<&str> = match &existing_comment.r#type {
        RuleCommentType::All => {
            // If it's an "all" directive, just return a no-op (can't add more rules)
            let start_position = offset_to_position(rope, comment_span.start, source_text);
            let end_position = offset_to_position(rope, comment_span.end, source_text);
            return FixedContent {
                message: format!("Disable {rule_name} for this line"),
                code: comment_text.to_string(),
                range: Range::new(start_position, end_position),
            };
        }
        RuleCommentType::Single(rules) => rules.iter().map(|r| r.rule_name.as_str()).collect(),
    };

    // The rule should not already be in the comment - if it were, the diagnostic
    // wouldn't have been raised. This is a defensive check.
    debug_assert!(
        !existing_rules.contains(&rule_name),
        "Rule '{rule_name}' should not already be in the disable comment"
    );
    if existing_rules.contains(&rule_name) {
        // Rule already exists, return a no-op fix (same content)
        let start_position = offset_to_position(rope, comment_span.start, source_text);
        let end_position = offset_to_position(rope, comment_span.end, source_text);
        return FixedContent {
            message: format!("Disable {rule_name} for this line"),
            code: comment_text.to_string(),
            range: Range::new(start_position, end_position),
        };
    }

    // Append the new rule to the comment using comma separation (ESLint standard format)
    // The comment_text is just the content inside the comment (without // prefix for line comments)
    let new_comment = format!("{comment_text}, {rule_name}");

    let start_position = offset_to_position(rope, comment_span.start, source_text);
    let end_position = offset_to_position(rope, comment_span.end, source_text);

    FixedContent {
        message: format!("Disable {rule_name} for this line"),
        code: new_comment,
        range: Range::new(start_position, end_position),
    }
}

/// Create a new disable-next-line comment when no existing comment is found.
fn create_new_disable_comment(
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
        message: format!("Disable {rule_name} for this line"),
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
        message: format!("Disable {rule_name} for this whole file"),
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
        let fix = super::disable_for_this_line("no-console", 0, 0, &rope, source, None);

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_with_spaces() {
        let source = "  console.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 10, 0, &rope, source, None);

        assert_eq!(fix.code, "  // oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_with_tabs() {
        let source = "\t\tconsole.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 10, 0, &rope, source, None);

        assert_eq!(fix.code, "\t\t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_mixed_tabs_spaces() {
        let source = "\t  \tconsole.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 12, 0, &rope, source, None);

        assert_eq!(fix.code, "\t  \t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_multiline_with_tabs() {
        let source = "function test() {\n\tconsole.log('hello');\n}";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 27, 0, &rope, source, None);

        assert_eq!(fix.code, "\t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_multiline_with_spaces() {
        let source = "function test() {\n    console.log('hello');\n}";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 30, 0, &rope, source, None);

        assert_eq!(fix.code, "    // oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_complex_indentation() {
        let source = "function test() {\n\t  \t  console.log('hello');\n}";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 33, 0, &rope, source, None);

        assert_eq!(fix.code, "\t  \t  // oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_no_indentation() {
        let source = "function test() {\nconsole.log('hello');\n}";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 26, 0, &rope, source, None);

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_crlf_with_tabs() {
        let source = "function test() {\r\n\tconsole.log('hello');\r\n}";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 28, 0, &rope, source, None);

        assert_eq!(fix.code, "\t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_deeply_nested() {
        let source = "if (true) {\n\t\tif (nested) {\n\t\t\tconsole.log('deep');\n\t\t}\n}";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 40, 0, &rope, source, None);

        assert_eq!(fix.code, "\t\t\t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 2);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_at_start_of_file() {
        let source = "console.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_line("no-console", 0, 0, &rope, source, None);

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
        let fix = super::disable_for_this_line("no-console", 21, 0, &rope, source, None);

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
        let fix = super::disable_for_this_line(
            "no-console",
            error_offset,
            section_offset,
            &rope,
            source,
            None,
        );

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
        let fix = super::disable_for_this_line(
            "no-console",
            error_offset,
            section_offset,
            &rope,
            source,
            None,
        );

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
        let fix = super::disable_for_this_line(
            "no-console",
            error_offset,
            section_offset,
            &rope,
            source,
            None,
        );

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
        let fix = super::disable_for_this_line(
            "no-console",
            error_offset,
            section_offset,
            &rope,
            source,
            None,
        );

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    fn assert_position(source: &str, offset: u32, expected: (u32, u32)) {
        let position = offset_to_position(&Rope::from_str(source), offset, source);
        assert_eq!(position.line, expected.0);
        assert_eq!(position.character, expected.1);
    }

    #[test]
    fn append_rule_to_existing_comment_single_rule() {
        use oxc_linter::{DisableRuleComment, RuleCommentRule, RuleCommentType};
        use oxc_span::Span;

        // Source: "// oxlint-disable-next-line no-console\nconsole.log('hello');"
        // The comment span is the content inside the comment (without the "// " prefix)
        let source = "// oxlint-disable-next-line no-console\nconsole.log('hello');";
        let rope = Rope::from_str(source);

        // Comment content span starts after "// " at position 3, ends before newline at 38
        let existing_comment = DisableRuleComment {
            span: Span::new(3, 38),
            r#type: RuleCommentType::Single(vec![RuleCommentRule {
                rule_name: "no-console".to_string(),
                name_span: Span::new(28, 38),
            }]),
            is_next_line: true,
        };

        let fix =
            super::append_rule_to_existing_comment("no-debugger", &existing_comment, &rope, source);

        assert_eq!(fix.code, "oxlint-disable-next-line no-console, no-debugger");
        assert_eq!(fix.message, "Disable no-debugger for this line");
    }

    #[test]
    fn append_rule_to_existing_comment_multiple_rules() {
        use oxc_linter::{DisableRuleComment, RuleCommentRule, RuleCommentType};
        use oxc_span::Span;

        let source = "// oxlint-disable-next-line no-console, no-alert\nconsole.log('hello');";
        let rope = Rope::from_str(source);

        let existing_comment = DisableRuleComment {
            span: Span::new(3, 48),
            r#type: RuleCommentType::Single(vec![
                RuleCommentRule {
                    rule_name: "no-console".to_string(),
                    name_span: Span::new(28, 38),
                },
                RuleCommentRule { rule_name: "no-alert".to_string(), name_span: Span::new(40, 48) },
            ]),
            is_next_line: true,
        };

        let fix =
            super::append_rule_to_existing_comment("no-debugger", &existing_comment, &rope, source);

        assert_eq!(fix.code, "oxlint-disable-next-line no-console, no-alert, no-debugger");
        assert_eq!(fix.message, "Disable no-debugger for this line");
    }

    #[test]
    fn append_rule_to_existing_comment_all_directive() {
        use oxc_linter::{DisableRuleComment, RuleCommentType};
        use oxc_span::Span;

        // When the existing comment is "all" (disables everything), we can't add more rules
        let source = "// oxlint-disable-next-line\nconsole.log('hello');";
        let rope = Rope::from_str(source);

        let existing_comment = DisableRuleComment {
            span: Span::new(3, 27),
            r#type: RuleCommentType::All,
            is_next_line: true,
        };

        let fix =
            super::append_rule_to_existing_comment("no-debugger", &existing_comment, &rope, source);

        // Should return a no-op (same content)
        assert_eq!(fix.code, "oxlint-disable-next-line");
        assert_eq!(fix.message, "Disable no-debugger for this line");
    }

    #[test]
    fn append_rule_to_existing_comment_in_framework_file() {
        use oxc_linter::{DisableRuleComment, RuleCommentRule, RuleCommentType};
        use oxc_span::Span;

        // Simulates a Vue/Svelte file where the script section starts after <script>
        let source =
            "<script>\n// oxlint-disable-next-line no-console\nconsole.log('hello');\n</script>";
        let rope = Rope::from_str(source);

        // Comment content span in the framework file context
        // "<script>\n" = 9 bytes, then "// " = 3 bytes, comment content starts at 12
        let existing_comment = DisableRuleComment {
            span: Span::new(12, 47), // "oxlint-disable-next-line no-console"
            r#type: RuleCommentType::Single(vec![RuleCommentRule {
                rule_name: "no-console".to_string(),
                name_span: Span::new(37, 47),
            }]),
            is_next_line: true,
        };

        let fix =
            super::append_rule_to_existing_comment("no-debugger", &existing_comment, &rope, source);

        assert_eq!(fix.code, "oxlint-disable-next-line no-console, no-debugger");
        assert_eq!(fix.message, "Disable no-debugger for this line");
        // Verify the range is correct for the framework file
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 3); // After "// "
    }
}
