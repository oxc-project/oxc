use oxc_syntax::es_target::ESTarget;

#[derive(Debug, Clone, Copy)]
pub struct CompressOptions {
    /// Enable features that are targeted above.
    ///
    /// e.g.
    ///
    /// * catch optional binding when >= es2019
    pub target: ESTarget,

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
        Self { target: ESTarget::ESNext, drop_debugger: true, drop_console: true }
    }

    pub fn all_false() -> Self {
        Self { target: ESTarget::ESNext, drop_debugger: false, drop_console: false }
    }
}
