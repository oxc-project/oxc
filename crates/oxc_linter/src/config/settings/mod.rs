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

#[derive(Deserialize, Default)]
// A private struct to deserialize well-known settings from parsed, merged, or extended JSON.
struct WellKnownOxlintSettings {
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
            vitest: well_known_settings.vitest,
        })
    }
}

impl OxlintSettings {
    /// Mutates `settings_to_override` by reading from `self`.
    pub fn override_settings(&self, settings_to_override: &mut OxlintSettings) {
        // If `None`, `self` has nothing configured, so we don't need to mutate `settings_to_override` at all.
        if let Some(self_json) = &self.json {
            if let Some(override_json) = &settings_to_override.json {
                let json = deep_merge(self_json, override_json);
                match serde_json::from_value::<WellKnownOxlintSettings>(serde_json::Value::Object(
                    json.clone(),
                )) {
                    Ok(well_known_settings) => {
                        settings_to_override.json = Some(json);
                        settings_to_override.jsx_a11y = well_known_settings.jsx_a11y;
                        settings_to_override.next = well_known_settings.next;
                        settings_to_override.react = well_known_settings.react;
                        settings_to_override.jsdoc = well_known_settings.jsdoc;
                        settings_to_override.vitest = well_known_settings.vitest;
                    }
                    Err(e) => {
                        panic!("Failed to parse override settings: {e:?}");
                    }
                }
            } else {
                settings_to_override.json = Some(self_json.clone());
                settings_to_override.jsx_a11y = self.jsx_a11y.clone();
                settings_to_override.next = self.next.clone();
                settings_to_override.react = self.react.clone();
                settings_to_override.jsdoc = self.jsdoc.clone();
                settings_to_override.vitest = self.vitest.clone();
            }
        }
    }

    pub fn merge(&self, other: &OxlintSettings) -> OxlintSettings {
        let json = match (&self.json, &other.json) {
            (Some(self_json), Some(other_json)) => Some(deep_merge(self_json, other_json)),
            (Some(self_json), None) => Some(self_json.clone()),
            (None, Some(other_json)) => Some(other_json.clone()),
            (None, None) => None,
        };

        let well_known_settings = match &json {
            Some(json) => serde_json::from_value::<WellKnownOxlintSettings>(
                serde_json::Value::Object(json.clone()),
            )
            .ok(),
            None => None,
        };

        if let Some(well_known_settings) = well_known_settings {
            OxlintSettings {
                json,
                jsx_a11y: well_known_settings.jsx_a11y,
                next: well_known_settings.next,
                react: well_known_settings.react,
                jsdoc: well_known_settings.jsdoc,
                vitest: well_known_settings.vitest,
            }
        } else {
            // TODO(perf): we can consume self to avoid clone
            OxlintSettings {
                json,
                jsx_a11y: self.jsx_a11y.clone(),
                next: self.next.clone(),
                react: self.react.clone(),
                jsdoc: self.jsdoc.clone(),
                vitest: self.vitest.clone(),
            }
        }
    }
}

fn deep_merge(a: &OxlintSettingsJson, b: &OxlintSettingsJson) -> OxlintSettingsJson {
    let mut result = b.clone();

    for (key, a_value) in a {
        match (a_value, result.get(key)) {
            (serde_json::Value::Object(self_obj), Some(serde_json::Value::Object(other_obj))) => {
                let merged_obj = deep_merge(self_obj, other_obj);
                result.insert(key.clone(), serde_json::Value::Object(merged_obj));
            }
            (self_val, _) => {
                // Either other doesn't have this key, or one of them is not an object
                // In both cases, use self's value
                result.insert(key.clone(), self_val.clone());
            }
        }
    }

    result
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
    fn test_extra_fields() {
        let json_value = serde_json::json!({
            "jsx-a11y": { "polymorphicPropName": "role" },
            "unknown-plugin": { "setting": "value" },
            "globalSetting": "value"
        });

        let settings = OxlintSettings::deserialize(&json_value).unwrap();

        // Known fields are parsed correctly
        assert_eq!(settings.jsx_a11y.polymorphic_prop_name, Some("role".into()));

        // Raw JSON preserves all fields (known and unknown)
        let raw_json = settings.json.unwrap();
        assert_eq!(raw_json["jsx-a11y"]["polymorphicPropName"], "role");
        assert_eq!(raw_json["unknown-plugin"]["setting"], "value");
        assert_eq!(raw_json["globalSetting"], "value");
    }

    #[test]
    fn test_merge() {
        let base = OxlintSettings::deserialize(&serde_json::json!({
            "jsx-a11y": { "polymorphicPropName": "role" },
            "unknown": { "a": 1, "nested": { "x": 1 } }
        }))
        .unwrap();

        let other = OxlintSettings::deserialize(&serde_json::json!({
            "jsx-a11y": { "attributes": { "for": ["htmlFor", "for"] } },
            "react": { "linkComponents": [{ "name": "Link", "linkAttribute": "to" }] },
            "unknown": { "b": 2, "nested": { "y": 2 } }
        }))
        .unwrap();

        let merged = base.merge(&other);

        assert_eq!(
            merged.react.get_link_component_attrs("Link").unwrap(),
            as_attrs(["to"]),
            "React settings from the other config are added to the merged settings."
        );

        assert_eq!(
            merged.jsx_a11y.polymorphic_prop_name,
            Some("role".into()),
            "JSX A11y settings from the base config are added to the merged settings."
        );
        assert_eq!(
            &merged.jsx_a11y.attributes["for"],
            &["htmlFor", "for"],
            "JSX A11y settings from one config get merged with JSX A11y settings from the other config."
        );

        // Raw JSON is deep merged
        let json = merged.json.unwrap();
        assert_eq!(json["unknown"]["a"], 1); // from base
        assert_eq!(json["unknown"]["b"], 2); // from other
        assert_eq!(json["unknown"]["nested"]["x"], 1); // from base
        assert_eq!(json["unknown"]["nested"]["y"], 2); // from other
    }
}
