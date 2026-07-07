//! Compiler options.
//!
//! [`PluginOptions`] is the public configuration surface, mirroring the upstream
//! Babel plugin's options. String-valued options are modelled as enums here; the
//! `FromStr` impls map the upstream string spellings (used by the snapshot test
//! pragma parser) onto those enums.

use std::str::FromStr;

use crate::react_compiler_hir::environment_config::EnvironmentConfig;

/// Which functions the compiler considers for compilation.
///
/// Mirrors the upstream `compilationMode` option.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CompilationMode {
    /// Compile functions inferred to be components or hooks (default).
    #[default]
    Infer,
    /// Compile only functions declared with component/hook syntax.
    Syntax,
    /// Compile only functions with an opt-in directive.
    Annotation,
    /// Compile every top-level function.
    All,
}

impl FromStr for CompilationMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "infer" => Self::Infer,
            "syntax" => Self::Syntax,
            "annotation" => Self::Annotation,
            "all" => Self::All,
            _ => return Err(()),
        })
    }
}

/// When a compiler error should escalate into a hard failure.
///
/// Mirrors the upstream `panicThreshold` option.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PanicThreshold {
    /// Never fail; skip the offending function (default).
    #[default]
    None,
    /// Fail on critical errors only.
    CriticalErrors,
    /// Fail on any error.
    AllErrors,
}

impl FromStr for PanicThreshold {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "none" => Self::None,
            "critical_errors" => Self::CriticalErrors,
            "all_errors" => Self::AllErrors,
            _ => return Err(()),
        })
    }
}

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

impl Default for CompilerTarget {
    /// Targets React 19. (A field-carrying variant can't be a derived `#[default]`.)
    fn default() -> Self {
        CompilerTarget::Version("19".to_string())
    }
}

/// Feature-flag gating configuration (see the `gating` option).
#[derive(Debug, Clone)]
pub struct GatingConfig {
    /// Module the gating flag is imported from.
    pub source: String,
    /// Name of the imported flag that guards compiled functions at runtime.
    pub import_specifier_name: String,
}

/// Directive-driven gating configuration (see the `dynamic_gating` option).
#[derive(Debug, Clone)]
pub struct DynamicGatingConfig {
    /// Module the `"use memo if(<flag>)"` gate imports its flag from.
    pub source: String,
}

/// Options for the React Compiler, mirroring the upstream
/// `babel-plugin-react-compiler` plugin options.
#[derive(Debug, Clone)]
pub struct PluginOptions {
    /// Which functions the compiler attempts to compile. See [`CompilationMode`].
    pub compilation_mode: CompilationMode,

    /// When a compilation error aborts with a hard failure instead of leaving the
    /// offending function untouched. See [`PanicThreshold`].
    pub panic_threshold: PanicThreshold,

    /// React version (or Meta-internal runtime) the output targets. Selects which
    /// `react/compiler-runtime` module the memo-cache `c` helper is imported from.
    pub target: CompilerTarget,

    /// Feature-flag gating. When set, each compiled function is emitted *alongside*
    /// its original and guarded at runtime by the imported flag, so the optimized
    /// version only runs when the flag is on. Applies to original (not JSX-outlined)
    /// functions.
    pub gating: Option<GatingConfig>,

    /// Enables per-function gating via a `"use memo if(<flag>)"` body directive. A
    /// directive's flag takes precedence over `gating`.
    pub dynamic_gating: Option<DynamicGatingConfig>,

    /// Run for diagnostics only, without rewriting the program. Set by [`lint`];
    /// resolves to [`CompilerOutputMode::Lint`] when no `output_mode` is given.
    ///
    /// [`lint`]: crate::lint
    pub no_emit: bool,

    /// Requested output mode. Resolved together with `no_emit` by
    /// [`CompilerOutputMode::from_opts`]; `None` defaults to `Client` (or `Lint`
    /// when `no_emit` is set).
    pub output_mode: Option<CompilerOutputMode>,

    /// ESLint rule names whose `eslint-disable` comments make the compiler skip the
    /// function they cover, so it doesn't optimize code the author has already
    /// flagged. `None` uses the defaults (`react-hooks/exhaustive-deps` and
    /// `react-hooks/rules-of-hooks`). Ignored when both the exhaustive-deps and
    /// hooks-usage validations are enabled.
    pub eslint_suppression_rules: Option<Vec<String>>,

    /// Whether Flow suppression comments (e.g. `$FlowFixMe`) likewise make the
    /// compiler skip the function they cover.
    pub flow_suppressions: bool,

    /// Compile a function even when it carries an opt-out directive
    /// (`"use no memo"` / `"use no forget"`), ignoring the opt-out.
    pub ignore_use_no_forget: bool,

    /// Additional directive strings that opt a function or module out of compilation,
    /// on top of the built-in `"use no memo"` / `"use no forget"`.
    pub custom_opt_out_directives: Option<Vec<String>>,

    /// Compiler environment: feature flags, validation toggles, and custom hook/type
    /// definitions for the compilation passes. See [`EnvironmentConfig`].
    pub environment: EnvironmentConfig,

    /// Enable debug logging (HIR formatting after each pass).
    /// Only set to true when a logger with debugLogIRs is configured on the JS side.
    pub debug: bool,
}

impl Default for PluginOptions {
    /// The compiler's standard defaults. Override fields with struct-update syntax:
    /// `PluginOptions { ..Default::default() }`.
    fn default() -> Self {
        PluginOptions {
            compilation_mode: CompilationMode::default(),
            panic_threshold: PanicThreshold::default(),
            target: CompilerTarget::default(),
            gating: None,
            dynamic_gating: None,
            no_emit: false,
            output_mode: None,
            eslint_suppression_rules: None,
            flow_suppressions: true,
            ignore_use_no_forget: false,
            custom_opt_out_directives: None,
            environment: EnvironmentConfig::default(),
            debug: false,
        }
    }
}

/// The compiler's output mode: the requested [`PluginOptions::output_mode`], or the
/// value [`from_opts`](Self::from_opts) resolves from the options.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CompilerOutputMode {
    /// Server-side rendering; enables SSR-specific optimizations.
    Ssr,
    /// Client rendering (the default).
    #[default]
    Client,
    /// Validate and report diagnostics only; the program is not rewritten.
    Lint,
}

impl FromStr for CompilerOutputMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "ssr" => Self::Ssr,
            "client" => Self::Client,
            "lint" => Self::Lint,
            _ => return Err(()),
        })
    }
}

impl CompilerOutputMode {
    pub fn from_opts(opts: &PluginOptions) -> Self {
        match opts.output_mode {
            Some(Self::Ssr) => Self::Ssr,
            Some(Self::Lint) => Self::Lint,
            _ if opts.no_emit => Self::Lint,
            _ => Self::Client,
        }
    }
}
