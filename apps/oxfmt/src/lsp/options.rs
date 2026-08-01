use serde::{Deserialize, Deserializer, Serialize, de::Error};
use serde_json::Value;

#[derive(Debug, Default, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FormatOptions {
    pub config_path: Option<String>,
    pub language: Option<LspLanguage>,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum LspLanguage {
    EtsStatic,
}

impl LspLanguage {
    pub const fn explicit(self) -> oxc_span::ExplicitLanguage {
        match self {
            Self::EtsStatic => oxc_span::ExplicitLanguage::EtsStatic,
        }
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
            language: object
                .get("fmt.language")
                .and_then(Value::as_str)
                .map(|language| match language {
                    "ets-static" => Ok(LspLanguage::EtsStatic),
                    _ => Err(format!(
                        "Unknown language '{language}'. Supported explicit languages: ets-static."
                    )),
                })
                .transpose()?,
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
            "fmt.configPath": "./.oxfmtrc.json"
        });

        let options = FormatOptions::try_from(json).unwrap();
        assert_eq!(options.config_path.unwrap(), "./.oxfmtrc.json");
        assert!(options.language.is_none());
    }

    #[test]
    fn test_empty_options_json() {
        let json = json!({});

        let options = FormatOptions::try_from(json).unwrap();
        assert!(options.config_path.is_none());
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
            "fmt.configPath": true // should be a string
        });

        let options = FormatOptions::try_from(json).unwrap();
        assert!(options.config_path.is_none());
    }

    #[test]
    fn test_empty_string_config_path() {
        let json = json!({
            "fmt.configPath": ""
        });

        let options = FormatOptions::try_from(json).unwrap();
        assert_eq!(options.config_path, Some(String::new()));
    }

    #[test]
    fn test_static_ets_language() {
        let options = FormatOptions::try_from(json!({ "fmt.language": "ets-static" })).unwrap();
        assert_eq!(options.language, Some(super::LspLanguage::EtsStatic));
    }

    #[test]
    fn test_unknown_language() {
        let error = FormatOptions::try_from(json!({ "fmt.language": "ets" })).unwrap_err();
        assert!(error.contains("ets-static"));
    }
}
