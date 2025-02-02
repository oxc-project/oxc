use tower_lsp::lsp_types::{
    ClientCapabilities, CodeActionKind, CodeActionOptions, CodeActionProviderCapability,
    ExecuteCommandOptions, OneOf, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, WorkDoneProgressOptions, WorkspaceFoldersServerCapabilities,
    WorkspaceServerCapabilities,
};

use crate::commands::LSP_COMMANDS;

pub const CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC: CodeActionKind =
    CodeActionKind::new("source.fixAll.oxc");

#[derive(Clone)]
pub struct Capabilities {
    pub code_action_provider: bool,
    pub workspace_edit: bool,
}

impl From<ClientCapabilities> for Capabilities {
    fn from(value: ClientCapabilities) -> Self {
        // check if the client support some code action literal support
        let code_action_provider = value.text_document.is_some_and(|capability| {
            capability.code_action.is_some_and(|code_action| {
                code_action.code_action_literal_support.is_some_and(|literal_support| {
                    !literal_support.code_action_kind.value_set.is_empty()
                })
            })
        });
        let workspace_edit =
            value.workspace.is_some_and(|capability| capability.workspace_edit.is_some());

        Self { code_action_provider, workspace_edit }
    }
}

impl From<Capabilities> for ServerCapabilities {
    fn from(value: Capabilities) -> Self {
        let commands = LSP_COMMANDS
            .iter()
            .filter_map(|c| match c.available(value.clone()) {
                true => Some(c.command_id()),
                false => None,
            })
            .collect();

        Self {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            workspace: Some(WorkspaceServerCapabilities {
                workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                    supported: Some(true),
                    change_notifications: Some(OneOf::Left(true)),
                }),
                file_operations: None,
            }),
            code_action_provider: if value.code_action_provider {
                Some(CodeActionProviderCapability::Options(CodeActionOptions {
                    code_action_kinds: Some(vec![
                        CodeActionKind::QUICKFIX,
                        CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC,
                    ]),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                    resolve_provider: None,
                }))
            } else {
                None
            },
            execute_command_provider: Some(ExecuteCommandOptions {
                commands,
                ..Default::default()
            }),
            ..ServerCapabilities::default()
        }
    }
}

#[cfg(test)]
mod test {
    use tower_lsp::lsp_types::{
        ClientCapabilities, CodeActionClientCapabilities, CodeActionKindLiteralSupport,
        CodeActionLiteralSupport, TextDocumentClientCapabilities, WorkspaceClientCapabilities,
        WorkspaceEditClientCapabilities,
    };

    use super::Capabilities;

    #[test]
    fn test_code_action_provider_vscode() {
        let client_capabilities = ClientCapabilities {
            text_document: Some(TextDocumentClientCapabilities {
                code_action: Some(CodeActionClientCapabilities {
                    code_action_literal_support: Some(CodeActionLiteralSupport {
                        code_action_kind: CodeActionKindLiteralSupport {
                            // this is from build (see help, about):
                            // Version: 1.95.3 (user setup)
                            // Commit: f1a4fb101478ce6ec82fe9627c43efbf9e98c813
                            value_set: vec![
                                #[allow(clippy::manual_string_new)]
                                "".into(),
                                "quickfix".into(),
                                "refactor".into(),
                                "refactor.extract".into(),
                                "refactor.inline".into(),
                                "refactor.rewrite".into(),
                                "source".into(),
                                "source.organizeImports".into(),
                            ],
                        },
                    }),
                    ..CodeActionClientCapabilities::default()
                }),
                ..TextDocumentClientCapabilities::default()
            }),
            workspace: Some(WorkspaceClientCapabilities {
                workspace_edit: Some(WorkspaceEditClientCapabilities {
                    document_changes: Some(true),
                    ..WorkspaceEditClientCapabilities::default()
                }),
                ..WorkspaceClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        };

        let capabilities = Capabilities::from(client_capabilities);

        assert!(capabilities.code_action_provider);
        assert!(capabilities.workspace_edit);
    }

    #[test]
    fn test_code_action_provider_intellij() {
        let client_capabilities = ClientCapabilities {
            text_document: Some(TextDocumentClientCapabilities {
                code_action: Some(CodeActionClientCapabilities {
                    code_action_literal_support: Some(CodeActionLiteralSupport {
                        code_action_kind: CodeActionKindLiteralSupport {
                            // this is from build (see help, about):
                            // Build #IU-243.22562.145, built on December 8, 2024
                            value_set: vec![
                                "quickfix".into(),
                                #[allow(clippy::manual_string_new)]
                                "".into(),
                                "source".into(),
                                "refactor".into(),
                            ],
                        },
                    }),
                    ..CodeActionClientCapabilities::default()
                }),
                ..TextDocumentClientCapabilities::default()
            }),
            workspace: Some(WorkspaceClientCapabilities {
                workspace_edit: Some(WorkspaceEditClientCapabilities {
                    document_changes: Some(true),
                    ..WorkspaceEditClientCapabilities::default()
                }),
                ..WorkspaceClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        };

        let capabilities = Capabilities::from(client_capabilities);

        assert!(capabilities.code_action_provider);
        assert!(capabilities.workspace_edit);
    }
}
