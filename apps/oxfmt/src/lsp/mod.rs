use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use oxc_language_server::{LanguageId, run_server};
use tower_lsp_server::ls_types::Uri;

use crate::core::{ExternalFormatter, JsConfigLoaderCb};

mod options;
mod server_formatter;

fn get_file_extension_from_language_id(language_id: &LanguageId) -> Option<&'static str> {
    match language_id.as_str() {
        "javascript" => Some("js"),
        "typescript" => Some("ts"),
        "javascriptreact" => Some("jsx"),
        "typescriptreact" => Some("tsx"),
        "toml" => Some("toml"),
        "css" => Some("css"),
        "graphql" => Some("graphql"),
        "handlebars" => Some("handlebars"),
        "json" => Some("json"),
        "jsonc" => Some("jsonc"),
        "json5" => Some("json5"),
        "markdown" => Some("md"),
        "mdx" => Some("mdx"),
        "mjml" => Some("mjml"),
        "html" => Some("html"),
        "scss" => Some("scss"),
        "less" => Some("less"),
        "vue" => Some("vue"),
        "yaml" => Some("yaml"),
        "angular" => Some("component.html"),
        _ => None,
    }
}

pub fn create_fake_file_path_from_language_id(
    language_id: &LanguageId,
    root: &Path,
    uri: &Uri,
) -> Option<PathBuf> {
    let file_extension = get_file_extension_from_language_id(language_id)?;
    let file_name = format!("{}.{}", uri.authority()?, file_extension);
    Some(root.join(file_name))
}

/// Run the language server
pub async fn run_lsp(js_config_loader: JsConfigLoaderCb, external_formatter: ExternalFormatter) {
    run_server(
        "oxfmt".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
        Arc::new(server_formatter::ServerFormatterBuilder::new(
            js_config_loader,
            external_formatter,
        )),
    )
    .await;
}
