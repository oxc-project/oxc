use oxc_diagnostics::Diagnostics;

use oxc_span::Span;
use oxc_str::Ident;

use crate::react_compiler_hir::ReactFunctionType;

use super::program::CompileOutput;

/// Main result type returned by the compile function.
#[allow(clippy::large_enum_variant)] // built once per compile; `ProgramContext` carries the options
pub enum CompileResult<'a> {
    /// Compilation succeeded (or no functions needed compilation).
    /// `output` is None if no changes are to be made to the program — always so
    /// in lint output mode.
    Success {
        output: Option<CompileOutput<'a>>,
        /// Errors and warnings accumulated during compilation.
        diagnostics: Diagnostics,
    },
    /// A fatal error occurred and panicThreshold dictates it should throw.
    Error { diagnostics: Diagnostics },
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
    pub name_hint: Option<Ident<'a>>,
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
