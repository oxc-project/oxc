use schematic::Config;
use serde::Deserialize;

/// <https://nextjs.org/docs/pages/building-your-application/configuring/eslint#eslint-plugin>
#[derive(Debug, Clone, PartialEq, Deserialize, Config)]
pub struct NextPluginSettings {
    #[serde(default, rename = "rootDir", skip_serializing)]
    root_dir: OneOrMany,
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

#[derive(Clone, Debug, Deserialize, PartialEq, Config)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}
