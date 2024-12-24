use tower_lsp::lsp_types::{
    ClientCapabilities, CodeActionKind, CodeActionOptions, CodeActionProviderCapability, OneOf,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, WorkDoneProgressOptions,
    WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
};

pub const CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC: CodeActionKind =
    CodeActionKind::new("source.fixAll.oxc");

pub struct Capabilities {
    pub code_action_provider: bool,
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

        Self { code_action_provider }
    }
}

impl From<Capabilities> for ServerCapabilities {
    fn from(value: Capabilities) -> Self {
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
            ..ServerCapabilities::default()
        }
    }
}

#[cfg(test)]
mod test {
    use tower_lsp::lsp_types::{
        ClientCapabilities, CodeActionClientCapabilities, CodeActionKindLiteralSupport,
        CodeActionLiteralSupport, TextDocumentClientCapabilities,
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
            ..ClientCapabilities::default()
        };

        let capabilities = Capabilities::from(client_capabilities);

        assert!(capabilities.code_action_provider);
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
            ..ClientCapabilities::default()
        };

        let capabilities = Capabilities::from(client_capabilities);

        assert!(capabilities.code_action_provider);
    }
}
