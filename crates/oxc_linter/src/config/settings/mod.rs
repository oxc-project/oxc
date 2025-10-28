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
        // Deep merge each plugin's settings in place
        self.jsx_a11y.merge_into(&mut base.jsx_a11y);
        self.next.merge_into(&mut base.next);
        self.react.merge_into(&mut base.react);
        self.jsdoc.merge_into(&mut base.jsdoc);
        self.vitest.merge_into(&mut base.vitest);
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

        // Child with explicit empty array should override parent
        let merged_explicit = child_explicit_empty.clone().merge(parent.clone());
        assert_eq!(merged_explicit.next.get_root_dirs().len(), 0);

        // Child with unset should inherit from parent
        let merged_unset = child_unset.merge(parent.clone());
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

        // Child with explicit empty arrays should override parent
        let merged_explicit = child_explicit_empty.clone().merge(parent.clone());
        assert!(merged_explicit.react.get_form_component_attrs("ParentForm").is_none());
        assert!(merged_explicit.react.get_link_component_attrs("ParentLink").is_none());

        // Child with unset should inherit from parent
        let merged_unset = child_unset.merge(parent.clone());
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
