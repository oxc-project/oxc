use std::borrow::Cow;

use schemars::JsonSchema;
use serde::Deserialize;

/// Configure Next.js plugin rules.
#[derive(Debug, Deserialize, Default, JsonSchema)]
pub struct NextPluginSettings {
    /// The root directory of the Next.js project.
    ///
    /// This is particularly useful when you have a monorepo and your Next.js
    /// project is in a subfolder.
    ///
    /// ## Example
    ///
    /// ```json
    /// {
    ///   "settings": {
    ///     "next": {
    ///       "rootDir": "apps/dashboard/"
    ///     }
    ///   }
    /// }
    /// ```
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
