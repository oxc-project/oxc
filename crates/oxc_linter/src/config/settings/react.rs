use serde::Deserialize;

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
