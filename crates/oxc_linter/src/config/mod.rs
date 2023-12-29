use std::{collections::HashSet, path::PathBuf};

pub mod errors;
use oxc_diagnostics::{Error, FailedToOpenFileError, Report};
use phf::{phf_map, Map};
use rustc_hash::FxHashMap;
use serde_json::Value;

use crate::{
    rules::{RuleEnum, RULES},
    AllowWarnDeny, JsxA11y, LintSettings,
};

use self::errors::{
    FailedToParseConfigError, FailedToParseConfigJsonError, FailedToParseConfigPropertyError,
    FailedToParseRuleValueError,
};

pub struct ESLintConfig {
    rules: std::vec::Vec<RuleEnum>,
    settings: LintSettings,
}

impl ESLintConfig {
    pub fn new(path: &PathBuf) -> Result<Self, Report> {
        let file = match std::fs::read_to_string(path) {
            Ok(file) => file,
            Err(e) => {
                return Err(FailedToParseConfigError(vec![Error::new(FailedToOpenFileError(
                    path.clone(),
                    e,
                ))])
                .into());
            }
        };

        let file = match serde_json::from_str::<serde_json::Value>(&file) {
            Ok(file) => file,
            Err(e) => {
                let guess = mime_guess::from_path(path);
                let err = match guess.first() {
                    // syntax error
                    Some(mime) if mime.subtype() == "json" => e.to_string(),
                    Some(_) => "only json configuration is supported".to_string(),
                    None => {
                        format!(
                            "{e}, if the configuration is not a json file, please use json instead."
                        )
                    }
                };
                return Err(FailedToParseConfigError(vec![Error::new(
                    FailedToParseConfigJsonError(path.clone(), err),
                )])
                .into());
            }
        };

        // See https://github.com/oxc-project/oxc/issues/1672
        let extends_hm: HashSet<&str> = HashSet::new();

        let roles_hm = match parse_rules(&file) {
            Ok(roles_hm) => roles_hm
                .into_iter()
                .map(|(plugin_name, rule_name, allow_warn_deny, config)| {
                    ((plugin_name, rule_name), (allow_warn_deny, config))
                })
                .collect::<std::collections::HashMap<_, _>>(),
            Err(e) => {
                return Err(e);
            }
        };

        let settings = parse_settings_from_root(&file);

        // `extends` provides the defaults
        // `rules` provides the overrides
        let rules = RULES.clone().into_iter().filter_map(|rule| {
            // Check if the extends set is empty or contains the plugin name
            let in_extends = extends_hm.contains(rule.plugin_name());

            // Check if there's a custom rule that explicitly handles this rule
            let (is_explicitly_handled, policy, config) =
                if let Some((policy, config)) = roles_hm.get(&(rule.plugin_name(), rule.name())) {
                    // Return true for handling, and also whether it's enabled or not
                    (true, *policy, config)
                } else {
                    // Not explicitly handled
                    (false, AllowWarnDeny::Allow, &None)
                };

            // The rule is included if it's in the extends set and not explicitly disabled,
            // or if it's explicitly enabled
            if (in_extends && !is_explicitly_handled) || policy.is_enabled() {
                Some(rule.read_json(config.clone()))
            } else {
                None
            }
        });

        Ok(Self { rules: rules.collect::<Vec<_>>(), settings })
    }

    pub fn into_rules(mut self) -> Self {
        self.rules.sort_unstable_by_key(RuleEnum::name);
        self
    }

    pub fn get_config(self) -> (std::vec::Vec<RuleEnum>, LintSettings) {
        (self.rules, self.settings)
    }
}

#[allow(unused)]
fn parse_extends(root_json: &Value) -> Result<Option<Vec<&'static str>>, Report> {
    let Some(extends) = root_json.get("extends") else {
        return Ok(None);
    };

    let extends_obj = match extends {
        Value::Array(v) => v,
        _ => {
            return Err(FailedToParseConfigPropertyError("extends", "Expected an array.").into());
        }
    };

    let extends_rule_groups = extends_obj
        .iter()
        .filter_map(|v| {
            let v = match v {
                Value::String(s) => s,
                _ => return None,
            };

            if let Some(m) = EXTENDS_MAP.get(v.as_str()) {
                return Some(*m);
            }

            None
        })
        .collect::<Vec<_>>();

    Ok(Some(extends_rule_groups))
}

#[allow(clippy::type_complexity)]
fn parse_rules(
    root_json: &Value,
) -> Result<Vec<(&str, &str, AllowWarnDeny, Option<Value>)>, Error> {
    let Value::Object(rules_object) = root_json else { return Ok(vec![]) };

    let Some(Value::Object(rules_object)) = rules_object.get("rules") else { return Ok(vec![]) };

    rules_object
        .iter()
        .map(|(key, value)| {
            let (plugin_name, name) = parse_rule_name(key);

            let (rule_severity, rule_config) = resolve_rule_value(value)?;

            Ok((plugin_name, name, rule_severity, rule_config))
        })
        .collect::<Result<Vec<_>, Error>>()
}

fn parse_settings_from_root(root_json: &Value) -> LintSettings {
    let Value::Object(root_object) = root_json else { return LintSettings::default() };

    let Some(settings_value) = root_object.get("settings") else { return LintSettings::default() };

    parse_settings(settings_value)
}

pub fn parse_settings(setting_value: &Value) -> LintSettings {
    if let Value::Object(settings_object) = setting_value {
        if let Some(Value::Object(jsx_a11y)) = settings_object.get("jsx-a11y") {
            let mut jsx_a11y_setting =
                JsxA11y { polymorphic_prop_name: None, components: FxHashMap::default() };

            if let Some(Value::Object(components)) = jsx_a11y.get("components") {
                let components_map: FxHashMap<String, String> = components
                    .iter()
                    .map(|(key, value)| (String::from(key), String::from(value.as_str().unwrap())))
                    .collect();

                jsx_a11y_setting.set_components(components_map);
            }

            if let Some(Value::String(polymorphic_prop_name)) = jsx_a11y.get("polymorphicPropName")
            {
                jsx_a11y_setting
                    .set_polymorphic_prop_name(Some(String::from(polymorphic_prop_name)));
            }

            return LintSettings { jsx_a11y: jsx_a11y_setting };
        }
    }

    LintSettings::default()
}

pub const EXTENDS_MAP: Map<&'static str, &'static str> = phf_map! {
    "eslint:recommended" => "eslint",
    "plugin:react/recommended" => "react",
    "plugin:@typescript-eslint/recommended" => "typescript",
    "plugin:react-hooks/recommended" => "react",
    "plugin:unicorn/recommended" => "unicorn",
    "plugin:jest/recommended" => "jest",
};

fn parse_rule_name(name: &str) -> (&str, &str) {
    if let Some((category, name)) = name.split_once('/') {
        let category = category.trim_start_matches('@');

        // if it matches typescript-eslint, map it to typescript
        let category = match category {
            "typescript-eslint" => "typescript",
            _ => category,
        };

        (category, name)
    } else {
        ("eslint", name)
    }
}

/// Resolves the level of a rule and its config
///
/// Three cases here
/// ```json
/// {
///     "rule": "off",
///     "rule": ["off", "config"],
///     "rule": ["off", "config1", "config2"],
/// }
/// ```
fn resolve_rule_value(value: &serde_json::Value) -> Result<(AllowWarnDeny, Option<Value>), Error> {
    if let Some(v) = value.as_str() {
        return Ok((AllowWarnDeny::try_from(v)?, None));
    }

    if let Some(v) = value.as_array() {
        let mut config = Vec::new();
        for item in v.iter().skip(1).take(2) {
            config.push(item.clone());
        }
        let config = if config.is_empty() { None } else { Some(Value::Array(config)) };
        if let Some(v_idx_0) = v.first() {
            return Ok((AllowWarnDeny::try_from(v_idx_0)?, config));
        }
    }

    Err(FailedToParseRuleValueError(value.to_string(), "Invalid rule value").into())
}

#[cfg(test)]
mod test {
    use super::parse_rules;
    use std::env;

    #[test]
    fn test_parse_rules() {
        let fixture_path = env::current_dir().unwrap().join("fixtures/eslint_config.json");
        let input = std::fs::read_to_string(fixture_path).unwrap();
        let file = serde_json::from_str::<serde_json::Value>(&input).unwrap();
        let rules = parse_rules(&file).unwrap();
        insta::assert_debug_snapshot!(rules);
    }
}
