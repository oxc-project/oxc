pub mod jsdoc;
mod jsx_a11y;
mod next;
mod react;

use schemars::JsonSchema;
use serde::Deserialize;

use self::{
    jsdoc::JSDocPluginSettings, jsx_a11y::JSXA11yPluginSettings, next::NextPluginSettings,
    react::ReactPluginSettings,
};

/// Shared settings for plugins
#[derive(Debug, Deserialize, Default, JsonSchema)]
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
}

#[cfg(test)]
mod test {
    use serde::Deserialize;

    use super::OxlintSettings;

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
        let settings = OxlintSettings::default();
        assert!(settings.jsx_a11y.polymorphic_prop_name.is_none());
        assert!(settings.jsx_a11y.components.is_empty());
    }
}
