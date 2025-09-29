use log::debug;
use tower_lsp_server::lsp_types::{CodeAction, CodeActionKind, TextEdit, Uri, WorkspaceEdit};

use crate::linter::error_with_position::{DiagnosticReport, FixedContent, PossibleFixContent};

pub const CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC: CodeActionKind =
    CodeActionKind::new("source.fixAll.oxc");

fn fix_content_to_code_action(
    fixed_content: &FixedContent,
    uri: &Uri,
    alternative_message: &str,
    is_preferred: bool,
) -> CodeAction {
    // 1) Use `fixed_content.message` if it exists
    // 2) Try to parse the report diagnostic message
    // 3) Fallback to "Fix this problem"
    let title = match fixed_content.message.clone() {
        Some(msg) => msg,
        None => {
            if let Some(code) = alternative_message.split(':').next() {
                format!("Fix this {code} problem")
            } else {
                "Fix this problem".to_string()
            }
        }
    };

    CodeAction {
        title,
        kind: Some(CodeActionKind::QUICKFIX),
        is_preferred: Some(is_preferred),
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
    }
}

pub fn apply_fix_code_actions(report: &DiagnosticReport, uri: &Uri) -> Option<Vec<CodeAction>> {
    match &report.fixed_content {
        PossibleFixContent::None => None,
        PossibleFixContent::Single(fixed_content) => Some(vec![fix_content_to_code_action(
            fixed_content,
            uri,
            &report.diagnostic.message,
            true,
        )]),
        PossibleFixContent::Multiple(fixed_contents) => {
            // only the first code action is preferred
            let mut preferred = true;
            Some(
                fixed_contents
                    .iter()
                    .map(|fixed_content| {
                        let action = fix_content_to_code_action(
                            fixed_content,
                            uri,
                            &report.diagnostic.message,
                            preferred,
                        );
                        preferred = false;
                        action
                    })
                    .collect(),
            )
        }
    }
}

pub fn apply_all_fix_code_action<'a>(
    reports: impl Iterator<Item = &'a DiagnosticReport>,
    uri: &Uri,
) -> Option<CodeAction> {
    let quick_fixes: Vec<TextEdit> = fix_all_text_edit(reports);

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

/// Collect all text edits from the provided diagnostic reports, which can be applied at once.
/// This is useful for implementing a "fix all" code action / command that applies multiple fixes in one go.
pub fn fix_all_text_edit<'a>(reports: impl Iterator<Item = &'a DiagnosticReport>) -> Vec<TextEdit> {
    let mut text_edits: Vec<TextEdit> = vec![];

    for report in reports {
        let fix = match &report.fixed_content {
            PossibleFixContent::None => None,
            PossibleFixContent::Single(fixed_content) => Some(fixed_content),
            // For multiple fixes, we take the first one as a representative fix.
            // Applying all possible fixes at once is not possible in this context.
            PossibleFixContent::Multiple(multi) => {
                // for a real linter fix, we expect at least 3 fixes
                if multi.len() > 2 {
                    multi.first()
                } else {
                    debug!("Multiple fixes found, but only ignore fixes available");
                    #[cfg(debug_assertions)]
                    {
                        if !multi.is_empty() {
                            debug_assert!(multi[0].message.as_ref().is_some());
                            debug_assert!(
                                multi[0].message.as_ref().unwrap().starts_with("Disable")
                            );
                            debug_assert!(
                                multi[0].message.as_ref().unwrap().ends_with("for this line")
                            );
                        }
                    }

                    // this fix is only for "ignore this line/file" fixes
                    // do not apply them for "fix all" code action
                    None
                }
            }
        };

        if let Some(fixed_content) = &fix {
            // when source.fixAll.oxc we collect all changes at ones
            // and return them as one workspace edit.
            // it is possible that one fix will change the range for the next fix
            // see oxc-project/oxc#10422
            text_edits.push(TextEdit {
                range: fixed_content.range,
                new_text: fixed_content.code.clone(),
            });
        }
    }

    text_edits
}
