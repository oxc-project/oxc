use oxc_allocator::{Allocator, ArenaVec};

mod diagnostics;
mod options;
mod prefilter;
mod scope;

mod react_compiler;
mod react_compiler_diagnostics;
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
use crate::react_compiler::entrypoint::imports::get_react_compiler_runtime_module;
use crate::react_compiler::entrypoint::program::compile_program;
use prefilter::{has_memo_cache_function_import, has_react_like_functions};

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
use oxc_span::GetSpan;
use rustc_hash::FxHashSet;

#[derive(Default)]
pub struct TransformResult<'a> {
    /// The rewritten program, when the compiler memoized something.
    ///
    /// Callers that want in-place behavior should replace their original
    /// `Program` with this value after the borrowed `Semantic` has gone out of
    /// scope.
    pub program: Option<Program<'a>>,
    /// Whether `program` contains a rewritten program. `false` means the
    /// compiler made no changes — no React-like functions, a bail-out, or
    /// nothing to memoize.
    pub changed: bool,
    /// Errors and warnings produced by the compile. Errors (e.g. Rules of Hooks
    /// violations) are hard problems in the source; the program is still left
    /// valid. Warnings include bail-outs where the compiler declined to optimize.
    pub diagnostics: Diagnostics,
}

pub struct LintResult {
    /// Errors and warnings produced by the compile.
    pub diagnostics: Diagnostics,
}

/// Run the React Compiler on a pre-parsed program, returning a rewritten
/// program when it memoizes something.
///
/// Must run **first**, on the pristine AST, before any other transform. The
/// borrowed `semantic` must have been built from that same pristine AST with
/// `SemanticBuilder::with_build_nodes(true)`.
pub fn transform<'a>(
    program: &Program<'a>,
    semantic: &Semantic<'_>,
    allocator: &'a Allocator,
    options: PluginOptions,
) -> TransformResult<'a> {
    let (program, diagnostics) = compile(program, semantic, allocator, options);
    let changed = program.is_some();
    TransformResult { program, changed, diagnostics }
}

/// Lint a pre-parsed program — like [`transform`] but read-only: it collects
/// diagnostics without rewriting the program.
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

    let (_program, diagnostics) = compile(program, semantic, allocator, options);
    LintResult { diagnostics }
}

/// Shared compile pipeline behind [`transform`] and [`lint`]. Borrows `program`
/// (so `lint` can stay read-only) and returns the compiled OXC program — `None`
/// when nothing was compiled — together with diagnostics and logger events.
fn compile<'a>(
    program: &Program<'a>,
    semantic: &Semantic<'_>,
    allocator: &'a Allocator,
    options: PluginOptions,
) -> (Option<Program<'a>>, Diagnostics) {
    // Check for existing runtime imports (file already compiled).
    if has_memo_cache_function_import(program, &get_react_compiler_runtime_module(&options.target))
    {
        return (None, Diagnostics::default());
    }

    // Skip files with no React-like functions, unless the mode compiles everything.
    if !matches!(options.compilation_mode, CompilationMode::All | CompilationMode::Annotation)
        && !has_react_like_functions(program)
    {
        return (None, Diagnostics::default());
    }

    let result = compile_program(allocator, semantic, program, options);

    let (program_ast, diagnostics) = match result {
        CompileResult::Success { ast, diagnostics, .. } => (ast, diagnostics),
        CompileResult::Error { diagnostics, .. } => (None, diagnostics),
    };

    let compiled = program_ast.map(|mut compiled: Program<'a>| {
        compiled.source_type = program.source_type;
        preserve_comments(&mut compiled, program, allocator);
        compiled
    });

    (compiled, diagnostics)
}

/// Carry over the comments attached to top-level statements of the compiled
/// program, so codegen can re-emit them. The `react_compiler_ast` roundtrip
/// drops comments, so we reuse the ones from the original `source` program
/// (already parsed) rather than re-parsing the source.
fn preserve_comments<'a>(
    compiled: &mut Program<'a>,
    source: &Program<'a>,
    allocator: &'a Allocator,
) {
    // Keep only comments attached to a top-level statement; inner comments have
    // `attached_to` positions that match no top-level statement.
    let mut top_level_starts = FxHashSet::default();
    top_level_starts.insert(0u32);
    for stmt in &compiled.body {
        let start = stmt.span().start;
        if start > 0 {
            top_level_starts.insert(start);
        }
    }

    // Copy only comments attached to top-level statements.
    let mut comments = ArenaVec::with_capacity_in(source.comments.len(), &allocator);
    for comment in &source.comments {
        if top_level_starts.contains(&comment.attached_to) {
            comments.push(*comment);
        }
    }
    compiled.comments = comments;

    // Codegen reads comment content from `source_text` via span offsets, so the
    // compiled program must point at the same source as the original.
    compiled.source_text = source.source_text;
}
