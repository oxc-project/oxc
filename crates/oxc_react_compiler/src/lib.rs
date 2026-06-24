use oxc_allocator::Allocator;

mod category;
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

use crate::react_compiler::entrypoint::compile_result::CompileResult;
use crate::react_compiler::entrypoint::imports::{
    get_react_compiler_runtime_module, has_memo_cache_function_import, validate_restricted_imports,
};
use crate::react_compiler::entrypoint::program::compile_program;

pub use crate::react_compiler::entrypoint::program::CompileOutput;

pub use category::{ReactCompilerCategory, ReactCompilerDiagnostic};
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
    BuiltInTypeRef, FunctionTypeConfig, HookTypeConfig, ObjectTypeConfig, TypeConfig,
    TypeReferenceConfig, ValueKind,
};
pub use crate::react_compiler_utils::FxIndexMap;

use oxc_ast::ast::Program;
use oxc_diagnostics::Diagnostics;
use oxc_semantic::Semantic;

pub struct LintResult {
    /// Errors and warnings produced by the compile, each paired with its
    /// [`ReactCompilerCategory`] so consumers can filter or suppress by category
    /// (e.g. per-sub-rule disable directives) without re-parsing the message.
    pub diagnostics: Vec<ReactCompilerDiagnostic>,
}

/// Run the React Compiler on a pre-parsed program.
///
/// Returns the compiled output — `None` when the compiler has nothing to change
/// (no React-like functions, a bail-out, nothing to memoize, or a fatal error) —
/// together with the diagnostics. Errors (e.g. Rules of Hooks violations) are
/// hard problems in the source; the program is still left valid. Warnings
/// include bail-outs where the compiler declined to optimize.
///
/// Must run **first**, on the pristine AST, before any other transform. The
/// borrowed `semantic` must have been built from that same pristine AST with
/// `SemanticBuilder::with_build_nodes(true)`. Rewrite the program by applying
/// the output with [`CompileOutput::transform`] once `semantic` is dropped:
///
/// ```ignore
/// let (output, diagnostics) = {
///     let semantic = SemanticBuilder::new().with_build_nodes(true).build(&program).semantic;
///     compile(&program, &semantic, &allocator, options)
/// }; // `semantic`'s borrow of `program` ends here
/// if let Some(output) = output {
///     output.transform(&mut program);
/// }
/// ```
pub fn compile<'a>(
    program: &Program<'a>,
    semantic: &Semantic<'_>,
    allocator: &'a Allocator,
    options: PluginOptions,
) -> (Option<CompileOutput<'a>>, Diagnostics) {
    // Check for existing runtime imports (file already compiled).
    if has_memo_cache_function_import(program, &get_react_compiler_runtime_module(&options.target))
    {
        return (None, Diagnostics::default());
    }

    // Blocklisted imports fail the whole file: report and bail without compiling.
    if let Some(diagnostics) =
        validate_restricted_imports(program, &options.environment.validate_blocklisted_imports)
    {
        return (None, diagnostics);
    }

    match compile_program(allocator, semantic, program, options) {
        CompileResult::Success { output, diagnostics } => (output.map(|b| *b), diagnostics),
        CompileResult::Error { diagnostics } => (None, diagnostics),
    }
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

    let (_output, diagnostics) = compile(program, semantic, allocator, options);
    // The compiler emits already-formatted `OxcDiagnostic`s (categories are
    // flattened into the message inside `diagnostics::detail_to_diagnostic`). Pair
    // each back up with its typed category here, so consumers match on the enum
    // instead of re-parsing the message.
    let diagnostics = diagnostics
        .into_vec()
        .into_iter()
        .map(|diagnostic| ReactCompilerDiagnostic {
            category: message_category(&diagnostic.message),
            diagnostic,
        })
        .collect();
    LintResult { diagnostics }
}

/// Recover a [`ReactCompilerCategory`] from a formatted diagnostic message.
///
/// The compiler formats per-detail diagnostics as
/// `[ReactCompiler] {Category:?}: {reason}` (see `diagnostics::detail_to_diagnostic`,
/// which stringifies the typed `ErrorCategory`), and its two synthetic error paths
/// as `[ReactCompiler] Pipeline error: …` / `[ReactCompiler] Unexpected error: …`
/// (see `program.rs`). This is the inverse of that formatting; anything else maps
/// to [`ReactCompilerCategory::Other`].
fn message_category(message: &str) -> ReactCompilerCategory {
    match message
        .strip_prefix("[ReactCompiler] ")
        .and_then(|rest| rest.split_once(": "))
        .map(|(category, _)| category)
    {
        Some("Pipeline error") => ReactCompilerCategory::PipelineError,
        Some("Unexpected error") => ReactCompilerCategory::UnexpectedError,
        Some(name) => ReactCompilerCategory::from_compiler_string(name),
        None => ReactCompilerCategory::Other,
    }
}
