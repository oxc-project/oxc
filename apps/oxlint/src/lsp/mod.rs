use oxc_linter::ExternalLinter;

#[cfg(feature = "napi")]
use crate::js_config::JsConfigLoaderCb;
use crate::lsp::server_linter::ServerLinterBuilder;

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
    let mut builder = ServerLinterBuilder::new(external_linter);
    #[cfg(feature = "napi")]
    if let Some(loader) = js_config_loader {
        builder = builder.with_js_config_loader(Some(loader));
    }
    run_lsp_with_builder(builder).await;
}

async fn run_lsp_with_builder(builder: crate::lsp::server_linter::ServerLinterBuilder) {
    oxc_language_server::run_server(
        "oxlint".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
        vec![Box::new(builder)],
    )
    .await;
}
