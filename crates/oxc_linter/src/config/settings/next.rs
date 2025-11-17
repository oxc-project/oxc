use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize, Serializer};

/// Configure Next.js plugin rules.
#[derive(Debug, Clone, Deserialize, Default, Serialize, JsonSchema, PartialEq)]
pub struct NextPluginSettings {
    /// The root directory of the Next.js project.
    ///
    /// This is particularly useful when you have a monorepo and your Next.js
    /// project is in a subfolder.
    ///
    /// Example:
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "rootDir")]
    root_dir: Option<OneOrMany<String>>,
}

impl NextPluginSettings {
    pub fn get_root_dirs(&self) -> Cow<'_, [String]> {
        match &self.root_dir {
            Some(OneOrMany::One(val)) => Cow::Owned(vec![val.clone()]),
            Some(OneOrMany::Many(vec)) => Cow::Borrowed(vec),
            None => Cow::Owned(vec![]),
        }
    }

    /// Deep merge self into other (self takes priority).
    /// Arrays are replaced, not merged (ESLint behavior).
    pub(crate) fn merge(mut self, other: Self) -> Self {
        // If self has no root_dir, use other's
        if self.root_dir.is_none() {
            self.root_dir = other.root_dir;
        }
        self
    }

    /// Deep merge self into base (self takes priority), mutating base in place.
    /// Arrays are replaced, not merged (ESLint behavior).
    pub(crate) fn merge_into(&self, base: &mut Self) {
        // If self has root_dir, replace base's
        if self.root_dir.is_some() {
            base.root_dir.clone_from(&self.root_dir);
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
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::One(val) => val.serialize(serializer),
            Self::Many(vec) => vec.serialize(serializer),
        }
    }
}
