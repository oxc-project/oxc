use rustc_hash::FxBuildHasher;
use tower_lsp_server::{
    LanguageServer, LspService, Server, UriExt,
    lsp_types::{
        ClientCapabilities, DocumentFormattingClientCapabilities, InitializeParams,
        InitializedParams, TextDocumentClientCapabilities, Uri,
    },
};

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

use crate::{
    backend::Backend,
    linter::{
        options::LintOptions,
        server_linter::{ServerLinter, ServerLinterRun},
    },
};

type ConcurrentHashMap<K, V> = papaya::HashMap<K, V, FxBuildHasher>;

const OXC_CONFIG_FILE: &str = ".oxlintrc.json";

// max range for LSP integer is 2^31 - 1
// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#baseTypes
const LSP_MAX_INT: u32 = 2u32.pow(31) - 1;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[tokio::main]
async fn main() {
    let _profiler = dhat::Profiler::new_heap();

    let cwd = std::env::current_dir().expect("Failed to get current working directory");
    let fixture_path = cwd.join("crates/oxc_language_server/fixtures/linter/memory");
    let fake_file_path = fixture_path.join("file.ts");
    let uri = Uri::from_file_path(fake_file_path).unwrap();

    let server =
        ServerLinter::new(&Uri::from_file_path(fixture_path).unwrap(), &LintOptions::default());

    for _i in 0..100 {
        let _ = server.run_single(&uri, None, ServerLinterRun::OnType).await;
        server.remove_diagnostics(&uri);
    }
}
