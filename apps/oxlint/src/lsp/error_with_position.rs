use std::borrow::Cow;

use oxc_span::Span;
use tower_lsp_server::ls_types::{
    self, CodeDescription, Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity,
    NumberOrString, Position, Range, Uri,
};

use oxc_diagnostics::{OxcCode, Severity};
use oxc_language_server::offset_to_position as lsp_offset_to_position;
use oxc_linter::{
    AllowWarnDeny, DisableDirectives, Fix, FixKind, Message, PossibleFixes, RuleCommentType,
};

use crate::lsp::{
    options::{RuleCustomizationSeverity, RulesCustomization},
    utils::get_full_rule_name,
};

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

#[derive(Debug, Clone)]
pub struct FixedContent {
    pub message: String,
    pub code: String,
    pub range: Range,
    pub kind: FixKind,
    pub lsp_kind: FixedContentKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixedContentKind {
    LintRule(OxcCode),
    IgnoreLintRuleLine,
    IgnoreLintRuleSection,
    UnusedDirective,
}

impl RulesCustomization {
    fn get_severity_for_rule(&self, code: &OxcCode) -> Option<RuleCustomizationSeverity> {
        let lookup = get_full_rule_name(code)?;
        self.rules.get(lookup.as_ref()).and_then(|customization| customization.severity.clone())
    }
}

impl TryFrom<RuleCustomizationSeverity> for DiagnosticSeverity {
    type Error = &'static str;

    fn try_from(value: RuleCustomizationSeverity) -> Result<Self, Self::Error> {
        match value {
            RuleCustomizationSeverity::Error => Ok(DiagnosticSeverity::ERROR),
            RuleCustomizationSeverity::Warn => Ok(DiagnosticSeverity::WARNING),
            RuleCustomizationSeverity::Hint => Ok(DiagnosticSeverity::HINT),
            RuleCustomizationSeverity::Info => Ok(DiagnosticSeverity::INFORMATION),
            RuleCustomizationSeverity::Off => Err(
                "Off severity should not be converted to DiagnosticSeverity as it means the rule is disabled and should not produce diagnostics.",
            ),
        }
    }
}

fn miette_severity_to_lsp_severity(value: Severity) -> DiagnosticSeverity {
    match value {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
        Severity::Advice => DiagnosticSeverity::HINT,
    }
}
pub fn message_to_lsp_diagnostic(
    message: Message,
    uri: &Uri,
    source_text: &str,
    rules_customization: Option<&RulesCustomization>,
) -> Option<DiagnosticReport> {
    let severity = if let Some(rules_customization) = rules_customization {
        if let Some(severity) = rules_customization.get_severity_for_rule(&message.error.code) {
            // filter off rules early
            DiagnosticSeverity::try_from(severity).ok()?
        } else {
            miette_severity_to_lsp_severity(message.error.severity)
        }
    } else {
        miette_severity_to_lsp_severity(message.error.severity)
    };

    let related_information = if message.error.labels.is_empty() {
        None
    } else {
        Some(
            message
                .error
                .labels
                .iter()
                .map(|span| {
                    let offset = span.offset();
                    let start_position = offset_to_position(offset, source_text);
                    let end_position = offset_to_position(offset + span.len(), source_text);

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
                .collect(),
        )
    };

    let start_position = offset_to_position(message.span.start, source_text);
    let end_position = offset_to_position(message.span.end, source_text);
    let range = Range::new(start_position, end_position);

    let code = message.error.code.to_string();
    let code_description = message
        .error
        .url
        .as_ref()
        .and_then(|url| url.parse().ok())
        .map(|href| CodeDescription { href });

    let mut diagnostic_message = String::with_capacity(
        message.error.message.len()
            + message.error.help.as_ref().map_or(0, |h| h.len() + 7) // "help: " prefix
            + message.error.note.as_ref().map_or(0, |n| n.len() + 7), // "note: " prefix
    );

    diagnostic_message.push_str(&message.error.message);
    if let Some(help) = &message.error.help {
        diagnostic_message.push_str("\nhelp: ");
        diagnostic_message.push_str(help);
    }

    if let Some(note) = &message.error.note {
        diagnostic_message.push_str("\nnote: ");
        diagnostic_message.push_str(note);
    }

    // 1) Use `fixed_content.message` if it exists
    // 2) Try to parse the report diagnostic message
    // 3) Fallback to "Fix this problem"
    let alternative_fix_title: Cow<'static, str> =
        if let Some(code) = message.error.message.split(':').next() {
            format!("Fix this {code} problem").into()
        } else {
            std::borrow::Cow::Borrowed("Fix this problem")
        };

    let diagnostic = Diagnostic {
        range,
        severity: Some(severity),
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
    match message.fixes {
        PossibleFixes::None => {}
        PossibleFixes::Single(mut fix) => {
            if fix.message.is_none() {
                fix.message = Some(alternative_fix_title);
            }
            fixed_content.push(fix_to_fixed_content(
                &fix,
                source_text,
                FixedContentKind::LintRule(message.error.code.clone()),
            ));
        }
        PossibleFixes::Multiple(fixes) => {
            fixed_content.extend(fixes.into_iter().map(|mut fix| {
                if fix.message.is_none() {
                    fix.message = Some(alternative_fix_title.clone());
                }
                fix_to_fixed_content(
                    &fix,
                    source_text,
                    FixedContentKind::LintRule(message.error.code.clone()),
                )
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
        return Some(DiagnosticReport {
            diagnostic,
            code_action: Some(LinterCodeAction { range, fixed_content }),
        });
    }

    add_ignore_fixes(
        &mut fixed_content,
        &message.error.code,
        error_offset,
        section_offset,
        message.jsx_child_offset,
        source_text,
    );

    let code_action = if fixed_content.is_empty() {
        None
    } else {
        Some(LinterCodeAction { range, fixed_content })
    };

    Some(DiagnosticReport { diagnostic, code_action })
}

fn fix_to_fixed_content(fix: &Fix, source_text: &str, fix_kind: FixedContentKind) -> FixedContent {
    let start_position = offset_to_position(fix.span.start, source_text);
    let end_position = offset_to_position(fix.span.end, source_text);

    debug_assert!(
        fix.message.is_some(),
        "Fix message should be present. `message_to_lsp_diagnostic` should modify fixes to include messages."
    );

    FixedContent {
        message: fix.message.as_ref().map(std::string::ToString::to_string).unwrap_or_default(),
        code: fix.content.to_string(),
        range: Range::new(start_position, end_position),
        kind: fix.kind,
        lsp_kind: fix_kind,
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

/// Generate diagnostics for unused disable directives, with fixes to remove them.
pub fn create_unused_directives_report(
    directives: &DisableDirectives,
    severity: AllowWarnDeny,
    source_text: &str,
) -> Vec<DiagnosticReport> {
    let mut reports = Vec::new();
    let fix_message = "remove unused disable directive";

    let severity = if severity == AllowWarnDeny::Deny {
        DiagnosticSeverity::ERROR
    } else {
        DiagnosticSeverity::WARNING
    };

    // Report unused disable comments
    let unused_disable = directives.collect_unused_disable_comments();
    for unused_comment in unused_disable {
        let span = unused_comment.span;
        let fix_span = unused_comment.fix_span;
        match unused_comment.r#type {
            RuleCommentType::All => {
                reports.push(build_unused_disable_diagnostic_report(
                    unused_comment.directive_prefix.unused_disable_message(),
                    span,
                    severity,
                    source_text,
                    Some(&Fix::delete(fix_span).with_message(fix_message)),
                ));
            }
            RuleCommentType::Single(rules) => {
                for rule in rules {
                    reports.push(build_unused_disable_diagnostic_report(
                        rule.directive_prefix.unused_disable_rule_message(&rule.rule_name),
                        rule.name_span,
                        severity,
                        source_text,
                        Some(&rule.create_fix(source_text, span).with_message(fix_message)),
                    ));
                }
            }
        }
    }

    // Report unused enable comments
    let unused_enable = directives.unused_enable_comments();
    for (directive_prefix, rule_name, span) in unused_enable {
        let message = if let Some(rule_name) = rule_name {
            directive_prefix.unused_enable_rule_message(rule_name)
        } else {
            directive_prefix.unused_enable_message()
        };
        reports.push(build_unused_disable_diagnostic_report(
            message,
            *span,
            severity,
            source_text,
            // TODO: fixer
            // copy the structure of disable directives
            None,
        ));
    }

    reports
}

fn build_unused_disable_diagnostic_report(
    message: String,
    span: Span,
    severity: DiagnosticSeverity,
    source_text: &str,
    fix: Option<&Fix>,
) -> DiagnosticReport {
    let start_position = offset_to_position(span.start, source_text);
    let end_position = offset_to_position(span.end, source_text);
    let range = Range::new(start_position, end_position);

    DiagnosticReport {
        diagnostic: Diagnostic {
            range,
            severity: Some(severity),
            code: Some("".into()),
            message,
            source: Some("oxc".into()),
            code_description: None,
            related_information: None,
            tags: None,
            data: None,
        },
        code_action: fix.map(|fix| LinterCodeAction {
            range,
            fixed_content: vec![fix_to_fixed_content(
                fix,
                source_text,
                FixedContentKind::UnusedDirective,
            )],
        }),
    }
}

pub fn offset_to_position(offset: u32, source_text: &str) -> Position {
    lsp_offset_to_position(source_text, offset)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisableDirective {
    NextLine,
    Line,
    Section,
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
    jsx_child_offset: Option<u32>,
    source_text: &str,
) {
    debug_assert!(
        !fixes.iter().any(|fix| fix.message.starts_with("remove unused disable directive")),
        "Unused disable directives should never pass pass this point, as they should be handled separately in `create_unused_directives_messages`."
    );

    let Some(rule_name_with_plugin) = get_full_rule_name(code) else {
        return;
    };
    // TODO: doesn't support disabling multiple rules by name for a given line.
    fixes.push(disable_for_this_line_with_jsx_child(
        &rule_name_with_plugin,
        error_offset,
        section_offset,
        jsx_child_offset,
        source_text,
    ));
    fixes.push(disable_for_this_section(&rule_name_with_plugin, section_offset, source_text));
}

fn disable_for_this_line(
    rule_name: &str,
    error_offset: u32,
    section_offset: u32,
    source_text: &str,
) -> FixedContent {
    disable_for_this_line_with_jsx_child(rule_name, error_offset, section_offset, None, source_text)
}

fn disable_for_this_line_with_jsx_child(
    rule_name: &str,
    error_offset: u32,
    section_offset: u32,
    jsx_child_offset: Option<u32>,
    source_text: &str,
) -> FixedContent {
    let bytes = source_text.as_bytes();
    let message = format!("Disable {rule_name} for this line");

    if let Some(jsx_child_offset) = jsx_child_offset {
        let child_line_start = line_start_offset(jsx_child_offset, section_offset, bytes);
        let error_line_start = line_start_offset(error_offset, section_offset, bytes);
        let target_offset =
            if child_line_start == error_line_start { jsx_child_offset } else { error_offset };
        let target_line_start = line_start_offset(target_offset, section_offset, bytes);
        let line_prefix = &bytes[target_line_start as usize..target_offset as usize];

        let (content_prefix, insert_offset, trailing_indent) =
            if line_prefix.iter().all(|byte| matches!(byte, b' ' | b'\t')) {
                let (content_prefix, insert_offset) =
                    get_section_insert_position(section_offset, target_line_start, bytes);
                (content_prefix, insert_offset, String::new())
            } else {
                ("", target_offset, " ".repeat((target_offset - target_line_start) as usize))
            };
        let whitespace_range = &bytes[insert_offset as usize..target_offset as usize];
        let whitespace_len =
            whitespace_range.iter().take_while(|c| matches!(c, b' ' | b'\t')).count();
        let whitespace = String::from_utf8_lossy(&whitespace_range[..whitespace_len]);

        if let Some(existing_comment_end) =
            get_existing_jsx_disable_comment_end(target_offset, bytes)
        {
            let position = offset_to_position(existing_comment_end, source_text);
            return FixedContent {
                message,
                code: format!(" {rule_name}"),
                range: Range::new(position, position),
                kind: FixKind::SafeFix,
                lsp_kind: FixedContentKind::IgnoreLintRuleLine,
            };
        }

        let position = offset_to_position(insert_offset, source_text);

        return FixedContent {
            message,
            code: format!(
                "{content_prefix}{whitespace}{{/* oxlint-disable-next-line {rule_name} */}}\n{trailing_indent}"
            ),
            range: Range::new(position, position),
            kind: FixKind::SafeFix,
            lsp_kind: FixedContentKind::IgnoreLintRuleLine,
        };
    }

    // Reuse an inline disable-line comment on the same line when present.
    if let Some(existing_comment_end) = get_inline_disable_line_comment_end(error_offset, bytes) {
        let position = offset_to_position(existing_comment_end, source_text);
        return FixedContent {
            message,
            code: format!(" {rule_name}"),
            range: Range::new(position, position),
            kind: FixKind::SafeFix,
            lsp_kind: FixedContentKind::IgnoreLintRuleLine,
        };
    }

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

    // Reuse an existing disable-next-line comment when present by appending the rule.
    if let Some(existing_comment_end) =
        get_existing_disable_comment_end(insert_offset, DisableDirective::NextLine, bytes)
    {
        let position = offset_to_position(existing_comment_end, source_text);
        return FixedContent {
            message,
            code: format!(" {rule_name}"),
            range: Range::new(position, position),
            kind: FixKind::SafeFix,
            lsp_kind: FixedContentKind::IgnoreLintRuleLine,
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

    let position = offset_to_position(insert_offset, source_text);
    FixedContent {
        message,
        code: format!(
            "{content_prefix}{whitespace_string}// oxlint-disable-next-line {rule_name}\n"
        ),
        range: Range::new(position, position),
        kind: FixKind::SafeFix,
        lsp_kind: FixedContentKind::IgnoreLintRuleLine,
    }
}

fn line_start_offset(offset: u32, section_offset: u32, bytes: &[u8]) -> u32 {
    let mut line_start = offset;
    for byte in bytes[section_offset as usize..offset as usize].iter().rev() {
        if *byte == b'\n' || *byte == b'\r' {
            break;
        }
        line_start -= 1;
    }
    line_start.max(section_offset)
}

#[expect(clippy::cast_possible_truncation)]
fn get_existing_jsx_disable_comment_end(target_offset: u32, bytes: &[u8]) -> Option<u32> {
    let target_offset = target_offset as usize;
    let prefix = bytes.get(..target_offset)?;
    let comment_start = prefix.windows(3).rposition(|window| window == b"{/*")?;
    let content_start = comment_start + 3;
    let close_start = content_start
        + bytes
            .get(content_start..target_offset)?
            .windows(3)
            .position(|window| window == b"*/}")?;

    let between_comment_and_target = bytes.get(close_start + 3..target_offset)?;
    if !between_comment_and_target.iter().all(u8::is_ascii_whitespace)
        || !between_comment_and_target.iter().any(|byte| matches!(byte, b'\n' | b'\r'))
    {
        return None;
    }

    let content = bytes.get(content_start..close_start)?;
    let mut index = 0;
    while index < content.len() && content[index].is_ascii_whitespace() {
        index += 1;
    }

    let directive = if content[index..].starts_with(b"oxlint-disable-next-line") {
        b"oxlint-disable-next-line".as_slice()
    } else if content[index..].starts_with(b"eslint-disable-next-line") {
        b"eslint-disable-next-line".as_slice()
    } else {
        return None;
    };
    index += directive.len();

    if index < content.len() && !content[index].is_ascii_whitespace() {
        return None;
    }

    let content_end =
        content.iter().rposition(|byte| !byte.is_ascii_whitespace()).map_or(index, |last| last + 1);
    let merge_end = find_description_start_offset(&content[index..content_end])
        .map_or(content_end, |offset| index + offset);

    Some((content_start + merge_end) as u32)
}

fn disable_for_this_section(
    rule_name: &str,
    section_offset: u32,
    source_text: &str,
) -> FixedContent {
    let bytes = source_text.as_bytes();
    let message = format!("Disable {rule_name} for this whole file");

    let (content_prefix, insert_offset) =
        get_section_insert_position(section_offset, section_offset, bytes);

    // Reuse an existing section disable comment when present by appending the rule.
    if let Some(existing_comment_end) =
        get_existing_disable_comment_end(insert_offset, DisableDirective::Section, bytes)
    {
        let position = offset_to_position(existing_comment_end, source_text);
        return FixedContent {
            message,
            code: format!(" {rule_name}"),
            range: Range::new(position, position),
            kind: FixKind::SafeFix,
            lsp_kind: FixedContentKind::IgnoreLintRuleSection,
        };
    }

    let content = format!("{content_prefix}// oxlint-disable {rule_name}\n");
    let position = offset_to_position(insert_offset, source_text);

    FixedContent {
        message,
        code: content,
        range: Range::new(position, position),
        kind: FixKind::SafeFix,
        lsp_kind: FixedContentKind::IgnoreLintRuleSection,
    }
}

/// Get the insert position and content prefix for section-based insertions.
///
/// For framework files (section_offset > 0), this handles proper line break detection.
/// For regular JS files (section_offset == 0), this handles shebang lines.
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
    if section_offset == 0 && target_offset == 0 {
        if bytes.starts_with(b"#!") {
            // Shebang present, insert after the first line
            let mut shebang_end = 0;
            for (i, &byte) in bytes.iter().enumerate() {
                if byte == b'\n' {
                    shebang_end = i + 1;
                    break;
                }
            }
            return ("", shebang_end as u32);
        }
        // Regular JS file without shebang, insert at start
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

#[cfg(test)]
#[expect(clippy::cast_possible_truncation)]
mod test {
    use oxc_diagnostics::OxcCode;

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
    fn unicode_line_and_paragraph_separators_are_not_lsp_line_breaks() {
        let source = "a\u{2028}b\nc\u{2029}d";
        assert_position(source, source.find('b').unwrap() as u32, (0, 2));
        assert_position(source, source.find('c').unwrap() as u32, (1, 0));
        assert_position(source, source.find('d').unwrap() as u32, (1, 2));
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn out_of_bounds() {
        offset_to_position(100, "foo");
    }

    #[test]
    fn add_ignore_fixes_uses_user_facing_plugin_names() {
        let source = "foo();";
        let code = OxcCode { scope: Some("jsx-a11y".into()), number: Some("alt-text".into()) };
        let mut fixes = vec![];

        super::add_ignore_fixes(&mut fixes, &code, 0, 0, None, source);

        assert_eq!(fixes[0].code, "// oxlint-disable-next-line jsx-a11y/alt-text\n");
        assert_eq!(fixes[1].code, "// oxlint-disable jsx-a11y/alt-text\n");
    }

    #[test]
    fn disable_for_section_js_file() {
        let source = "console.log('hello');";
        let fix = super::disable_for_this_section("no-console", 0, source);

        assert_eq!(fix.code, "// oxlint-disable no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_section_after_lf() {
        let source = "<script>\nconsole.log('hello');";
        let fix = super::disable_for_this_section("no-console", 8, source);

        assert_eq!(fix.code, "// oxlint-disable no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_section_after_crlf() {
        let source = "<script>\r\nconsole.log('hello');";
        let fix = super::disable_for_this_section("no-console", 8, source);

        assert_eq!(fix.code, "// oxlint-disable no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_section_with_shebang() {
        let source = "#!/usr/bin/env node\nconsole.log('hello');";
        let fix = super::disable_for_this_section("no-console", 0, source);

        assert_eq!(fix.code, "// oxlint-disable no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_section_with_shebang_crlf() {
        let source = "#!/usr/bin/env node\r\nconsole.log('hello');";
        let fix = super::disable_for_this_section("no-console", 0, source);

        assert_eq!(fix.code, "// oxlint-disable no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_section_mid_line() {
        let source = "const x = 5;";
        let fix = super::disable_for_this_section("no-unused-vars", 6, source);

        assert_eq!(fix.code, "\n// oxlint-disable no-unused-vars\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 6);
    }

    #[test]
    fn disable_for_section_vue_script_block_after_template() {
        let source =
            "<template>\n  <div />\n</template>\n<script>\nconsole.log('hello');\n</script>";
        let section_offset = source.find("<script>").unwrap() as u32 + "<script>".len() as u32;
        let fix = super::disable_for_this_section("no-console", section_offset, source);

        assert_eq!(fix.code, "// oxlint-disable no-console\n");
        assert_eq!(fix.range.start.line, 4);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_section_vue_script_block_after_template_crlf() {
        let source = "<template>\r\n  <div />\r\n</template>\r\n<script>\r\nconsole.log('hello');\r\n</script>";
        let section_offset = source.find("<script>").unwrap() as u32 + "<script>".len() as u32;
        let fix = super::disable_for_this_section("no-console", section_offset, source);

        assert_eq!(fix.code, "// oxlint-disable no-console\n");
        assert_eq!(fix.range.start.line, 4);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_section_vue_script_setup_mid_line() {
        let source = "<template><div /></template>\n<script setup>const x = 1;\n</script>";
        let section_offset = source.find("const x").unwrap() as u32;
        let fix = super::disable_for_this_section("no-unused-vars", section_offset, source);

        assert_eq!(fix.code, "\n// oxlint-disable no-unused-vars\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, "<script setup>".len() as u32);
    }

    #[test]
    fn disable_for_section_vue_script_block_merges_existing_ignore_line() {
        let existing = "// oxlint-disable no-alert";
        let source = format!(
            "<template>\n</template>\n<script>\n{existing}\nconsole.log('hello');\n</script>"
        );
        let section_offset = source.find(existing).unwrap() as u32;

        let fix = super::disable_for_this_section("no-console", section_offset, &source);

        assert_eq!(fix.code, " no-console");
        assert_eq!(fix.range.start.line, 3);
        assert_eq!(fix.range.start.character, existing.len() as u32);
        assert_eq!(fix.range.end.line, 3);
        assert_eq!(fix.range.end.character, existing.len() as u32);
    }

    #[test]
    fn disable_for_this_line_single_line() {
        let source = "console.log('hello');";
        let fix = super::disable_for_this_line("no-console", 0, 0, source);

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_in_jsx_uses_brace_escaped_comment_before_element() {
        let source = include_str!("fixtures/jsx-disable-multiline.tsx");
        let jsx_child_offset = source.find("<div").unwrap() as u32;
        let fix = super::disable_for_this_line_with_jsx_child(
            "jsx-a11y/interactive-supports-focus",
            jsx_child_offset,
            0,
            Some(jsx_child_offset),
            source,
        );

        insta::assert_snapshot!(
            fix.code,
            @"      {/* oxlint-disable-next-line jsx-a11y/interactive-supports-focus */}"
        );
        assert_eq!(fix.range.start.line, 4);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_in_jsx_inserts_before_same_line_nested_child() {
        let source = "return (\n  <div><button onClick={submit} role=\"button\" /></div>\n);";
        let error_offset = source.find("onClick").unwrap() as u32;
        let jsx_child_offset = source.find("<button").unwrap() as u32;
        let fix = super::disable_for_this_line_with_jsx_child(
            "jsx-a11y/interactive-supports-focus",
            error_offset,
            0,
            Some(jsx_child_offset),
            source,
        );

        assert_eq!(
            fix.code,
            "{/* oxlint-disable-next-line jsx-a11y/interactive-supports-focus */}\n       "
        );
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 7);
    }

    #[test]
    fn disable_for_this_line_in_jsx_uses_error_line_for_multiline_descendant() {
        let source = "const node = (\n  <div>\n    <p>\n      \"\n    </p>\n  </div>\n);";
        let error_offset = source.find('"').unwrap() as u32;
        let jsx_child_offset = source.find("<p>").unwrap() as u32;
        let fix = super::disable_for_this_line_with_jsx_child(
            "react/no-unescaped-entities",
            error_offset,
            0,
            Some(jsx_child_offset),
            source,
        );

        assert_eq!(
            fix.code,
            "      {/* oxlint-disable-next-line react/no-unescaped-entities */}\n"
        );
        assert_eq!(fix.range.start.line, 3);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_in_jsx_wraps_direct_child_expression() {
        let source = "const node = (\n  <div>\n    {foo}\n  </div>\n);";
        let error_offset = source.find("foo").unwrap() as u32;
        let jsx_child_offset = source.find("{foo}").unwrap() as u32;
        let fix = super::disable_for_this_line_with_jsx_child(
            "no-undef",
            error_offset,
            0,
            Some(jsx_child_offset),
            source,
        );

        assert_eq!(fix.code, "    {/* oxlint-disable-next-line no-undef */}\n");
        assert_eq!(fix.range.start.line, 2);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_in_jsx_attribute_expression_uses_js_comment() {
        let source = "const node = (\n  <div value={\n    foo\n  } />\n);";
        let error_offset = source.find("foo").unwrap() as u32;
        let fix =
            super::disable_for_this_line_with_jsx_child("no-undef", error_offset, 0, None, source);

        assert_eq!(fix.code, "    // oxlint-disable-next-line no-undef\n");
        assert_eq!(fix.range.start.line, 2);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_in_jsx_attribute_anchors_before_child() {
        let source = "const node = (\n  <Parent>\n    <Child value={foo} />\n  </Parent>\n);";
        let error_offset = source.find("foo").unwrap() as u32;
        let jsx_child_offset = source.find("<Child").unwrap() as u32;
        let fix = super::disable_for_this_line_with_jsx_child(
            "no-undef",
            error_offset,
            0,
            Some(jsx_child_offset),
            source,
        );

        assert_eq!(fix.code, "    {/* oxlint-disable-next-line no-undef */}\n");
        assert_eq!(fix.range.start.line, 2);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_in_multiline_jsx_expression_uses_js_comment() {
        let source = "const node = <div>{foo &&\n  bar}</div>;";
        let error_offset = source.find("bar").unwrap() as u32;
        let fix =
            super::disable_for_this_line_with_jsx_child("no-undef", error_offset, 0, None, source);

        assert_eq!(fix.code, "  // oxlint-disable-next-line no-undef\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_in_jsx_anchors_before_root_closing_tag() {
        let source = "const node = (\n  <div>\n  </div>\n);";
        let error_offset = source.find("</div>").unwrap() as u32;
        let fix = super::disable_for_this_line_with_jsx_child(
            "react/self-closing-comp",
            error_offset,
            0,
            Some(error_offset),
            source,
        );

        assert_eq!(fix.code, "  {/* oxlint-disable-next-line react/self-closing-comp */}\n");
        assert_eq!(fix.range.start.line, 2);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_merges_with_existing_jsx_comment() {
        let existing = "{/* oxlint-disable-next-line no-alert */}";
        let source = format!(
            "const node = (\n  <div>\n    {existing}\n    <button onClick={{foo}} />\n  </div>\n);"
        );
        let error_offset = source.find("foo").unwrap() as u32;
        let jsx_child_offset = source.find("<button").unwrap() as u32;
        let fix = super::disable_for_this_line_with_jsx_child(
            "no-undef",
            error_offset,
            0,
            Some(jsx_child_offset),
            &source,
        );

        assert_eq!(fix.code, " no-undef");
        assert_eq!(
            fix.range.start.character,
            ("    ".len() + existing.find(" */}").unwrap()) as u32
        );
        assert_eq!(fix.range.start.line, 2);
    }

    #[test]
    fn disable_for_this_line_with_spaces() {
        let source = "  console.log('hello');";
        let fix = super::disable_for_this_line("no-console", 10, 0, source);

        assert_eq!(fix.code, "  // oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_with_tabs() {
        let source = "\t\tconsole.log('hello');";
        let fix = super::disable_for_this_line("no-console", 10, 0, source);

        assert_eq!(fix.code, "\t\t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_mixed_tabs_spaces() {
        let source = "\t  \tconsole.log('hello');";
        let fix = super::disable_for_this_line("no-console", 12, 0, source);

        assert_eq!(fix.code, "\t  \t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_multiline_with_tabs() {
        let source = "function test() {\n\tconsole.log('hello');\n}";
        let fix = super::disable_for_this_line("no-console", 27, 0, source);

        assert_eq!(fix.code, "\t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_multiline_with_spaces() {
        let source = "function test() {\n    console.log('hello');\n}";
        let fix = super::disable_for_this_line("no-console", 30, 0, source);

        assert_eq!(fix.code, "    // oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_complex_indentation() {
        let source = "function test() {\n\t  \t  console.log('hello');\n}";
        let fix = super::disable_for_this_line("no-console", 33, 0, source);

        assert_eq!(fix.code, "\t  \t  // oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_no_indentation() {
        let source = "function test() {\nconsole.log('hello');\n}";
        let fix = super::disable_for_this_line("no-console", 26, 0, source);

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_crlf_with_tabs() {
        let source = "function test() {\r\n\tconsole.log('hello');\r\n}";
        let fix = super::disable_for_this_line("no-console", 28, 0, source);

        assert_eq!(fix.code, "\t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_deeply_nested() {
        let source = "if (true) {\n\t\tif (nested) {\n\t\t\tconsole.log('deep');\n\t\t}\n}";
        let fix = super::disable_for_this_line("no-console", 40, 0, source);

        assert_eq!(fix.code, "\t\t\t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 2);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_at_start_of_file() {
        let source = "console.log('hello');";
        let fix = super::disable_for_this_line("no-console", 0, 0, source);

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_whitespace_only_continuous() {
        // Test that only continuous whitespace from line start is captured
        let source = "function test() {\n  \tcode  \there\n}";
        // Error at position of 'code' (after "  \t")
        let fix = super::disable_for_this_line("no-console", 21, 0, source);

        // Should only capture "  \t" at the beginning, not the spaces around "here"
        assert_eq!(fix.code, "  \t// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_with_section_offset() {
        // Test framework file with section offset (like Vue/Svelte)
        let source = "<script>\nconsole.log('hello');\n</script>";
        let section_offset = 8; // At the \n after "<script>"
        let error_offset = 17; // At 'console'
        let fix = super::disable_for_this_line("no-console", error_offset, section_offset, source);

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_section_offset_mid_line() {
        // Test framework file where section starts mid-line
        let source = "<script>console.log('hello');\n</script>";
        let section_offset = 8; // After "<script>"
        let error_offset = 16; // At 'console'
        let fix = super::disable_for_this_line("no-console", error_offset, section_offset, source);

        assert_eq!(fix.code, "\n// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, 8);
    }

    #[test]
    fn disable_for_this_line_section_offset_with_indentation() {
        // Test framework file with indented code
        let source = "<template>\n</template>\n<script>\n  console.log('hello');\n</script>";
        let section_offset = 31; // At \n after "<script>"
        let error_offset = 36; // At 'console' (after "  ")
        let fix = super::disable_for_this_line("no-console", error_offset, section_offset, source);

        assert_eq!(fix.code, "  // oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 3);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_section_offset_start() {
        // Test framework file where error is exactly at section offset
        let source = "<script>\nconsole.log('hello');\n</script>";
        let section_offset = 8; // At the \n after "<script>"
        let error_offset = 8; // Error exactly at section offset
        let fix = super::disable_for_this_line("no-console", error_offset, section_offset, source);

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_with_shebang() {
        let source = "#!/usr/bin/env node\nconsole.log('hello');";
        let fix = super::disable_for_this_line("no-console", 0, 0, source);

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_with_shebang_crlf() {
        let source = "#!/usr/bin/env node\r\nconsole.log('hello');";
        let fix = super::disable_for_this_line("no-console", 0, 0, source);

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
        assert_eq!(fix.range.start.line, 1);
        assert_eq!(fix.range.start.character, 0);
    }

    #[test]
    fn disable_for_this_line_merges_with_existing_ignore_comment_above() {
        let existing = "// oxlint-disable-next-line no-alert";
        let source = format!("{existing}\nconsole.log('hello');");
        let error_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, &source);

        assert_eq!(fix.code, " no-console");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, existing.len() as u32);
        assert_eq!(fix.range.end.line, 0);
        assert_eq!(fix.range.end.character, existing.len() as u32);
    }

    #[test]
    fn disable_for_this_line_merges_with_inline_disable_line_comment() {
        let existing = "// oxlint-disable-line no-alert";
        let source = format!("console.log('hello'); {existing}");
        let error_offset = source.find("console").unwrap() as u32;
        let insert_offset = source.find(existing).unwrap() as u32 + existing.len() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, &source);

        assert_eq!(fix.code, " no-console");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, insert_offset);
        assert_eq!(fix.range.end.line, 0);
        assert_eq!(fix.range.end.character, insert_offset);
    }

    #[test]
    fn disable_for_this_line_merges_inline_disable_line_before_description() {
        let existing = "// oxlint-disable-line no-alert -- reason";
        let source = format!("console.log('hello'); {existing}");
        let error_offset = source.find("console").unwrap() as u32;
        let insert_offset = source.find("--").unwrap() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, &source);

        assert_eq!(fix.code, " no-console");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, insert_offset);
        assert_eq!(fix.range.end.line, 0);
        assert_eq!(fix.range.end.character, insert_offset);
    }

    #[test]
    fn disable_for_this_line_merges_before_description_suffix() {
        let existing = "// oxlint-disable-next-line no-alert -- description";
        let source = format!("{existing}\nconsole.log('hello');");
        let error_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, &source);

        assert_eq!(fix.code, " no-console");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, existing.find("--").unwrap() as u32);
        assert_eq!(fix.range.end.line, 0);
        assert_eq!(fix.range.end.character, existing.find("--").unwrap() as u32);
    }

    #[test]
    fn disable_for_this_line_merges_before_single_dash_description_suffix() {
        let existing = "// oxlint-disable-next-line no-alert\t-\treason";
        let source = format!("{existing}\nconsole.log('hello');");
        let error_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, &source);

        assert_eq!(fix.code, " no-console");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, existing.find("-\t").unwrap() as u32);
        assert_eq!(fix.range.end.line, 0);
        assert_eq!(fix.range.end.character, existing.find("-\t").unwrap() as u32);
    }

    #[test]
    fn disable_for_this_line_merges_before_double_dash_without_leading_space() {
        let existing = "// oxlint-disable-next-line no-alert-- reason";
        let source = format!("{existing}\nconsole.log('hello');");
        let error_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, &source);

        assert_eq!(fix.code, " no-console");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, existing.find("--").unwrap() as u32);
        assert_eq!(fix.range.end.line, 0);
        assert_eq!(fix.range.end.character, existing.find("--").unwrap() as u32);
    }

    #[test]
    fn disable_for_this_line_merges_with_eslint_disable_comment_above() {
        let existing = "// eslint-disable-next-line no-alert";
        let source = format!("{existing}\nconsole.log('hello');");
        let error_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, &source);

        assert_eq!(fix.code, " no-console");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, existing.len() as u32);
        assert_eq!(fix.range.end.line, 0);
        assert_eq!(fix.range.end.character, existing.len() as u32);
    }

    #[test]
    fn disable_for_this_section_merges_with_existing_ignore_comment_above() {
        let existing = "// oxlint-disable no-alert";
        let source = format!("{existing}\nconsole.log('hello');");
        let section_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_section("no-console", section_offset, &source);

        assert_eq!(fix.code, " no-console");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, existing.len() as u32);
        assert_eq!(fix.range.end.line, 0);
        assert_eq!(fix.range.end.character, existing.len() as u32);
    }

    #[test]
    fn disable_for_this_section_merges_with_eslint_disable_comment_above() {
        let existing = "// eslint-disable no-alert";
        let source = format!("{existing}\nconsole.log('hello');");
        let section_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_section("no-console", section_offset, &source);

        assert_eq!(fix.code, " no-console");
        assert_eq!(fix.range.start.line, 0);
        assert_eq!(fix.range.start.character, existing.len() as u32);
        assert_eq!(fix.range.end.line, 0);
        assert_eq!(fix.range.end.character, existing.len() as u32);
    }

    #[test]
    fn disable_for_this_line_does_not_merge_with_non_disable_comment_above() {
        let source = "// this is not a disable comment\nconsole.log('hello');";
        let error_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, source);

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
    }

    #[test]
    fn disable_for_this_line_does_not_merge_with_lookalike_comment_above() {
        let source = "// oxlint-disable-next-line-foo no-alert\nconsole.log('hello');";
        let error_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_line("no-console", error_offset, 0, source);

        assert_eq!(fix.code, "// oxlint-disable-next-line no-console\n");
    }

    #[test]
    fn disable_for_this_section_does_not_merge_with_non_disable_comment_above() {
        let source = "// tslint:disable no-alert\nconsole.log('hello');";
        let section_offset = source.find("console").unwrap() as u32;

        let fix = super::disable_for_this_section("no-console", section_offset, source);

        assert_eq!(fix.code, "\n// oxlint-disable no-console\n");
    }

    fn assert_position(source: &str, offset: u32, expected: (u32, u32)) {
        let position = offset_to_position(offset, source);
        assert_eq!(position.line, expected.0);
        assert_eq!(position.character, expected.1);
    }
}
