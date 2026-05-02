use oxc_linter::FixKind;
use tower_lsp_server::ls_types::{CodeAction, CodeActionKind, TextEdit, Uri, WorkspaceEdit};
use tracing::debug;

use crate::lsp::{
    error_with_position::{FixedContent, FixedContentKind, LinterCodeAction},
    utils::range_overlaps,
};

pub const CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC: CodeActionKind =
    CodeActionKind::new("source.fixAll.oxc");

/// A custom code action kind for dangerous fix-all operations.
/// This is intentionally outside the `source.fixAll.*` hierarchy because the LSP spec
/// restricts `source.fixAll` to safe fixes only.
pub const CODE_ACTION_KIND_SOURCE_FIX_ALL_DANGEROUS_OXC: CodeActionKind =
    CodeActionKind::new("source.fixAllDangerous.oxc");

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
        title: "fix all safe fixable oxlint issues".to_string(),
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

pub fn apply_dangerous_fix_code_action(
    actions: impl Iterator<Item = LinterCodeAction>,
    uri: Uri,
) -> Option<CodeAction> {
    let quick_fixes: Vec<TextEdit> = dangerous_fix_all_text_edit(actions);

    if quick_fixes.is_empty() {
        return None;
    }

    Some(CodeAction {
        title: "fix all fixable oxlint issues".to_string(),
        kind: Some(CODE_ACTION_KIND_SOURCE_FIX_ALL_DANGEROUS_OXC),
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

/// Collect safe text edits from the provided diagnostic reports, which can be applied at once.
pub fn fix_all_text_edit(actions: impl Iterator<Item = LinterCodeAction>) -> Vec<TextEdit> {
    let mut text_edits: Vec<TextEdit> = vec![];

    for action in actions {
        // Applying all possible fixes at once is not possible in this context.
        // Search for the first "real" fix for the rule, and ignore the rest of the fixes for the same rule.
        let Some(fixed_content) = action.fixed_content.into_iter().find(|fixed| {
            matches!(fixed.lsp_kind, FixedContentKind::LintRule | FixedContentKind::UnusedDirective)
        }) else {
            continue;
        };

        // Only safe fixes or suggestions are applied in "fix all" action/command.
        if !FixKind::SafeFixOrSuggestion.can_apply(fixed_content.kind) {
            debug!("Skipping unsafe fix for fix all action: {}", fixed_content.message);
            continue;
        }

        text_edits.push(TextEdit { range: fixed_content.range, new_text: fixed_content.code });
    }

    remove_overlapping_edits(text_edits)
}

fn dangerous_fix_all_text_edit(actions: impl Iterator<Item = LinterCodeAction>) -> Vec<TextEdit> {
    let mut text_edits: Vec<TextEdit> = vec![];

    for action in actions {
        let Some(fixed_content) = action.fixed_content.into_iter().find(|fixed| {
            matches!(fixed.lsp_kind, FixedContentKind::LintRule | FixedContentKind::UnusedDirective)
        }) else {
            continue;
        };

        text_edits.push(TextEdit { range: fixed_content.range, new_text: fixed_content.code });
    }

    remove_overlapping_edits(text_edits)
}

fn remove_overlapping_edits(mut edits: Vec<TextEdit>) -> Vec<TextEdit> {
    // LSP spec requires text edits in a WorkspaceEdit to be non-overlapping.
    // Sort by start position so adjacent fixes are applied in order, then drop
    // any edit whose range overlaps (or touches) the previous one via range_overlaps.
    edits.sort_unstable_by(|a, b| {
        a.range
            .start
            .line
            .cmp(&b.range.start.line)
            .then(a.range.start.character.cmp(&b.range.start.character))
    });

    let mut result: Vec<TextEdit> = Vec::with_capacity(edits.len());
    for edit in edits {
        if let Some(last) = result.last()
            && range_overlaps(last.range, edit.range)
        {
            debug!("Skipping overlapping fix at {:?}", edit.range);
            continue;
        }
        result.push(edit);
    }
    result
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use tower_lsp_server::ls_types::{Position, Range};

    use oxc_linter::FixKind;

    use super::*;
    use crate::lsp::error_with_position::{FixedContent, FixedContentKind, LinterCodeAction};

    fn make_ranged_action(
        start_line: u32,
        start_char: u32,
        end_line: u32,
        end_char: u32,
    ) -> LinterCodeAction {
        let range =
            Range::new(Position::new(start_line, start_char), Position::new(end_line, end_char));
        LinterCodeAction {
            range,
            fixed_content: vec![FixedContent {
                message: "fix".to_string(),
                code: String::new(),
                range,
                kind: FixKind::SafeFix,
                lsp_kind: FixedContentKind::LintRule,
            }],
        }
    }

    #[test]
    fn test_fix_all_text_edit_sorts_by_position() {
        // Edits provided in reverse order should be sorted by start position.
        let actions = vec![make_ranged_action(2, 0, 2, 5), make_ranged_action(0, 0, 0, 5)];
        let result = fix_all_text_edit(actions.into_iter());
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].range.start.line, 0);
        assert_eq!(result[1].range.start.line, 2);
    }

    #[test]
    fn test_fix_all_text_edit_drops_overlapping() {
        // The second edit overlaps the first; only the first should be kept.
        let actions = vec![make_ranged_action(0, 0, 0, 10), make_ranged_action(0, 5, 0, 15)];
        let result = fix_all_text_edit(actions.into_iter());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].range.end.character, 10);
    }

    #[test]
    fn test_fix_all_text_edit_drops_adjacent_edits() {
        // range_overlaps uses non-strict comparisons, so adjacent edits (where the end of one
        // equals the start of the next) are conservatively treated as overlapping and the second
        // is dropped. This is a safe trade-off: the fix can be applied on the next invocation,
        // and it avoids any risk of producing unexpected output for boundary-sharing edits.
        let actions = vec![make_ranged_action(0, 0, 0, 5), make_ranged_action(0, 5, 0, 10)];
        let result = fix_all_text_edit(actions.into_iter());
        assert_eq!(result.len(), 1);
    }

    fn make_fix_kind_action(kind: FixKind) -> LinterCodeAction {
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
    fn test_fix_all_text_edit_skips_dangerous_fix() {
        let text_edits =
            fix_all_text_edit(std::iter::once(make_fix_kind_action(FixKind::DangerousFix)));
        assert!(text_edits.is_empty(), "dangerous fix should always be skipped in safe fix-all");
    }

    #[test]
    fn test_fix_all_text_edit_includes_safe_fix() {
        let text_edits = fix_all_text_edit(std::iter::once(make_fix_kind_action(FixKind::SafeFix)));
        assert!(!text_edits.is_empty(), "safe fix should be included");
    }

    #[test]
    fn test_apply_all_fix_code_action_uses_safe_kind() {
        let action = apply_all_fix_code_action(
            std::iter::once(make_fix_kind_action(FixKind::SafeFix)),
            Uri::from_str("file:///test.js").unwrap(),
        )
        .unwrap();
        assert_eq!(action.kind, Some(CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC));
    }

    #[test]
    fn test_apply_dangerous_fix_code_action_uses_dangerous_kind() {
        let action = apply_dangerous_fix_code_action(
            std::iter::once(make_fix_kind_action(FixKind::DangerousFix)),
            Uri::from_str("file:///test.js").unwrap(),
        )
        .unwrap();
        assert_eq!(action.kind, Some(CODE_ACTION_KIND_SOURCE_FIX_ALL_DANGEROUS_OXC));
    }
}
