use crate::react_compiler_diagnostics::SourceLocation;
use crate::react_compiler_hir::ReactFunctionType;

/// Source location with index and filename fields for logger event serialization.
/// Matches the Babel SourceLocation format that the TS compiler emits in logger events.
#[derive(Debug, Clone)]
pub struct LoggerSourceLocation {
    pub start: LoggerPosition,
    pub end: LoggerPosition,
    pub filename: Option<String>,
    pub identifier_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoggerPosition {
    pub line: u32,
    pub column: u32,
    pub index: Option<u32>,
}

impl LoggerSourceLocation {
    /// Create from a diagnostics SourceLocation, adding index and filename.
    pub fn from_loc(
        loc: &SourceLocation,
        filename: Option<&str>,
        start_index: Option<u32>,
        end_index: Option<u32>,
    ) -> Self {
        Self {
            start: LoggerPosition {
                line: loc.start.line,
                column: loc.start.column,
                index: start_index,
            },
            end: LoggerPosition { line: loc.end.line, column: loc.end.column, index: end_index },
            filename: filename.map(|s| s.to_string()),
            identifier_name: None,
        }
    }

    /// Create from a diagnostics SourceLocation without index or filename.
    pub fn from_loc_simple(loc: &SourceLocation) -> Self {
        Self {
            start: LoggerPosition { line: loc.start.line, column: loc.start.column, index: None },
            end: LoggerPosition { line: loc.end.line, column: loc.end.column, index: None },
            filename: None,
            identifier_name: None,
        }
    }
}

/// A variable rename from lowering, serialized for the JS shim.
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
        events: Vec<LoggerEvent>,
        /// Unified ordered log interleaving events and debug entries.
        /// Items appear in the order they were emitted during compilation.
        /// The JS side uses this as the single source of truth (preferred over
        /// separate events/debugLogs arrays).
        ordered_log: Vec<OrderedLogItem>,
        /// Variable renames from lowering, for applying back to the Babel AST.
        /// Each entry maps an original binding name to its renamed version,
        /// identified by the binding's declaration start position in the source.
        renames: Vec<BindingRenameInfo>,
    },
    /// A fatal error occurred and panicThreshold dictates it should throw.
    Error { error: CompilerErrorInfo, events: Vec<LoggerEvent>, ordered_log: Vec<OrderedLogItem> },
}

/// An item in the ordered log, which can be either a logger event or a debug entry.
#[derive(Debug, Clone)]
pub enum OrderedLogItem {
    Event { event: LoggerEvent },
    Debug { entry: DebugLogEntry },
}

/// Structured error information for the JS shim.
#[derive(Debug, Clone)]
pub struct CompilerErrorInfo {
    pub reason: String,
    pub description: Option<String>,
    pub details: Vec<CompilerErrorDetailInfo>,
    /// When set, the JS shim should throw an Error with this exact message
    /// instead of formatting through formatCompilerError(). This is used
    /// for simulated unknown exceptions (throwUnknownException__testonly)
    /// which in the TS compiler are plain Error objects, not CompilerErrors.
    pub raw_message: Option<String>,
    /// Pre-formatted error message produced by Rust, matching the JS
    /// formatCompilerError() output. When present, the JS shim uses this
    /// directly instead of calling formatCompilerError() on the JS side.
    pub formatted_message: Option<String>,
}

/// Serializable error detail — flat plain object matching the TS
/// `formatDetailForLogging()` output. All fields are direct properties.
#[derive(Debug, Clone)]
pub struct CompilerErrorDetailInfo {
    pub category: String,
    pub reason: String,
    pub description: Option<String>,
    pub severity: String,
    pub suggestions: Option<Vec<LoggerSuggestionInfo>>,
    pub details: Option<Vec<CompilerErrorItemInfo>>,
    pub loc: Option<LoggerSourceLocation>,
}

/// Serializable suggestion info for logger events.
#[derive(Debug, Clone)]
pub struct LoggerSuggestionInfo {
    pub description: String,
    pub op: LoggerSuggestionOp,
    pub range: (usize, usize),
    pub text: Option<String>,
}

/// Numeric enum matching TS `CompilerSuggestionOperation`.
#[derive(Debug, Clone, Copy)]
pub enum LoggerSuggestionOp {
    InsertBefore = 0,
    InsertAfter = 1,
    Remove = 2,
    Replace = 3,
}

/// Individual error or hint item within a CompilerErrorDetailInfo.
#[derive(Debug, Clone)]
pub struct CompilerErrorItemInfo {
    pub kind: String,
    pub loc: Option<LoggerSourceLocation>,
    /// Serialized as `null` when None (not omitted), matching TS behavior.
    pub message: Option<String>,
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

/// Logger events emitted during compilation.
/// These are returned to JS for the logger callback.
#[derive(Debug, Clone)]
pub enum LoggerEvent {
    CompileSuccess {
        fn_loc: Option<LoggerSourceLocation>,
        fn_name: Option<String>,
        memo_slots: u32,
        memo_blocks: u32,
        memo_values: u32,
        pruned_memo_blocks: u32,
        pruned_memo_values: u32,
    },
    CompileError {
        detail: CompilerErrorDetailInfo,
        fn_loc: Option<LoggerSourceLocation>,
    },
    /// Same as CompileError but serializes fnLoc before detail (matching TS program.ts output)
    CompileErrorWithLoc {
        fn_loc: LoggerSourceLocation,
        detail: CompilerErrorDetailInfo,
    },
    CompileSkip {
        fn_loc: Option<LoggerSourceLocation>,
        reason: String,
        loc: Option<LoggerSourceLocation>,
    },
    CompileUnexpectedThrow {
        fn_loc: Option<LoggerSourceLocation>,
        data: String,
    },
    PipelineError {
        fn_loc: Option<LoggerSourceLocation>,
        data: String,
    },
}
