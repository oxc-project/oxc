/// Run the language server
pub async fn run_lsp(external_linter: Option<oxc_linter::ExternalLinter>) {
    oxc_language_server::run_server(vec![Box::new(oxc_language_server::ServerLinterBuilder::new(
        external_linter,
    ))])
    .await;
}
