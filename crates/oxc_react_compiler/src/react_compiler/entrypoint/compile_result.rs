use oxc_diagnostics::Diagnostics;

use crate::react_compiler_diagnostics::Span;
use crate::react_compiler_hir::ReactFunctionType;

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
    },
    /// A fatal error occurred and panicThreshold dictates it should throw.
    Error { diagnostics: Diagnostics },
}

/// Debug log entry for debugLogIRs support.
/// Currently only supports the 'debug' variant (string values).
#[derive(Debug, Clone)]
pub struct DebugLogEntry;

impl DebugLogEntry {
    pub fn new(_name: impl Into<String>, _value: impl Into<String>) -> Self {
        Self
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
    pub span: Option<Span>,
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
