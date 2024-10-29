use bitflags::bitflags;
use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};
use serde::{
    de::{self, Deserializer},
    ser::Serializer,
    Deserialize, Serialize,
};

bitflags! {
    // NOTE: may be increased to a u32 if needed
    #[derive(Debug, Clone, Copy, PartialEq, Hash)]
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
        /// Custom security rules made by the Oxc team
        const SECURITY = 1 << 13;
    }
}
impl Default for LintPlugins {
    #[inline]
    fn default() -> Self {
        LintPlugins::REACT | LintPlugins::UNICORN | LintPlugins::TYPESCRIPT | LintPlugins::OXC
    }
}

impl From<LintPluginOptions> for LintPlugins {
    fn from(options: LintPluginOptions) -> Self {
        let mut plugins = LintPlugins::empty();
        plugins.set(LintPlugins::REACT, options.react);
        plugins.set(LintPlugins::UNICORN, options.unicorn);
        plugins.set(LintPlugins::TYPESCRIPT, options.typescript);
        plugins.set(LintPlugins::OXC, options.oxc);
        plugins.set(LintPlugins::IMPORT, options.import);
        plugins.set(LintPlugins::JSDOC, options.jsdoc);
        plugins.set(LintPlugins::JEST, options.jest);
        plugins.set(LintPlugins::VITEST, options.vitest);
        plugins.set(LintPlugins::JSX_A11Y, options.jsx_a11y);
        plugins.set(LintPlugins::NEXTJS, options.nextjs);
        plugins.set(LintPlugins::REACT_PERF, options.react_perf);
        plugins.set(LintPlugins::PROMISE, options.promise);
        plugins.set(LintPlugins::NODE, options.node);
        plugins.set(LintPlugins::SECURITY, options.security);
        plugins
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
        self.intersects(LintPlugins::JEST.union(LintPlugins::VITEST))
    }

    /// Returns `true` if the import plugin is enabled.
    #[inline]
    pub fn has_import(self) -> bool {
        self.contains(LintPlugins::IMPORT)
    }
}

impl From<&str> for LintPlugins {
    fn from(value: &str) -> Self {
        match value {
            "react" | "react-hooks" | "react_hooks" => LintPlugins::REACT,
            "unicorn" => LintPlugins::UNICORN,
            "typescript" | "typescript-eslint" | "typescript_eslint" | "@typescript-eslint" => {
                LintPlugins::TYPESCRIPT
            }
            // deepscan for backwards compatibility. Those rules have been moved into oxc
            "oxc" | "deepscan" => LintPlugins::OXC,
            "import" => LintPlugins::IMPORT,
            "jsdoc" => LintPlugins::JSDOC,
            "jest" => LintPlugins::JEST,
            "vitest" => LintPlugins::VITEST,
            "jsx-a11y" | "jsx_a11y" => LintPlugins::JSX_A11Y,
            "nextjs" => LintPlugins::NEXTJS,
            "react-perf" | "react_perf" => LintPlugins::REACT_PERF,
            "promise" => LintPlugins::PROMISE,
            "node" => LintPlugins::NODE,
            "security" | "oxc-security" => LintPlugins::SECURITY,
            // "eslint" is not really a plugin, so it's 'empty'. This has the added benefit of
            // making it the default value.
            _ => LintPlugins::empty(),
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
            LintPlugins::SECURITY => "security",
            _ => "",
        }
    }
}

impl<S: AsRef<str>> FromIterator<S> for LintPlugins {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        iter.into_iter()
            .map(|plugin| plugin.as_ref().into())
            .fold(LintPlugins::default(), LintPlugins::union)
    }
}

impl<'de> Deserialize<'de> for LintPlugins {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct LintPluginsVisitor;
        impl<'de> de::Visitor<'de> for LintPluginsVisitor {
            type Value = LintPlugins;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a list of plugin names")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(LintPlugins::from(value))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(LintPlugins::from(v.as_str()))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut plugins = LintPlugins::default();
                while let Some(plugin) = seq.next_element::<&str>()? {
                    plugins |= plugin.into();
                }
                Ok(plugins)
            }
        }

        deserializer.deserialize_any(LintPluginsVisitor)
    }
}

impl Serialize for LintPlugins {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let vec: Vec<&str> = self.iter().map(Into::into).collect();
        vec.serialize(serializer)
    }
}

impl JsonSchema for LintPlugins {
    fn schema_name() -> String {
        "LintPlugins".to_string()
    }
    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("LintPlugins")
    }
    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        gen.subschema_for::<Vec<&str>>()
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct LintPluginOptions {
    /// On by default.
    pub react: bool,
    /// On by default.
    pub unicorn: bool,
    /// On by default.
    pub typescript: bool,
    /// On by default.
    pub oxc: bool,
    pub import: bool,
    pub jsdoc: bool,
    pub jest: bool,
    pub vitest: bool,
    pub jsx_a11y: bool,
    pub nextjs: bool,
    pub react_perf: bool,
    pub promise: bool,
    pub node: bool,
    pub security: bool,
}

impl Default for LintPluginOptions {
    fn default() -> Self {
        Self {
            react: true,
            unicorn: true,
            typescript: true,
            oxc: true,
            import: false,
            jsdoc: false,
            jest: false,
            vitest: false,
            jsx_a11y: false,
            nextjs: false,
            react_perf: false,
            promise: false,
            node: false,
            security: false,
        }
    }
}

impl LintPluginOptions {
    /// Create a new instance with all plugins disabled.
    #[must_use]
    pub fn none() -> Self {
        Self {
            react: false,
            unicorn: false,
            typescript: false,
            oxc: false,
            import: false,
            jsdoc: false,
            jest: false,
            vitest: false,
            jsx_a11y: false,
            nextjs: false,
            react_perf: false,
            promise: false,
            node: false,
            security: false,
        }
    }

    /// Create a new instance with all plugins enabled.
    #[must_use]
    pub fn all() -> Self {
        Self {
            react: true,
            unicorn: true,
            typescript: true,
            oxc: true,
            import: true,
            jsdoc: true,
            jest: true,
            vitest: true,
            jsx_a11y: true,
            nextjs: true,
            react_perf: true,
            promise: true,
            node: true,
            security: true,
        }
    }
}

impl<S: AsRef<str>> FromIterator<(S, bool)> for LintPluginOptions {
    fn from_iter<I: IntoIterator<Item = (S, bool)>>(iter: I) -> Self {
        let mut options = Self::default();
        for (s, enabled) in iter {
            let flags = LintPlugins::from(s.as_ref());
            match flags {
                LintPlugins::REACT => options.react = enabled,
                LintPlugins::UNICORN => options.unicorn = enabled,
                LintPlugins::TYPESCRIPT => options.typescript = enabled,
                LintPlugins::OXC => options.oxc = enabled,
                LintPlugins::IMPORT => options.import = enabled,
                LintPlugins::JSDOC => options.jsdoc = enabled,
                LintPlugins::JEST => options.jest = enabled,
                LintPlugins::VITEST => options.vitest = enabled,
                LintPlugins::JSX_A11Y => options.jsx_a11y = enabled,
                LintPlugins::NEXTJS => options.nextjs = enabled,
                LintPlugins::REACT_PERF => options.react_perf = enabled,
                LintPlugins::PROMISE => options.promise = enabled,
                LintPlugins::NODE => options.node = enabled,
                LintPlugins::SECURITY => options.security = enabled,
                _ => {} // ignored
            }
        }

        options
    }
}

impl<'s> FromIterator<&'s str> for LintPluginOptions {
    fn from_iter<T: IntoIterator<Item = &'s str>>(iter: T) -> Self {
        iter.into_iter().map(|s| (s, true)).collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    impl PartialEq for LintPluginOptions {
        fn eq(&self, other: &Self) -> bool {
            self.react == other.react
                && self.unicorn == other.unicorn
                && self.typescript == other.typescript
                && self.oxc == other.oxc
                && self.import == other.import
                && self.jsdoc == other.jsdoc
                && self.jest == other.jest
                && self.vitest == other.vitest
                && self.jsx_a11y == other.jsx_a11y
                && self.nextjs == other.nextjs
                && self.react_perf == other.react_perf
                && self.promise == other.promise
                && self.node == other.node
                && self.security == other.security
        }
    }

    #[test]
    fn test_default_conversion() {
        let plugins = LintPlugins::default();
        let options = LintPluginOptions::default();
        assert_eq!(LintPlugins::from(options), plugins);
    }

    #[test]
    fn test_collect_empty() {
        let empty: &[&str] = &[];
        let plugins: LintPluginOptions = empty.iter().copied().collect();
        assert_eq!(plugins, LintPluginOptions::default());

        let empty: Vec<(String, bool)> = vec![];
        let plugins: LintPluginOptions = empty.into_iter().collect();
        assert_eq!(plugins, LintPluginOptions::default());
    }

    #[test]
    fn test_collect_strings() {
        let enabled = vec!["react", "typescript", "jest"];
        let plugins: LintPluginOptions = enabled.into_iter().collect();
        let expected = LintPluginOptions {
            react: true,
            unicorn: true,
            typescript: true,
            oxc: true,
            import: false,
            jsdoc: false,
            jest: true,
            vitest: false,
            jsx_a11y: false,
            nextjs: false,
            react_perf: false,
            promise: false,
            node: false,
            security: false,
        };
        assert_eq!(plugins, expected);
    }
}
