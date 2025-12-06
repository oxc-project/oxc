use rustc_hash::FxBuildHasher;
use tower_lsp_server::{LspService, Server, lsp_types::ServerInfo};

mod backend;
mod capabilities;
mod file_system;
#[cfg(feature = "formatter")]
mod formatter;
#[cfg(feature = "linter")]
mod linter;
mod options;
#[cfg(any(test, feature = "benchmark"))]
mod tests;
mod tool;
mod utils;
mod worker;

pub use crate::backend::Backend;
#[cfg(feature = "formatter")]
pub use crate::formatter::ServerFormatterBuilder;
#[cfg(feature = "linter")]
pub use crate::linter::ServerLinterBuilder;
#[cfg(feature = "benchmark")]
pub use crate::tests::*;
pub use crate::tool::{Tool, ToolBuilder, ToolRestartChanges, ToolShutdownChanges};

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
        Backend::new(client, ServerInfo { name: server_name, version: Some(server_version) }, tools)
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
