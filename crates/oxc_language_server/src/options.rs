use serde::{Deserialize, Serialize};
use tower_lsp_server::lsp_types::Uri;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceOption {
    pub workspace_uri: Uri,
    pub options: serde_json::Value,
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::WorkspaceOption;

    #[test]
    fn test_workspace_options_json() {
        let json = json!([{
            "workspaceUri": "file:///root/",
            "options": {
                "run": true,
                "configPath": "./custom.json",
                "fmt.experimental": true
            }
        }]);

        let workspace = serde_json::from_value::<Vec<WorkspaceOption>>(json).unwrap();

        assert_eq!(workspace.len(), 1);
        assert_eq!(workspace[0].workspace_uri.path().as_str(), "/root/");

        let options = &workspace[0].options;
        assert_eq!(options["run"], true);
    }
}
