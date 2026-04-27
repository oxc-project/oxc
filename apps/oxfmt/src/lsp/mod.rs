use std::{
    fmt::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use oxc_language_server::{LanguageId, run_server};
use tower_lsp_server::ls_types::Uri;

use crate::core::{ExternalFormatter, JsConfigLoaderCb, utils};

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
    // Use the authority (if available) or the last segment of the path as the file name, defaulting to "Untitled" if neither is available
    let mut name = uri.authority().map_or_else(
        || uri.path().rsplit_once('/').map_or_else(|| "Untitled", |(_, s)| s.as_str()),
        |s| s.as_str(),
    );
    // if the last character is `/`, the name will be empty, so we need to check for that as well
    if name.is_empty() {
        name = "Untitled";
    }

    let file_name = format!("{name}.{file_extension}");
    Some(root.join(file_name))
}

/// Run the language server
pub async fn run_lsp(js_config_loader: JsConfigLoaderCb, external_formatter: ExternalFormatter) {
    let version = {
        let mut version = env!("CARGO_PKG_VERSION").to_string();
        if let Some(vp_version) = utils::vp_version() {
            let _ = write!(version, " (VP: {})", vp_version.to_string_lossy());
        }
        version
    };

    run_server(
        "oxfmt".to_string(),
        version,
        Arc::new(server_formatter::ServerFormatterBuilder::new(
            js_config_loader,
            external_formatter,
        )),
    )
    .await;
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use oxc_language_server::LanguageId;
    use tower_lsp_server::ls_types::Uri;

    use crate::lsp::create_fake_file_path_from_language_id;

    #[test]
    fn test_create_fake_file_path_from_language_id() {
        let language_id = LanguageId::new("jsonc".to_string());
        let root = std::env::temp_dir();

        let uri = Uri::from_str("vscode-userdata:/c%3A/Users/User/settings.json").unwrap();
        let result = create_fake_file_path_from_language_id(&language_id, &root, &uri).unwrap();
        assert!(result.extension().unwrap() == "jsonc");
        assert!(result.starts_with(&root));

        let uri = Uri::from_str("Untitled://Untitled-1").unwrap();
        let result = create_fake_file_path_from_language_id(&language_id, &root, &uri).unwrap();
        assert!(result.extension().unwrap() == "jsonc");
        assert!(result.starts_with(&root));
    }
}
