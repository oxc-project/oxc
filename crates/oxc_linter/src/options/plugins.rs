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
        }
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
