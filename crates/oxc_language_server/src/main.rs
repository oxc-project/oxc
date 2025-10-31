use oxc_language_server::{Backend, LintTool};
use tower_lsp_server::{LspService, Server};

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) =
        LspService::build(|client| Backend::new(client, vec![LintTool::new().into()])).finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
