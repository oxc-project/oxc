use rustc_hash::FxHashMap;
use serde::Deserialize;

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

/// https://github.com/jsx-eslint/eslint-plugin-jsx-a11y#configurations
#[derive(Debug, Deserialize, Default)]
pub struct ESLintSettingsJSXA11y {
    #[serde(rename = "polymorphicPropName")]
    pub polymorphic_prop_name: Option<String>,
    #[serde(default)]
    pub components: FxHashMap<String, String>,
}

/// https://nextjs.org/docs/pages/building-your-application/configuring/eslint#eslint-plugin
#[derive(Debug, Deserialize, Default)]
pub struct ESLintSettingsNext {
    #[serde(default)]
    #[serde(rename = "rootDir")]
    root_dir: OneOrMany<String>,
}

impl ESLintSettingsNext {
    pub fn get_root_dirs(&self) -> Vec<String> {
        match &self.root_dir {
            OneOrMany::One(val) => vec![val.clone()],
            OneOrMany::Many(vec) => vec.clone(),
        }
    }
}

/// https://github.com/jsx-eslint/eslint-plugin-react#configuration-legacy-eslintrc-
#[derive(Debug, Deserialize, Default)]
pub struct ESLintSettingsReact {
    #[serde(default)]
    #[serde(rename = "formComponents")]
    form_components: Vec<FormComponent>,
    #[serde(default)]
    #[serde(rename = "linkComponents")]
    link_components: Vec<LinkComponent>,

    // TODO: More properties should be added
}

impl ESLintSettingsReact {
    pub fn get_form_component_attr(&self, name: &str) -> Option<Vec<String>> {
        for item in &self.form_components {
            let comp = match item {
                FormComponent::NameOnly(name) => (name, vec![]),
                FormComponent::ObjectWithOneAttr { name, form_attribute } => {
                    (name, vec![form_attribute.to_string()])
                }
                FormComponent::ObjectWithMaynAttrs { name, form_attribute } => {
                    (name, form_attribute.clone())
                }
            };
            if comp.0 == name {
                return Some(comp.1);
            }
        }

        None
    }

    pub fn get_link_component_attr(&self, name: &str) -> Option<Vec<String>> {
        for item in &self.link_components {
            let comp = match item {
                LinkComponent::NameOnly(name) => (name, vec![]),
                LinkComponent::ObjectWithOneAttr { name, link_attribute } => {
                    (name, vec![link_attribute.to_string()])
                }
                LinkComponent::ObjectWithMaynAttrs { name, link_attribute } => {
                    (name, link_attribute.clone())
                }
            };
            if comp.0 == name {
                return Some(comp.1);
            }
        }

        None
    }
}

// Deserialize helper types

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}
impl<T> Default for OneOrMany<T> {
    fn default() -> Self {
        OneOrMany::Many(Vec::new())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
enum FormComponent {
    NameOnly(String),
    ObjectWithOneAttr {
        name: String,
        #[serde(rename = "formAttribute")]
        form_attribute: String,
    },
    ObjectWithMaynAttrs {
        name: String,
        #[serde(rename = "formAttribute")]
        form_attribute: Vec<String>,
    },
}

// It seems above and below are almost the same,
// but original code uses different names. So keep it as is.

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
enum LinkComponent {
    NameOnly(String),
    ObjectWithOneAttr {
        name: String,
        #[serde(rename = "linkAttribute")]
        link_attribute: String,
    },
    ObjectWithMaynAttrs {
        name: String,
        #[serde(rename = "linkAttribute")]
        link_attribute: Vec<String>,
    },
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
        assert_eq!(settings.react.get_form_component_attr("CustomForm"), Some(vec![]));
        assert_eq!(
            settings.react.get_form_component_attr("SimpleForm"),
            Some(vec!["endpoint".to_string()])
        );
        assert_eq!(
            settings.react.get_form_component_attr("Form"),
            Some(vec!["registerEndpoint".to_string(), "loginEndpoint".to_string()])
        );
        assert_eq!(
            settings.react.get_link_component_attr("Link"),
            Some(vec!["to".to_string(), "href".to_string()])
        );
        assert_eq!(settings.react.get_link_component_attr("Noop"), None);
    }

    #[test]
    fn test_parse_settings_default() {
        let settings = ESLintSettings::default();
        assert!(settings.jsx_a11y.polymorphic_prop_name.is_none());
        assert!(settings.jsx_a11y.components.is_empty());
    }
}
