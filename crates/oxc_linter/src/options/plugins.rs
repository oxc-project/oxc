#[derive(Debug, Clone)]
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
