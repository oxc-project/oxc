use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Default, JsonSchema, Serialize)]
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

#[derive(Clone, Debug, Deserialize, PartialEq, JsonSchema, Serialize)]
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
