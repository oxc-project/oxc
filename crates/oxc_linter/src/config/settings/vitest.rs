use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_config::GlobSet;

/// Configure Vitest plugin rules.
///
/// See [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest)'s
/// configuration for a full reference.
#[derive(Debug, Clone, Deserialize, Serialize, Default, JsonSchema, PartialEq, Eq)]
pub struct VitestPluginSettings {
    /// Whether to enable typecheck mode for Vitest rules.
    /// When enabled, some rules will skip certain checks for describe blocks
    /// to accommodate TypeScript type checking scenarios.
    #[serde(default)]
    pub typecheck: bool,

    /// Extra glob patterns that mark a file as a Vitest test file.
    ///
    /// Without an import from `vitest`, a file is only recognized as a Jest test file, if at
    /// all. Use this for helpers that call `test()` or `expect()` without importing them:
    ///
    /// ```json
    /// {
    ///   "settings": {
    ///     "vitest": {
    ///       "additionalTestPatterns": ["**/*.steps.ts", "**/*.helper.ts"]
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// Patterns are matched against the file path relative to the directory holding the
    /// config file, the same way `overrides[].files` is matched. A pattern without a `/`
    /// is made recursive, so `"*.steps.ts"` and `"**/*.steps.ts"` are equivalent.
    ///
    /// Only adds files; a file that is also recognized as Jest is treated as both.
    #[serde(default, rename = "additionalTestPatterns")]
    pub additional_test_patterns: GlobSet,
}
