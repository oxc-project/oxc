//! React Compiler options for the transform binding.
//!
//! A curated, JS-facing mirror of the React Compiler `PluginOptions` (the deep
//! `environment` config is not surfaced), resolved into the concrete options the
//! transformer consumes.
//!
//! The option types below are deliberately **not** gated on the `react_compiler`
//! feature, for two reasons: `#[napi(object)]` ignores `#[cfg]` on fields (it emits
//! them into the generated `FromNapiValue`/`ToNapiValue` regardless), and keeping the
//! types unconditional makes the generated `index.d.ts` identical in every feature
//! config, so a lean build can never drift from the checked-in one. Only the
//! conversion into the compiler's `PluginOptions` is gated; without the feature,
//! `is_enabled` lets the caller reject the option rather than silently ignore it.

use napi::Either;
use napi_derive::napi;

#[cfg(feature = "react_compiler")]
use std::str::FromStr;

#[cfg(feature = "react_compiler")]
use oxc::react_compiler::{CompilerTarget, DynamicGatingConfig, GatingConfig, PluginOptions};

/// Options for the experimental [React Compiler](https://github.com/react/react/tree/main/compiler).
///
/// Mirrors the compiler's `PluginOptions`. The deep `environment` configuration
/// (inference / validation flags) is not surfaced here.
///
/// @see {@link PluginsOptions#reactCompiler}
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

/// Whether the `reactCompiler` option asks for the compiler to run. `false` and an
/// absent option both mean "disabled", and must not be treated as a request.
#[cfg(not(feature = "react_compiler"))]
pub fn is_enabled(option: Option<&Either<bool, ReactCompilerOptions>>) -> bool {
    matches!(option, Some(Either::A(true) | Either::B(_)))
}

/// Resolve the JS `reactCompiler` option into the compiler's `PluginOptions`:
/// `true` → default options, an object → those options, `false`/absent → disabled.
///
/// # Errors
///
/// Returns an error if a string-valued option is not one of its documented variants.
#[cfg(feature = "react_compiler")]
pub fn resolve(
    option: Option<Either<bool, ReactCompilerOptions>>,
) -> Result<Option<PluginOptions>, String> {
    match option {
        Some(Either::A(true)) => Ok(Some(PluginOptions::default())),
        Some(Either::B(options)) => options.into_plugin_options().map(Some),
        Some(Either::A(false)) | None => Ok(None),
    }
}

/// Parse a string-valued option into the enum the compiler takes. The `ts_type`
/// annotations constrain these at the type level only, so a plain-JS caller can still
/// reach this with anything.
#[cfg(feature = "react_compiler")]
fn parse<T: FromStr>(value: &str, option: &str) -> Result<T, String> {
    T::from_str(value)
        .map_err(|_| format!("Invalid plugins.reactCompiler.{option} option: `{value}`."))
}

#[cfg(feature = "react_compiler")]
impl ReactCompilerOptions {
    fn into_plugin_options(self) -> Result<PluginOptions, String> {
        let mut options = PluginOptions::default();
        if let Some(compilation_mode) = self.compilation_mode {
            options.compilation_mode = parse(&compilation_mode, "compilationMode")?;
        }
        if let Some(panic_threshold) = self.panic_threshold {
            options.panic_threshold = parse(&panic_threshold, "panicThreshold")?;
        }
        if let Some(target) = self.target {
            // `CompilerTarget::Version` takes any string and silently falls back to the
            // React 19 runtime for unrecognized ones, so reject typos here instead.
            if !matches!(target.as_str(), "17" | "18" | "19") {
                return Err(format!("Invalid plugins.reactCompiler.target option: `{target}`."));
            }
            options.target = CompilerTarget::Version(target);
        }
        if let Some(no_emit) = self.no_emit {
            options.no_emit = no_emit;
        }
        if let Some(output_mode) = self.output_mode {
            options.output_mode = Some(parse(&output_mode, "outputMode")?);
        }
        if let Some(ignore_use_no_forget) = self.ignore_use_no_forget {
            options.ignore_use_no_forget = ignore_use_no_forget;
        }
        if let Some(flow_suppressions) = self.flow_suppressions {
            options.flow_suppressions = flow_suppressions;
        }
        if self.eslint_suppression_rules.is_some() {
            options.eslint_suppression_rules = self.eslint_suppression_rules;
        }
        if self.custom_opt_out_directives.is_some() {
            options.custom_opt_out_directives = self.custom_opt_out_directives;
        }
        if let Some(gating) = self.gating {
            options.gating = Some(GatingConfig {
                source: gating.source,
                import_specifier_name: gating.import_specifier_name,
            });
        }
        if let Some(dynamic_gating) = self.dynamic_gating {
            options.dynamic_gating = Some(DynamicGatingConfig { source: dynamic_gating.source });
        }
        Ok(options)
    }
}
