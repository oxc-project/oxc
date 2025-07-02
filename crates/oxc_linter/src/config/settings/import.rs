use lazy_regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// See: [settings](https://github.com/import-js/eslint-plugin-import/blob/main/README.md#settings)
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct ImportPluginSettings {
    /// An array of folders. Resolved modules only from those folders will be considered as "external".
    ///
    /// See: [import/external-module-folders](https://github.com/import-js/eslint-plugin-import/blob/main/README.md#importexternal-module-folders)
    #[serde(default = "default_external_module_folders", rename = "external-module-folders")]
    external_module_folders: Vec<String>,

    /// A regex for packages should be treated as internal.
    /// Useful when you are utilizing a monorepo setup or developing a set of packages that depend on each other.
    ///
    /// See: [import/internal-regex](https://github.com/import-js/eslint-plugin-import/blob/main/README.md#importinternal-regex)
    ///
    /// Example:
    ///
    /// ```json
    /// {
    ///   "settings": {
    ///     "import": {
    ///       "internal-regex": "^@scope/"
    ///     }
    ///   }
    /// }
    /// ```
    #[serde(default, rename = "internal-regex")]
    internal_regex: String,
}

impl Default for ImportPluginSettings {
    fn default() -> Self {
        Self {
            external_module_folders: default_external_module_folders(),
            internal_regex: String::default(),
        }
    }
}

impl ImportPluginSettings {
    pub fn get_external_module_folders(&self) -> Vec<String> {
        self.external_module_folders.clone()
    }

    pub fn get_internal_regex(&self) -> Regex {
        Regex::new(&self.internal_regex).unwrap()
    }
}

fn default_external_module_folders() -> Vec<String> {
    vec!["node_modules".to_string()]
}

#[cfg(test)]
mod test {
    use serde::Deserialize;

    use super::ImportPluginSettings;

    #[test]
    fn parse_defaults() {
        let settings = ImportPluginSettings::deserialize(serde_json::json!({})).unwrap();
        assert!(settings.get_internal_regex().as_str().is_empty());
        assert_eq!(settings.get_external_module_folders(), ["node_modules"]);
    }
}
