use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_str::CompactStr;

/// File extensions considered by default, mirroring common resolver configurations.
pub const DEFAULT_IMPORT_EXTENSIONS: [&str; 9] =
    [".js", ".mjs", ".cjs", ".jsx", ".ts", ".tsx", ".mts", ".cts", ".json"];

/// Configure import plugin rules.
///
/// See [eslint-plugin-import's settings](https://github.com/import-js/eslint-plugin-import#settings).
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub struct ImportPluginSettings {
    /// File extensions that import rules consider when resolving relative specifiers
    /// and detecting directory `index` files.
    ///
    /// Mirrors eslint-plugin-import's `import/extensions` setting.
    ///
    /// Example:
    ///
    /// ```json
    /// {
    ///   "settings": {
    ///     "import": {
    ///       "extensions": [".js", ".jsx"]
    ///     }
    ///   }
    /// }
    /// ```
    #[serde(default = "default_extensions")]
    pub extensions: Vec<CompactStr>,
}

impl Default for ImportPluginSettings {
    fn default() -> Self {
        Self { extensions: default_extensions() }
    }
}

fn default_extensions() -> Vec<CompactStr> {
    DEFAULT_IMPORT_EXTENSIONS.iter().copied().map(CompactStr::from).collect()
}
