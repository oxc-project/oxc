use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
}

impl VitestPluginSettings {
    pub(crate) fn is_empty(&self) -> bool {
        !self.typecheck
    }

    /// Deep merge self into other (self takes priority).
    pub(crate) fn merge(self, other: Self) -> Self {
        // If self is empty (default), use other's values
        if self.is_empty() {
            return other;
        }
        self
    }

    /// Deep merge self into base (self takes priority), mutating base in place.
    pub(crate) fn merge_into(&self, base: &mut Self) {
        // If self is not empty, override base's values
        if !self.is_empty() {
            base.typecheck = self.typecheck;
        }
    }
}
