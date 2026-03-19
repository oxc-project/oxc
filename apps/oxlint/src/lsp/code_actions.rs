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
    fix_kind: FixKind,
) -> Option<CodeAction> {
    let quick_fixes: Vec<TextEdit> = fix_all_text_edit(actions, fix_kind);

    if quick_fixes.is_empty() {
        return None;
    }

    let title = if fix_kind.is_dangerous() {
        "fix all fixable oxlint issues"
    } else {
        "fix all safe fixable oxlint issues"
    };

    Some(CodeAction {
        title: title.to_string(),
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
pub fn fix_all_text_edit(
    actions: impl Iterator<Item = LinterCodeAction>,
    fix_kind: FixKind,
) -> Vec<TextEdit> {
    let mut text_edits: Vec<TextEdit> = vec![];

    for action in actions {
        // Applying all possible fixes at once is not possible in this context.
        // Search for the first "real" fix for the rule, and ignore the rest of the fixes for the same rule.
        let Some(fixed_content) = action.fixed_content.into_iter().find(|fixed| {
            matches!(fixed.lsp_kind, FixedContentKind::LintRule | FixedContentKind::UnusedDirective)
        }) else {
            continue;
        };

        if !(fix_kind | FixKind::SafeFixOrSuggestion).can_apply(fixed_content.kind) {
            debug!("Skipping fix for fix all action: {}", fixed_content.message);
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use tower_lsp_server::ls_types::{Position, Range};

    use super::*;
    use crate::lsp::error_with_position::FixedContentKind;

    fn make_action(kind: FixKind) -> LinterCodeAction {
        LinterCodeAction {
            range: Range::default(),
            fixed_content: vec![FixedContent {
                message: "remove unused import".to_string(),
                code: String::new(),
                range: Range::new(Position::new(0, 0), Position::new(0, 10)),
                kind,
                lsp_kind: FixedContentKind::LintRule,
            }],
        }
    }

    #[test]
    fn test_fix_all_text_edit_skips_dangerous_fix_by_default() {
        let text_edits = fix_all_text_edit(
            std::iter::once(make_action(FixKind::DangerousFix)),
            FixKind::SafeFixOrSuggestion,
        );
        assert!(
            text_edits.is_empty(),
            "dangerous fix should be skipped when fix_kind is SafeFixOrSuggestion (default)"
        );
    }

    #[test]
    fn test_fix_all_text_edit_includes_safe_fix() {
        let text_edits = fix_all_text_edit(
            std::iter::once(make_action(FixKind::SafeFix)),
            FixKind::SafeFixOrSuggestion,
        );
        assert!(!text_edits.is_empty(), "safe fix should be included");
    }

    #[test]
    fn test_fix_all_text_edit_with_dangerous_fix_kind_includes_dangerous_fix() {
        let text_edits = fix_all_text_edit(
            std::iter::once(make_action(FixKind::DangerousFix)),
            FixKind::DangerousFix,
        );
        assert!(
            !text_edits.is_empty(),
            "dangerous fix should be included when fix_kind is DangerousFix"
        );
    }

    #[test]
    fn test_fix_all_text_edit_with_none_still_includes_safe_fix() {
        let text_edits =
            fix_all_text_edit(std::iter::once(make_action(FixKind::SafeFix)), FixKind::None);
        assert!(
            !text_edits.is_empty(),
            "safe fix should still be included even when fix_kind is None"
        );
    }

    #[test]
    fn test_fix_all_text_edit_with_dangerous_fix_kind_also_includes_safe_fix() {
        let text_edits = fix_all_text_edit(
            std::iter::once(make_action(FixKind::SafeFix)),
            FixKind::DangerousFix,
        );
        assert!(
            !text_edits.is_empty(),
            "safe fix should also be included when fix_kind is DangerousFix"
        );
    }

    #[test]
    fn test_apply_all_fix_code_action_title_safe_by_default() {
        let action = apply_all_fix_code_action(
            std::iter::once(make_action(FixKind::SafeFix)),
            Uri::from_str("file:///test.js").unwrap(),
            FixKind::SafeFixOrSuggestion,
        )
        .unwrap();
        assert!(
            action.title.contains("safe"),
            "title should contain 'safe' when fix_kind is SafeFixOrSuggestion, got: {}",
            action.title
        );
    }

    #[test]
    fn test_apply_all_fix_code_action_title_dangerous_when_dangerous_fix_kind() {
        let action = apply_all_fix_code_action(
            std::iter::once(make_action(FixKind::DangerousFix)),
            Uri::from_str("file:///test.js").unwrap(),
            FixKind::DangerousFix,
        )
        .unwrap();
        assert!(
            !action.title.contains("safe"),
            "title should not contain 'safe' when fix_kind is DangerousFix, got: {}",
            action.title
        );
    }
}
