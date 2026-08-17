use std::sync::Arc;

use rustc_hash::FxBuildHasher;
use tower_lsp_server::{
    Client, LanguageServer, LspService, LspServiceBuilder, Server,
    ls_types::{
        ServerInfo, Uri,
        notification::{self, Notification as _},
        request::{self, Request as _},
    },
};

use crate::backend::Backend;

mod backend;
mod capabilities;
mod file_system;
mod language_id;
mod options;
mod position;
#[cfg(test)]
mod tests;
mod tool;
mod tool_params;
pub mod utils;
mod worker;
mod worker_manager;

pub use crate::capabilities::{Capabilities, DiagnosticMode};
pub use crate::language_id::LanguageId;
pub use crate::position::offset_to_position;
pub use crate::tool::{
    ClientMessage, DiagnosticResult, Tool, ToolBuildResult, ToolBuilder, ToolRestartChanges,
};
pub use crate::tool_params::CodeActionParams;
pub use crate::worker::WorkspaceWorker;
pub use crate::worker_manager::WorkerManager;

pub type ConcurrentHashMap<K, V> = papaya::HashMap<K, V, FxBuildHasher>;

#[derive(Debug)]
pub struct TextDocument<'a> {
    pub uri: &'a Uri,
    pub language_id: LanguageId,
    pub text: Option<Arc<str>>,
}

impl<'a> TextDocument<'a> {
    pub fn new(uri: &'a Uri, language_id: LanguageId, text: Option<Arc<str>>) -> Self {
        Self { uri, language_id, text }
    }
}

fn build_lsp_service<F>(init: F) -> LspServiceBuilder<Backend>
where
    F: FnOnce(Client) -> Backend,
{
    LspService::build_with_lifecycle_methods(init)
        .custom_method(
            notification::DidChangeConfiguration::METHOD,
            Backend::did_change_configuration,
        )
        .custom_method(
            notification::DidChangeWatchedFiles::METHOD,
            Backend::did_change_watched_files,
        )
        .custom_method(
            notification::DidChangeWorkspaceFolders::METHOD,
            Backend::did_change_workspace_folders,
        )
        .custom_method(notification::DidSaveTextDocument::METHOD, Backend::did_save)
        .custom_method(notification::DidChangeTextDocument::METHOD, Backend::did_change)
        .custom_method(notification::DidOpenTextDocument::METHOD, Backend::did_open)
        .custom_method(notification::DidCloseTextDocument::METHOD, Backend::did_close)
        .custom_method(request::CodeActionRequest::METHOD, Backend::code_action)
        .custom_method(request::ExecuteCommand::METHOD, Backend::execute_command)
        .custom_method(request::DocumentDiagnosticRequest::METHOD, Backend::diagnostic)
        .custom_method(request::Formatting::METHOD, Backend::formatting)
}

/// Run the language server
pub async fn run_server(
    server_name: String,
    server_version: String,
    worker_manager: WorkerManager,
) {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = build_lsp_service(|client| {
        Backend::new(
            client,
            ServerInfo { name: server_name, version: Some(server_version) },
            worker_manager,
        )
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
