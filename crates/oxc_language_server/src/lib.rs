use rustc_hash::FxBuildHasher;
use tower_lsp_server::{LspService, Server};

mod backend;
mod capabilities;
mod file_system;
#[cfg(feature = "formatter")]
mod formatter;
#[cfg(feature = "linter")]
mod linter;
mod options;
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

/// Run the language server
pub async fn run_server(tools: Vec<Box<dyn ToolBuilder>>) {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend::new(client, tools)).finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
