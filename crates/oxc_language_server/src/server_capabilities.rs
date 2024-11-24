use tower_lsp::lsp_types::{
    self, CodeActionKind, CodeActionOptions, CodeActionProviderCapability, DiagnosticOptions,
    DiagnosticServerCapabilities, OneOf, TextDocumentSyncCapability, TextDocumentSyncKind,
    WorkDoneProgressOptions, WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
};

pub struct ServerCapabilities {
    diagnostic_enabled: bool,
    code_action_enabled: bool,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self { diagnostic_enabled: false, code_action_enabled: false }
    }
}

impl From<lsp_types::ClientCapabilities> for ServerCapabilities {
    fn from(params: lsp_types::ClientCapabilities) -> Self {
        if params.text_document.is_none() {
            return Self::default();
        }

        let doc = params.text_document.unwrap();

        Self {
            diagnostic_enabled: doc.diagnostic.is_some(),
            code_action_enabled: doc.code_action.is_some_and(|code_action| {
                code_action.code_action_literal_support.is_some_and(|literal_support| {
                    !literal_support.code_action_kind.value_set.is_empty()
                })
            }),
        }
    }
}

impl Into<lsp_types::ServerCapabilities> for ServerCapabilities {
    fn into(self) -> lsp_types::ServerCapabilities {
        lsp_types::ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            workspace: Some(WorkspaceServerCapabilities {
                workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                    supported: Some(true),
                    change_notifications: Some(OneOf::Left(true)),
                }),
                file_operations: None,
            }),
            diagnostic_provider: if self.diagnostic_enabled {
                Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
                    identifier: Some("oxc".into()),
                    inter_file_dependencies: false,
                    workspace_diagnostics: false,
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }))
            } else {
                None
            },
            code_action_provider: if self.code_action_enabled {
                Some(CodeActionProviderCapability::Options(CodeActionOptions {
                    code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                    resolve_provider: None,
                }))
            } else {
                None
            },
            ..lsp_types::ServerCapabilities::default()
        }
    }
}
