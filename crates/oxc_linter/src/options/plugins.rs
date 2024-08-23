use bitflags::bitflags;
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Hash)]
    pub(crate) struct LintPlugins: u32 {
        const REACT = 1 << 0;
        const UNICORN = 1 << 1;
        const TYPESCRIPT = 1 << 2;
        const OXC = 1 << 3;
        const IMPORT = 1 << 4;
        const JSDOC = 1 << 5;
        const JEST = 1 << 6;
        const VITEST = 1 << 7;
        const JSX_A11Y = 1 << 8;
        const NEXTJS = 1 << 9;
        const REACT_PERF = 1 << 10;
        const PROMISE = 1 << 11;
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
        plugins
    }
}

impl LintPlugins {
    /// Returns `true` if the Vitest plugin is enabled.
    #[inline]
    pub fn has_vitest(self) -> bool {
        self.contains(LintPlugins::VITEST)
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

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
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
        }
    }
}

impl<S: AsRef<str>> FromIterator<(S, bool)> for LintPluginOptions {
    fn from_iter<I: IntoIterator<Item = (S, bool)>>(iter: I) -> Self {
        let mut options = Self::default();
        for (s, enabled) in iter {
            match s.as_ref() {
                "react" | "react-hooks" => options.react = enabled,
                "unicorn" => options.unicorn = enabled,
                "typescript" | "typescript-eslint" | "@typescript-eslint" => {
                    options.typescript = enabled;
                }
                // deepscan for backwards compatibility. Those rules have been
                // moved into oxc
                "oxc" | "deepscan" => options.oxc = enabled,
                "import" => options.import = enabled,
                "jsdoc" => options.jsdoc = enabled,
                "jest" => options.jest = enabled,
                "vitest" => options.vitest = enabled,
                "jsx-a11y" => options.jsx_a11y = enabled,
                "nextjs" => options.nextjs = enabled,
                "react-perf" => options.react_perf = enabled,
                "promise" => options.promise = enabled,
                _ => { /* ignored */ }
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
        };
        assert_eq!(plugins, expected);
    }
}
