use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::utils::is_vitest_import_source;

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

    /// Import sources whose exported `test`/`describe`/`it`/`expect` bindings
    /// should be treated as Vitest functions, in addition to the built-in
    /// Vitest import sources (`vitest`, `vite-plus/test`, `@effect/vitest`).
    ///
    /// Useful when tests import their test functions from a custom fixture or
    /// wrapper module instead of `vitest` directly, for example:
    ///
    /// ```jsonc
    /// {
    ///   "settings": {
    ///     "vitest": {
    ///       "vitestImports": ["@/test/fixtures"]
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// See the [custom fixtures](https://github.com/vitest-dev/eslint-plugin-vitest#custom-fixtures)
    /// documentation of `eslint-plugin-vitest`. Note that regular expressions
    /// are not supported yet, only exact module names.
    #[serde(default, rename = "vitestImports")]
    pub vitest_imports: Vec<String>,
}

impl VitestPluginSettings {
    /// Returns `true` if `source` is a Vitest import source, either one of the
    /// built-in sources (`vitest`, `vite-plus/test`, `@effect/vitest`) or one
    /// configured via the `vitestImports` setting.
    pub fn is_vitest_import_source(&self, source: &str) -> bool {
        is_vitest_import_source(source)
            || self.vitest_imports.iter().any(|import| import.as_str() == source)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::VitestPluginSettings;

    #[test]
    fn test_parse_vitest_imports() {
        let settings: VitestPluginSettings = serde_json::from_value(json!({
            "typecheck": true,
            "vitestImports": ["@/test/fixtures", "$test/setup/fixtures"]
        }))
        .unwrap();

        assert!(settings.typecheck);
        assert_eq!(
            settings.vitest_imports,
            vec!["@/test/fixtures".to_string(), "$test/setup/fixtures".to_string()]
        );

        // Built-in sources are always recognized.
        assert!(settings.is_vitest_import_source("vitest"));
        assert!(settings.is_vitest_import_source("vite-plus/test"));
        assert!(settings.is_vitest_import_source("@effect/vitest"));
        // Configured sources are recognized too.
        assert!(settings.is_vitest_import_source("@/test/fixtures"));
        assert!(settings.is_vitest_import_source("$test/setup/fixtures"));
        // Unrelated sources are not.
        assert!(!settings.is_vitest_import_source("some-other-module"));
    }

    #[test]
    fn test_parse_vitest_imports_default() {
        let settings: VitestPluginSettings = serde_json::from_value(json!({})).unwrap();

        assert!(!settings.typecheck);
        assert!(settings.vitest_imports.is_empty());
        assert!(settings.is_vitest_import_source("vitest"));
        assert!(!settings.is_vitest_import_source("./fixtures"));
    }
}
