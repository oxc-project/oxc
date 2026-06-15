//! React Compiler options for the transform binding.
//!
//! A curated, JS-facing mirror of the React Compiler `PluginOptions` (the deep
//! `environment` config is not surfaced), resolved into the concrete options the
//! transformer consumes.

use napi::Either;
use napi_derive::napi;

/// Options for the experimental [React Compiler](https://github.com/react/react/tree/main/compiler).
///
/// Mirrors the compiler's `PluginOptions`. The deep `environment` configuration
/// (inference / validation flags) is not surfaced here.
///
/// @see {@link TransformOptions#reactCompiler}
#[napi(object)]
#[derive(Default, Debug)]
pub struct ReactCompilerOptions {
    /// Which functions to compile.
    ///
    /// @default 'infer'
    #[napi(ts_type = "'infer' | 'syntax' | 'annotation' | 'all'")]
    pub compilation_mode: Option<String>,

    /// What to do when a function cannot be compiled.
    ///
    /// @default 'none'
    #[napi(ts_type = "'none' | 'critical_errors' | 'all_errors'")]
    pub panic_threshold: Option<String>,

    /// React runtime version target. `17` and `18` require the
    /// `react-compiler-runtime` package; `19` ships the runtime in `react`.
    ///
    /// @default '19'
    #[napi(ts_type = "'17' | '18' | '19'")]
    pub target: Option<String>,

    /// Analyze and report diagnostics only; emit no transformed code.
    ///
    /// @default false
    pub no_emit: Option<bool>,

    /// Compiler output mode.
    ///
    /// @default undefined
    #[napi(ts_type = "'client' | 'ssr' | 'lint'")]
    pub output_mode: Option<String>,

    /// Compile even functions marked with the `"use no memo"` / `"use no forget"`
    /// opt-out directives.
    ///
    /// @default false
    pub ignore_use_no_forget: Option<bool>,

    /// Treat Flow suppression comments as opt-outs.
    ///
    /// @default true
    pub flow_suppressions: Option<bool>,

    /// Enable `react-native-reanimated` support.
    ///
    /// @default false
    pub enable_reanimated: Option<bool>,

    /// Development mode (extra validation / instrumentation).
    ///
    /// @default false
    pub is_dev: Option<bool>,

    /// Source file name, used for the fast-refresh hash and in diagnostics.
    pub filename: Option<String>,

    /// ESLint rules whose suppressions opt a function out of compilation.
    pub eslint_suppression_rules: Option<Vec<String>>,

    /// Extra directives that opt a function out of compilation.
    pub custom_opt_out_directives: Option<Vec<String>>,

    /// Also emit a gated (feature-flagged) version of each compiled function.
    pub gating: Option<ReactCompilerGating>,

    /// Dynamically-gated compilation.
    pub dynamic_gating: Option<ReactCompilerDynamicGating>,
}

/// Static gating for {@link ReactCompilerOptions#gating}.
#[napi(object)]
#[derive(Debug)]
pub struct ReactCompilerGating {
    /// Module the gating import comes from.
    pub source: String,
    /// Imported specifier used as the gate.
    pub import_specifier_name: String,
}

/// Dynamic gating for {@link ReactCompilerOptions#dynamicGating}.
#[napi(object)]
#[derive(Debug)]
pub struct ReactCompilerDynamicGating {
    /// Module the gating import comes from.
    pub source: String,
}

/// Resolve the JS `reactCompiler` option into the compiler's `PluginOptions`:
/// `true` → default options, an object → those options, `false`/absent → disabled.
pub fn resolve(
    option: Option<Either<bool, ReactCompilerOptions>>,
) -> Option<oxc_react_compiler::PluginOptions> {
    match option {
        Some(Either::A(true)) => Some(oxc_react_compiler::default_plugin_options()),
        Some(Either::B(options)) => Some(options.into_plugin_options()),
        Some(Either::A(false)) | None => None,
    }
}

impl ReactCompilerOptions {
    fn into_plugin_options(self) -> oxc_react_compiler::PluginOptions {
        let mut options = oxc_react_compiler::default_plugin_options();
        if let Some(compilation_mode) = self.compilation_mode {
            options.compilation_mode = compilation_mode;
        }
        if let Some(panic_threshold) = self.panic_threshold {
            options.panic_threshold = panic_threshold;
        }
        if let Some(target) = self.target {
            options.target = oxc_react_compiler::CompilerTarget::Version(target);
        }
        if let Some(no_emit) = self.no_emit {
            options.no_emit = no_emit;
        }
        if self.output_mode.is_some() {
            options.output_mode = self.output_mode;
        }
        if let Some(ignore_use_no_forget) = self.ignore_use_no_forget {
            options.ignore_use_no_forget = ignore_use_no_forget;
        }
        if let Some(flow_suppressions) = self.flow_suppressions {
            options.flow_suppressions = flow_suppressions;
        }
        if let Some(enable_reanimated) = self.enable_reanimated {
            options.enable_reanimated = enable_reanimated;
        }
        if let Some(is_dev) = self.is_dev {
            options.is_dev = is_dev;
        }
        if self.filename.is_some() {
            options.filename = self.filename;
        }
        if self.eslint_suppression_rules.is_some() {
            options.eslint_suppression_rules = self.eslint_suppression_rules;
        }
        if self.custom_opt_out_directives.is_some() {
            options.custom_opt_out_directives = self.custom_opt_out_directives;
        }
        if let Some(gating) = self.gating {
            options.gating = Some(oxc_react_compiler::GatingConfig {
                source: gating.source,
                import_specifier_name: gating.import_specifier_name,
            });
        }
        if let Some(dynamic_gating) = self.dynamic_gating {
            options.dynamic_gating =
                Some(oxc_react_compiler::DynamicGatingConfig { source: dynamic_gating.source });
        }
        options
    }
}
