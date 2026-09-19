use serde::{Deserialize, Deserializer, Serialize, de::Error};
use serde_json::Value;

use oxc_language_server::WorkingDirectory;

#[derive(Debug, Default, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FormatOptions {
    /// An empty string is treated as unset.
    pub config_path: Option<String>,
    pub disable_nested_config: bool,
    /// Additional project roots below the workspace folder, each formatted as if it were its own
    /// workspace folder (like `eslint.workingDirectories`). Strings are paths or globs relative to
    /// the workspace folder; `[{ "mode": "auto" }]` detects directories containing both a
    /// `package.json` and an oxfmt config file. Handled by the language server, not by the tool.
    ///
    /// A directory listed explicitly is a working directory even when it is gitignored; the
    /// `auto` detection follows the `.gitignore` of the workspace folder instead.
    ///
    /// A working directory does not inherit the configuration of its workspace folder. It resolves
    /// its own config from its own root, the same way opening that directory as a workspace folder
    /// would.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub working_directories: Vec<WorkingDirectory>,
}

impl FormatOptions {
    /// Off with an explicit `fmt.configPath` or `fmt.disableNestedConfig`.
    pub fn use_nested_configs(&self) -> bool {
        !self.disable_nested_config && self.config_path.is_none()
    }

    /// Whether the formatter has to be rebuilt for these new options.
    ///
    /// `workingDirectories` is deliberately not compared: the language server owns it and rebuilds
    /// this formatter itself when the option *resolves* to a different set of roots. A change
    /// which only rewrites the entries resolves to the same roots and must not restart.
    pub fn needs_restart(&self, other: &Self) -> bool {
        self.config_path != other.config_path
            || self.disable_nested_config != other.disable_nested_config
    }
}

impl<'de> Deserialize<'de> for FormatOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        FormatOptions::try_from(value).map_err(Error::custom)
    }
}

impl TryFrom<Value> for FormatOptions {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        // null is treated as default options
        if value == Value::Null {
            return Ok(Self::default());
        }

        let Some(object) = value.as_object() else {
            return Err("no object passed".to_string());
        };

        Ok(Self {
            config_path: object
                .get("fmt.configPath")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_owned),
            disable_nested_config: object
                .get("fmt.disableNestedConfig")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            working_directories: object
                .get("workingDirectories")
                .and_then(|value| Vec::<WorkingDirectory>::deserialize(value).ok())
                .unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::FormatOptions;

    #[test]
    fn test_valid_options_json() {
        let json = json!({
            "fmt.configPath": "./.oxfmtrc.json",
            "fmt.disableNestedConfig": true
        });

        let options = FormatOptions::try_from(json).unwrap();
        assert_eq!(options.config_path.unwrap(), "./.oxfmtrc.json");
        assert!(options.disable_nested_config);
    }

    #[test]
    fn test_empty_options_json() {
        let json = json!({});

        let options = FormatOptions::try_from(json).unwrap();
        assert!(options.config_path.is_none());
        assert!(!options.disable_nested_config);
    }

    #[test]
    fn test_null_json() {
        let json = json!(null);
        let options = FormatOptions::try_from(json).unwrap();
        assert_eq!(options, FormatOptions::default());
    }

    #[test]
    fn test_invalid_options_json() {
        let json = json!({
            "fmt.configPath": true, // should be a string
            "fmt.disableNestedConfig": "true" // should be a boolean
        });

        let options = FormatOptions::try_from(json).unwrap();
        assert!(options.config_path.is_none());
        assert!(!options.disable_nested_config);
    }

    #[test]
    fn test_empty_string_config_path() {
        let json = json!({
            "fmt.configPath": ""
        });

        let options = FormatOptions::try_from(json).unwrap();
        assert_eq!(options.config_path, None);
    }

    #[test]
    fn test_use_nested_configs() {
        let options = FormatOptions::default();
        assert!(options.use_nested_configs());

        let options =
            FormatOptions { config_path: Some("config.json".into()), ..Default::default() };
        assert!(!options.use_nested_configs());

        let options = FormatOptions { disable_nested_config: true, ..Default::default() };
        assert!(!options.use_nested_configs());
    }
}
