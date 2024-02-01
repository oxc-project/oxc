mod env;
pub mod errors;
mod settings;

use std::path::Path;

use oxc_diagnostics::{Error, FailedToOpenFileError, Report};
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::{Map, Value};

use crate::{rules::RuleEnum, AllowWarnDeny};

pub use self::{
    env::ESLintEnv,
    settings::{ESLintSettings, JsxA11y, Nextjs},
};
use self::{
    errors::{
        FailedToParseConfigError, FailedToParseConfigJsonError, FailedToParseJsonc,
        FailedToParseRuleValueError,
    },
    settings::CustomComponents,
};

/// ESLint Config
/// <https://eslint.org/docs/latest/use/configure/configuration-files-new#configuration-objects>
pub struct ESLintConfig {
    rules: Vec<ESLintRuleConfig>,
    settings: ESLintSettings,
    env: ESLintEnv,
}

#[derive(Debug)]
pub struct ESLintRuleConfig {
    plugin_name: String,
    rule_name: String,
    severity: AllowWarnDeny,
    config: Option<serde_json::Value>,
}

impl ESLintConfig {
    pub fn new(path: &Path) -> Result<Self, Report> {
        let json = Self::read_json(path)?;
        let rules = parse_rules(&json)?;
        let settings = parse_settings_from_root(&json);
        let env = parse_env_from_root(&json);
        Ok(Self { rules, settings, env })
    }

    pub fn properties(self) -> (ESLintSettings, ESLintEnv) {
        (self.settings, self.env)
    }

    fn read_json(path: &Path) -> Result<serde_json::Value, Error> {
        let mut string = std::fs::read_to_string(path).map_err(|e| {
            FailedToParseConfigError(vec![Error::new(FailedToOpenFileError(path.to_path_buf(), e))])
        })?;

        // jsonc support
        json_strip_comments::strip(&mut string)
            .map_err(|_| FailedToParseJsonc(path.to_path_buf()))?;

        serde_json::from_str::<serde_json::Value>(&string).map_err(|err| {
            let guess = mime_guess::from_path(path);
            let err = match guess.first() {
                // syntax error
                Some(mime) if mime.subtype() == "json" => err.to_string(),
                Some(_) => "only json configuration is supported".to_string(),
                None => {
                    format!(
                        "{err}, if the configuration is not a json file, please use json instead."
                    )
                }
            };
            FailedToParseConfigError(vec![Error::new(FailedToParseConfigJsonError(
                path.to_path_buf(),
                err,
            ))])
            .into()
        })
    }

    #[allow(clippy::option_if_let_else)]
    pub fn override_rules(
        &self,
        rules_for_override: &mut FxHashSet<RuleEnum>,
        all_rules: &[RuleEnum],
    ) {
        use itertools::Itertools;
        let mut rules_to_replace = vec![];
        let mut rules_to_remove = vec![];

        // Rules can have the same name but different plugin names
        let lookup = self.rules.iter().into_group_map_by(|r| r.rule_name.as_str());

        for (name, rule_configs) in &lookup {
            match rule_configs.len() {
                0 => unreachable!(),
                1 => {
                    let rule_config = &rule_configs[0];
                    let rule_name = &rule_config.rule_name;
                    let plugin_name = &rule_config.plugin_name;
                    match rule_config.severity {
                        AllowWarnDeny::Warn | AllowWarnDeny::Deny => {
                            if let Some(rule) = all_rules
                                .iter()
                                .find(|r| r.name() == rule_name && r.plugin_name() == plugin_name)
                            {
                                rules_to_replace.push(rule.read_json(rule_config.config.clone()));
                            }
                        }
                        AllowWarnDeny::Allow => {
                            if let Some(rule) = rules_for_override
                                .iter()
                                .find(|r| r.name() == rule_name && r.plugin_name() == plugin_name)
                            {
                                rules_to_remove.push(rule.clone());
                            }
                        }
                    }
                }
                _ => {
                    // For overlapping rule names, use the "error" one
                    // "no-loss-of-precision": "off",
                    // "@typescript-eslint/no-loss-of-precision": "error"
                    if let Some(rule_config) =
                        rule_configs.iter().find(|r| r.severity.is_warn_deny())
                    {
                        if let Some(rule) = rules_for_override.iter().find(|r| r.name() == *name) {
                            rules_to_replace.push(rule.read_json(rule_config.config.clone()));
                        }
                    } else if rule_configs.iter().all(|r| r.severity.is_allow()) {
                        if let Some(rule) = rules_for_override.iter().find(|r| r.name() == *name) {
                            rules_to_remove.push(rule.clone());
                        }
                    }
                }
            }
        }

        for rule in rules_to_remove {
            rules_for_override.remove(&rule);
        }
        for rule in rules_to_replace {
            rules_for_override.replace(rule);
        }
    }
}

fn parse_rules(root_json: &Value) -> Result<Vec<ESLintRuleConfig>, Error> {
    let Value::Object(rules_object) = root_json else { return Ok(Vec::default()) };

    let Some(Value::Object(rules_object)) = rules_object.get("rules") else {
        return Ok(Vec::default());
    };

    rules_object
        .into_iter()
        .map(|(key, value)| {
            let (plugin_name, rule_name) = parse_rule_name(key);
            let (severity, config) = resolve_rule_value(value)?;
            Ok(ESLintRuleConfig {
                plugin_name: plugin_name.to_string(),
                rule_name: rule_name.to_string(),
                severity,
                config,
            })
        })
        .collect::<Result<Vec<_>, Error>>()
}

fn parse_settings_from_root(root_json: &Value) -> ESLintSettings {
    let Value::Object(root_object) = root_json else { return ESLintSettings::default() };

    let Some(settings_value) = root_object.get("settings") else {
        return ESLintSettings::default();
    };

    parse_settings(settings_value)
}

pub fn parse_settings(setting_value: &Value) -> ESLintSettings {
    if let Value::Object(settings_object) = setting_value {
        let mut jsx_a11y_setting = JsxA11y::new(None, FxHashMap::default());
        let mut nextjs_setting = Nextjs::new(vec![]);
        if let Some(Value::Object(jsx_a11y)) = settings_object.get("jsx-a11y") {
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
        }

        if let Some(Value::Object(nextjs)) = settings_object.get("next") {
            if let Some(Value::String(root_dir)) = nextjs.get("rootDir") {
                nextjs_setting.set_root_dir(vec![String::from(root_dir)]);
            }
            if let Some(Value::Array(root_dir)) = nextjs.get("rootDir") {
                nextjs_setting.set_root_dir(
                    root_dir.iter().map(|v| v.as_str().unwrap().to_string()).collect(),
                );
            }
        }

        let link_components_setting =
            parse_custom_components(settings_object, &CustomComponentEnum::LinkComponents);
        let form_components_setting =
            parse_custom_components(settings_object, &CustomComponentEnum::FormComponents);

        return ESLintSettings::new(
            jsx_a11y_setting,
            nextjs_setting,
            link_components_setting,
            form_components_setting,
        );
    }

    ESLintSettings::default()
}

enum CustomComponentEnum {
    LinkComponents,
    FormComponents,
}

fn parse_custom_components(
    settings_object: &Map<String, Value>,
    components_type: &CustomComponentEnum,
) -> CustomComponents {
    fn parse_obj(obj: &Map<String, Value>, attribute_name: &str, setting: &mut CustomComponents) {
        if let Some(Value::String(name)) = obj.get("name") {
            let mut arr: Vec<String> = vec![];
            if let Some(Value::String(attribute)) = obj.get(attribute_name) {
                arr.push(attribute.to_string());
            } else if let Some(Value::Array(attributes)) = obj.get(attribute_name) {
                for attribute in attributes {
                    if let Value::String(attribute) = attribute {
                        arr.push(attribute.to_string());
                    }
                }
            }
            setting.insert(name.to_string(), arr);
        }
    }

    fn parse_component(
        settings_object: &Map<String, Value>,
        component_name: &str,
        attribute_name: &str,
        setting: &mut CustomComponents,
    ) {
        match settings_object.get(component_name) {
            Some(Value::Array(component)) => {
                for component in component {
                    if let Value::String(name) = component {
                        setting.insert(name.to_string(), [].to_vec());
                        continue;
                    }
                    if let Value::Object(obj) = component {
                        parse_obj(obj, attribute_name, setting);
                    }
                }
            }
            Some(Value::Object(obj)) => {
                parse_obj(obj, attribute_name, setting);
            }
            _ => {}
        };
    }

    let mut setting: CustomComponents = FxHashMap::default();

    match components_type {
        CustomComponentEnum::FormComponents => {
            parse_component(settings_object, "formComponents", "formAttribute", &mut setting);
        }
        CustomComponentEnum::LinkComponents => {
            parse_component(settings_object, "linkComponents", "linkAttribute", &mut setting);
        }
    }
    setting
}

fn parse_env_from_root(root_json: &Value) -> ESLintEnv {
    let Value::Object(root_object) = root_json else { return ESLintEnv::default() };

    let Some(env_value) = root_object.get("env") else { return ESLintEnv::default() };

    let env_object = match env_value {
        Value::Object(env_object) => env_object,
        _ => return ESLintEnv::default(),
    };

    let mut result = vec![];
    for (k, v) in env_object {
        if let Value::Bool(v) = v {
            if *v {
                result.push(String::from(k));
            }
        }
    }

    ESLintEnv::new(result)
}

fn parse_rule_name(name: &str) -> (&str, &str) {
    if let Some((category, name)) = name.split_once('/') {
        let category = category.trim_start_matches('@');

        let category = match category {
            // if it matches typescript-eslint, map it to typescript
            "typescript-eslint" => "typescript",
            // plugin name in RuleEnum is in snake_case
            "jsx-a11y" => "jsx_a11y",
            "next" => "nextjs",
            _ => category,
        };

        // since next.js eslint rule starts with @next/next/
        let name = name.trim_start_matches("next/");

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
    use super::ESLintConfig;
    use std::env;

    #[test]
    fn test_parse_rules() {
        let fixture_path = env::current_dir().unwrap().join("fixtures/eslint_config.json");
        let config = ESLintConfig::new(&fixture_path).unwrap();
        assert!(!config.rules.is_empty());
    }
}
