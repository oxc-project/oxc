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
#[derive(Debug, Clone, Deserialize, Serialize, Default, JsonSchema, PartialEq)]
pub struct OxlintSettings {
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

impl OxlintSettings {
    /// Deep merge settings (ESLint compatible).
    /// Self takes priority over other. Nested objects are merged recursively,
    /// but arrays are replaced (not merged).
    pub fn merge(self, other: Self) -> Self {
        Self {
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
        // Deep merge each plugin's settings
        let merged = self.clone().merge(base.clone());
        *base = merged;
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
}
