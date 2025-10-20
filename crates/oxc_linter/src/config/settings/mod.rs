pub mod jsdoc;
mod jsx_a11y;
mod next;
mod react;
pub mod vitest;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use self::{
    jsdoc::JSDocPluginSettings, jsx_a11y::JSXA11yPluginSettings, next::NextPluginSettings,
    react::ReactPluginSettings, vitest::VitestPluginSettings,
};

/// # Oxlint Plugin Settings
///
/// Configure the behavior of linter plugins.
///
/// Here's an example if you're using Next.js in a monorepo:
///
/// ```json
/// {
///   "settings": {
///     "next": {
///       "rootDir": "apps/dashboard/"
///     },
///     "react": {
///       "linkComponents": [
///         { "name": "Link", "linkAttribute": "to" }
///       ]
///     },
///     "jsx-a11y": {
///       "components": {
///         "Link": "a",
///         "Button": "button"
///       }
///     }
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Default, JsonSchema, PartialEq)]
pub struct OxlintSettings {
    #[serde(skip)]
    pub json: Option<OxlintSettingsJson>,

    #[serde(default)]
    #[serde(rename = "jsx-a11y")]
    pub jsx_a11y: JSXA11yPluginSettings,

    #[serde(default)]
    pub next: NextPluginSettings,

    #[serde(default)]
    pub react: ReactPluginSettings,

    #[serde(default)]
    pub jsdoc: JSDocPluginSettings,

    #[serde(default)]
    pub vitest: VitestPluginSettings,
}

pub type OxlintSettingsJson = serde_json::Map<String, serde_json::Value>;

impl<'de> Deserialize<'de> for OxlintSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Create a temporary struct without the value field for deserialization
        #[derive(serde::Deserialize)]
        struct WellKnownOxlintSettings {
            #[serde(default)]
            #[serde(rename = "jsx-a11y")]
            jsx_a11y: JSXA11yPluginSettings,

            #[serde(default)]
            next: NextPluginSettings,

            #[serde(default)]
            react: ReactPluginSettings,

            #[serde(default)]
            jsdoc: JSDocPluginSettings,
        }

        // Capture the raw JSON value first
        let raw_value = serde_json::Value::deserialize(deserializer)?;

        let well_known_settings: WellKnownOxlintSettings =
            serde_json::from_value(raw_value.clone()).map_err(serde::de::Error::custom)?;

        let arbitrary_settings =
            if let serde_json::Value::Object(json) = raw_value { Some(json) } else { None };

        Ok(OxlintSettings {
            json: arbitrary_settings,
            jsx_a11y: well_known_settings.jsx_a11y,
            next: well_known_settings.next,
            react: well_known_settings.react,
            jsdoc: well_known_settings.jsdoc,
        })
    }
}

impl OxlintSettings {
    /// Create a new OxlintSettings instance with both parsed settings and raw JSON.
    /// This allows external plugins to access unknown configuration fields.
    pub fn from_json_with_raw(json_value: &serde_json::Value) -> Result<Self, serde_json::Error> {
        let settings = Self::deserialize(json_value)?;
        Ok(settings)
    }

    pub fn override_settings(&self, settings_to_override: &mut OxlintSettings) {
        // TODO: we need to distinguish between overrides of well known settings being default vs actually being empty.
        // Switch to Option?
        settings_to_override.react = self.react.clone();
        settings_to_override.jsx_a11y = self.jsx_a11y.clone();
        settings_to_override.next = self.next.clone();
        settings_to_override.jsdoc = self.jsdoc.clone();

        if let Some(base_json) = &self.json {
            if let Some(override_json) = &mut settings_to_override.json {
                for (key, value) in base_json {
                    override_json[key] = value.clone();
                }
            } else {
                settings_to_override.json = Some(base_json.clone());
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use oxc_span::CompactStr;
    use serde::Deserialize;

    use super::OxlintSettings;
    use crate::config::settings::react::ComponentAttrs;

    fn as_attrs<S: Into<CompactStr>, I: IntoIterator<Item = S>>(
        attrs: I,
    ) -> ComponentAttrs<'static> {
        ComponentAttrs::from(Cow::Owned(attrs.into_iter().map(Into::into).collect()))
    }

    #[test]
    fn test_parse_settings() {
        let settings = OxlintSettings::deserialize(&serde_json::json!({
            "jsx-a11y": {
                "polymorphicPropName": "role",
                "components": {
                    "Link": "Anchor",
                    "Link2": "Anchor2"
                }
            },
            "next": {
                "rootDir": "app"
            },
            "react": {
                "formComponents": [
                    "CustomForm",
                    {"name": "SimpleForm", "formAttribute": "endpoint"},
                    {"name": "Form", "formAttribute": ["registerEndpoint", "loginEndpoint"]},
                ],
                "linkComponents": [
                    "Hyperlink",
                    {"name": "MyLink", "linkAttribute": "to"},
                    {"name": "Link", "linkAttribute": ["to", "href"]},
                ]
            }
        }))
        .unwrap();

        assert_eq!(settings.jsx_a11y.polymorphic_prop_name, Some("role".into()));
        assert_eq!(settings.jsx_a11y.components.get("Link"), Some(&"Anchor".into()));
        assert!(settings.next.get_root_dirs().contains(&"app".to_string()));
        assert_eq!(
            settings.react.get_form_component_attrs("CustomForm").unwrap(),
            as_attrs::<CompactStr, _>(vec![])
        );
        assert_eq!(
            settings.react.get_form_component_attrs("SimpleForm").unwrap(),
            as_attrs(["endpoint"])
        );
        assert_eq!(
            settings.react.get_form_component_attrs("Form").unwrap(),
            as_attrs(["registerEndpoint", "loginEndpoint"])
        );
        assert_eq!(
            settings.react.get_link_component_attrs("Link").unwrap(),
            as_attrs(["to", "href"])
        );
        assert_eq!(settings.react.get_link_component_attrs("Noop"), None);
    }

    #[test]
    fn test_parse_settings_default() {
        let settings = OxlintSettings::default();
        assert!(settings.jsx_a11y.polymorphic_prop_name.is_none());
        assert!(settings.jsx_a11y.components.is_empty());
        assert!(settings.jsx_a11y.attributes.is_empty());
    }

    #[test]
    fn test_parse_jsx_a11y_attributes() {
        let settings = OxlintSettings::deserialize(&serde_json::json!({
            "jsx-a11y": {
                "attributes": {
                    "for": ["htmlFor", "for"],
                    "class": ["className"]
                }
            }
        }))
        .unwrap();

        let for_attrs = &settings.jsx_a11y.attributes["for"];
        assert_eq!(for_attrs.len(), 2);
        assert_eq!(for_attrs[0], "htmlFor");
        assert_eq!(for_attrs[1], "for");

        let class_attrs = &settings.jsx_a11y.attributes["class"];
        assert_eq!(class_attrs.len(), 1);
        assert_eq!(class_attrs[0], "className");

        assert_eq!(settings.jsx_a11y.attributes.get("nonexistent"), None);
    }

    #[test]
    fn test_parse_jsx_a11y_attributes_empty() {
        let settings = OxlintSettings::deserialize(&serde_json::json!({
            "jsx-a11y": {
                "attributes": {}
            }
        }))
        .unwrap();

        assert!(settings.jsx_a11y.attributes.is_empty());
    }

    #[test]
    fn test_parse_settings_with_extra_fields_and_raw_json() {
        // Helper struct to demonstrate external plugin configuration parsing
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct CustomPluginConfig {
            enabled: bool,
            severity: String,
            config: CustomPluginInnerConfig,
        }

        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct CustomPluginInnerConfig {
            #[serde(rename = "maxDepth")]
            max_depth: i32,
            #[serde(rename = "ignorePatterns")]
            ignore_patterns: Vec<String>,
        }
        // Test JSON with both known plugin settings and unknown extra fields
        let json_value = serde_json::json!({
            "jsx-a11y": {
                "polymorphicPropName": "role",
                "components": {
                    "Link": "a",
                    "Button": "button"
                }
            },
            "react": {
                "linkComponents": [
                    { "name": "Link", "linkAttribute": "to" }
                ]
            },
            // Extra fields for external plugins
            "eslint-plugin-custom": {
                "enabled": true,
                "severity": "error",
                "config": {
                    "maxDepth": 3,
                    "ignorePatterns": ["*.test.js"]
                }
            },
            "typescript-plugin": {
                "strict": true,
                "parserOptions": {
                    "project": "./tsconfig.json"
                }
            },
            // Another unknown field at the root level
            "globalSetting": "value"
        });

        // Test the enhanced parsing that captures raw JSON
        let settings = OxlintSettings::from_json_with_raw(&json_value).unwrap();

        // Verify that known plugin settings are properly parsed
        assert_eq!(settings.jsx_a11y.polymorphic_prop_name, Some("role".into()));
        assert_eq!(settings.jsx_a11y.components.get("Link"), Some(&"a".into()));
        assert_eq!(settings.jsx_a11y.components.get("Button"), Some(&"button".into()));

        let link_attrs = settings.react.get_link_component_attrs("Link").unwrap();
        assert!(link_attrs.contains(&"to".into()));

        // Verify that raw JSON is captured
        assert!(settings.json.is_some());
        let raw_json = settings.json.unwrap();

        // Verify known fields are present in raw JSON
        assert_eq!(raw_json["jsx-a11y"]["polymorphicPropName"], "role");
        assert_eq!(raw_json["react"]["linkComponents"][0]["name"], "Link");

        // Verify unknown fields are preserved in raw JSON
        assert_eq!(raw_json["eslint-plugin-custom"]["enabled"], true);
        assert_eq!(raw_json["eslint-plugin-custom"]["severity"], "error");
        assert_eq!(raw_json["eslint-plugin-custom"]["config"]["maxDepth"], 3);
        assert_eq!(raw_json["eslint-plugin-custom"]["config"]["ignorePatterns"][0], "*.test.js");

        assert_eq!(raw_json["typescript-plugin"]["strict"], true);
        assert_eq!(raw_json["typescript-plugin"]["parserOptions"]["project"], "./tsconfig.json");

        assert_eq!(raw_json["globalSetting"], "value");

        // Demonstrate how an external plugin would access its configuration
        if let Some(custom_config) = raw_json.get("eslint-plugin-custom") {
            let custom_plugin_config: CustomPluginConfig =
                serde_json::from_value(custom_config.clone()).unwrap();

            // Simulate external plugin processing
            assert!(custom_plugin_config.enabled);
            assert_eq!(custom_plugin_config.severity, "error");
            assert_eq!(custom_plugin_config.config.max_depth, 3);
            assert_eq!(custom_plugin_config.config.ignore_patterns, vec!["*.test.js".to_string()]);
        }
    }
}
