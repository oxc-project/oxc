use std::sync::Arc;

use rustc_hash::FxBuildHasher;
use tower_lsp_server::ls_types::Uri;
use tower_lsp_server::{LspService, Server, ls_types::ServerInfo};

mod backend;
mod capabilities;
mod file_system;
mod language_id;
mod options;
#[cfg(test)]
mod tests;
mod tool;
mod worker;
mod worker_manager;

pub use crate::capabilities::{Capabilities, DiagnosticMode};
pub use crate::language_id::LanguageId;
pub use crate::tool::{DiagnosticResult, Tool, ToolBuilder, ToolRestartChanges};

pub type ConcurrentHashMap<K, V> = papaya::HashMap<K, V, FxBuildHasher>;

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

/// Run the language server
pub async fn run_server(server_name: String, server_version: String, tool: Arc<dyn ToolBuilder>) {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| {
        crate::backend::Backend::new(
            client,
            ServerInfo { name: server_name, version: Some(server_version) },
            tool,
        )
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
