#[derive(Debug, Clone, Copy)]
pub struct CompressOptions {
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
        Self { drop_console: false, ..Self::all_true() }
    }
}

impl CompressOptions {
    pub fn all_true() -> Self {
        Self { drop_debugger: true, drop_console: true }
    }

    pub fn all_false() -> Self {
        Self { drop_debugger: false, drop_console: false }
    }
}
