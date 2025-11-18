#[cfg(test)]
mod test {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde_json::json;
    use tower_lsp_server::{UriExt, lsp_types::Uri};

    use super::super::server_linter::ServerLinterBuilder;

    #[test]
    fn test_new_with_js_plugins_in_config_does_not_crash() {
        // Create a unique temp directory
        let mut dir = std::env::temp_dir();
        let uniq = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        dir.push(format!("oxc_ls_test_{uniq}"));
        fs::create_dir_all(&dir).unwrap();

        // Write a config containing jsPlugins and a rule belonging to an external plugin
        let config_path = dir.join(".oxlintrc.json");
        let config = json!({
            "jsPlugins": ["custom-plugin"],
            "rules": {
                "custom/some-rule": ["warn", {"flag": true}]
            }
        });
        fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();

        // Initialize language server linter over the directory; should not panic/crash
        let uri = Uri::from_file_path(PathBuf::from(&dir)).unwrap();
        let _server = ServerLinterBuilder::build(&uri, json!({}));

        // Cleanup best-effort
        let _ = fs::remove_file(config_path);
        let _ = fs::remove_dir(dir);
    }
}
