use std::borrow::Cow;

use oxc_span::CompactStr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configure React plugin rules.
///
/// Derived from [eslint-plugin-react](https://github.com/jsx-eslint/eslint-plugin-react#configuration-legacy-eslintrc-)
#[derive(Debug, Clone, Deserialize, Default, Serialize, JsonSchema, PartialEq)]
pub struct ReactPluginSettings {
    /// Components used as alternatives to `<form>` for forms, such as `<Formik>`.
    ///
    /// Example:
    ///
    /// ```jsonc
    /// {
    ///   "settings": {
    ///     "react": {
    ///       "formComponents": [
    ///         "CustomForm",
    ///         // OtherForm is considered a form component and has an endpoint attribute
    ///         { "name": "OtherForm", "formAttribute": "endpoint" },
    ///         // allows specifying multiple properties if necessary
    ///         { "name": "Form", "formAttribute": ["registerEndpoint", "loginEndpoint"] }
    ///       ]
    ///     }
    ///   }
    /// }
    /// ```
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "formComponents")]
    form_components: Option<Vec<CustomComponent>>,

    /// Components used as alternatives to `<a>` for linking, such as `<Link>`.
    ///
    /// Example:
    ///
    /// ```jsonc
    /// {
    ///   "settings": {
    ///     "react": {
    ///       "linkComponents": [
    ///         "HyperLink",
    ///         // Use `linkAttribute` for components that use a different prop name
    ///         // than `href`.
    ///         { "name": "MyLink", "linkAttribute": "to" },
    ///         // allows specifying multiple properties if necessary
    ///         { "name": "Link", "linkAttribute": ["to", "href"] }
    ///       ]
    ///     }
    ///   }
    /// }
    /// ```
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "linkComponents")]
    link_components: Option<Vec<CustomComponent>>,
    // TODO: More properties should be added
}

pub type ComponentAttrs<'c> = Cow<'c, Vec<CompactStr>>;
impl ReactPluginSettings {
    pub fn get_form_component_attrs(&self, name: &str) -> Option<ComponentAttrs<'_>> {
        self.form_components
            .as_ref()
            .and_then(|components| get_component_attrs_by_name(components, name))
    }

    pub fn get_link_component_attrs(&self, name: &str) -> Option<ComponentAttrs<'_>> {
        self.link_components
            .as_ref()
            .and_then(|components| get_component_attrs_by_name(components, name))
    }

    /// Deep merge self into other (self takes priority).
    /// Arrays are replaced, not merged (ESLint behavior).
    pub(crate) fn merge(mut self, other: Self) -> Self {
        // If self has components, use them; otherwise use other's
        if self.form_components.is_none() {
            self.form_components = other.form_components;
        }
        if self.link_components.is_none() {
            self.link_components = other.link_components;
        }
        self
    }

    /// Deep merge self into base (self takes priority), mutating base in place.
    /// Arrays are replaced, not merged (ESLint behavior).
    pub(crate) fn merge_into(&self, base: &mut Self) {
        // If self has components, replace base's
        if self.form_components.is_some() {
            base.form_components.clone_from(&self.form_components);
        }
        if self.link_components.is_some() {
            base.link_components.clone_from(&self.link_components);
        }
    }
}

// Deserialize helper types

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq)]
#[serde(untagged)]
enum CustomComponent {
    NameOnly(CompactStr),
    ObjectWithOneAttr {
        name: CompactStr,
        #[serde(alias = "formAttribute", alias = "linkAttribute")]
        attribute: CompactStr,
    },
    ObjectWithManyAttrs {
        name: CompactStr,
        #[serde(alias = "formAttribute", alias = "linkAttribute")]
        attributes: Vec<CompactStr>,
    },
}

fn get_component_attrs_by_name<'c>(
    components: &'c Vec<CustomComponent>,
    name: &str,
) -> Option<ComponentAttrs<'c>> {
    for item in components {
        match item {
            CustomComponent::NameOnly(comp_name) if comp_name == name => {
                return Some(Cow::Owned(vec![]));
            }
            CustomComponent::ObjectWithOneAttr { name: comp_name, attribute }
                if comp_name == name =>
            {
                return Some(Cow::Owned(vec![attribute.clone()]));
            }
            CustomComponent::ObjectWithManyAttrs { name: comp_name, attributes }
                if comp_name == name =>
            {
                return Some(Cow::Borrowed(attributes));
            }
            _ => {}
        }
    }

    None
}
