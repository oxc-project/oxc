use oxc_syntax::es_target::ESTarget;

pub use oxc_ecmascript::side_effects::PropertyReadSideEffects;

/// Configuration options for code compression and minification.
///
/// These options control various aspects of how the minifier transforms
/// and optimizes JavaScript/TypeScript code. Different presets are available
/// for common use cases like maximum compression vs. safer transformations.
#[derive(Debug, Clone)]
pub struct CompressOptions {
    /// Target ECMAScript version for output compatibility.
    ///
    /// This affects which language features can be used in the output:
    /// - ES5: Basic ES5 compatibility
    /// - ES2015+: Modern features like arrow functions, const/let, etc.
    /// - ESNext: Latest language features
    ///
    /// Default: `ESTarget::ESNext`
    pub target: ESTarget,

    /// Whether to remove `debugger;` statements from the output.
    ///
    /// Debugger statements are typically only needed during development
    /// and can be safely removed in production builds.
    ///
    /// Default: `true`
    pub drop_debugger: bool,

    /// Whether to remove `console.*` method calls.
    ///
    /// Console calls can be removed to reduce bundle size and prevent
    /// potential issues in production environments. However, this should
    /// be used carefully as it may affect debugging capabilities.
    ///
    /// Default: `false`
    pub drop_console: bool,

    /// Whether to join consecutive variable declarations.
    ///
    /// Transforms multiple var/let/const statements into single declarations:
    /// ```javascript
    /// // Before
    /// var a = 1;
    /// var b = 2;
    ///
    /// // After
    /// var a = 1, b = 2;
    /// ```
    ///
    /// Default: `true`
    pub join_vars: bool,

    /// Whether to join consecutive simple statements using the comma operator.
    ///
    /// Transforms multiple expression statements into sequence expressions:
    /// ```javascript
    /// // Before
    /// a();
    /// b();
    ///
    /// // After
    /// a(), b();
    /// ```
    ///
    /// Default: `true`
    pub sequences: bool,

    /// Configuration for removing unused variables and functions.
    pub unused: CompressOptionsUnused,

    /// Configuration for preserving function and class names.
    pub keep_names: CompressOptionsKeepNames,

    /// Tree-shaking options for eliminating dead code.
    ///
    /// These options control how aggressively the minifier can eliminate
    /// code that appears to be unused, based on static analysis.
    pub treeshake: TreeShakeOptions,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self::smallest()
    }
}

impl CompressOptions {
    /// Create compression options optimized for the smallest possible output.
    ///
    /// This preset enables all size-reducing optimizations while maintaining
    /// correctness. It may be more aggressive than other presets and could
    /// potentially affect debugging or runtime behavior in edge cases.
    pub fn smallest() -> Self {
        Self {
            target: ESTarget::ESNext,
            keep_names: CompressOptionsKeepNames::all_false(),
            drop_debugger: true,
            drop_console: false,
            join_vars: true,
            sequences: true,
            unused: CompressOptionsUnused::Remove,
            treeshake: TreeShakeOptions::default(),
        }
    }

    /// Create compression options optimized for safety and compatibility.
    ///
    /// This preset applies conservative optimizations that are less likely
    /// to cause issues but may result in larger output. Recommended for
    /// production builds where correctness is more important than size.
    pub fn safest() -> Self {
        Self {
            target: ESTarget::ESNext,
            keep_names: CompressOptionsKeepNames::all_true(),
            drop_debugger: false,
            drop_console: false,
            join_vars: true,
            sequences: true,
            unused: CompressOptionsUnused::Keep,
            treeshake: TreeShakeOptions::default(),
        }
    }

    /// Create compression options focused on dead code elimination only.
    ///
    /// This preset applies only dead code elimination optimizations while
    /// preserving most other aspects of the code structure. Useful when
    /// you want to remove unused code but maintain readability.
    pub fn dce() -> Self {
        Self {
            target: ESTarget::ESNext,
            keep_names: CompressOptionsKeepNames::all_true(),
            drop_debugger: false,
            drop_console: false,
            join_vars: false,
            sequences: false,
            unused: CompressOptionsUnused::Remove,
            treeshake: TreeShakeOptions::default(),
        }
    }

    /// Check if any compression optimizations are enabled.
    pub fn has_optimizations(&self) -> bool {
        self.drop_debugger
            || self.drop_console
            || self.join_vars
            || self.sequences
            || matches!(
                self.unused,
                CompressOptionsUnused::Remove | CompressOptionsUnused::KeepAssign
            )
    }

    /// Check if unused code removal is enabled.
    pub fn removes_unused_code(&self) -> bool {
        matches!(self.unused, CompressOptionsUnused::Remove | CompressOptionsUnused::KeepAssign)
    }
}

/// Configuration for handling unused variables and functions.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum CompressOptionsUnused {
    /// Remove all unused variables and functions.
    ///
    /// This is the most aggressive option and provides the best size reduction,
    /// but may remove code that is used through dynamic means (eval, etc.).
    #[default]
    Remove,

    /// Keep unused variables but remove their assignments when safe.
    ///
    /// This preserves variable declarations but may remove their initializers
    /// if the assignments have no side effects.
    KeepAssign,

    /// Keep all unused variables and functions.
    ///
    /// This is the safest option but provides no dead code elimination benefits.
    Keep,
}

/// Configuration for preserving function and class names.
///
/// In JavaScript, function and class names can be accessed at runtime
/// through the `name` property. These options control whether to preserve
/// these names during minification.
#[derive(Debug, Clone, Copy, Default)]
pub struct CompressOptionsKeepNames {
    /// Whether to preserve function names.
    ///
    /// When enabled, function names are preserved so that `Function.prototype.name`
    /// returns the original name. This may increase bundle size but preserves
    /// runtime introspection capabilities.
    ///
    /// Note: This does not guarantee preservation of anonymous function names.
    ///
    /// Default: `false`
    pub function: bool,

    /// Whether to preserve class names.     
    ///
    /// When enabled, class names are preserved so that `Class.prototype.name`
    /// returns the original name. This may increase bundle size but preserves
    /// runtime introspection capabilities.
    ///
    /// Note: This does not guarantee preservation of anonymous class names.
    ///
    /// Default: `false`
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
