//! Bookkeeping for the pull diagnostics model (`textDocument/diagnostic`).

use std::hash::Hasher;

use rustc_hash::FxHasher;
use tower_lsp_server::ls_types::Uri;

use crate::{ConcurrentHashMap, ConcurrentHashSet, TextDocument};

/// The state needed to hand out diagnostic result ids and to answer a pull with `unchanged`.
///
/// In the pull model a full report may carry a `resultId`, which the client sends back as
/// `previousResultId` on its next request for the same document. When the server knows that the
/// diagnostics did not change, it answers with an `unchanged` report instead of computing them
/// again. That is worth doing here because a lint can spawn the slow type-aware `tsgolint` process.
///
/// To be able to answer without linting, a result id is derived from the *inputs* of a lint rather
/// than from its output (see [`DiagnosticState::result_id`]). An id that matches is therefore a
/// guarantee that the diagnostics are the same, and an input that changed conservatively produces a
/// new id.
#[derive(Debug, Default)]
pub struct DiagnosticState {
    /// The linter generation of every workspace root, bumped when that workspace's linter is
    /// rebuilt.
    ///
    /// A rebuilt linter can produce different diagnostics for text that did not change, but only
    /// for the documents of its own workspace, so the ids of the other workspaces stay valid.
    generations: ConcurrentHashMap<Uri, u64>,
    /// Documents that changed after the diagnostics handed to the client were computed.
    ///
    /// A change clears the code action cache of the tool and only a lint repopulates it, so such a
    /// document is linted on the next pull even when its text still hashes to the id the client
    /// sent back.
    changed: ConcurrentHashSet<Uri>,
}

impl DiagnosticState {
    /// The result id for the diagnostics of `document` as produced by the workspace rooted at
    /// `workspace_root`, or `None` when the document is not open.
    ///
    /// An unopened document is read from disk, where it can change without any notification, so no
    /// id is handed out for it and every pull lints it.
    pub fn result_id(&self, document: &TextDocument<'_>, workspace_root: &Uri) -> Option<String> {
        let text = document.text.as_deref()?;
        let mut hasher = FxHasher::default();
        hasher.write(text.as_bytes());
        hasher.write(workspace_root.as_str().as_bytes());
        hasher.write_u64(self.generation(workspace_root));
        Some(format!("{:016x}", hasher.finish()))
    }

    /// Whether `uri` changed after the last lint, so the next pull has to lint it again.
    pub fn is_changed(&self, uri: &Uri) -> bool {
        self.changed.pin().contains(uri)
    }

    /// Record that `uri` changed, so its diagnostics have to be computed again on the next pull.
    pub fn mark_changed(&self, uri: &Uri) {
        self.changed.pin().insert(uri.clone());
    }

    /// Record that the diagnostics of `uri` were computed from its current content.
    pub fn mark_linted(&self, uri: &Uri) {
        self.changed.pin().remove(uri);
    }

    /// Forget a document that is not open any more.
    pub fn remove_document(&self, uri: &Uri) {
        self.changed.pin().remove(uri);
    }

    /// The linter serving `workspace_root` was rebuilt, so every result id of its documents is
    /// stale.
    pub fn invalidate(&self, workspace_root: &Uri) {
        self.generations.pin().update_or_insert(
            workspace_root.clone(),
            |generation| generation + 1,
            1,
        );
    }

    fn generation(&self, workspace_root: &Uri) -> u64 {
        self.generations.pin().get(workspace_root).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::DiagnosticState;
    use crate::{LanguageId, TextDocument};

    fn uri(value: &str) -> tower_lsp_server::ls_types::Uri {
        value.parse().unwrap()
    }

    fn document<'a>(
        uri: &'a tower_lsp_server::ls_types::Uri,
        text: Option<&str>,
    ) -> TextDocument<'a> {
        TextDocument::new(uri, LanguageId::new("plaintext".to_string()), text.map(Into::into))
    }

    #[test]
    fn result_id_is_stable_for_unchanged_inputs() {
        let state = DiagnosticState::default();
        let file = uri("file:///workspace/file.ts");
        let root = uri("file:///workspace");

        let document = document(&file, Some("const a = 1;"));
        let first = state.result_id(&document, &root);
        assert!(first.is_some());
        assert_eq!(first, state.result_id(&document, &root));
    }

    #[test]
    fn result_id_changes_with_text_and_workspace_root() {
        let state = DiagnosticState::default();
        let file = uri("file:///workspace/file.ts");
        let root = uri("file:///workspace");
        let other_root = uri("file:///other");

        let id = state.result_id(&document(&file, Some("const a = 1;")), &root);
        assert_ne!(id, state.result_id(&document(&file, Some("const a = 2;")), &root));
        // The same text can produce different diagnostics under a different (nested) config.
        assert_ne!(id, state.result_id(&document(&file, Some("const a = 1;")), &other_root));
    }

    #[test]
    fn unopened_documents_have_no_result_id() {
        let state = DiagnosticState::default();
        let file = uri("file:///workspace/file.ts");
        assert_eq!(state.result_id(&document(&file, None), &uri("file:///workspace")), None);
    }

    #[test]
    fn invalidating_a_workspace_only_changes_its_own_ids() {
        let state = DiagnosticState::default();
        let file = uri("file:///workspace/file.ts");
        let other_file = uri("file:///other/file.ts");
        let root = uri("file:///workspace");
        let other_root = uri("file:///other");

        let id = state.result_id(&document(&file, Some("const a = 1;")), &root);
        let other_id = state.result_id(&document(&other_file, Some("const a = 1;")), &other_root);

        state.invalidate(&root);

        assert_ne!(id, state.result_id(&document(&file, Some("const a = 1;")), &root));
        assert_eq!(
            other_id,
            state.result_id(&document(&other_file, Some("const a = 1;")), &other_root)
        );
    }

    #[test]
    fn invalidate_bumps_an_unknown_workspace_from_zero() {
        let state = DiagnosticState::default();
        let root = uri("file:///workspace");

        // A workspace that was never invalidated has generation 0, and invalidating it has to change
        // the ids it serves.
        assert_eq!(state.generation(&root), 0);
        state.invalidate(&root);
        assert_eq!(state.generation(&root), 1);
    }

    #[test]
    fn changed_documents_are_tracked_until_they_are_linted() {
        let state = DiagnosticState::default();
        let file = uri("file:///workspace/file.ts");

        assert!(!state.is_changed(&file));
        state.mark_changed(&file);
        assert!(state.is_changed(&file));
        state.mark_linted(&file);
        assert!(!state.is_changed(&file));

        state.mark_changed(&file);
        state.remove_document(&file);
        assert!(!state.is_changed(&file));
    }
}
