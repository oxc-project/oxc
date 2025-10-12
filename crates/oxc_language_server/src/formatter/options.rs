use serde::{Deserialize, Deserializer, Serialize, de::Error};
use serde_json::Value;

#[derive(Debug, Default, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FormatOptions {
    pub experimental: bool,
    pub config_path: Option<String>,
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
        let Some(object) = value.as_object() else {
            return Err("no object passed".to_string());
        };

        Ok(Self {
            experimental: object
                .get("fmt.experimental")
                .is_some_and(|run| serde_json::from_value::<bool>(run.clone()).unwrap_or_default()),
            config_path: object
                .get("fmt.configPath")
                .and_then(|config_path| serde_json::from_value::<String>(config_path.clone()).ok()),
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
            "fmt.experimental": true,
            "fmt.configPath": "./.oxfmtrc.json"
        });

        let options = FormatOptions::try_from(json).unwrap();
        assert!(options.experimental);
        assert_eq!(options.config_path.unwrap(), "./.oxfmtrc.json");
    }

    #[test]
    fn test_empty_options_json() {
        let json = json!({});

        let options = FormatOptions::try_from(json).unwrap();
        assert!(!options.experimental);
        assert!(options.config_path.is_none());
    }

    #[test]
    fn test_invalid_options_json() {
        let json = json!({
            "fmt.experimental": "what", // should be bool
            "fmt.configPath": "./.oxfmtrc.json"
        });

        let options = FormatOptions::try_from(json).unwrap();
        assert!(!options.experimental);
        assert_eq!(options.config_path.unwrap(), "./.oxfmtrc.json");
    }
}
