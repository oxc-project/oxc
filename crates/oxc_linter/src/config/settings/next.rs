use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default, JsonSchema)]
pub struct NextPluginSettings {
    #[serde(default)]
    #[serde(rename = "rootDir")]
    root_dir: OneOrMany<String>,
}

impl NextPluginSettings {
    pub fn get_root_dirs(&self) -> Vec<String> {
        match &self.root_dir {
            OneOrMany::One(val) => vec![val.clone()],
            OneOrMany::Many(vec) => vec.clone(),
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
