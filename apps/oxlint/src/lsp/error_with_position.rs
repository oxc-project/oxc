use std::borrow::Cow;

use oxc_span::Span;
use tower_lsp_server::gen_lsp_types::{
    Code, CodeDescription, Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location,
    Message as LspMessage, Range, Uri,
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
    pub message: Cow<'static, str>,
    pub code: Cow<'static, str>,
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

    fn try_from(value: RuleCustomizationSeverity) -> Result<Self, &'static str> {
        match value {
            RuleCustomizationSeverity::Error => Ok(DiagnosticSeverity::Error),
            RuleCustomizationSeverity::Warn => Ok(DiagnosticSeverity::Warning),
            RuleCustomizationSeverity::Hint => Ok(DiagnosticSeverity::Hint),
            RuleCustomizationSeverity::Info => Ok(DiagnosticSeverity::Information),
            RuleCustomizationSeverity::Off => Err(
                "Off severity should not be converted to DiagnosticSeverity as it means the rule is disabled and should not produce diagnostics.",
            ),
        }
    }
}

fn severity_to_lsp_severity(value: Severity) -> DiagnosticSeverity {
    match value {
        Severity::Error => DiagnosticSeverity::Error,
        Severity::Warning => DiagnosticSeverity::Warning,
        Severity::Advice => DiagnosticSeverity::Hint,
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
                    let start_position = lsp_offset_to_position(source_text, offset);
                    let end_position = lsp_offset_to_position(source_text, offset + span.len());

                    DiagnosticRelatedInformation {
                        location: Location {
                            uri: uri.clone(),
                            range: Range::new(start_position, end_position),
                        },
                        message: span
                            .label()
                            .map_or_else(String::new, std::string::ToString::to_string),
                    }
                })
                .collect(),
        )
    };

    let start_position = lsp_offset_to_position(source_text, message.span.start);
    let end_position = lsp_offset_to_position(source_text, message.span.end);
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

    let diagnostic = Diagnostic {
        range,
        severity: Some(severity),
        code: Some(Code::String(code)),
        message: LspMessage::String(diagnostic_message),
        source: Some("oxc".into()),
        code_description,
        related_information,
        tags: None,
        data: None,
    };

    let mut fixed_content = Vec::with_capacity(message.fixes.len());

    // Convert PossibleFixes directly to FixedContent
    match message.fixes {
        PossibleFixes::None => {}
        PossibleFixes::Single(fix) => {
            fixed_content.push(fix_to_fixed_content(
                fix,
                source_text,
                FixedContentKind::LintRule(message.error.code.clone()),
            ));
        }
        PossibleFixes::Multiple(fixes) => {
            fixed_content.extend(fixes.into_iter().map(|fix| {
                fix_to_fixed_content(
                    fix,
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

fn fix_to_fixed_content(fix: Fix, source_text: &str, fix_kind: FixedContentKind) -> FixedContent {
    let start_position = lsp_offset_to_position(source_text, fix.span.start);
    let end_position = lsp_offset_to_position(source_text, fix.span.end);

    let message = fix.message.unwrap_or_else(|| {
        let rule_name = match &fix_kind {
            FixedContentKind::LintRule(code) => {
                get_full_rule_name(code).unwrap_or(Cow::Borrowed("this"))
            }
            FixedContentKind::UnusedDirective => Cow::Borrowed("this"),
        };
        Cow::Owned(format!("Fix {rule_name} problem"))
    });

    FixedContent {
        message,
        code: fix.content,
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
            location: Location { uri: uri.clone(), range: d.diagnostic.range },
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
                    severity: Some(DiagnosticSeverity::Hint),
                    code: None,
                    message: LspMessage::String(r.message.clone()),
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
        DiagnosticSeverity::Error
    } else {
        DiagnosticSeverity::Warning
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
                    Some(Fix::delete(fix_span).with_message(fix_message)),
                ));
            }
            RuleCommentType::Single(rules) => {
                for rule in rules {
                    reports.push(build_unused_disable_diagnostic_report(
                        rule.directive_prefix.unused_disable_rule_message(&rule.rule_name),
                        rule.name_span,
                        severity,
                        source_text,
                        Some(rule.create_fix(source_text, span).with_message(fix_message)),
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
    fix: Option<Fix>,
) -> DiagnosticReport {
    let start_position = lsp_offset_to_position(source_text, span.start);
    let end_position = lsp_offset_to_position(source_text, span.end);
    let range = Range::new(start_position, end_position);

    DiagnosticReport {
        diagnostic: Diagnostic {
            range,
            severity: Some(severity),
            code: Some("".into()),
            message: LspMessage::String(message),
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
