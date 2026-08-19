use oxc_allocator::Allocator;

mod diagnostics;
mod options;
mod scope;

mod react_compiler;
mod react_compiler_hir;
mod react_compiler_inference;
mod react_compiler_lowering;
mod react_compiler_optimization;
mod react_compiler_reactive_scopes;
mod react_compiler_ssa;
mod react_compiler_typeinference;
mod react_compiler_utils;
mod react_compiler_validation;

use crate::react_compiler::entrypoint::imports::{
    get_react_compiler_runtime_module, has_memo_cache_function_import, validate_restricted_imports,
};
use crate::react_compiler::entrypoint::program::compile_program;

pub use crate::diagnostics::{ErrorCategory, LintDiagnostic};
pub use crate::react_compiler::entrypoint::compile_result::CompileResult;
pub use crate::react_compiler::entrypoint::program::CompileOutput;

// Re-exported so integrations needn't depend on the upstream `react_compiler` crates.
pub use crate::options::{
    CompilationMode, CompilerOutputMode, CompilerTarget, DynamicGatingConfig, GatingConfig,
    PanicThreshold, PluginOptions,
};
pub use crate::react_compiler_hir::Effect;
pub use crate::react_compiler_hir::environment_config::{
    EnvironmentConfig, ExhaustiveEffectDepsMode, ExternalFunctionConfig, HookConfig,
    InstrumentationConfig,
};
pub use crate::react_compiler_hir::type_config::{
    AliasingEffectConfig, AliasingSignatureConfig, ApplyArgConfig, BuiltInTypeRef,
    FunctionTypeConfig, HookTypeConfig, ObjectTypeConfig, TypeConfig, TypeReferenceConfig,
    ValueKind, ValueReason,
};
pub use crate::react_compiler_utils::FxIndexMap;

use oxc_ast::ast::Program;
use oxc_diagnostics::Diagnostics;
use oxc_semantic::Semantic;

pub struct LintResult {
    /// Errors and warnings produced by the compile, paired with their
    /// [`ErrorCategory`] for routing to category-specific lint rules.
    pub diagnostics: Vec<LintDiagnostic>,
    /// Whether compilation was aborted according to `panic_threshold`.
    pub fatal: bool,
}

/// Run the React Compiler on a pre-parsed program.
///
/// Returns [`CompileResult::Success`] when compilation completed, even if one or
/// more functions bailed out with diagnostics. Returns
/// [`CompileResult::Fatal`] only when compilation was aborted according to
/// [`PluginOptions::panic_threshold`]. Diagnostic severity and fatality are
/// intentionally separate.
///
/// Must run **first**, on the pristine AST, before any other transform. The
/// borrowed `semantic` must have been built from that same pristine AST with
/// `SemanticBuilder::with_build_nodes(true)`. Rewrite the program by applying
/// the output with [`CompileOutput::transform`] once `semantic` is dropped:
///
/// ```ignore
/// let result = {
///     let semantic = SemanticBuilder::new().with_build_nodes(true).build(&program).semantic;
///     compile(&program, &semantic, &allocator, options)
/// }; // `semantic`'s borrow of `program` ends here
/// match result {
///     CompileResult::Success { output: Some(output), .. } => output.transform(&mut program),
///     CompileResult::Success { output: None, .. } => {}
///     CompileResult::Fatal { diagnostics } => report(diagnostics),
/// }
/// ```
pub fn compile<'a>(
    program: &Program<'a>,
    semantic: &Semantic<'_>,
    allocator: &'a Allocator,
    options: PluginOptions,
) -> CompileResult<'a> {
    // Check for existing runtime imports (file already compiled).
    if has_memo_cache_function_import(program, &get_react_compiler_runtime_module(&options.target))
    {
        return CompileResult::Success { output: None, diagnostics: Diagnostics::default() };
    }

    // Blocklisted imports bail out the whole file. Whether that aborts the
    // surrounding transform is controlled by `panic_threshold`.
    if let Some(diagnostics) =
        validate_restricted_imports(program, &options.environment.validate_blocklisted_imports)
    {
        return if diagnostics::should_panic(&diagnostics, options.panic_threshold) {
            CompileResult::Fatal { diagnostics }
        } else {
            CompileResult::Success { output: None, diagnostics }
        };
    }

    compile_program(allocator, semantic, program, options)
}

/// Lint a pre-parsed program — like [`compile`] but read-only: it collects
/// diagnostics without producing a rewrite.
///
/// The borrowed `semantic` must have been built from `program` with
/// `SemanticBuilder::with_build_nodes(true)`.
pub fn lint<'a>(
    program: &Program<'a>,
    semantic: &Semantic<'_>,
    allocator: &'a Allocator,
    options: PluginOptions,
) -> LintResult {
    let mut options = options;
    options.no_emit = true;

    let (diagnostics, fatal) = match compile(program, semantic, allocator, options) {
        CompileResult::Success { diagnostics, .. } => (diagnostics, false),
        CompileResult::Fatal { diagnostics } => (diagnostics, true),
    };
    let diagnostics = diagnostics.into_vec().into_iter().map(diagnostics::categorize).collect();
    LintResult { diagnostics, fatal }
}
