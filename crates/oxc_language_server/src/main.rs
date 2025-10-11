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
mod utils;
mod worker;

use crate::backend::Backend;

type ConcurrentHashMap<K, V> = papaya::HashMap<K, V, FxBuildHasher>;

const LINT_CONFIG_FILE: &str = ".oxlintrc.json";
const FORMAT_CONFIG_FILE: &str = ".oxfmtrc.json";

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new).finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
