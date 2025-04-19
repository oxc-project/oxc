use tower_lsp_server::lsp_types::{
    CodeAction, CodeActionKind, Diagnostic, NumberOrString, Position, Range, TextEdit, Uri,
    WorkspaceEdit,
};

use crate::linter::error_with_position::DiagnosticReport;

pub const CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC: CodeActionKind =
    CodeActionKind::new("source.fixAll.oxc");

// TODO: Would be better if we had exact rule name from the diagnostic instead of having to parse it.
fn get_rule_name(diagnostic: &Diagnostic) -> Option<String> {
    if let Some(NumberOrString::String(code)) = &diagnostic.code {
        let open_paren = code.chars().position(|c| c == '(')?;
        let close_paren = code.chars().position(|c| c == ')')?;

        return Some(code[(open_paren + 1)..close_paren].to_string());
    }

    None
}

pub fn apply_fix_code_action(report: &DiagnosticReport, uri: &Uri) -> Option<CodeAction> {
    let Some(fixed_content) = &report.fixed_content else {
        return None;
    };

    // 1) Use `fixed_content.message` if it exists
    // 2) Try to parse the report diagnostic message
    // 3) Fallback to "Fix this problem"
    let title = match fixed_content.message.clone() {
        Some(msg) => msg,
        None => {
            if let Some(code) = report.diagnostic.message.split(':').next() {
                format!("Fix this {code} problem")
            } else {
                "Fix this problem".to_string()
            }
        }
    };

    Some(CodeAction {
        title,
        kind: Some(CodeActionKind::QUICKFIX),
        is_preferred: Some(true),
        edit: Some(WorkspaceEdit {
            #[expect(clippy::disallowed_types)]
            changes: Some(std::collections::HashMap::from([(
                uri.clone(),
                vec![TextEdit { range: fixed_content.range, new_text: fixed_content.code.clone() }],
            )])),
            ..WorkspaceEdit::default()
        }),
        disabled: None,
        data: None,
        diagnostics: None,
        command: None,
    })
}

pub fn apply_all_fix_code_action<'a>(
    reports: impl Iterator<Item = &'a DiagnosticReport>,
    uri: &Uri,
) -> Option<CodeAction> {
    let mut quick_fixes: Vec<TextEdit> = vec![];

    for report in reports {
        if let Some(fixed_content) = &report.fixed_content {
            // when source.fixAll.oxc we collect all changes at ones
            // and return them as one workspace edit.
            // it is possible that one fix will change the range for the next fix
            // see oxc-project/oxc#10422
            quick_fixes.push(TextEdit {
                range: fixed_content.range,
                new_text: fixed_content.code.clone(),
            });
        }
    }

    if quick_fixes.is_empty() {
        return None;
    }

    Some(CodeAction {
        title: "quick fix".to_string(),
        kind: Some(CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC),
        is_preferred: Some(true),
        edit: Some(WorkspaceEdit {
            #[expect(clippy::disallowed_types)]
            changes: Some(std::collections::HashMap::from([(uri.clone(), quick_fixes)])),
            ..WorkspaceEdit::default()
        }),
        disabled: None,
        data: None,
        diagnostics: None,
        command: None,
    })
}

pub fn ignore_this_line_code_action(report: &DiagnosticReport, uri: &Uri) -> CodeAction {
    let rule_name = get_rule_name(&report.diagnostic);

    // TODO: This CodeAction doesn't support disabling multiple rules by name for a given line.
    //  To do that, we need to read `report.diagnostic.range.start.line` and check if a disable comment already exists.
    //  If it does, it needs to be appended to instead of a completely new line inserted.
    CodeAction {
        title: rule_name.as_ref().map_or_else(
            || "Disable oxlint for this line".into(),
            |s| format!("Disable {s} for this line"),
        ),
        kind: Some(CodeActionKind::QUICKFIX),
        is_preferred: Some(false),
        edit: Some(WorkspaceEdit {
            #[expect(clippy::disallowed_types)]
            changes: Some(std::collections::HashMap::from([(
                uri.clone(),
                vec![TextEdit {
                    range: Range {
                        start: Position {
                            line: report.diagnostic.range.start.line,
                            // TODO: character should be set to match the first non-whitespace character in the source text to match the existing indentation.
                            character: 0,
                        },
                        end: Position {
                            line: report.diagnostic.range.start.line,
                            // TODO: character should be set to match the first non-whitespace character in the source text to match the existing indentation.
                            character: 0,
                        },
                    },
                    new_text: rule_name.as_ref().map_or_else(
                        || "// eslint-disable-next-line\n".into(),
                        |s| format!("// eslint-disable-next-line {s}\n"),
                    ),
                }],
            )])),
            ..WorkspaceEdit::default()
        }),
        disabled: None,
        data: None,
        diagnostics: None,
        command: None,
    }
}

pub fn ignore_this_rule_code_action(report: &DiagnosticReport, uri: &Uri) -> CodeAction {
    let rule_name = get_rule_name(&report.diagnostic);

    CodeAction {
        title: rule_name.as_ref().map_or_else(
            || "Disable oxlint for this file".into(),
            |s| format!("Disable {s} for this file"),
        ),
        kind: Some(CodeActionKind::QUICKFIX),
        is_preferred: Some(false),
        edit: Some(WorkspaceEdit {
            #[expect(clippy::disallowed_types)]
            changes: Some(std::collections::HashMap::from([(
                uri.clone(),
                vec![TextEdit {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 0 },
                    },
                    new_text: rule_name.as_ref().map_or_else(
                        || "// eslint-disable\n".into(),
                        |s| format!("// eslint-disable {s}\n"),
                    ),
                }],
            )])),
            ..WorkspaceEdit::default()
        }),
        disabled: None,
        data: None,
        diagnostics: None,
        command: None,
    }
}
