use oxc_language_server::{ServerFormatterBuilder, run_server};

/// Run the language server
pub async fn run_lsp() {
    run_server(vec![Box::new(ServerFormatterBuilder)]).await;
}
