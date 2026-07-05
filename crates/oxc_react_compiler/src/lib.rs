use oxc_allocator::{Allocator, ArenaVec};

mod diagnostics;
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
use prefilter::{has_react_like_functions, has_resource_management_declarations};
use scope::ScopeResolver;

// Re-exported so integrations needn't depend on the upstream `react_compiler` crates.
pub use crate::react_compiler::entrypoint::plugin_options::{
    CompilerOutputMode, CompilerTarget, DynamicGatingConfig, GatingConfig, PluginOptions,
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
/// borrowed `semantic` must have been built from that same pristine AST; the
/// `AstNodes` table is not needed, so `SemanticBuilder::with_build_nodes(false)`
/// is sufficient (and cheaper).
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
    // Check for existing runtime imports (file already compiled).
    if has_memo_cache_function_import(program, &get_react_compiler_runtime_module(&options.target))
    {
        return (None, Diagnostics::default());
    }

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
    let scope = ScopeResolver::new(semantic, program);
    // Function discovery and lowering both walk the oxc `Program` directly.
    let result = compile_program(&ast_builder, program, &scope, options);

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

fn has_memo_cache_function_import(program: &Program<'_>, module_name: &str) -> bool {
    for stmt in &program.body {
        if let oxc_ast::ast::Statement::ImportDeclaration(import) = stmt
            && import.source.value == module_name
            && import.import_kind.is_value()
            && let Some(specifiers) = &import.specifiers
        {
            for specifier in specifiers {
                if let oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(data) = specifier
                    && data.import_kind.is_value()
                {
                    let imported_name = match &data.imported {
                        oxc_ast::ast::ModuleExportName::IdentifierName(id) => {
                            Some(id.name.as_str())
                        }
                        oxc_ast::ast::ModuleExportName::IdentifierReference(id) => {
                            Some(id.name.as_str())
                        }
                        oxc_ast::ast::ModuleExportName::StringLiteral(s) => Some(s.value.as_str()),
                    };
                    if imported_name == Some("c") {
                        return true;
                    }
                }
            }
        }
    }
    false
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
/// The borrowed `semantic` must have been built from `program`; the `AstNodes`
/// table is not needed, so `SemanticBuilder::with_build_nodes(false)` suffices.
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
