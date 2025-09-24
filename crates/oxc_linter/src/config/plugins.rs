use bitflags::bitflags;
use rustc_hash::FxHashSet;
use schemars::{JsonSchema, r#gen::SchemaGenerator, schema::Schema};
use serde::{Deserialize, Serialize, de::Deserializer, ser::Serializer};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LintPlugins {
    pub builtin: BuiltinLintPlugins,
    pub external: FxHashSet<String>,
}

impl LintPlugins {
    pub fn new(builtin: BuiltinLintPlugins, external: FxHashSet<String>) -> Self {
        Self { builtin, external }
    }
}

bitflags! {
    // NOTE: may be increased to a u32 if needed
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BuiltinLintPlugins: u16 {
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
        /// `eslint-plugin-regex`
        const REGEX = 1 << 13;
        /// `eslint-plugin-vue`
        const VUE = 1 << 14;
    }
}

impl Default for BuiltinLintPlugins {
    #[inline]
    fn default() -> Self {
        // update `oxc_linter::table::RuleTable` when changing the defaults
        BuiltinLintPlugins::UNICORN | BuiltinLintPlugins::TYPESCRIPT | BuiltinLintPlugins::OXC
    }
}

impl From<BuiltinLintPlugins> for LintPlugins {
    fn from(builtin: BuiltinLintPlugins) -> Self {
        LintPlugins { builtin, external: FxHashSet::default() }
    }
}

impl LintPlugins {
    /// Returns `true` if the Vitest plugin is enabled.
    #[inline]
    pub fn has_vitest(&self) -> bool {
        self.builtin.contains(BuiltinLintPlugins::VITEST)
    }

    /// Returns `true` if the Jest plugin is enabled.
    #[inline]
    pub fn has_jest(&self) -> bool {
        self.builtin.contains(BuiltinLintPlugins::JEST)
    }

    /// Returns `true` if Jest or Vitest plugins are enabled.
    #[inline]
    pub fn has_test(&self) -> bool {
        self.builtin.intersects(BuiltinLintPlugins::JEST.union(BuiltinLintPlugins::VITEST))
    }

    /// Returns `true` if the import plugin is enabled.
    #[inline]
    pub fn has_import(&self) -> bool {
        self.builtin.contains(BuiltinLintPlugins::IMPORT)
    }

    /// Returns the union of two `LintPlugins` sets.
    #[must_use]
    pub fn union(&self, other: &LintPlugins) -> LintPlugins {
        let builtin = self.builtin | other.builtin;
        let mut external = self.external.clone();
        external.extend(other.external.iter().cloned());
        LintPlugins { builtin, external }
    }
}

impl From<&str> for BuiltinLintPlugins {
    fn from(value: &str) -> Self {
        match value {
            "react" | "react-hooks" | "react_hooks" => BuiltinLintPlugins::REACT,
            "unicorn" => BuiltinLintPlugins::UNICORN,
            "typescript" | "typescript-eslint" | "typescript_eslint" | "@typescript-eslint" => {
                BuiltinLintPlugins::TYPESCRIPT
            }
            // deepscan for backwards compatibility. Those rules have been moved into oxc
            "oxc" | "deepscan" => BuiltinLintPlugins::OXC,
            // import-x has the same rules but better performance
            "import" | "import-x" => BuiltinLintPlugins::IMPORT,
            "jsdoc" => BuiltinLintPlugins::JSDOC,
            "jest" => BuiltinLintPlugins::JEST,
            "vitest" => BuiltinLintPlugins::VITEST,
            "jsx-a11y" | "jsx_a11y" => BuiltinLintPlugins::JSX_A11Y,
            "nextjs" => BuiltinLintPlugins::NEXTJS,
            "react-perf" | "react_perf" => BuiltinLintPlugins::REACT_PERF,
            "promise" => BuiltinLintPlugins::PROMISE,
            "node" => BuiltinLintPlugins::NODE,
            "regex" => BuiltinLintPlugins::REGEX,
            "vue" => BuiltinLintPlugins::VUE,
            // "eslint" is not really a plugin, so it's 'empty'. This has the added benefit of
            // making it the default value.
            _ => BuiltinLintPlugins::empty(),
        }
    }
}

impl From<BuiltinLintPlugins> for &'static str {
    fn from(value: BuiltinLintPlugins) -> Self {
        match value {
            BuiltinLintPlugins::REACT => "react",
            BuiltinLintPlugins::UNICORN => "unicorn",
            BuiltinLintPlugins::TYPESCRIPT => "typescript",
            BuiltinLintPlugins::OXC => "oxc",
            BuiltinLintPlugins::IMPORT => "import",
            BuiltinLintPlugins::JSDOC => "jsdoc",
            BuiltinLintPlugins::JEST => "jest",
            BuiltinLintPlugins::VITEST => "vitest",
            BuiltinLintPlugins::JSX_A11Y => "jsx-a11y",
            BuiltinLintPlugins::NEXTJS => "nextjs",
            BuiltinLintPlugins::REACT_PERF => "react-perf",
            BuiltinLintPlugins::PROMISE => "promise",
            BuiltinLintPlugins::NODE => "node",
            BuiltinLintPlugins::REGEX => "regex",
            BuiltinLintPlugins::VUE => "vue",
            _ => "",
        }
    }
}

impl<S: AsRef<str>> FromIterator<S> for LintPlugins {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let mut builtin = BuiltinLintPlugins::empty();
        let mut external = FxHashSet::default();

        for plugin in iter {
            let plugin_str = plugin.as_ref();
            let plugin_flag: BuiltinLintPlugins = BuiltinLintPlugins::from(plugin_str);
            if plugin_flag == BuiltinLintPlugins::empty() && plugin_str != "eslint" {
                external.insert(plugin_str.to_string());
            } else {
                builtin |= plugin_flag;
            }
        }

        Self { builtin, external }
    }
}

impl<'de> Deserialize<'de> for LintPlugins {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let plugins: Vec<String> = Vec::deserialize(deserializer)?;
        Ok(plugins.into_iter().collect())
    }
}

impl Serialize for LintPlugins {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;

        let mut seq = serializer
            .serialize_seq(Some(self.builtin.bits().count_ones() as usize + self.external.len()))?;

        for flag in BuiltinLintPlugins::all().iter() {
            if self.builtin.contains(flag) {
                let s: &'static str = flag.into();
                if !s.is_empty() {
                    seq.serialize_element(s)?;
                }
            }
        }

        for ext in &self.external {
            seq.serialize_element(ext)?;
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
            Regex,
            Vue,
        }

        let enum_schema = r#gen.subschema_for::<LintPluginOptionsSchema>();

        let string_schema = Schema::Object(schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::SingleOrVec::Single(Box::new(
                schemars::schema::InstanceType::String,
            ))),
            ..Default::default()
        });

        let item_schema = Schema::Object(schemars::schema::SchemaObject {
            subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
                any_of: Some(vec![enum_schema, string_schema]),
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
    use serde_json::{from_str, to_string};

    #[test]
    fn test_builtin_plugins_default() {
        let default = BuiltinLintPlugins::default();
        assert!(default.contains(BuiltinLintPlugins::UNICORN));
        assert!(default.contains(BuiltinLintPlugins::TYPESCRIPT));
        assert!(default.contains(BuiltinLintPlugins::OXC));
        assert!(!default.contains(BuiltinLintPlugins::REACT));
    }

    #[test]
    fn test_builtin_plugin_from_str() {
        assert_eq!(BuiltinLintPlugins::from("react"), BuiltinLintPlugins::REACT);
        assert_eq!(BuiltinLintPlugins::from("typescript-eslint"), BuiltinLintPlugins::TYPESCRIPT);
        assert_eq!(BuiltinLintPlugins::from("deepscan"), BuiltinLintPlugins::OXC);
        assert_eq!(BuiltinLintPlugins::from("unknown"), BuiltinLintPlugins::empty());
    }

    #[test]
    fn test_builtin_plugin_to_str() {
        assert_eq!(<&'static str>::from(BuiltinLintPlugins::REACT), "react");
        assert_eq!(<&'static str>::from(BuiltinLintPlugins::JEST), "jest");
        assert_eq!(<&'static str>::from(BuiltinLintPlugins::ESLINT), "");
    }

    #[test]
    fn test_has_helpers() {
        let plugins: LintPlugins = (BuiltinLintPlugins::JEST | BuiltinLintPlugins::IMPORT).into();
        assert!(plugins.has_jest());
        assert!(!plugins.has_vitest());
        assert!(plugins.has_test());
        assert!(plugins.has_import());
    }

    #[test]
    fn test_lint_plugins_from_iter() {
        let input = vec!["react", "some-custom", "oxc", "import-x"];
        let plugins: LintPlugins = input.into_iter().collect();

        assert!(plugins.builtin.contains(BuiltinLintPlugins::REACT));
        assert!(plugins.builtin.contains(BuiltinLintPlugins::OXC));
        assert!(plugins.builtin.contains(BuiltinLintPlugins::IMPORT));
        assert!(plugins.external.contains("some-custom"));
    }

    #[test]
    fn test_serialize_lint_plugins() {
        let plugins: LintPlugins = vec!["react", "custom-plugin"].into_iter().collect();
        let json = to_string(&plugins).unwrap();
        let parsed = serde_json::from_str::<Vec<String>>(&json).unwrap();

        assert!(parsed.contains(&"react".to_string()));
        assert!(parsed.contains(&"custom-plugin".to_string()));
    }

    #[test]
    fn test_deserialize_lint_plugins() {
        let json = r#"["react", "jsdoc", "custom-foo"]"#;
        let plugins: LintPlugins = from_str(json).unwrap();

        assert!(plugins.builtin.contains(BuiltinLintPlugins::REACT));
        assert!(plugins.builtin.contains(BuiltinLintPlugins::JSDOC));
        assert!(plugins.external.contains("custom-foo"));
    }
}
