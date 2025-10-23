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

            #[serde(default)]
            vitest: VitestPluginSettings,
        }

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

    pub fn merge(&self, other: &OxlintSettings) -> OxlintSettings {
        let json = match (&self.json, &other.json) {
            (Some(self_json), Some(other_json)) => Some(deep_merge(self_json, other_json)),
            (Some(self_json), None) => Some(self_json.clone()),
            (None, Some(other_json)) => Some(other_json.clone()),
            (None, None) => None,
        };
        // Comparing well known settings to their defaults is a hack because we don't know if they were actually left out or not.
        // TODO: Switch to Option?
        OxlintSettings {
            json,
            jsx_a11y: if self.jsx_a11y == JSXA11yPluginSettings::default() {
                other.jsx_a11y.clone()
            } else {
                self.jsx_a11y.clone()
            },
            next: if self.next == NextPluginSettings::default() {
                other.next.clone()
            } else {
                self.next.clone()
            },
            react: if self.react == ReactPluginSettings::default() {
                other.react.clone()
            } else {
                self.react.clone()
            },
            jsdoc: if self.jsdoc == JSDocPluginSettings::default() {
                other.jsdoc.clone()
            } else {
                self.jsdoc.clone()
            },
            vitest: if self.vitest == VitestPluginSettings::default() {
                other.vitest.clone()
            } else {
                self.vitest.clone()
            },
        }
    }
}

fn deep_merge(
    self_json: &OxlintSettingsJson,
    other_json: &OxlintSettingsJson,
) -> OxlintSettingsJson {
    let mut result = other_json.clone();

    for (key, self_value) in self_json {
        match (self_value, result.get(key)) {
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
            "react": { "linkComponents": [{ "name": "Link", "linkAttribute": "to" }] },
            "unknown": { "b": 2, "nested": { "y": 2 } }
        }))
        .unwrap();

        let merged = base.merge(&other);

        // Non-default base values take precedence
        assert_eq!(merged.jsx_a11y.polymorphic_prop_name, Some("role".into()));
        // Default base values use other
        assert_eq!(merged.react.get_link_component_attrs("Link").unwrap(), as_attrs(["to"]));

        // Raw JSON is deep merged
        let json = merged.json.unwrap();
        assert_eq!(json["unknown"]["a"], 1); // from base
        assert_eq!(json["unknown"]["b"], 2); // from other
        assert_eq!(json["unknown"]["nested"]["x"], 1); // from base
        assert_eq!(json["unknown"]["nested"]["y"], 2); // from other
    }
}
