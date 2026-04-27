use std::{fmt::Write, sync::Arc};

use oxc_language_server::{WorkerManager, run_server};
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
    let version = {
        let mut version = env!("CARGO_PKG_VERSION").to_string();
        if let Some(vp_version) = crate::vp_version() {
            let _ = write!(version, " (VP: {})", vp_version.to_string_lossy());
        }
        version
    };
    run_server(
        "oxlint".to_string(),
        version,
        WorkerManager::new(Arc::new(crate::lsp::server_linter::ServerLinterBuilder::new(
            external_linter,
            #[cfg(feature = "napi")]
            js_config_loader,
        ))),
    )
    .await;
}
