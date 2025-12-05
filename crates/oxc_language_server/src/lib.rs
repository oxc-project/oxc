use rustc_hash::FxBuildHasher;
use tower_lsp_server::{Client, LspService, Server, ls_types::ServerInfo};

mod backend;
mod capabilities;
mod file_system;
#[cfg(feature = "formatter")]
mod formatter;
#[cfg(feature = "linter")]
mod linter;
mod options;
#[cfg(test)]
mod tests;
mod tool;
mod utils;
mod worker;

use crate::backend::Backend;
#[cfg(feature = "formatter")]
pub use crate::formatter::ServerFormatterBuilder;
#[cfg(feature = "linter")]
pub use crate::linter::ServerLinterBuilder;
pub use crate::tool::{Tool, ToolBuilder, ToolRestartChanges, ToolShutdownChanges};

pub type ConcurrentHashMap<K, V> = papaya::HashMap<K, V, FxBuildHasher>;

#[cfg(feature = "benchmark")]
pub fn build_backend(
    client: Client,
    server_name: String,
    server_version: String,
    tools: Vec<Box<dyn ToolBuilder>>,
) -> Backend {
    Backend::new(client, ServerInfo { name: server_name, version: Some(server_version) }, tools)
}
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
        Backend::new(client, ServerInfo { name: server_name, version: Some(server_version) }, tools)
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
