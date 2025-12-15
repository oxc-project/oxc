use oxc_language_server::{ServerFormatterBuilder, run_server};

/// Run the language server
pub async fn run_lsp() {
    run_server(
        "oxfmt".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
        vec![Box::new(ServerFormatterBuilder)],
    )
    .await;
}
