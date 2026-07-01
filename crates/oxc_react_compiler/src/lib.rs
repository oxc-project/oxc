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

use crate::react_compiler::entrypoint::compile_result::{CompileResult, LoggerEvent};
use crate::react_compiler::entrypoint::program::compile_program;
use convert_scope::convert_scope_info;
use diagnostics::compile_result_to_diagnostics;
use prefilter::{has_react_like_functions, has_resource_management_declarations};

// Re-exported so integrations needn't depend on the upstream `react_compiler` crates.
pub use crate::react_compiler::entrypoint::plugin_options::{
    CompilerTarget, DynamicGatingConfig, GatingConfig, PluginOptions,
};
pub use crate::react_compiler_hir::environment_config::EnvironmentConfig;

use oxc_ast::ast::Program;
use oxc_diagnostics::Diagnostics;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType};
use rustc_hash::FxHashSet;

/// [`PluginOptions`] with the compiler's standard defaults (it has no `Default`).
/// Override fields with struct-update syntax: `PluginOptions { ..default_plugin_options() }`.
pub fn default_plugin_options() -> PluginOptions {
    PluginOptions {
        should_compile: true,
        enable_reanimated: false,
        is_dev: false,
        filename: None,
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
        source_code: None,
        profiling: false,
        debug: false,
    }
}

#[derive(Default)]
pub struct TransformResult {
    /// Whether `program` was rewritten. `false` means the compiler made no
    /// changes — no React-like functions, a bail-out, or nothing to memoize.
    pub changed: bool,
    /// Errors and warnings produced by the compile. Errors (e.g. Rules of Hooks
    /// violations) are hard problems in the source; the program is still left
    /// valid. Warnings include bail-outs where the compiler declined to optimize.
    pub diagnostics: Diagnostics,
    /// Raw structured logger events from the upstream compiler (compile
    /// success/skip/error with memoization stats), for tooling and profiling.
    /// Unlike `diagnostics`, these are not meant for user-facing reporting.
    pub events: Vec<LoggerEvent>,
}

pub struct LintResult {
    /// Errors and warnings produced by the compile.
    pub diagnostics: Diagnostics,
}

/// Run the React Compiler on a pre-parsed program, rewriting `program` in place
/// when it memoizes something. Builds the semantic model internally.
///
/// Must run **first**, on the pristine AST, before any other transform.
pub fn transform<'a>(
    program: &mut Program<'a>,
    allocator: &'a Allocator,
    options: PluginOptions,
) -> TransformResult {
    let (compiled, diagnostics, events) = compile(program, allocator, options);
    let changed = compiled.is_some();
    if let Some(compiled) = compiled {
        *program = compiled;
    }
    TransformResult { changed, diagnostics, events }
}

/// Shared compile pipeline behind [`transform`] and [`lint`]. Borrows `program`
/// (so `lint` can stay read-only) and returns the compiled OXC program — `None`
/// when nothing was compiled — together with diagnostics and logger events.
/// Containing the semantic borrow inside this call is what lets `transform`
/// write the result back into its `&mut program` afterwards.
fn compile<'a>(
    program: &Program<'a>,
    allocator: &'a Allocator,
    options: PluginOptions,
) -> (Option<Program<'a>>, Diagnostics, Vec<LoggerEvent>) {
    let source_text = program.source_text;

    // The HIR lowering computes `SourceLocation` line/column from a line-offset
    // table built off `context.code` (see `pipeline::compile_fn`). The oxc
    // front-end derives locations on demand from the source text, so the source
    // must be threaded through. Without it, every loc collapses to
    // `line = 1, column = byte_offset`, which surfaces as wrong `(line:col)`
    // suffixes in diagnostics.
    let mut options = options;
    if options.source_code.is_none() {
        options.source_code = Some(source_text.to_string());
    }

    // Skip files with no React-like functions, unless the mode compiles everything.
    if !matches!(options.compilation_mode.as_str(), "all" | "annotation")
        && !has_react_like_functions(program)
    {
        return (None, Diagnostics::default(), Vec::new());
    }

    // `using`/`await using` disposal semantics aren't preserved yet — skip the file.
    if has_resource_management_declarations(program) {
        return (None, Diagnostics::default(), Vec::new());
    }

    let semantic = SemanticBuilder::new().with_build_nodes(true).build(program).semantic;

    // The codegen back-end builds oxc nodes directly via this `AstBuilder`, and the
    // compiled program is spliced/returned as an arena-allocated `Program<'a>`.
    let ast_builder = oxc_ast::builder::AstBuilder::new(allocator);
    let scope_info = convert_scope_info(&semantic, program);
    // Function discovery and lowering both walk the oxc `Program` directly.
    let result = compile_program(&ast_builder, program, scope_info, options);

    let diagnostics = compile_result_to_diagnostics(&result);
    let (program_ast, events) = match result {
        CompileResult::Success { ast, events, .. } => (ast, events),
        CompileResult::Error { events, .. } => (None, events),
    };

    let compiled = program_ast.map(|mut compiled: Program<'a>| {
        compiled.source_type = program.source_type;
        preserve_comments(&mut compiled, program, allocator);
        compiled
    });

    (compiled, diagnostics, events)
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

/// Convenience wrapper — parses source text then transforms it in place, returning
/// the (possibly rewritten) program together with the result.
pub fn transform_source<'a>(
    source_text: &'a str,
    source_type: SourceType,
    allocator: &'a Allocator,
    options: PluginOptions,
) -> (Program<'a>, TransformResult) {
    let mut program = Parser::new(allocator, source_text, source_type).parse().program;
    let result = transform(&mut program, allocator, options);
    (program, result)
}

/// Lint a pre-parsed program — like [`transform`] but read-only: it collects
/// diagnostics without rewriting the program.
pub fn lint(program: &Program, options: PluginOptions) -> LintResult {
    let mut options = options;
    options.no_emit = true;

    // `no_emit` produces no program; a local arena for the conversion suffices.
    let allocator = Allocator::default();
    let (_program, diagnostics, _events) = compile(program, &allocator, options);
    LintResult { diagnostics }
}

/// Convenience wrapper — parses source text, runs semantic analysis, then lints.
pub fn lint_source(
    source_text: &str,
    source_type: SourceType,
    options: PluginOptions,
) -> LintResult {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source_text, source_type).parse();
    lint(&parsed.program, options)
}
