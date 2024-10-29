#[derive(Debug, Clone, Copy)]
pub struct CompressOptions {
    pub dead_code_elimination: bool,

    /// Remove `debugger;` statements.
    ///
    /// Default `true`
    pub drop_debugger: bool,

    /// Remove `console.*` statements.
    ///
    /// Default `false`
    pub drop_console: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for CompressOptions {
    fn default() -> Self {
        Self { dead_code_elimination: false, drop_console: false, ..Self::all_true() }
    }
}

impl CompressOptions {
    pub fn all_true() -> Self {
        Self { dead_code_elimination: false, drop_debugger: true, drop_console: true }
    }

    pub fn all_false() -> Self {
        Self { dead_code_elimination: false, drop_debugger: false, drop_console: false }
    }

    pub fn dead_code_elimination() -> Self {
        Self { dead_code_elimination: true, ..Self::all_false() }
    }
}
