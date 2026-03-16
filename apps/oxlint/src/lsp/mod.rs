use oxc_linter::ExternalLinter;

#[cfg(feature = "napi")]
use crate::js_config::JsConfigLoaderCb;

mod code_actions;
mod commands;
mod error_with_position;
mod lsp_file_system;
mod options;
mod server_linter;
#[cfg(test)]
mod tester;
mod utils;

/// Run the language server
pub async fn run_lsp(
    external_linter: Option<ExternalLinter>,
    #[cfg(feature = "napi")] js_config_loader: Option<JsConfigLoaderCb>,
) {
    oxc_language_server::run_server(
        "oxlint".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
        vec![Box::new(crate::lsp::server_linter::ServerLinterBuilder::new(
            external_linter,
            #[cfg(feature = "napi")]
            js_config_loader,
        ))],
    )
    .await;
}
