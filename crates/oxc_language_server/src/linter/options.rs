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
    pub disable_nested_config: bool,
    pub fix_kind: LintFixKindFlag,
}

#[derive(Debug, Default, Serialize, PartialEq, Eq, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum LintFixKindFlag {
    #[default]
    SafeFix,
    SafeFixOrSuggestion,
    DangerousFix,
    DangerousFixOrSuggestion,
    None,
    All,
}

impl From<LintFixKindFlag> for FixKind {
    fn from(flag: LintFixKindFlag) -> Self {
        match flag {
            LintFixKindFlag::SafeFix => FixKind::SafeFix,
            LintFixKindFlag::SafeFixOrSuggestion => FixKind::SafeFixOrSuggestion,
            LintFixKindFlag::DangerousFix => FixKind::DangerousFix,
            LintFixKindFlag::DangerousFixOrSuggestion => FixKind::DangerousFixOrSuggestion,
            LintFixKindFlag::None => FixKind::None,
            LintFixKindFlag::All => FixKind::All,
        }
    }
}

impl LintOptions {
    pub fn use_nested_configs(&self) -> bool {
        !self.disable_nested_config && self.config_path.is_none()
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

        // deprecated flags field
        let mut flags = FxHashMap::with_capacity_and_hasher(2, FxBuildHasher);
        if let Some(json_flags) = object.get("flags").and_then(|value| value.as_object()) {
            if let Some(disable_nested_config) =
                json_flags.get("disable_nested_config").and_then(|value| value.as_str())
            {
                flags.insert("disable_nested_config".to_string(), disable_nested_config);
            }

            if let Some(fix_kind) = json_flags.get("fix_kind").and_then(|value| value.as_str()) {
                flags.insert("fix_kind".to_string(), fix_kind);
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
            disable_nested_config: object
                .get("disableNestedConfig")
                .and_then(|key| serde_json::from_value::<bool>(key.clone()).ok())
                .unwrap_or(flags.contains_key("disable_nested_config")),
            fix_kind: object
                .get("fixKind")
                .and_then(|key| serde_json::from_value::<LintFixKindFlag>(key.clone()).ok())
                .unwrap_or_else(|| match flags.get("fix_kind") {
                    Some(&"safe_fix") => LintFixKindFlag::SafeFix,
                    Some(&"safe_fix_or_suggestion") => LintFixKindFlag::SafeFixOrSuggestion,
                    Some(&"dangerous_fix") => LintFixKindFlag::DangerousFix,
                    Some(&"dangerous_fix_or_suggestion") => {
                        LintFixKindFlag::DangerousFixOrSuggestion
                    }
                    Some(&"none") => LintFixKindFlag::None,
                    Some(&"all") => LintFixKindFlag::All,
                    _ => LintFixKindFlag::default(),
                }),
        })
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::{LintOptions, Run, UnusedDisableDirectives};

    #[test]
    fn test_valid_options_json() {
        let json = json!({
            "run": "onSave",
            "configPath": "./custom.json",
            "unusedDisableDirectives": "warn",
            "typeAware": true,
            "disableNestedConfig": true,
            "fixKind": "dangerous_fix"
        });

        let options = LintOptions::try_from(json).unwrap();
        assert_eq!(options.run, Run::OnSave);
        assert_eq!(options.config_path, Some("./custom.json".into()));
        assert_eq!(options.unused_disable_directives, UnusedDisableDirectives::Warn);
        assert!(options.type_aware);
        assert!(options.disable_nested_config);
        assert_eq!(options.fix_kind, super::LintFixKindFlag::DangerousFix);
    }

    #[test]
    fn test_empty_options_json() {
        let json = json!({});

        let options = LintOptions::try_from(json).unwrap();
        assert_eq!(options.run, Run::OnType);
        assert_eq!(options.config_path, None);
        assert_eq!(options.unused_disable_directives, UnusedDisableDirectives::Allow);
        assert!(!options.type_aware);
        assert!(!options.disable_nested_config);
        assert_eq!(options.fix_kind, super::LintFixKindFlag::SafeFix);
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
    }

    #[test]
    fn test_use_nested_configs() {
        let options = LintOptions::default();
        assert!(options.use_nested_configs());

        let options =
            LintOptions { config_path: Some("config.json".to_string()), ..Default::default() };
        assert!(!options.use_nested_configs());

        let options = LintOptions { disable_nested_config: true, ..Default::default() };
        assert!(!options.use_nested_configs());
    }

    mod deprecated_flags {
        use serde_json::json;

        use crate::linter::options::LintFixKindFlag;

        use super::{LintOptions, Run, UnusedDisableDirectives};

        #[test]
        fn test_valid_options_json_deprecated_flags() {
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
            assert!(options.disable_nested_config);
            assert_eq!(options.fix_kind, LintFixKindFlag::DangerousFix);
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
            assert!(!options.disable_nested_config); // fallback
            assert_eq!(options.fix_kind, LintFixKindFlag::DangerousFix);
        }

        #[test]
        fn test_root_options_overrides_flags() {
            let json = json!({
                "disableNestedConfig": false,
                "fixKind": "safe_fix_or_suggestion",
                "flags": {
                    "disable_nested_config": "true",
                    "fix_kind": "dangerous_fix"
                }
            });

            let options = LintOptions::try_from(json).unwrap();
            assert!(!options.disable_nested_config); // root option takes precedence
            assert_eq!(options.fix_kind, LintFixKindFlag::SafeFixOrSuggestion);
        }
    }
}
