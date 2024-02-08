use self::{jsx_a11y::ESLintSettingsJSXA11y, next::ESLintSettingsNext, react::ESLintSettingsReact};
use serde::Deserialize;

mod jsx_a11y;
mod next;
mod react;

/// The `settings` field from ESLint config
/// An object containing name-value pairs of information that should be available to all rules
///
/// TS type is `Object`
/// https://github.com/eslint/eslint/blob/ce838adc3b673e52a151f36da0eedf5876977514/lib/shared/types.js#L53
/// But each plugin extends this with their own properties.
#[derive(Debug, Deserialize, Default)]
pub struct ESLintSettings {
    #[serde(default)]
    #[serde(rename = "jsx-a11y")]
    pub jsx_a11y: ESLintSettingsJSXA11y,
    #[serde(default)]
    pub next: ESLintSettingsNext,
    #[serde(default)]
    pub react: ESLintSettingsReact,
}

#[cfg(test)]
mod test {
    use super::ESLintSettings;
    use serde::Deserialize;

    #[test]
    fn test_parse_settings() {
        let settings = ESLintSettings::deserialize(&serde_json::json!({
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

        assert_eq!(settings.jsx_a11y.polymorphic_prop_name, Some("role".to_string()));
        assert_eq!(settings.jsx_a11y.components.get("Link"), Some(&"Anchor".to_string()));
        assert!(settings.next.get_root_dirs().contains(&"app".to_string()));
        assert_eq!(settings.react.get_form_component_attrs("CustomForm"), Some(vec![]));
        assert_eq!(
            settings.react.get_form_component_attrs("SimpleForm"),
            Some(vec!["endpoint".to_string()])
        );
        assert_eq!(
            settings.react.get_form_component_attrs("Form"),
            Some(vec!["registerEndpoint".to_string(), "loginEndpoint".to_string()])
        );
        assert_eq!(
            settings.react.get_link_component_attrs("Link"),
            Some(vec!["to".to_string(), "href".to_string()])
        );
        assert_eq!(settings.react.get_link_component_attrs("Noop"), None);
    }

    #[test]
    fn test_parse_settings_default() {
        let settings = ESLintSettings::default();
        assert!(settings.jsx_a11y.polymorphic_prop_name.is_none());
        assert!(settings.jsx_a11y.components.is_empty());
    }
}
