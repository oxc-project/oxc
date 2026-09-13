use std::{num::NonZero, sync::Arc};

use futures::future::BoxFuture;
use rustc_hash::FxBuildHasher;
use tower_lsp_server::ls_types::Uri;
use tower_lsp_server::{LspService, Server, ls_types::ServerInfo};

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
pub use crate::file_system::ResolvedPath;
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

/// Run the language server.
///
/// The future is type-erased to reduce binary size by preventing CLI and NAPI execution paths from
/// each generating a copy of the LSP server state machine.
pub fn run_server(
    server_name: String,
    server_version: String,
    worker_manager: WorkerManager,
) -> BoxFuture<'static, ()> {
    Box::pin(run_server_impl(server_name, server_version, worker_manager))
}

async fn run_server_impl(
    server_name: String,
    server_version: String,
    worker_manager: WorkerManager,
) {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| {
        crate::backend::Backend::new(
            client,
            ServerInfo { name: server_name, version: Some(server_version) },
            worker_manager,
        )
    })
    .finish();

    // Baseline concurrency level from `available_parallelism()`. On systems with 6+ threads, leave 2
    // threads for other work to avoid overwhelming the host (the minimum is clamped below).
    let current_threads = std::thread::available_parallelism().map_or(1, NonZero::get);
    let capped_threads = if current_threads >= 6 { current_threads - 2 } else { current_threads };
    // Ensure that the concurrency level is at least 4, defaulting to the old behavior if the system has fewer than 4 threads.
    // Server-Requests can trigger Client-Requests, which will fill up the thread pool quickly, so we want to ensure that we have enough threads to handle both.
    // https://github.com/ebkalderon/tower-lsp/blob/49e1ce54549d5efc53b75510517c2f0b86f5c827/src/transport.rs#L78-L80
    let level = std::cmp::max(4, capped_threads);

    Server::new(stdin, stdout, socket).concurrency_level(level).serve(service).await;
}
