use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configure Vitest plugin rules.
///
/// See [eslint-plugin-vitest](https://github.com/veritem/eslint-plugin-vitest)'s
/// configuration for a full reference.
#[derive(Debug, Clone, Deserialize, Serialize, Default, JsonSchema)]
#[cfg_attr(test, derive(PartialEq))]
pub struct VitestPluginSettings {
    /// Whether to enable typecheck mode for Vitest rules.
    /// When enabled, some rules will skip certain checks for describe blocks
    /// to accommodate TypeScript type checking scenarios.
    #[serde(default)]
    pub typecheck: bool,
}
