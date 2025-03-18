use oxc_syntax::es_target::ESTarget;

#[derive(Debug, Clone, Copy)]
pub struct CompressOptions {
    /// Set desired EcmaScript standard version for output.
    ///
    /// e.g.
    ///
    /// * catch optional binding when >= es2019
    /// * `??` operator >=  es2020
    ///
    /// Default `ESTarget::ESNext`
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

#[expect(clippy::derivable_impls)]
impl Default for CompressOptions {
    fn default() -> Self {
        Self { drop_console: false, ..Self::smallest() }
    }
}

impl CompressOptions {
    pub fn smallest() -> Self {
        Self { target: ESTarget::ESNext, drop_debugger: true, drop_console: true }
    }

    pub fn safest() -> Self {
        Self { target: ESTarget::ESNext, drop_debugger: false, drop_console: false }
    }
}
