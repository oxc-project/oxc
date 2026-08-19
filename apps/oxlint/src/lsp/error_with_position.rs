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

fn severity_to_lsp_severity(value: Severity) -> DiagnosticSeverity {
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
            severity_to_lsp_severity(message.error.severity)
        }
    } else {
        severity_to_lsp_severity(message.error.severity)
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

    let mut fixed_content = Vec::with_capacity(message.fixes.len());

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

#[cfg(test)]
#[expect(clippy::cast_possible_truncation)]
mod test {
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

    fn assert_position(source: &str, offset: u32, expected: (u32, u32)) {
        let position = offset_to_position(offset, source);
        assert_eq!(position.line, expected.0);
        assert_eq!(position.character, expected.1);
    }
}
