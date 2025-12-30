use oxc_language_server::run_server;

mod options;
mod server_formatter;
#[cfg(test)]
mod tester;
const FORMAT_CONFIG_FILES: &[&str; 2] = &[".oxfmtrc.json", ".oxfmtrc.jsonc"];

/// Run the language server
pub async fn run_lsp() {
    run_server(
        "oxfmt".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
        vec![Box::new(server_formatter::ServerFormatterBuilder)],
    )
    .await;
}
