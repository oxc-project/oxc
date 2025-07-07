use bitflags::bitflags;
use schemars::{JsonSchema, r#gen::SchemaGenerator, schema::Schema};
use serde::{
    Deserialize, Serialize,
    de::{self, Deserializer},
    ser::Serializer,
};

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
    }
}

impl Default for BuiltinLintPlugins {
    #[inline]
    fn default() -> Self {
        BuiltinLintPlugins::UNICORN | BuiltinLintPlugins::TYPESCRIPT | BuiltinLintPlugins::OXC
    }
}

impl BuiltinLintPlugins {
    /// Returns `true` if the Vitest plugin is enabled.
    #[inline]
    pub fn has_vitest(self) -> bool {
        self.contains(BuiltinLintPlugins::VITEST)
    }

    /// Returns `true` if the Jest plugin is enabled.
    #[inline]
    pub fn has_jest(self) -> bool {
        self.contains(BuiltinLintPlugins::JEST)
    }

    /// Returns `true` if Jest or Vitest plugins are enabled.
    #[inline]
    pub fn has_test(self) -> bool {
        self.intersects(BuiltinLintPlugins::JEST.union(BuiltinLintPlugins::VITEST))
    }

    /// Returns `true` if the import plugin is enabled.
    #[inline]
    pub fn has_import(self) -> bool {
        self.contains(BuiltinLintPlugins::IMPORT)
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
            _ => "",
        }
    }
}

impl<S: AsRef<str>> FromIterator<S> for BuiltinLintPlugins {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        iter.into_iter()
            .map(|plugin| plugin.as_ref().into())
            .fold(BuiltinLintPlugins::empty(), BuiltinLintPlugins::union)
    }
}

impl<'de> Deserialize<'de> for BuiltinLintPlugins {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct LintPluginsVisitor;
        impl<'de> de::Visitor<'de> for LintPluginsVisitor {
            type Value = BuiltinLintPlugins;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a list of plugin names")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(BuiltinLintPlugins::from(value))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(BuiltinLintPlugins::from(v.as_str()))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut plugins = BuiltinLintPlugins::empty();
                loop {
                    // serde_json::from_str will provide an &str, while
                    // serde_json::from_value provides a String. The former is
                    // used in almost all cases, but the latter is more
                    // convenient for test cases.
                    match seq.next_element::<String>() {
                        Ok(Some(next)) => {
                            plugins |= next.as_str().into();
                        }
                        Ok(None) => break,
                        Err(_) => match seq.next_element::<&str>() {
                            Ok(Some(next)) => {
                                plugins |= next.into();
                            }
                            Ok(None) | Err(_) => break,
                        },
                    }
                }

                Ok(plugins)
            }
        }

        deserializer.deserialize_any(LintPluginsVisitor)
    }
}

impl Serialize for BuiltinLintPlugins {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let vec: Vec<&str> = self.iter().map(Into::into).collect();
        vec.serialize(serializer)
    }
}

impl JsonSchema for BuiltinLintPlugins {
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
        }
        r#gen.subschema_for::<Vec<LintPluginOptionsSchema>>()
    }
}
