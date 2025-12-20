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
    /// Deep merge settings (ESLint compatible).
    /// Self takes priority over other. Nested objects are merged recursively,
    /// but arrays are replaced (not merged).
    pub fn merge(self, other: Self) -> Self {
        Self {
            json: match (self.json, other.json) {
                (Some(a), Some(b)) => Some(deep_merge(&a, &b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
            jsx_a11y: self.jsx_a11y.merge(other.jsx_a11y),
            next: self.next.merge(other.next),
            react: self.react.merge(other.react),
            jsdoc: self.jsdoc.merge(other.jsdoc),
            vitest: self.vitest.merge(other.vitest),
        }
    }

    /// Deep merge override settings into base settings (for use in overrides).
    /// This mutates the base settings in place using deep merge semantics.
    pub(crate) fn merge_into(&self, base: &mut Self) {
        // Deep merge each plugin's settings in place
        self.jsx_a11y.merge_into(&mut base.jsx_a11y);
        self.next.merge_into(&mut base.next);
        self.react.merge_into(&mut base.react);
        self.jsdoc.merge_into(&mut base.jsdoc);
        self.vitest.merge_into(&mut base.vitest);
    }

    // Note: We don't merge settings in overrides at present.
    // So this is dead code, but keeping it for now, as we may want to enable merging settings in the future.
    #[expect(dead_code)]
    /// Mutates `settings_to_override` by reading from `self`.
    fn override_settings(&self, settings_to_override: &mut OxlintSettings) {
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
    fn test_next_merge_empty_array_vs_unset() {
        // Test that explicitly setting rootDir: [] overrides parent config
        let parent = OxlintSettings::deserialize(&serde_json::json!({
            "next": {
                "rootDir": ["parent/dir"]
            }
        }))
        .unwrap();

        let child_explicit_empty = OxlintSettings::deserialize(&serde_json::json!({
            "next": {
                "rootDir": []
            }
        }))
        .unwrap();

        let child_unset = OxlintSettings::deserialize(&serde_json::json!({})).unwrap();
        let parent_clone = parent.clone();

        // Child with explicit empty array should override parent
        let merged_explicit = child_explicit_empty.merge(parent);
        assert_eq!(merged_explicit.next.get_root_dirs().len(), 0);

        // Child with unset should inherit from parent
        let merged_unset = child_unset.merge(parent_clone);
        assert_eq!(merged_unset.next.get_root_dirs().len(), 1);
        assert_eq!(merged_unset.next.get_root_dirs()[0], "parent/dir");
    }

    #[test]
    fn test_react_merge_empty_array_vs_unset() {
        // Test that explicitly setting components: [] overrides parent config
        let parent = OxlintSettings::deserialize(&serde_json::json!({
            "react": {
                "formComponents": ["ParentForm"],
                "linkComponents": ["ParentLink"]
            }
        }))
        .unwrap();

        let child_explicit_empty = OxlintSettings::deserialize(&serde_json::json!({
            "react": {
                "formComponents": [],
                "linkComponents": []
            }
        }))
        .unwrap();

        let child_unset = OxlintSettings::deserialize(&serde_json::json!({})).unwrap();
        let parent_clone = parent.clone();

        // Child with explicit empty arrays should override parent
        let merged_explicit = child_explicit_empty.merge(parent);
        assert!(merged_explicit.react.get_form_component_attrs("ParentForm").is_none());
        assert!(merged_explicit.react.get_link_component_attrs("ParentLink").is_none());

        // Child with unset should inherit from parent
        let merged_unset = child_unset.merge(parent_clone);
        assert!(merged_unset.react.get_form_component_attrs("ParentForm").is_some());
        assert!(merged_unset.react.get_link_component_attrs("ParentLink").is_some());
    }

    #[test]
    fn test_merge_into_with_explicit_empty() {
        // Test merge_into with explicit empty arrays
        let mut base = OxlintSettings::deserialize(&serde_json::json!({
            "next": {
                "rootDir": ["base/dir"]
            },
            "react": {
                "formComponents": ["BaseForm"]
            }
        }))
        .unwrap();

        let override_empty = OxlintSettings::deserialize(&serde_json::json!({
            "next": {
                "rootDir": []
            },
            "react": {
                "formComponents": []
            }
        }))
        .unwrap();

        override_empty.merge_into(&mut base);

        // Empty arrays should override base values
        assert_eq!(base.next.get_root_dirs().len(), 0);
        assert!(base.react.get_form_component_attrs("BaseForm").is_none());
    }

    #[test]
    fn test_merge_into_with_unset() {
        // Test merge_into with unset values (should not override)
        let mut base = OxlintSettings::deserialize(&serde_json::json!({
            "next": {
                "rootDir": ["base/dir"]
            },
            "react": {
                "formComponents": ["BaseForm"]
            }
        }))
        .unwrap();

        let override_unset = OxlintSettings::deserialize(&serde_json::json!({})).unwrap();

        override_unset.merge_into(&mut base);

        // Unset values should not override base
        assert_eq!(base.next.get_root_dirs().len(), 1);
        assert_eq!(base.next.get_root_dirs()[0], "base/dir");
        assert!(base.react.get_form_component_attrs("BaseForm").is_some());
    }
}
