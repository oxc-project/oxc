use serde::{Deserialize, Serialize};
use tower_lsp_server::lsp_types::Uri;

use crate::{formatter::options::FormatOptions, linter::options::LintOptions};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    #[serde(flatten)]
    pub lint: LintOptions,
    #[serde(flatten)]
    pub format: FormatOptions,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceOption {
    pub workspace_uri: Uri,
    pub options: Options,
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::linter::options::Run;

    use super::WorkspaceOption;

    #[test]
    fn test_invalid_workspace_options_json() {
        let json = json!([{
            "workspaceUri": "file:///root/",
            "options": {
                "run": true,
                "configPath": "./custom.json",
            }
        }]);

        let workspace = serde_json::from_value::<Vec<WorkspaceOption>>(json).unwrap();

        assert_eq!(workspace.len(), 1);
        assert_eq!(workspace[0].workspace_uri.path().as_str(), "/root/");

        let options = &workspace[0].options;
        assert_eq!(options.lint.run, Run::OnType); // fallback
        assert_eq!(options.lint.config_path, Some("./custom.json".into()));
        assert!(options.lint.flags.is_empty());
    }
}
