use tower_lsp_server::lsp_types::{
    ClientCapabilities, OneOf, SaveOptions, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextDocumentSyncOptions, TextDocumentSyncSaveOptions,
    WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
};

#[derive(Default)]
pub struct Capabilities {
    pub workspace_apply_edit: bool,
    pub workspace_configuration: bool,
    pub dynamic_watchers: bool,
}

impl From<ClientCapabilities> for Capabilities {
    fn from(value: ClientCapabilities) -> Self {
        let workspace_apply_edit =
            value.workspace.as_ref().is_some_and(|workspace| workspace.apply_edit.is_some());
        let workspace_configuration = value
            .workspace
            .as_ref()
            .is_some_and(|workspace| workspace.configuration.is_some_and(|config| config));
        let dynamic_watchers = value.workspace.is_some_and(|workspace| {
            workspace.did_change_watched_files.is_some_and(|watched_files| {
                watched_files.dynamic_registration.is_some_and(|dynamic| dynamic)
            })
        });

        Self { workspace_apply_edit, workspace_configuration, dynamic_watchers }
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
    use tower_lsp_server::lsp_types::{
        ClientCapabilities, DidChangeWatchedFilesClientCapabilities, WorkspaceClientCapabilities,
    };

    use super::Capabilities;

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
