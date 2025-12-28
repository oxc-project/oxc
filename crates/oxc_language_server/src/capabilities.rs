use tower_lsp_server::ls_types::{
    ClientCapabilities, OneOf, SaveOptions, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextDocumentSyncOptions, TextDocumentSyncSaveOptions,
    WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
};

#[derive(Default)]
pub struct Capabilities {
    pub workspace_apply_edit: bool,
    pub workspace_configuration: bool,
    pub dynamic_watchers: bool,
    pub show_message: bool,
    /// Whether the client supports pull diagnostics.
    pull_diagnostics: bool,
    /// Whether the client supports the `workspace/diagnostic/refresh` request.
    refresh_diagnostics: bool,
}

impl From<ClientCapabilities> for Capabilities {
    fn from(value: ClientCapabilities) -> Self {
        let workspace_apply_edit =
            value.workspace.as_ref().is_some_and(|workspace| workspace.apply_edit.is_some());

        let workspace_configuration = value
            .workspace
            .as_ref()
            .is_some_and(|workspace| workspace.configuration.is_some_and(|config| config));

        let dynamic_watchers = value.workspace.as_ref().is_some_and(|workspace| {
            workspace.did_change_watched_files.is_some_and(|watched_files| {
                watched_files.dynamic_registration.is_some_and(|dynamic| dynamic)
            })
        });

        let show_message =
            value.window.as_ref().is_some_and(|window| window.show_message.is_some());

        let pull_diagnostics = value
            .text_document
            .as_ref()
            .is_some_and(|text_document| text_document.diagnostic.is_some());

        let refresh_diagnostics = value.workspace.as_ref().is_some_and(|workspace| {
            workspace.diagnostics.as_ref().is_some_and(|diagnostics| {
                diagnostics.refresh_support.is_some_and(|refresh| refresh)
            })
        });

        Self {
            workspace_apply_edit,
            workspace_configuration,
            dynamic_watchers,
            show_message,
            pull_diagnostics,
            refresh_diagnostics,
        }
    }
}

impl Capabilities {
    /// The server supports pull and push diagnostics.
    /// Only use push diagnostics if the client does not support pull diagnostics,
    /// or we cannot ask the client to refresh diagnostics.
    pub fn use_push_diagnostics(&self) -> bool {
        !self.pull_diagnostics || !self.refresh_diagnostics
    }
}

pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
            change: Some(TextDocumentSyncKind::FULL),
            open_close: Some(true),
            save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                include_text: Some(false),
            })),
            ..Default::default()
        })),
        workspace: Some(WorkspaceServerCapabilities {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Left(true)),
            }),
            file_operations: None,
        }),
        ..ServerCapabilities::default()
    }
}

#[cfg(test)]
mod test {
    use tower_lsp_server::ls_types::{
        ClientCapabilities, DidChangeWatchedFilesClientCapabilities, WorkspaceClientCapabilities,
    };

    use super::Capabilities;

    #[test]
    fn test_use_push_diagnostics() {
        let capabilities = Capabilities {
            pull_diagnostics: true,
            refresh_diagnostics: true,
            ..Default::default()
        };
        assert!(!capabilities.use_push_diagnostics());

        let capabilities = Capabilities {
            pull_diagnostics: false,
            refresh_diagnostics: true,
            ..Default::default()
        };
        assert!(capabilities.use_push_diagnostics());

        let capabilities = Capabilities {
            pull_diagnostics: true,
            refresh_diagnostics: false,
            ..Default::default()
        };
        assert!(capabilities.use_push_diagnostics());

        let capabilities = Capabilities {
            pull_diagnostics: false,
            refresh_diagnostics: false,
            ..Default::default()
        };
        assert!(capabilities.use_push_diagnostics());
    }

    #[test]
    fn test_workspace_edit_nvim() {
        let client_capabilities = ClientCapabilities {
            workspace: Some(WorkspaceClientCapabilities {
                // Nvim 0.10.3
                apply_edit: Some(true),
                ..WorkspaceClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        };

        let capabilities = Capabilities::from(client_capabilities);

        assert!(capabilities.workspace_apply_edit);
    }

    #[test]
    fn test_dynamic_watchers_vscode() {
        let client_capabilities = ClientCapabilities {
            workspace: Some(WorkspaceClientCapabilities {
                did_change_watched_files: Some(DidChangeWatchedFilesClientCapabilities {
                    dynamic_registration: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let capabilities = Capabilities::from(client_capabilities);
        assert!(capabilities.dynamic_watchers);
    }

    #[test]
    fn test_dynamic_watchers_intellij() {
        let client_capabilities = ClientCapabilities {
            workspace: Some(WorkspaceClientCapabilities {
                did_change_watched_files: Some(DidChangeWatchedFilesClientCapabilities {
                    dynamic_registration: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let capabilities = Capabilities::from(client_capabilities);
        assert!(capabilities.dynamic_watchers);
    }
}
