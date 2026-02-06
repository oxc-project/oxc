use oxc_linter::FixKind;
use tower_lsp_server::ls_types::{CodeAction, CodeActionKind, TextEdit, Uri, WorkspaceEdit};
use tracing::debug;

use crate::lsp::error_with_position::{FixedContent, FixedContentKind, LinterCodeAction};

pub const CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC: CodeActionKind =
    CodeActionKind::new("source.fixAll.oxc");

fn fix_content_to_code_action(
    fixed_content: FixedContent,
    uri: Uri,
    is_preferred: bool,
) -> CodeAction {
    CodeAction {
        title: fixed_content.message,
        kind: Some(CodeActionKind::QUICKFIX),
        is_preferred: Some(is_preferred),
        edit: Some(WorkspaceEdit {
            #[expect(clippy::disallowed_types)]
            changes: Some(std::collections::HashMap::from([(
                uri,
                vec![TextEdit { range: fixed_content.range, new_text: fixed_content.code }],
            )])),
            ..WorkspaceEdit::default()
        }),
        disabled: None,
        data: None,
        diagnostics: None,
        command: None,
    }
}

pub fn apply_fix_code_actions(action: LinterCodeAction, uri: &Uri) -> Vec<CodeAction> {
    let mut code_actions = vec![];

    let mut preferred_possible = true;
    for fixed in action.fixed_content {
        // only rule fixes and unused directive fixes can be preferred, ignore fixes are not preferred.
        let preferred = preferred_possible
            && matches!(
                fixed.lsp_kind,
                FixedContentKind::LintRule | FixedContentKind::UnusedDirective
            );
        if preferred {
            // only the first fix can be preferred, if there are multiple fixes available.
            preferred_possible = false;
        }
        let action = fix_content_to_code_action(fixed, uri.clone(), preferred);
        code_actions.push(action);
    }

    code_actions
}

pub fn apply_all_fix_code_action(
    actions: impl Iterator<Item = LinterCodeAction>,
    uri: Uri,
) -> Option<CodeAction> {
    let quick_fixes: Vec<TextEdit> = fix_all_text_edit(actions);

    if quick_fixes.is_empty() {
        return None;
    }

    Some(CodeAction {
        title: "quick fix".to_string(),
        kind: Some(CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC),
        is_preferred: Some(true),
        edit: Some(WorkspaceEdit {
            #[expect(clippy::disallowed_types)]
            changes: Some(std::collections::HashMap::from([(uri, quick_fixes)])),
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
pub fn fix_all_text_edit(actions: impl Iterator<Item = LinterCodeAction>) -> Vec<TextEdit> {
    let mut text_edits: Vec<TextEdit> = vec![];

    for action in actions {
        if action.fixed_content.is_empty() {
            continue;
        }

        // for a real linter fix, we expect at least 3 fixes
        if action.fixed_content.len() == 2 {
            debug!("Multiple fixes found, but only ignore fixes available");
            #[cfg(debug_assertions)]
            {
                debug_assert!(action.fixed_content[0].message.starts_with("Disable"));
                debug_assert!(action.fixed_content[0].message.ends_with("for this line"));
            }
            continue;
        }

        // For multiple fixes, we take the first one as a representative fix.
        // Applying all possible fixes at once is not possible in this context.
        let fixed_content = action.fixed_content.into_iter().next().unwrap();

        // Only safe fixes or suggestions are applied in "fix all" action/command.
        if !FixKind::SafeFixOrSuggestion.can_apply(fixed_content.kind) {
            debug!("Skipping unsafe fix for fix all action: {}", fixed_content.message);
            continue;
        }

        // when source.fixAll.oxc we collect all changes at ones
        // and return them as one workspace edit.
        // it is possible that one fix will change the range for the next fix
        // see oxc-project/oxc#10422
        text_edits.push(TextEdit { range: fixed_content.range, new_text: fixed_content.code });
    }

    text_edits
}
