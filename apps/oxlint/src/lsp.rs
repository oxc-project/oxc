/// Run the language server
pub async fn run_lsp() {
    oxc_language_server::run_server().await;
}
