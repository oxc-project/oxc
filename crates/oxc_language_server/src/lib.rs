use rustc_hash::FxBuildHasher;
use tower_lsp_server::{LspService, Server};

mod backend;
mod capabilities;
mod file_system;
mod formatter;
mod linter;
mod options;
mod utils;
mod worker;

pub use crate::backend::Backend;
pub use crate::linter::server_linter::ServerLinter;
pub use crate::worker::WorkspaceWorker;

pub type ConcurrentHashMap<K, V> = papaya::HashMap<K, V, FxBuildHasher>;

pub const LINT_CONFIG_FILE: &str = ".oxlintrc.json";
pub const FORMAT_CONFIG_FILES: &[&str; 2] = &[".oxfmtrc.json", ".oxfmtrc.jsonc"];

/// Run the language server
pub async fn run_server() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new).finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
