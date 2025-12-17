/// Run the language server
pub async fn run_lsp() {
    oxc_language_server::run_server(
        "oxlint".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
        vec![Box::new(oxc_language_server::ServerLinterBuilder)],
    )
    .await;
}
