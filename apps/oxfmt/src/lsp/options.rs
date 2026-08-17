use serde::{Deserialize, Deserializer, Serialize, de::Error};
use serde_json::Value;

#[derive(Debug, Default, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FormatOptions {
    pub config_path: Option<String>,
    pub disable_nested_config: bool,
}

impl FormatOptions {
    /// `fmt.configPath` with the empty string treated as unset.
    pub fn explicit_config_path(&self) -> Option<&str> {
        self.config_path.as_deref().filter(|s| !s.is_empty())
    }

    /// Whether to search for nested config files per file.
    /// An explicit `fmt.configPath` takes absolute precedence,
    /// and `fmt.disableNestedConfig` opts out explicitly.
    pub fn use_nested_configs(&self) -> bool {
        !self.disable_nested_config && self.explicit_config_path().is_none()
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
            config_path: object.get("fmt.configPath").and_then(Value::as_str).map(str::to_owned),
            disable_nested_config: object
                .get("fmt.disableNestedConfig")
                .and_then(Value::as_bool)
                .unwrap_or(false),
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
        assert_eq!(options.config_path, Some(String::new()));
        assert!(options.explicit_config_path().is_none());
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

        // Empty `fmt.configPath` is treated as unset
        let options = FormatOptions { config_path: Some(String::new()), ..Default::default() };
        assert!(options.use_nested_configs());
    }
}
