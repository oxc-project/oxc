use std::sync::Arc;

use oxc_language_server::{ExternalFormatterBridge, ServerFormatterBuilder, run_server};
use serde_json::Value;
use tokio::task::block_in_place;

use crate::core::{
    ExternalFormatter, JsFormatEmbeddedCb, JsFormatFileCb, JsInitExternalFormatterCb,
};

struct NapiExternalFormatterBridge {
    formatter: ExternalFormatter,
}

impl ExternalFormatterBridge for NapiExternalFormatterBridge {
    fn init(&self, num_threads: usize) -> Result<(), String> {
        block_in_place(|| self.formatter.init(num_threads).map(|_| ()))
    }

    fn format_file(
        &self,
        options: &Value,
        parser: &str,
        file: &str,
        code: &str,
    ) -> Result<String, String> {
        block_in_place(|| self.formatter.format_file(options, parser, file, code))
    }
}

/// Run the language server
pub async fn run_lsp(
    init_external_formatter_cb: JsInitExternalFormatterCb,
    format_embedded_cb: JsFormatEmbeddedCb,
    format_file_cb: JsFormatFileCb,
) {
    let external_formatter =
        ExternalFormatter::new(init_external_formatter_cb, format_embedded_cb, format_file_cb);
    let bridge = Arc::new(NapiExternalFormatterBridge { formatter: external_formatter });

    run_server(
        "oxfmt".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
        vec![Box::new(ServerFormatterBuilder::new(Some(bridge)))],
    )
    .await;
}
