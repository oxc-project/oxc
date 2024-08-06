use std::borrow::Cow;

use oxc_span::CompactStr;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{set, utils::Set};

// <https://github.com/jsx-eslint/eslint-plugin-react#configuration-legacy-eslintrc->
#[derive(Debug, Deserialize, Default, JsonSchema)]
pub struct ReactPluginSettings {
    #[serde(default)]
    #[serde(rename = "formComponents")]
    form_components: Set<CustomComponent>,

    #[serde(default)]
    #[serde(rename = "linkComponents")]
    link_components: Set<CustomComponent>,
    // TODO: More properties should be added
}

pub type ComponentAttrs<'c> = Cow<'c, Set<CompactStr>>;
impl ReactPluginSettings {
    pub fn get_form_component_attrs(&self, name: &str) -> Option<ComponentAttrs<'_>> {
        get_component_attrs_by_name(&self.form_components, name)
    }

    pub fn get_link_component_attrs(&self, name: &str) -> Option<ComponentAttrs<'_>> {
        get_component_attrs_by_name(&self.link_components, name)
    }
}

// Deserialize helper types

#[derive(Clone, Debug, Deserialize, JsonSchema)]
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
        attributes: Set<CompactStr>,
    },
}
impl CustomComponent {
    pub fn name(&self) -> &CompactStr {
        match self {
            Self::NameOnly(name)
            | Self::ObjectWithOneAttr { name, .. }
            | Self::ObjectWithManyAttrs { name, .. } => name,
        }
    }
}

impl PartialEq for CustomComponent {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}
impl Eq for CustomComponent {}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for CustomComponent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name().partial_cmp(other.name())
    }
}

impl Ord for CustomComponent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name().cmp(other.name())
    }
}

fn get_component_attrs_by_name<'c>(
    components: &'c Set<CustomComponent>,
    name: &str,
) -> Option<ComponentAttrs<'c>> {
    for item in components {
        match item {
            CustomComponent::NameOnly(comp_name) if comp_name == name => {
                return Some(Cow::Owned(set![]))
            }
            CustomComponent::ObjectWithOneAttr { name: comp_name, attribute }
                if comp_name == name =>
            {
                return Some(Cow::Owned(set![attribute.clone()]));
            }
            CustomComponent::ObjectWithManyAttrs { name: comp_name, attributes }
                if comp_name == name =>
            {
                return Some(Cow::Borrowed(attributes));
            }
            _ => {}
        };
    }

    None
}
