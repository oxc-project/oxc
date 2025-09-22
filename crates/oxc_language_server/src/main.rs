use rustc_hash::FxBuildHasher;
use tower_lsp_server::{LspService, Server};

mod backend;
mod capabilities;
mod code_actions;
mod commands;
mod file_system;
mod formatter;
mod linter;
mod options;
#[cfg(test)]
mod tester;
mod worker;

use crate::backend::Backend;

type ConcurrentHashMap<K, V> = papaya::HashMap<K, V, FxBuildHasher>;

const OXC_CONFIG_FILE: &str = ".oxlintrc.json";

// max range for LSP integer is 2^31 - 1
// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#baseTypes
const LSP_MAX_INT: u32 = 2u32.pow(31) - 1;

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new).finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
