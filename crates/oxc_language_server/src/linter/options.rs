use log::info;
use rustc_hash::{FxBuildHasher, FxHashMap};
use serde::{Deserialize, Deserializer, Serialize, de::Error};
use serde_json::Value;

use oxc_linter::FixKind;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum UnusedDisableDirectives {
    #[default]
    Allow,
    Warn,
    Deny,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum Run {
    OnSave,
    #[default]
    OnType,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LintOptions {
    pub run: Run, // TODO: the client wants maybe only the formatter, make it optional
    pub config_path: Option<String>,
    pub ts_config_path: Option<String>,
    pub unused_disable_directives: UnusedDisableDirectives,
    pub type_aware: bool,
    pub flags: FxHashMap<String, String>,
}

impl LintOptions {
    pub fn use_nested_configs(&self) -> bool {
        !self.flags.contains_key("disable_nested_config") && self.config_path.is_none()
    }

    pub fn fix_kind(&self) -> FixKind {
        self.flags.get("fix_kind").map_or(FixKind::SafeFix, |kind| match kind.as_str() {
            "safe_fix" => FixKind::SafeFix,
            "safe_fix_or_suggestion" => FixKind::SafeFixOrSuggestion,
            "dangerous_fix" => FixKind::DangerousFix,
            "dangerous_fix_or_suggestion" => FixKind::DangerousFixOrSuggestion,
            "none" => FixKind::None,
            "all" => FixKind::All,
            _ => {
                info!("invalid fix_kind flag `{kind}`, fallback to `safe_fix`");
                FixKind::SafeFix
            }
        })
    }
}

impl<'de> Deserialize<'de> for LintOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        LintOptions::try_from(value).map_err(Error::custom)
    }
}

impl TryFrom<Value> for LintOptions {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let Some(object) = value.as_object() else {
            return Err("no object passed".to_string());
        };

        let mut flags = FxHashMap::with_capacity_and_hasher(2, FxBuildHasher);
        if let Some(json_flags) = object.get("flags").and_then(|value| value.as_object()) {
            if let Some(disable_nested_config) =
                json_flags.get("disable_nested_config").and_then(|value| value.as_str())
            {
                flags
                    .insert("disable_nested_config".to_string(), disable_nested_config.to_string());
            }

            if let Some(fix_kind) = json_flags.get("fix_kind").and_then(|value| value.as_str()) {
                flags.insert("fix_kind".to_string(), fix_kind.to_string());
            }
        }

        Ok(Self {
            run: object
                .get("run")
                .map(|run| serde_json::from_value::<Run>(run.clone()).unwrap_or_default())
                .unwrap_or_default(),
            unused_disable_directives: object
                .get("unusedDisableDirectives")
                .map(|key| {
                    serde_json::from_value::<UnusedDisableDirectives>(key.clone())
                        .unwrap_or_default()
                })
                .unwrap_or_default(),
            config_path: object
                .get("configPath")
                .and_then(|config_path| serde_json::from_value::<String>(config_path.clone()).ok()),
            ts_config_path: object
                .get("tsConfigPath")
                .and_then(|config_path| serde_json::from_value::<String>(config_path.clone()).ok()),
            type_aware: object
                .get("typeAware")
                .is_some_and(|key| serde_json::from_value::<bool>(key.clone()).unwrap_or_default()),
            flags,
        })
    }
}

#[cfg(test)]
mod test {
    use rustc_hash::FxHashMap;
    use serde_json::json;

    use super::{LintOptions, Run, UnusedDisableDirectives};

    #[test]
    fn test_valid_options_json() {
        let json = json!({
            "run": "onSave",
            "configPath": "./custom.json",
            "unusedDisableDirectives": "warn",
            "typeAware": true,
            "flags": {
                "disable_nested_config": "true",
                "fix_kind": "dangerous_fix"
            }
        });

        let options = LintOptions::try_from(json).unwrap();
        assert_eq!(options.run, Run::OnSave);
        assert_eq!(options.config_path, Some("./custom.json".into()));
        assert_eq!(options.unused_disable_directives, UnusedDisableDirectives::Warn);
        assert!(options.type_aware);
        assert_eq!(options.flags.get("disable_nested_config"), Some(&"true".to_string()));
        assert_eq!(options.flags.get("fix_kind"), Some(&"dangerous_fix".to_string()));
    }

    #[test]
    fn test_empty_options_json() {
        let json = json!({});

        let options = LintOptions::try_from(json).unwrap();
        assert_eq!(options.run, Run::OnType);
        assert_eq!(options.config_path, None);
        assert_eq!(options.unused_disable_directives, UnusedDisableDirectives::Allow);
        assert!(!options.type_aware);
        assert!(options.flags.is_empty());
    }

    #[test]
    fn test_invalid_options_json() {
        let json = json!({
            "run": true,
            "configPath": "./custom.json"
        });

        let options = LintOptions::try_from(json).unwrap();
        assert_eq!(options.run, Run::OnType); // fallback
        assert_eq!(options.config_path, Some("./custom.json".into()));
        assert!(options.flags.is_empty());
    }

    #[test]
    fn test_invalid_flags_options_json() {
        let json = json!({
            "configPath": "./custom.json",
            "flags": {
                "disable_nested_config": true, // should be string
                "fix_kind": "dangerous_fix"
            }
        });

        let options = LintOptions::try_from(json).unwrap();
        assert_eq!(options.run, Run::OnType); // fallback
        assert_eq!(options.config_path, Some("./custom.json".into()));
        assert_eq!(options.flags.get("disable_nested_config"), None);
        assert_eq!(options.flags.get("fix_kind"), Some(&"dangerous_fix".to_string()));
    }

    #[test]
    fn test_use_nested_configs() {
        let options = LintOptions::default();
        assert!(options.use_nested_configs());

        let options =
            LintOptions { config_path: Some("config.json".to_string()), ..Default::default() };
        assert!(!options.use_nested_configs());

        let mut flags = FxHashMap::default();
        flags.insert("disable_nested_config".to_string(), "true".to_string());

        let options = LintOptions { flags, ..Default::default() };
        assert!(!options.use_nested_configs());
    }
}
