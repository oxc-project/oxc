use oxc_allocator::{Allocator, ArenaVec};

pub mod convert_scope;
pub mod diagnostics;
pub mod prefilter;
pub mod scope;

// Vendored React Compiler core crates (from oxc-project/forked-react-compiler),
// each crate flattened to a module. Kept near byte-for-byte with upstream for easy
// re-syncing, so `clippy::all` is relaxed here rather than editing the vendored code.
// They are `pub` so the public API may name types that originate inside them (e.g.
// `TransformResult::events`); being `pub` also keeps `dead_code` quiet on the parts
// the conversion layer doesn't reach, so only the few genuinely-dead private items
// carry their own targeted `#[allow(dead_code)]`.
#[allow(clippy::all)]
pub mod react_compiler;
#[allow(clippy::all)]
pub mod react_compiler_diagnostics;
#[allow(clippy::all)]
pub mod react_compiler_hir;
#[allow(clippy::all)]
pub mod react_compiler_inference;
#[allow(clippy::all)]
pub mod react_compiler_lowering;
#[allow(clippy::all)]
pub mod react_compiler_optimization;
#[allow(clippy::all)]
pub mod react_compiler_reactive_scopes;
#[allow(clippy::all)]
pub mod react_compiler_ssa;
#[allow(clippy::all)]
pub mod react_compiler_typeinference;
#[allow(clippy::all)]
pub mod react_compiler_utils;
#[allow(clippy::all)]
pub mod react_compiler_validation;

use crate::react_compiler::entrypoint::compile_result::CompileResult;
use crate::react_compiler::entrypoint::program::compile_program;
use convert_scope::convert_scope_info;
use prefilter::{has_react_like_functions, has_resource_management_declarations};

// Re-exported so integrations needn't depend on the upstream `react_compiler` crates.
pub use crate::react_compiler::entrypoint::plugin_options::{
    CompilerTarget, DynamicGatingConfig, GatingConfig, PluginOptions,
};
pub use crate::react_compiler_hir::environment_config::EnvironmentConfig;

use oxc_ast::ast::Program;
use oxc_diagnostics::Diagnostics;
use oxc_semantic::Semantic;
use oxc_span::GetSpan;
use rustc_hash::FxHashSet;

impl Default for PluginOptions {
    /// The compiler's standard defaults. Override fields with struct-update syntax:
    /// `PluginOptions { ..Default::default() }`.
    fn default() -> Self {
        PluginOptions {
            compilation_mode: "infer".to_string(),
            panic_threshold: "none".to_string(),
            target: CompilerTarget::Version("19".to_string()),
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

/// Shared compile pipeline behind [`transform`] and [`lint`]. Borrows `program`
/// (so `lint` can stay read-only) and returns the compiled OXC program — `None`
/// when nothing was compiled — together with diagnostics and logger events.
fn compile<'a>(
    program: &Program<'a>,
    semantic: &Semantic<'_>,
    allocator: &'a Allocator,
    options: PluginOptions,
) -> (Option<Program<'a>>, Diagnostics) {
    // Skip files with no React-like functions, unless the mode compiles everything.
    if !matches!(options.compilation_mode.as_str(), "all" | "annotation")
        && !has_react_like_functions(program)
    {
        return (None, Diagnostics::default());
    }

    // `using`/`await using` disposal semantics aren't preserved yet — skip the file.
    if has_resource_management_declarations(program) {
        return (None, Diagnostics::default());
    }

    // The codegen back-end builds oxc nodes directly via this `AstBuilder`, and the
    // compiled program is spliced/returned as an arena-allocated `Program<'a>`.
    let ast_builder = oxc_ast::builder::AstBuilder::new(allocator);
    let scope_info = convert_scope_info(semantic, program);
    // Function discovery and lowering both walk the oxc `Program` directly.
    let result = compile_program(&ast_builder, program, scope_info, options);

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
