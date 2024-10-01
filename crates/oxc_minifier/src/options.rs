#[derive(Debug, Clone, Copy)]
pub struct CompressOptions {
    pub dead_code_elimination: bool,

    /// Various optimizations for boolean context, for example `!!a ? b : c` → `a ? b : c`.
    ///
    /// Default `true`
    pub booleans: bool,

    /// Remove `debugger;` statements.
    ///
    /// Default `true`
    pub drop_debugger: bool,

    /// Remove `console.*` statements.
    ///
    /// Default `false`
    pub drop_console: bool,

    /// Attempt to evaluate constant expressions
    ///
    /// Default `true`
    pub evaluate: bool,

    /// Join consecutive var statements.
    ///
    /// Default `true`
    pub join_vars: bool,

    /// Optimizations for do, while and for loops when we can statically determine the condition
    ///
    /// Default `true`
    pub loops: bool,

    /// Transforms `typeof foo == "undefined" into `foo === void 0`
    ///
    /// Default `true`
    pub typeofs: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for CompressOptions {
    fn default() -> Self {
        Self { dead_code_elimination: false, drop_console: false, ..Self::all_true() }
    }
}

impl CompressOptions {
    pub fn all_true() -> Self {
        Self {
            dead_code_elimination: false,
            booleans: true,
            drop_debugger: true,
            drop_console: true,
            evaluate: true,
            join_vars: true,
            loops: true,
            typeofs: true,
        }
    }

    pub fn all_false() -> Self {
        Self {
            dead_code_elimination: false,
            booleans: false,
            drop_debugger: false,
            drop_console: false,
            evaluate: false,
            join_vars: false,
            loops: false,
            typeofs: false,
        }
    }

    pub fn dead_code_elimination() -> Self {
        Self { dead_code_elimination: true, ..Self::all_false() }
    }
}
