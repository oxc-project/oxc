use serde::Deserialize;

/// https://github.com/jsx-eslint/eslint-plugin-react#configuration-legacy-eslintrc-
#[derive(Debug, Deserialize, Default)]
pub struct ESLintSettingsReact {
    #[serde(default)]
    #[serde(rename = "formComponents")]
    form_components: Vec<CustomComponent>,
    #[serde(default)]
    #[serde(rename = "linkComponents")]
    link_components: Vec<CustomComponent>,
    // TODO: More properties should be added
}

impl ESLintSettingsReact {
    pub fn get_form_component_attr(&self, name: &str) -> Option<Vec<String>> {
        get_component_attr_by_name(&self.form_components, name)
    }

    pub fn get_link_component_attr(&self, name: &str) -> Option<Vec<String>> {
        get_component_attr_by_name(&self.link_components, name)
    }
}

// Deserialize helper types

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
enum CustomComponent {
    NameOnly(String),
    ObjectWithOneAttr {
        name: String,
        #[serde(alias = "formAttribute", alias = "linkAttribute")]
        attributes: String,
    },
    ObjectWithMaynAttrs {
        name: String,
        #[serde(alias = "formAttribute", alias = "linkAttribute")]
        attributes: Vec<String>,
    },
}

fn get_component_attr_by_name(
    components: &Vec<CustomComponent>,
    name: &str,
) -> Option<Vec<String>> {
    for item in components {
        let comp = match item {
            CustomComponent::NameOnly(name) => (name, vec![]),
            CustomComponent::ObjectWithOneAttr { name, attributes } => {
                (name, vec![attributes.to_string()])
            }
            CustomComponent::ObjectWithMaynAttrs { name, attributes } => (name, attributes.clone()),
        };

        if comp.0 == name {
            return Some(comp.1);
        }
    }

    None
}
