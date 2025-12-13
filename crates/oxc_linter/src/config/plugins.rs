use std::borrow::Cow;

use bitflags::bitflags;
use schemars::{JsonSchema, r#gen::SchemaGenerator, schema::Schema};
use serde::{Deserialize, Serialize, de::Deserializer, ser::Serializer};

/// Normalizes plugin names by stripping common ESLint plugin prefixes and suffixes.
///
/// This handles the various naming conventions used in the ESLint ecosystem:
/// - `eslint-plugin-foo` → `foo`
/// - `@scope/eslint-plugin` → `@scope`
/// - `@scope/eslint-plugin-foo` → `@scope/foo`
///
/// # Examples
///
/// ```
/// use oxc_linter::normalize_plugin_name;
///
/// assert_eq!(normalize_plugin_name("eslint-plugin-react"), "react");
/// assert_eq!(normalize_plugin_name("@typescript-eslint/eslint-plugin"), "@typescript-eslint");
/// assert_eq!(normalize_plugin_name("@foo/eslint-plugin-bar"), "@foo/bar");
/// ```
pub fn normalize_plugin_name(plugin_name: &str) -> Cow<'_, str> {
    // Handle scoped packages (@scope/...)
    if let Some(scope_end) = plugin_name.find('/') {
        let scope = &plugin_name[..scope_end]; // e.g., "@foo"
        let rest = &plugin_name[scope_end + 1..]; // e.g., "eslint-plugin" or "eslint-plugin-bar"

        // Check if it's @scope/eslint-plugin or @scope/eslint-plugin-something
        if rest == "eslint-plugin" {
            // @foo/eslint-plugin -> @foo
            return Cow::Borrowed(scope);
        } else if let Some(suffix) = rest.strip_prefix("eslint-plugin-") {
            // @foo/eslint-plugin-bar -> @foo/bar
            return Cow::Owned(format!("{scope}/{suffix}"));
        }
    }

    // Handle non-scoped packages
    if let Some(suffix) = plugin_name.strip_prefix("eslint-plugin-") {
        // eslint-plugin-foo -> foo
        return Cow::Borrowed(suffix);
    }

    // No normalization needed
    Cow::Borrowed(plugin_name)
}

bitflags! {
    // NOTE: may be increased to a u32 if needed
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LintPlugins: u16 {
        /// Not really a plugin. Included for completeness.
        const ESLINT = 0;
        /// `eslint-plugin-react`, plus `eslint-plugin-react-hooks`
        const REACT = 1 << 0;
        /// `eslint-plugin-unicorn`
        const UNICORN = 1 << 1;
        /// `@typescript-eslint/eslint-plugin`
        const TYPESCRIPT = 1 << 2;
        /// Custom rules for Oxc, plus some ported from Deepscan
        const OXC = 1 << 3;
        /// `eslint-plugin-import`
        const IMPORT = 1 << 4;
        /// `eslint-plugin-jsdoc`
        const JSDOC = 1 << 5;
        /// `eslint-plugin-jest`
        const JEST = 1 << 6;
        /// `eslint-plugin-vitest`
        const VITEST = 1 << 7;
        /// `eslint-plugin-jsx-a11y`
        const JSX_A11Y = 1 << 8;
        /// `eslint-plugin-next`
        const NEXTJS = 1 << 9;
        /// `eslint-plugin-react-perf`
        const REACT_PERF = 1 << 10;
        /// `eslint-plugin-promise`
        const PROMISE = 1 << 11;
        /// `eslint-plugin-node`
        const NODE = 1 << 12;
        /// `eslint-plugin-vue`
        const VUE = 1 << 13;
    }
}

impl Default for LintPlugins {
    #[inline]
    fn default() -> Self {
        // update `oxc_linter::table::RuleTable` when changing the defaults
        LintPlugins::UNICORN | LintPlugins::TYPESCRIPT | LintPlugins::OXC
    }
}

impl LintPlugins {
    /// Returns `true` if the Vitest plugin is enabled.
    #[inline]
    pub fn has_vitest(self) -> bool {
        self.contains(LintPlugins::VITEST)
    }

    /// Returns `true` if the Jest plugin is enabled.
    #[inline]
    pub fn has_jest(self) -> bool {
        self.contains(LintPlugins::JEST)
    }

    /// Returns `true` if Jest or Vitest plugins are enabled.
    #[inline]
    pub fn has_test(self) -> bool {
        self.intersects(LintPlugins::JEST | LintPlugins::VITEST)
    }

    /// Returns `true` if the import plugin is enabled.
    #[inline]
    pub fn has_import(self) -> bool {
        self.contains(LintPlugins::IMPORT)
    }
}

impl TryFrom<&str> for LintPlugins {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Normalize plugin name first to handle eslint-plugin-* naming
        let normalized = normalize_plugin_name(value);
        let value = normalized.as_ref();

        match value {
            "react" | "react-hooks" | "react_hooks" => Ok(LintPlugins::REACT),
            "unicorn" => Ok(LintPlugins::UNICORN),
            "typescript" | "typescript-eslint" | "typescript_eslint" | "@typescript-eslint" => {
                Ok(LintPlugins::TYPESCRIPT)
            }
            // deepscan for backwards compatibility. Those rules have been moved into oxc
            "oxc" | "deepscan" => Ok(LintPlugins::OXC),
            // import-x has the same rules but better performance
            "import" | "import-x" => Ok(LintPlugins::IMPORT),
            "jsdoc" => Ok(LintPlugins::JSDOC),
            "jest" => Ok(LintPlugins::JEST),
            "vitest" => Ok(LintPlugins::VITEST),
            "jsx-a11y" | "jsx_a11y" => Ok(LintPlugins::JSX_A11Y),
            "nextjs" => Ok(LintPlugins::NEXTJS),
            "react-perf" | "react_perf" => Ok(LintPlugins::REACT_PERF),
            "promise" => Ok(LintPlugins::PROMISE),
            "node" => Ok(LintPlugins::NODE),
            "vue" => Ok(LintPlugins::VUE),
            // "eslint" is not really a plugin, so it's 'empty'. This has the added benefit of
            // making it the default value.
            "eslint" => Ok(LintPlugins::ESLINT),
            _ => Err(()),
        }
    }
}

impl From<LintPlugins> for &'static str {
    fn from(value: LintPlugins) -> Self {
        match value {
            LintPlugins::REACT => "react",
            LintPlugins::UNICORN => "unicorn",
            LintPlugins::TYPESCRIPT => "typescript",
            LintPlugins::OXC => "oxc",
            LintPlugins::IMPORT => "import",
            LintPlugins::JSDOC => "jsdoc",
            LintPlugins::JEST => "jest",
            LintPlugins::VITEST => "vitest",
            LintPlugins::JSX_A11Y => "jsx-a11y",
            LintPlugins::NEXTJS => "nextjs",
            LintPlugins::REACT_PERF => "react-perf",
            LintPlugins::PROMISE => "promise",
            LintPlugins::NODE => "node",
            LintPlugins::VUE => "vue",
            _ => "",
        }
    }
}

impl<'de> Deserialize<'de> for LintPlugins {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let plugin_names: Vec<String> = Vec::deserialize(deserializer)?;

        let mut lint_plugins = LintPlugins::empty();

        for plugin in &plugin_names {
            if let Ok(plugin_flag) = LintPlugins::try_from(plugin.as_str()) {
                lint_plugins |= plugin_flag;
            } else {
                return Err(serde::de::Error::custom(format!("Unknown plugin: '{plugin}'.")));
            }
        }

        Ok(lint_plugins)
    }
}

impl Serialize for LintPlugins {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.bits().count_ones() as usize))?;

        for flag in LintPlugins::all().iter() {
            if self.contains(flag) {
                let s: &'static str = flag.into();
                if !s.is_empty() {
                    seq.serialize_element(s)?;
                }
            }
        }

        seq.end()
    }
}

impl JsonSchema for LintPlugins {
    fn schema_name() -> String {
        "LintPlugins".to_string()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("LintPlugins")
    }

    fn json_schema(r#gen: &mut SchemaGenerator) -> Schema {
        #[derive(JsonSchema)]
        #[serde(rename_all = "kebab-case")]
        #[expect(dead_code)]
        enum LintPluginOptionsSchema {
            Eslint,
            React,
            Unicorn,
            Typescript,
            Oxc,
            Import,
            Jsdoc,
            Jest,
            Vitest,
            JsxA11y,
            Nextjs,
            ReactPerf,
            Promise,
            Node,
            Vue,
        }

        let enum_schema = r#gen.subschema_for::<LintPluginOptionsSchema>();

        let item_schema = Schema::Object(schemars::schema::SchemaObject {
            subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
                any_of: Some(vec![enum_schema]),
                ..Default::default()
            })),
            ..Default::default()
        });

        Schema::Object(schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::SingleOrVec::Single(Box::new(
                schemars::schema::InstanceType::Array,
            ))),
            array: Some(Box::new(schemars::schema::ArrayValidation {
                items: Some(schemars::schema::SingleOrVec::Single(Box::new(item_schema))),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugins_default() {
        let default = LintPlugins::default();
        assert_eq!(default, LintPlugins::UNICORN | LintPlugins::TYPESCRIPT | LintPlugins::OXC);
        assert!(!default.contains(LintPlugins::REACT));
    }

    #[test]
    fn test_plugin_from_str() {
        assert_eq!(LintPlugins::try_from("react"), Ok(LintPlugins::REACT));
        assert_eq!(LintPlugins::try_from("typescript-eslint"), Ok(LintPlugins::TYPESCRIPT));
        assert_eq!(LintPlugins::try_from("deepscan"), Ok(LintPlugins::OXC));
        assert_eq!(LintPlugins::try_from("unknown"), Err(()));
    }

    #[test]
    fn test_plugin_to_str() {
        assert_eq!(<&'static str>::from(LintPlugins::REACT), "react");
        assert_eq!(<&'static str>::from(LintPlugins::JEST), "jest");
        assert_eq!(<&'static str>::from(LintPlugins::ESLINT), "");
    }

    #[test]
    fn test_has_helpers() {
        let plugins = LintPlugins::JEST | LintPlugins::IMPORT;
        assert!(plugins.has_jest());
        assert!(!plugins.has_vitest());
        assert!(plugins.has_test());
        assert!(plugins.has_import());
    }

    #[test]
    fn test_serialize_lint_plugins() {
        let plugins = LintPlugins::OXC | LintPlugins::REACT;
        let json = serde_json::to_string(&plugins).unwrap();
        let mut parsed = serde_json::from_str::<Vec<String>>(&json).unwrap();
        parsed.sort_unstable();
        assert_eq!(parsed, ["oxc", "react"]);
    }

    #[test]
    fn test_deserialize_lint_plugins() {
        // `eslint` is ignored
        let json = r#"["react", "eslint", "jsdoc"]"#;
        let plugins: LintPlugins = serde_json::from_str(json).unwrap();
        assert_eq!(plugins, LintPlugins::REACT | LintPlugins::JSDOC);
    }

    #[test]
    fn test_deserialize_lint_plugins_with_unknown_plugin() {
        let json = r#"["react", "not-a-real-plugin"]"#;
        let result = serde_json::from_str::<LintPlugins>(json);

        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert_eq!(error, "Unknown plugin: 'not-a-real-plugin'.");
    }

    #[test]
    fn test_plugin_normalization() {
        // Test eslint-plugin- prefix normalization
        assert_eq!(LintPlugins::try_from("eslint-plugin-react"), Ok(LintPlugins::REACT));
        assert_eq!(LintPlugins::try_from("eslint-plugin-unicorn"), Ok(LintPlugins::UNICORN));
        assert_eq!(LintPlugins::try_from("eslint-plugin-import"), Ok(LintPlugins::IMPORT));
        assert_eq!(LintPlugins::try_from("eslint-plugin-jest"), Ok(LintPlugins::JEST));

        // Test @scope/eslint-plugin normalization
        assert_eq!(
            LintPlugins::try_from("@typescript-eslint/eslint-plugin"),
            Ok(LintPlugins::TYPESCRIPT)
        );

        // Verify existing plugin names still work
        assert_eq!(LintPlugins::try_from("react"), Ok(LintPlugins::REACT));
        assert_eq!(LintPlugins::try_from("unicorn"), Ok(LintPlugins::UNICORN));
        assert_eq!(LintPlugins::try_from("@typescript-eslint"), Ok(LintPlugins::TYPESCRIPT));
    }

    #[test]
    fn test_normalize_plugin_name() {
        use super::normalize_plugin_name;

        // Test eslint-plugin- prefix stripping
        assert_eq!(normalize_plugin_name("eslint-plugin-foo"), "foo");
        assert_eq!(normalize_plugin_name("eslint-plugin-react"), "react");

        // Test @scope/eslint-plugin suffix stripping
        assert_eq!(normalize_plugin_name("@foo/eslint-plugin"), "@foo");
        assert_eq!(normalize_plugin_name("@bar/eslint-plugin"), "@bar");

        // Test @scope/eslint-plugin-name normalization
        assert_eq!(normalize_plugin_name("@foo/eslint-plugin-bar"), "@foo/bar");

        // Test no change for already normalized names
        assert_eq!(normalize_plugin_name("react"), "react");
        assert_eq!(normalize_plugin_name("@typescript-eslint"), "@typescript-eslint");
    }
}
