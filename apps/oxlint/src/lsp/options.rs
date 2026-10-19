use rustc_hash::{FxBuildHasher, FxHashMap};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de::Error};
use serde_json::Value;

use oxc_linter::{FixKind, normalize_rule_name};
use tracing::error;

use crate::config_loader::config_discovery;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum UnusedDisableDirectives {
    #[default]
    Allow,
    Warn,
    Deny,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Run {
    OnSave,
    #[default]
    OnType,
}

/// LSP Options
///
/// These options can be defined for each workspace folder separately.
/// File references in the options (e.g. `configPath`, `tsConfigPath`) are resolved relative to the workspace folder.
///
/// They can be sent by the client in `initialize` or `workspace/didChangeConfiguration` requests.
/// If the client supports `workspace/configuration`, the server will request the options from the client.
///
/// Example of `initialize` request:
/// ```json
/// {
///   "processId": 123,
///   "rootUri": null,
///   "workspaceFolders": [],
///   "capabilities": {},
///   "initializationOptions": [
///     {
///       "workspaceUri": "file:///home/user/project",
///       "options": {
///         "unusedDisableDirectives": "deny",
///         "typeAware": true
///       }
///     }
///   ]
/// }
/// ```
///
/// Example of `workspace/didChangeConfiguration` request:
/// ```json
/// {
///   "settings": [
///     {
///       "workspaceUri": "file:///home/user/project",
///       "options": {
///         "unusedDisableDirectives": "deny",
///         "disableNestedConfig": true
///       }
///     }
///   ]
/// }
/// ```
#[derive(Debug, Default, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LintOptions {
    /// If your editor does not support `textDocument/diagnostic`,
    /// this option handles when diagnostics are sent to the client.
    ///
    /// This option only applies to clients which use the push model. Clients which support
    /// `textDocument/diagnostic` ask for diagnostics themselves (the pull model), so `run`
    /// currently has no effect for them.
    /// See [#26613](https://github.com/oxc-project/oxc/issues/26613)
    #[schemars(with = "Option<Run>")]
    pub run: Run,
    /// Path to the config file. Similar to `--config` CLI option.
    /// If set, it disables searching for config files.
    /// An empty string is treated as unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
    /// Path to the tsconfig file. Similar to `--tsconfig` CLI option.
    /// If set, it disables auto discovery for tsconfig files.
    /// An empty string is treated as unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts_config_path: Option<String>,
    /// How to handle unused disable directives. By default, they are allowed and ignored.
    pub unused_disable_directives: Option<UnusedDisableDirectives>,
    /// Whether to enable/disable type-aware linting.
    /// It will override the root config's `typeAware` option if set.
    pub type_aware: Option<bool>,
    /// Whether to disable nested config support. Similar to `--disable-nested-config` CLI option.
    /// It gets automatically enabled when `configPath` is set.
    /// Nested config support is always disabled in Vite+ mode.
    #[schemars(with = "Option<bool>")]
    pub disable_nested_config: bool,
    /// What kind of fixes to generate for code actions.
    #[schemars(with = "Option<LintFixKindFlag>")]
    pub fix_kind: LintFixKindFlag,
    /// Customization for rules, allows to override the linter's diagnostics and autofix.
    /// Keys may contain `*` to match any sequence of characters, such as `perfectionist/*`.
    /// Each property uses its most specific matching key: exact names first, then patterns with
    /// the most literal characters. Ties use the lexicographically last key.
    /// Example of lowering the severity of "no-unused-vars" rule to "hint" and disabling autofix for it:
    /// ```json
    /// {
    ///   "rulesCustomization": {
    ///     "no-unused-vars": {
    ///       "severity": "hint",
    ///       "autofix": false
    ///     }
    ///   }
    /// }
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules_customization: Option<RulesCustomization>,
}

#[derive(Debug, Default, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(transparent)]
pub struct RulesCustomization {
    #[serde(flatten)]
    pub rules: FxHashMap<String, RuleCustomization>,
}

impl RulesCustomization {
    pub(super) fn get<T>(
        &self,
        rule_name: &str,
        property: impl Fn(&RuleCustomization) -> Option<T>,
    ) -> Option<T> {
        self.rules.get(rule_name).and_then(&property).or_else(|| {
            self.rules
                .iter()
                .filter_map(|(pattern, customization)| {
                    let value = property(customization)?;
                    let matches = if let Some(pattern) = pattern.strip_prefix("eslint/") {
                        !rule_name.contains('/') && matches_rule_pattern(pattern, rule_name)
                    } else {
                        matches_rule_pattern(pattern, rule_name)
                    };
                    matches.then_some((pattern, value))
                })
                .max_by_key(|(pattern, _)| {
                    (pattern.bytes().filter(|&byte| byte != b'*').count(), *pattern)
                })
                .map(|(_, value)| value)
        })
    }
}

fn matches_rule_pattern(pattern: &str, mut rule_name: &str) -> bool {
    let mut parts = pattern.split('*');
    let prefix = parts.next().unwrap_or_default();
    let Some(rest) = rule_name.strip_prefix(prefix) else { return false };
    rule_name = rest;
    let Some(suffix) = parts.next_back() else { return rule_name.is_empty() };
    for part in parts {
        let Some(index) = rule_name.find(part) else { return false };
        rule_name = &rule_name[index + part.len()..];
    }
    rule_name.ends_with(suffix)
}

impl<'de> Deserialize<'de> for RulesCustomization {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let Some(object) = value.as_object() else {
            return Err(D::Error::custom("rulesCustomization must be an object"));
        };

        let mut rules = FxHashMap::with_capacity_and_hasher(object.len(), FxBuildHasher);
        for (rule_name, rule_config) in object {
            let Ok(customization) = RuleCustomization::deserialize(rule_config) else {
                error!("failed to deserialize customization for rule {rule_name}, skipping.");
                continue;
            };
            let mut normalized = normalize_rule_name(rule_name);
            // Preserve the ESLint namespace on patterns: `eslint/*` must not match plugin rules.
            if rule_name.contains('*') && rule_name.contains('/') && !normalized.contains('/') {
                normalized.insert_str(0, "eslint/");
            }
            rules.insert(normalized, customization);
        }

        Ok(Self { rules })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum RuleCustomizationSeverity {
    Error,
    Warn,
    Info,
    Hint,
    Off,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RuleCustomization {
    // note: this will not enable "off" rules; it only customizes active rules
    // it only modifies the severity of returned diagnostics, not the linter configuration
    pub severity: Option<RuleCustomizationSeverity>,
    // this flag indicates whether to include this rule in "fix all" and "fix all dangerous" code action
    pub autofix: Option<bool>,
}

#[derive(Debug, Default, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LintFixKindFlag {
    SafeFix,
    #[default]
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
    /// Nested config search: off with `disableNestedConfig`, an explicit `configPath`, or in Vite+ mode.
    pub fn use_nested_configs(&self) -> bool {
        !self.disable_nested_config
            && self.config_path.is_none()
            && config_discovery().nested_configs()
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
        // null is treated as default options
        if value == Value::Null {
            return Ok(Self::default());
        }

        let Some(object) = value.as_object() else {
            return Err("no object passed".to_string());
        };

        // deprecated flags field
        let mut flags = FxHashMap::with_capacity_and_hasher(2, FxBuildHasher);
        if let Some(json_flags) = object.get("flags").and_then(Value::as_object) {
            if let Some(disable_nested_config) =
                json_flags.get("disable_nested_config").and_then(Value::as_str)
            {
                flags.insert("disable_nested_config".to_string(), disable_nested_config);
            }

            if let Some(fix_kind) = json_flags.get("fix_kind").and_then(Value::as_str) {
                flags.insert("fix_kind".to_string(), fix_kind);
            }
        }
        Ok(Self {
            run: object.get("run").and_then(|run| Run::deserialize(run).ok()).unwrap_or_default(),
            unused_disable_directives: object
                .get("unusedDisableDirectives")
                .and_then(|key| UnusedDisableDirectives::deserialize(key).ok()),
            config_path: object
                .get("configPath")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_owned),
            ts_config_path: object
                .get("tsConfigPath")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_owned),
            type_aware: object.get("typeAware").and_then(Value::as_bool),
            disable_nested_config: object
                .get("disableNestedConfig")
                .and_then(Value::as_bool)
                .unwrap_or(flags.contains_key("disable_nested_config")),
            fix_kind: object
                .get("fixKind")
                .and_then(|key| LintFixKindFlag::deserialize(key).ok())
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
            rules_customization: object
                .get("rulesCustomization")
                .and_then(|key| RulesCustomization::deserialize(key).ok()),
        })
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::{
        LintOptions, RuleCustomizationSeverity, RulesCustomization, Run, UnusedDisableDirectives,
        matches_rule_pattern,
    };

    #[test]
    fn test_rule_patterns() {
        for (pattern, rule, expected) in [
            ("perfectionist/*", "perfectionist/sort-objects", true),
            ("perfectionist/*", "unicorn/prefer-set-has", false),
            ("*", "unicorn/prefer-set-has", true),
            ("*unused*", "typescript/no-unused-vars", true),
            ("no-*-vars", "no-unused-vars", true),
            ("no-*-vars", "no-unused-expressions", false),
            ("no-*", "no-", true),
            ("no-unused-vars", "no-unused-vars-extra", false),
            ("abc*bc", "abc", false),
            ("*a*b*", "ba", false),
            ("réact/*", "réact/rule", true),
        ] {
            assert_eq!(matches_rule_pattern(pattern, rule), expected, "{pattern}: {rule}");
        }
    }

    #[test]
    fn test_rule_pattern_precedence_and_aliases() {
        let rules: RulesCustomization = serde_json::from_value(json!({
            "*": { "severity": "hint", "autofix": false },
            "perfectionist/*": { "severity": "warn" },
            "perfectionist/sort-*": { "severity": "error" },
            "perfectionist/sort-objects": { "autofix": true },
            "@typescript-eslint/*": { "severity": "info" },
            "eslint/no-*": { "severity": "off" },
            "no-debugger": { "severity": "warn" }
        }))
        .unwrap();

        for (rule, expected) in [
            ("perfectionist/sort-objects", RuleCustomizationSeverity::Error),
            ("perfectionist/sort-arrays", RuleCustomizationSeverity::Error),
            ("typescript/no-unused-vars", RuleCustomizationSeverity::Info),
            ("no-console", RuleCustomizationSeverity::Off),
            ("no-debugger", RuleCustomizationSeverity::Warn),
            ("react/jsx-key", RuleCustomizationSeverity::Hint),
        ] {
            assert_eq!(rules.get(rule, |c| c.severity.clone()), Some(expected), "{rule}");
        }
        assert_eq!(rules.get("perfectionist/sort-objects", |c| c.autofix), Some(true));
        assert_eq!(rules.get("perfectionist/sort-arrays", |c| c.autofix), Some(false));
    }

    #[test]
    fn test_eslint_rule_pattern_scope() {
        let rules: RulesCustomization = serde_json::from_value(json!({
            "eslint/*": { "autofix": false }
        }))
        .unwrap();
        assert_eq!(rules.get("no-unused-vars", |c| c.autofix), Some(false));
        assert_eq!(rules.get("typescript/no-unused-vars", |c| c.autofix), None);
    }

    #[test]
    fn test_rule_pattern_ties_are_deterministic() {
        let rules: RulesCustomization = serde_json::from_value(json!({
            "no-*vars": { "autofix": false },
            "no-un*rs": { "autofix": true }
        }))
        .unwrap();
        assert_eq!(rules.get("no-unused-vars", |c| c.autofix), Some(true));
    }

    #[test]
    fn test_valid_options_json() {
        let json = json!({
            "run": "onSave",
            "configPath": "./custom.json",
            "unusedDisableDirectives": "warn",
            "typeAware": true,
            "disableNestedConfig": true,
            "fixKind": "dangerous_fix",
            "rulesCustomization": {
                "no-unused-vars": {
                    "severity": "error",
                    "autofix": true
                },
                "eqeqeq": {
                    "severity": "warn"
                }
            }
        });

        let options = LintOptions::try_from(json).unwrap();
        assert_eq!(options.run, Run::OnSave);
        assert_eq!(options.config_path, Some("./custom.json".into()));
        assert_eq!(options.unused_disable_directives, Some(UnusedDisableDirectives::Warn));
        assert_eq!(options.type_aware, Some(true));
        assert!(options.disable_nested_config);
        assert_eq!(options.fix_kind, super::LintFixKindFlag::DangerousFix);

        assert!(options.rules_customization.is_some());
        let rules_customization = options.rules_customization.unwrap();
        assert_eq!(rules_customization.rules.len(), 2);
        let no_unused_vars = &rules_customization.rules["no-unused-vars"];
        assert_eq!(no_unused_vars.severity, Some(RuleCustomizationSeverity::Error));
        assert_eq!(no_unused_vars.autofix, Some(true));
        let eqeqeq = &rules_customization.rules["eqeqeq"];
        assert_eq!(eqeqeq.severity, Some(RuleCustomizationSeverity::Warn));
        assert_eq!(eqeqeq.autofix, None);
    }

    #[test]
    fn test_rule_customization_normalizes_rule_name() {
        let options = LintOptions::try_from(json!({
            "rulesCustomization": {
                "eslint/no-unused-vars": {
                    "severity": "error"
                }
            }
        }))
        .unwrap();

        let rules_customization = options.rules_customization.unwrap();
        assert!(rules_customization.rules.contains_key("no-unused-vars"));
    }

    #[test]
    fn test_empty_path_options_json() {
        let json = json!({
            "configPath": "",
            "tsConfigPath": "",
        });

        let options = LintOptions::try_from(json).unwrap();
        assert_eq!(options.config_path, None);
        assert_eq!(options.ts_config_path, None);
    }

    #[test]
    fn test_empty_options_json() {
        let json = json!({});

        let options = LintOptions::try_from(json).unwrap();
        assert_eq!(options.run, Run::OnType);
        assert_eq!(options.config_path, None);
        assert_eq!(options.unused_disable_directives, None);
        assert_eq!(options.type_aware, None);
        assert!(!options.disable_nested_config);
        assert_eq!(options.fix_kind, super::LintFixKindFlag::SafeFixOrSuggestion);
        assert!(options.rules_customization.is_none());
    }

    #[test]
    fn test_null_json() {
        let json = json!(null);
        let options = LintOptions::try_from(json).unwrap();
        assert_eq!(options, LintOptions::default());
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
    fn test_invalid_rule_customization_json() {
        let json = json!({
            "rulesCustomization": {
                "valid-rule": {
                    "severity": "error",
                },
                "invalid-rule": {
                    "severity": "invalid_severity",
                }
            }
        });

        let options = LintOptions::try_from(json).unwrap();
        assert!(options.rules_customization.is_some());
        let rules_customization = options.rules_customization.unwrap();
        assert_eq!(rules_customization.rules.len(), 1);
        // rule settings are valid
        assert!(rules_customization.rules.contains_key("valid-rule"));
        // rule settings are invalid, should be skipped without affecting the deserialization of other rules
        assert!(!rules_customization.rules.contains_key("invalid-rule"));
    }

    #[test]
    fn test_null_options_json() {
        let json = json!({
            "configPath": null,
            "tsConfigPath": null,
            "typeAware": null,
            "unusedDisableDirectives": null
        });

        let options = LintOptions::try_from(json).unwrap();
        assert_eq!(options.type_aware, None); // null should be treated as None
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

        use crate::lsp::options::LintFixKindFlag;

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
            assert_eq!(options.unused_disable_directives, Some(UnusedDisableDirectives::Warn));
            assert_eq!(options.type_aware, Some(true));
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
