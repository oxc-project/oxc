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

    /// Keep function / class names.
    pub keep_names: CompressOptionsKeepNames,

    /// Remove `debugger;` statements.
    ///
    /// Default `true`
    pub drop_debugger: bool,

    /// Remove `console.*` statements.
    ///
    /// Default `false`
    pub drop_console: bool,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            target: ESTarget::ESNext,
            keep_names: CompressOptionsKeepNames::default(),
            drop_debugger: true,
            drop_console: false,
        }
    }
}

impl CompressOptions {
    pub fn smallest() -> Self {
        Self {
            target: ESTarget::ESNext,
            keep_names: CompressOptionsKeepNames::all_false(),
            drop_debugger: true,
            drop_console: true,
        }
    }

    pub fn safest() -> Self {
        Self {
            target: ESTarget::ESNext,
            keep_names: CompressOptionsKeepNames::all_true(),
            drop_debugger: false,
            drop_console: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CompressOptionsKeepNames {
    /// Keep function names so that `Function.prototype.name` is preserved.
    ///
    /// This does not guarantee that the `undefined` name is preserved.
    ///
    /// Default `false`
    pub function: bool,

    /// Keep class names so that `Class.prototype.name` is preserved.
    ///
    /// This does not guarantee that the `undefined` name is preserved.
    ///
    /// Default `false`
    pub class: bool,
}

impl CompressOptionsKeepNames {
    pub fn all_false() -> Self {
        Self { function: false, class: false }
    }

    pub fn all_true() -> Self {
        Self { function: true, class: true }
    }

    pub fn function_only() -> Self {
        Self { function: true, class: false }
    }

    pub fn class_only() -> Self {
        Self { function: false, class: true }
    }
}
