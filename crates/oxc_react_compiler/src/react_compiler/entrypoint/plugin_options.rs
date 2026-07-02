use crate::react_compiler_hir::environment_config::EnvironmentConfig;

/// Target configuration for the compiler
#[derive(Debug, Clone)]
pub enum CompilerTarget {
    /// Standard React version target
    Version(String), // "17", "18", "19"
    /// Meta-internal target with custom runtime module
    MetaInternal {
        kind: String, // "donotuse_meta_internal"
        runtime_module: String,
    },
}

/// Gating configuration
#[derive(Debug, Clone)]
pub struct GatingConfig {
    pub source: String,
    pub import_specifier_name: String,
}

/// Dynamic gating configuration
#[derive(Debug, Clone)]
pub struct DynamicGatingConfig {
    pub source: String,
}

/// Serializable plugin options.
#[derive(Debug, Clone)]
pub struct PluginOptions {
    pub compilation_mode: String,
    pub panic_threshold: String,
    pub target: CompilerTarget,
    pub gating: Option<GatingConfig>,
    pub dynamic_gating: Option<DynamicGatingConfig>,
    pub no_emit: bool,
    pub output_mode: Option<String>,
    pub eslint_suppression_rules: Option<Vec<String>>,
    pub flow_suppressions: bool,
    pub ignore_use_no_forget: bool,
    pub custom_opt_out_directives: Option<Vec<String>>,
    pub environment: EnvironmentConfig,

    /// Enable debug logging (HIR formatting after each pass).
    /// Only set to true when a logger with debugLogIRs is configured on the JS side.
    pub debug: bool,
}

/// Output mode for the compiler, derived from PluginOptions.
/// Matches the TS `compilerOutputMode` logic in Program.ts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerOutputMode {
    Ssr,
    Client,
    Lint,
}

impl CompilerOutputMode {
    pub fn from_opts(opts: &PluginOptions) -> Self {
        match opts.output_mode.as_deref() {
            Some("ssr") => Self::Ssr,
            Some("lint") => Self::Lint,
            _ if opts.no_emit => Self::Lint,
            _ => Self::Client,
        }
    }
}
