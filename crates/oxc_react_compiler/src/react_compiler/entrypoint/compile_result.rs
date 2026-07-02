use oxc_diagnostics::Diagnostics;

use crate::react_compiler_diagnostics::SourceLocation;
use crate::react_compiler_hir::ReactFunctionType;

/// A variable rename from lowering, applied back to the program's scoping.
#[derive(Debug, Clone)]
pub struct BindingRenameInfo {
    pub original: String,
    pub renamed: String,
    pub declaration_start: u32,
}

/// Main result type returned by the compile function.
///
/// Stage 2: the compiled program is an arena-allocated oxc
/// [`oxc_ast::ast::Program`] (lifetime `'a` of the arena), built directly by the
/// codegen back-end (see `compile_program`) instead of the former Babel `File`.
#[derive(Debug)]
pub enum CompileResult<'a> {
    /// Compilation succeeded (or no functions needed compilation).
    /// `ast` is None if no changes were made to the program.
    Success {
        ast: Option<oxc_ast::ast::Program<'a>>,
        /// Errors and warnings accumulated during compilation.
        diagnostics: Diagnostics,
        /// Debug-log entries; populated only with the `debug` feature.
        ordered_log: Vec<OrderedLogItem>,
        /// Variable renames from lowering, for applying back to the AST.
        renames: Vec<BindingRenameInfo>,
    },
    /// A fatal error occurred and panicThreshold dictates it should throw.
    Error { diagnostics: Diagnostics, ordered_log: Vec<OrderedLogItem> },
}

/// A debug-log entry emitted during compilation (see `ProgramContext::log_debug`).
#[derive(Debug, Clone)]
pub enum OrderedLogItem {
    Debug { entry: DebugLogEntry },
}

/// Debug log entry for debugLogIRs support.
/// Currently only supports the 'debug' variant (string values).
#[derive(Debug, Clone)]
pub struct DebugLogEntry {
    pub kind: &'static str,
    pub name: String,
    pub value: String,
}

impl DebugLogEntry {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self { kind: "debug", name: name.into(), value: value.into() }
    }
}

/// Codegen output for a single compiled function.
///
/// Stage 2: the generated AST fields are now arena-allocated oxc nodes (lifetime
/// `'a`) instead of the former Babel-shaped `Identifier`/`PatternLike`/
/// `BlockStatement`. This is the type the back-end (`codegen_function`) produces
/// and the pipeline threads up to `compile_program`.
#[derive(Debug)]
pub struct CodegenFunction<'a> {
    pub loc: Option<SourceLocation>,
    pub id: Option<oxc_ast::ast::BindingIdentifier<'a>>,
    pub name_hint: Option<String>,
    pub params: oxc_allocator::Box<'a, oxc_ast::ast::FormalParameters<'a>>,
    pub body: oxc_allocator::Box<'a, oxc_ast::ast::FunctionBody<'a>>,
    pub generator: bool,
    pub is_async: bool,
    pub memo_slots_used: u32,
    pub memo_blocks: u32,
    pub memo_values: u32,
    pub pruned_memo_blocks: u32,
    pub pruned_memo_values: u32,
    pub outlined: Vec<OutlinedFunction<'a>>,
}

/// An outlined function extracted during compilation.
#[derive(Debug)]
pub struct OutlinedFunction<'a> {
    pub func: CodegenFunction<'a>,
    pub fn_type: Option<ReactFunctionType>,
}
