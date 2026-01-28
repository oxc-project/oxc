mod code_actions;
mod commands;
mod config_walker;
mod error_with_position;
mod lsp_file_system;
mod options;
mod server_linter;
#[cfg(test)]
mod tester;
mod utils;

/// Run the language server
pub async fn run_lsp(external_linter: Option<oxc_linter::ExternalLinter>) {
    oxc_language_server::run_server(
        "oxlint".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
        vec![Box::new(crate::lsp::server_linter::ServerLinterBuilder::new(external_linter))],
    )
    .await;
}
