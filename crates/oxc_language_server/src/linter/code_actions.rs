use log::debug;
use tower_lsp_server::lsp_types::{CodeAction, CodeActionKind, TextEdit, Uri, WorkspaceEdit};

use crate::linter::error_with_position::{FixedContent, LinterCodeAction};

pub const CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC: CodeActionKind =
    CodeActionKind::new("source.fixAll.oxc");

fn fix_content_to_code_action(
    fixed_content: &FixedContent,
    uri: &Uri,
    is_preferred: bool,
) -> CodeAction {
    CodeAction {
        title: fixed_content.message.clone(),
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

pub fn apply_fix_code_actions(report: &LinterCodeAction, uri: &Uri) -> Vec<CodeAction> {
    let mut code_actions = vec![];

    // only the first code action is preferred
    let mut preferred = true;
    for fixed in &report.fixed_content {
        let action = fix_content_to_code_action(fixed, uri, preferred);
        preferred = false;
        code_actions.push(action);
    }

    code_actions
}

pub fn apply_all_fix_code_action<'a>(
    reports: impl Iterator<Item = &'a LinterCodeAction>,
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
pub fn fix_all_text_edit<'a>(reports: impl Iterator<Item = &'a LinterCodeAction>) -> Vec<TextEdit> {
    let mut text_edits: Vec<TextEdit> = vec![];

    for report in reports {
        if report.fixed_content.is_empty() {
            continue;
        }

        // for a real linter fix, we expect at least 3 fixes
        if report.fixed_content.len() == 2 {
            debug!("Multiple fixes found, but only ignore fixes available");
            #[cfg(debug_assertions)]
            {
                debug_assert!(report.fixed_content[0].message.starts_with("Disable"));
                debug_assert!(report.fixed_content[0].message.ends_with("for this line"));
            }
            continue;
        }

        // For multiple fixes, we take the first one as a representative fix.
        // Applying all possible fixes at once is not possible in this context.
        let fixed_content = report.fixed_content.first().unwrap();
        // when source.fixAll.oxc we collect all changes at ones
        // and return them as one workspace edit.
        // it is possible that one fix will change the range for the next fix
        // see oxc-project/oxc#10422
        text_edits
            .push(TextEdit { range: fixed_content.range, new_text: fixed_content.code.clone() });
    }

    text_edits
}
