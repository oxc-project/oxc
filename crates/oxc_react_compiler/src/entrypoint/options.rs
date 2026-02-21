/// Plugin options and configuration for the React Compiler.
///
/// Port of `Entrypoint/Options.ts` from the React Compiler.
use crate::hir::environment::{CompilerOutputMode, EnvironmentConfig, ExternalFunction};

/// Controls when the compiler panics (throws an exception) on errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanicThreshold {
    /// Any errors will panic the compiler by throwing an exception.
    AllErrors,
    /// Panic only on critical or unrecognized errors.
    CriticalErrors,
    /// Never panic by throwing an exception.
    #[default]
    None,
}

/// Configuration for dynamic gating via `use memo if(...)` directives.
#[derive(Debug, Clone)]
pub struct DynamicGatingOptions {
    pub source: String,
}

/// Determines the strategy for deciding which functions to compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompilationMode {
    /// Compiles functions annotated with "use forget" or component/hook-like functions.
    #[default]
    Infer,
    /// Compile only components using component syntax and hooks using hook syntax.
    Syntax,
    /// Compile only functions which are explicitly annotated with "use forget".
    Annotation,
    /// Compile all top-level functions.
    All,
}

/// The minimum major version of React the compiler targets.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
pub enum CompilerReactTarget {
    /// React 17
    React17,
    /// React 18
    React18,
    /// React 19
    #[default]
    React19,
    /// Meta-internal target (re-exports useMemoCache from react directly).
    MetaInternal { runtime_module: String },
}


/// The parsed (validated) plugin options for the React Compiler.
#[derive(Debug, Clone)]
pub struct PluginOptions {
    pub environment: EnvironmentConfig,
    pub compilation_mode: CompilationMode,
    pub panic_threshold: PanicThreshold,
    pub gating: Option<ExternalFunction>,
    pub dynamic_gating: Option<DynamicGatingOptions>,
    pub no_emit: bool,
    pub output_mode: Option<CompilerOutputMode>,
    pub eslint_suppression_rules: Option<Vec<String>>,
    pub flow_suppressions: bool,
    pub ignore_use_no_forget: bool,
    pub custom_opt_out_directives: Option<Vec<String>>,
    pub enable_reanimated_check: bool,
    pub target: CompilerReactTarget,
}

impl Default for PluginOptions {
    fn default() -> Self {
        Self {
            compilation_mode: CompilationMode::Infer,
            panic_threshold: PanicThreshold::None,
            environment: EnvironmentConfig::default(),
            gating: None,
            no_emit: false,
            output_mode: None,
            dynamic_gating: None,
            eslint_suppression_rules: None,
            flow_suppressions: true,
            ignore_use_no_forget: false,
            custom_opt_out_directives: None,
            enable_reanimated_check: true,
            target: CompilerReactTarget::React19,
        }
    }
}

/// Directives that opt out of compilation.
pub const OPT_OUT_DIRECTIVES: &[&str] = &["use no forget", "use no memo"];

/// Directives that opt in to compilation.
pub const OPT_IN_DIRECTIVES: &[&str] = &["use forget", "use memo"];
