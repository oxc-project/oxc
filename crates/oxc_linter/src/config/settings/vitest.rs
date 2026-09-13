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
    /// The built-in conventions — a `__tests__` path segment, or a file name whose
    /// second-to-last dot-separated segment is `test` or `spec` — already make a file a test
    /// file, but they imply the *Jest* dialect. Vitest itself is recognized only through a
    /// static import with bindings from `vitest`, `vite-plus/test`, or `@effect/vitest`, so a
    /// helper module that pulls `expect` in from elsewhere is skipped by most Vitest rules.
    /// List such files here to lint them:
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
    /// Matching here sets the Vitest flag, which the rules shared between the two plugins
    /// consult to pick the dialect they enforce. It does not clear Jest — a path that also
    /// follows the built-in conventions, or that a `settings.jest.additionalTestPatterns`
    /// entry matches, is treated as both. Takes effect only when the `jest` or `vitest`
    /// plugin is enabled.
    #[serde(default, rename = "additionalTestPatterns")]
    pub additional_test_patterns: GlobSet,
}
