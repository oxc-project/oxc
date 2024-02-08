use serde::Deserialize;

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
