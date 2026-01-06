use rustc_hash::FxBuildHasher;
use tower_lsp_server::{LspService, Server, ls_types::ServerInfo};

mod backend;
mod capabilities;
mod file_system;
mod options;
#[cfg(test)]
mod tests;
mod tool;
pub mod utils;
mod worker;

pub use crate::capabilities::Capabilities;
pub use crate::tool::{DiagnosticResult, Tool, ToolBuilder, ToolRestartChanges};

pub type ConcurrentHashMap<K, V> = papaya::HashMap<K, V, FxBuildHasher>;

/// Run the language server
pub async fn run_server(
    server_name: String,
    server_version: String,
    tools: Vec<Box<dyn ToolBuilder>>,
) {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| {
        crate::backend::Backend::new(
            client,
            ServerInfo { name: server_name, version: Some(server_version) },
            tools,
        )
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
