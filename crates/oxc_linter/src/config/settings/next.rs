use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Deserialize, Default, Serialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq))]
pub struct NextPluginSettings {
    #[serde(default)]
    #[serde(rename = "rootDir")]
    root_dir: OneOrMany<String>,
}

impl NextPluginSettings {
    pub fn get_root_dirs(&self) -> Cow<'_, [String]> {
        match &self.root_dir {
            OneOrMany::One(val) => Cow::Owned(vec![val.clone()]),
            OneOrMany::Many(vec) => Cow::Borrowed(vec),
        }
    }
}

// Deserialize helper types

#[derive(Clone, Debug, Deserialize, PartialEq, JsonSchema)]
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

impl<T: Serialize> Serialize for OneOrMany<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::One(val) => val.serialize(serializer),
            Self::Many(vec) => vec.serialize(serializer),
        }
    }
}
