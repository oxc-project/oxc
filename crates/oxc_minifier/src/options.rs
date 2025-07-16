use oxc_syntax::es_target::ESTarget;

pub use oxc_ecmascript::side_effects::PropertyReadSideEffects;

#[derive(Debug, Clone)]
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

    /// Drop unreferenced functions and variables.
    pub unused: CompressOptionsUnused,

    /// Keep function / class names.
    pub keep_names: CompressOptionsKeepNames,

    /// Treeshake Options .
    /// <https://rollupjs.org/configuration-options/#treeshake>
    pub treeshake: TreeShakeOptions,
}

#[expect(clippy::derivable_impls)]
impl Default for CompressOptions {
    fn default() -> Self {
        Self { drop_console: false, ..Self::smallest() }
    }
}

impl CompressOptions {
    pub fn smallest() -> Self {
        Self {
            target: ESTarget::ESNext,
            keep_names: CompressOptionsKeepNames::all_false(),
            drop_debugger: true,
            drop_console: true,
            unused: CompressOptionsUnused::Remove,
            treeshake: TreeShakeOptions::default(),
        }
    }

    pub fn safest() -> Self {
        Self {
            target: ESTarget::ESNext,
            keep_names: CompressOptionsKeepNames::all_true(),
            drop_debugger: false,
            drop_console: false,
            unused: CompressOptionsUnused::Keep,
            treeshake: TreeShakeOptions::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum CompressOptionsUnused {
    #[default]
    Remove,
    KeepAssign,
    Keep,
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

#[derive(Debug, Clone)]
pub struct TreeShakeOptions {
    /// Whether to respect the pure annotations.
    ///
    /// Pure annotations are the comments that marks that a expression is pure.
    /// For example, `/* @__PURE__ */`, `/* #__NO_SIDE_EFFECTS__ */`.
    ///
    /// <https://rollupjs.org/configuration-options/#treeshake-annotations>
    ///
    /// Default `true`
    pub annotations: bool,

    /// Whether to treat this function call as pure.
    ///
    /// This function is called for normal function calls, new calls, and
    /// tagged template calls (`foo()`, `new Foo()`, ``foo`b` ``).
    ///
    /// <https://rollupjs.org/configuration-options/#treeshake-manualpurefunctions>
    pub manual_pure_functions: Vec<String>,

    /// Whether property read accesses have side effects.
    ///
    /// <https://rollupjs.org/configuration-options/#treeshake-propertyreadsideeffects>
    ///
    /// Default [PropertyReadSideEffects::All]
    pub property_read_side_effects: PropertyReadSideEffects,

    /// Whether accessing a global variable has side effects.
    ///
    /// Accessing a non-existing global variable will throw an error.
    /// Global variable may be a getter that has side effects.
    ///
    /// <https://rollupjs.org/configuration-options/#treeshake-unknownglobalsideeffects>
    ///
    /// Default `true`
    pub unknown_global_side_effects: bool,
}

impl Default for TreeShakeOptions {
    fn default() -> Self {
        Self {
            annotations: true,
            manual_pure_functions: vec![],
            property_read_side_effects: PropertyReadSideEffects::default(),
            unknown_global_side_effects: true,
        }
    }
}
