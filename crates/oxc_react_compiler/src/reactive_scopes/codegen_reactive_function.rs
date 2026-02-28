/// Code generation from reactive function to output AST.
///
/// Port of `ReactiveScopes/CodegenReactiveFunction.ts` from the React Compiler.
///
/// Converts the reactive function tree into an output AST (using oxc_ast types).
/// This is the final pass in the compilation pipeline that produces the
/// optimized JavaScript code with memoization.
///
/// Key output structures:
/// - `useMemoCache(N)` call at the top of the function for cache initialization
/// - `$[idx] !== dep` checks for dependency changes
/// - `$[idx] = value` assignments to cache new values
/// - `$[idx]` reads for cached values
use hmac::{Hmac, Mac};
use oxc_syntax::operator::LogicalOperator;
use rustc_hash::{FxHashMap, FxHashSet};
use sha2::Sha256;

use crate::{
    compiler_error::{CompilerError, SourceLocation},
    hir::{
        ArrayExpressionElement, ArrayPatternElement, CallArg, DeclarationId,
        FunctionExpressionType, HIRFunction, IdentifierId, IdentifierName, InstructionKind,
        InstructionValue, JsxAttribute, JsxTag, ObjectExpression, ObjectMethodValue,
        ObjectPatternProperty, ObjectPropertyKey, ObjectPropertyType, Pattern, Place,
        PrimitiveValueKind, ReactiveBlock, ReactiveBreakTerminal, ReactiveContinueTerminal,
        ReactiveFunction, ReactiveInstruction, ReactiveParam, ReactiveScope,
        ReactiveScopeDeclaration, ReactiveScopeDependency, ReactiveStatement, ReactiveTerminal,
        ReactiveTerminalTargetKind, ReactiveValue,
        environment::{CompilerOutputMode, ExternalFunction, InstrumentationConfig},
        object_shape::ShapeRegistry,
        types::Type,
    },
    utils::runtime_diagnostic_constants::GuardKind,
};

use super::visitors::{ReactiveVisitor, visit_reactive_block};

/// Sentinel values used in the output code.
pub const MEMO_CACHE_SENTINEL: &str = "react.memo_cache_sentinel";
pub const EARLY_RETURN_SENTINEL: &str = "react.early_return_sentinel";

// =====================================================================================
// CodegenFunction — the final output of code generation
// =====================================================================================

/// Result of code generation.
#[derive(Debug, Clone)]
pub struct CodegenFunction {
    /// Name of the function (if it had one).
    pub id: Option<String>,
    /// Name hint for anonymous functions.
    pub name_hint: Option<String>,
    /// Function parameters as formatted strings.
    pub params: Vec<String>,
    /// Whether the function is a generator.
    pub generator: bool,
    /// Whether the function is async.
    pub is_async: bool,
    /// Source location.
    pub loc: SourceLocation,
    /// Number of memo cache slots used.
    pub memo_slots_used: u32,
    /// Number of memo blocks (reactive scopes).
    pub memo_blocks: u32,
    /// Number of individual memo values.
    pub memo_values: u32,
    /// Number of pruned memo blocks.
    pub pruned_memo_blocks: u32,
    /// Number of pruned memo values.
    pub pruned_memo_values: u32,
    /// The generated body statements.
    pub body: Vec<CodegenStatement>,
    /// Directives (e.g. "use strict").
    pub directives: Vec<String>,
    /// Outlined functions that were extracted from this function.
    pub outlined: Vec<OutlinedFunction>,
}

/// A function that was outlined (extracted) from the main function.
#[derive(Debug, Clone)]
pub struct OutlinedFunction {
    pub fn_: CodegenFunction,
    pub fn_type: Option<crate::hir::ReactFunctionType>,
}

// =====================================================================================
// CodegenStatement / CodegenExpression — lightweight IR for codegen output
// =====================================================================================

/// A generated statement in the output.
#[derive(Debug, Clone)]
pub enum CodegenStatement {
    /// A variable declaration: `const x = expr;` or `let x;`
    VariableDeclaration { kind: VarKind, name: String, init: Option<String> },
    /// An expression statement: `expr;`
    ExpressionStatement(String),
    /// A block: `{ ... }`
    Block(Vec<CodegenStatement>),
    /// An if statement: `if (test) { consequent } else { alternate }`
    If { test: String, consequent: Vec<CodegenStatement>, alternate: Option<Vec<CodegenStatement>> },
    /// A return statement: `return expr;`
    /// `is_conditional` is true when the returned expression is a ternary,
    /// sequence, or optional-call expression that needs parenthesisation in
    /// concise arrow body position.
    Return { value: Option<String>, is_conditional: bool },
    /// A for statement: `for (init; test; update) { body }`
    For {
        init: Option<String>,
        test: Option<String>,
        update: Option<String>,
        body: Vec<CodegenStatement>,
    },
    /// A for-of statement: `for (left of right) { body }`
    ForOf { kind: VarKind, left: String, right: String, body: Vec<CodegenStatement> },
    /// A for-in statement: `for (left in right) { body }`
    ForIn { kind: VarKind, left: String, right: String, body: Vec<CodegenStatement> },
    /// A while statement: `while (test) { body }`
    While { test: String, body: Vec<CodegenStatement> },
    /// A do-while statement: `do { body } while (test);`
    DoWhile { body: Vec<CodegenStatement>, test: String },
    /// A switch statement: `switch (discriminant) { cases }`
    Switch { discriminant: String, cases: Vec<CodegenSwitchCase> },
    /// A break statement: `break;` or `break label;`
    Break(Option<String>),
    /// A continue statement: `continue;` or `continue label;`
    Continue(Option<String>),
    /// A try statement: `try { block } catch (param) { handler } finally { finalizer }`
    Try {
        block: Vec<CodegenStatement>,
        handler_param: Option<String>,
        handler: Option<Vec<CodegenStatement>>,
        finalizer: Option<Vec<CodegenStatement>>,
    },
    /// A throw statement: `throw expr;`
    Throw(String),
    /// A labeled statement: `label: stmt`
    Labeled { label: String, body: Box<CodegenStatement> },
    /// A function declaration: `function name(params) { body }`
    FunctionDeclaration {
        name: String,
        params: Vec<String>,
        body: Vec<CodegenStatement>,
        generator: bool,
        is_async: bool,
    },
    /// A debugger statement: `debugger;`
    Debugger,
    /// An empty statement (used for suppressed statements).
    Empty,
}

/// A switch case.
#[derive(Debug, Clone)]
pub struct CodegenSwitchCase {
    /// `None` for the default case.
    pub test: Option<String>,
    pub body: Vec<CodegenStatement>,
}

/// Kind of variable declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarKind {
    Const,
    Let,
}

impl VarKind {
    fn as_str(self) -> &'static str {
        match self {
            VarKind::Const => "const",
            VarKind::Let => "let",
        }
    }
}

// =====================================================================================
// CodegenOptions and CodegenContext
// =====================================================================================

/// Options for code generation.
#[derive(Debug, Clone)]
pub struct CodegenOptions {
    /// Unique identifiers used in the function (from RenameVariables).
    pub unique_identifiers: FxHashSet<String>,
    /// Identifiers that are fbt operands (from MemoizeFbt).
    pub fbt_operands: FxHashSet<IdentifierId>,
    /// Whether to enable HMR/Fast Refresh cache reset on source file changes.
    pub enable_reset_cache_on_source_file_changes: bool,
    /// The source code of the component, used for computing the hash for HMR cache reset.
    pub code: Option<String>,
    /// Hook guard configuration. When set, wraps function body and hook calls
    /// with runtime hook guard diagnostics.
    pub enable_emit_hook_guards: Option<ExternalFunction>,
    /// Instrumentation configuration for instrument forget emission.
    /// When set (and output mode is Client and function has a name),
    /// emits an instrumentation call at the start of compiled components.
    pub enable_emit_instrument_forget: Option<InstrumentationConfig>,
    /// The function name, needed for the instrument forget call argument.
    pub fn_id: Option<String>,
    /// The source filename, needed for the instrument forget call argument.
    pub filename: Option<String>,
    /// The compiler output mode.
    pub output_mode: CompilerOutputMode,
    /// The shape registry for looking up function signatures (needed for hook detection).
    pub shapes: ShapeRegistry,
    /// Whether to wrap anonymous functions in a naming expression.
    pub enable_name_anonymous_functions: bool,
}

/// Codegen context — tracks state during code generation.
pub struct CodegenContext {
    /// Next cache slot index to allocate.
    pub next_cache_index: u32,
    /// Tracks which declarations have been emitted, keyed by DeclarationId.
    declarations: FxHashSet<DeclarationId>,
    /// Maps temporary DeclarationId to expression string (or None if declared but no value yet).
    pub temp: FxHashMap<DeclarationId, Option<String>>,
    /// Unique identifiers set (for synthesized name deduplication).
    unique_identifiers: FxHashSet<String>,
    /// Map from original name to synthesized unique name.
    synthesized_names: FxHashMap<String, String>,
    /// Identifiers that are fbt operands (used for JSX attribute codegen).
    fbt_operands: FxHashSet<IdentifierId>,
    /// Hook guard configuration for emitting runtime hook diagnostics.
    enable_emit_hook_guards: Option<ExternalFunction>,
    /// The compiler output mode (needed for hook guard checks).
    output_mode: CompilerOutputMode,
    /// The shape registry for looking up function signatures (needed for hook detection).
    shapes: ShapeRegistry,
    /// Stored ObjectMethod values keyed by their lvalue's IdentifierId.
    /// These are stored during instruction codegen and consumed in ObjectExpression codegen.
    object_methods: FxHashMap<IdentifierId, ObjectMethodValue>,
    /// Whether to wrap anonymous functions in a naming expression.
    enable_name_anonymous_functions: bool,
    /// Temporaries that hold JSX text values (render as raw text in JSX children).
    jsx_text_temps: FxHashSet<DeclarationId>,
    /// Temporaries that hold JSX element/fragment values (render without {…} wrapper).
    jsx_element_temps: FxHashSet<DeclarationId>,
    /// Temporaries that hold assignment expressions (need parens when used in expression context).
    /// e.g., `x = value` needs wrapping as `(x = value)` when used as call args or initializers.
    assignment_temps: FxHashSet<DeclarationId>,
    /// Temporaries that hold logical expressions, keyed by `DeclarationId`, valued by
    /// the `LogicalOperator` of the expression. Used to add mandatory parentheses when
    /// `??` is mixed with `||`/`&&` (a JavaScript syntax error without parens).
    logical_temps: FxHashMap<DeclarationId, LogicalOperator>,
    /// Temporaries that hold function expressions (arrow or regular).
    /// Used for IIFE parenthesization: when a function expression temp is used
    /// as a callee, it needs wrapping as `(fn)()` to avoid parsing ambiguity.
    fn_expr_temps: FxHashSet<DeclarationId>,
    /// Temporaries that hold ternary (conditional), sequence, or optional-call
    /// expressions.  These need parenthesisation when used as a concise arrow
    /// function body (Babel's printer adds parens automatically; we must do it
    /// explicitly).
    arrow_paren_temps: FxHashSet<DeclarationId>,
    /// Temporaries that hold ternary (conditional) expressions.
    /// These need wrapping in parens when used as binary expression operands
    /// because `?:` has lower precedence than any arithmetic/relational operator.
    /// e.g., `x + cond ? a : b` must become `x + (cond ? a : b)`.
    ternary_temps: FxHashSet<DeclarationId>,
    /// Whether we are currently generating code inside an optional-chain Sequence
    /// (the `Sequence` node wrapped by an `OptionalCall`).
    ///
    /// When `true`, a `PropertyLoad` or `ComputedLoad` whose object is an
    /// optional-chain expression should NOT be parenthesised, because the
    /// `.prop` / `[prop]` access is a continuation of the chain (e.g. the `.c`
    /// in `props?.b.c`) rather than a separate, unconditional access applied to
    /// the chain's result (e.g. the `.b` in `(props?.a).b`).
    inside_optional_chain: bool,
    /// Structured `CodegenFunction` data for function expression temporaries
    /// that have `FunctionDeclaration` type. When a `StoreLocal` with
    /// `InstructionKind::Function` or `HoistedFunction` is emitted, this map
    /// is consulted to produce a proper `CodegenStatement::FunctionDeclaration`
    /// instead of an expression statement.
    fn_decl_data: FxHashMap<DeclarationId, CodegenFunction>,
}

impl CodegenContext {
    fn new(
        unique_identifiers: FxHashSet<String>,
        fbt_operands: FxHashSet<IdentifierId>,
        enable_emit_hook_guards: Option<ExternalFunction>,
        output_mode: CompilerOutputMode,
        shapes: ShapeRegistry,
        enable_name_anonymous_functions: bool,
    ) -> Self {
        Self {
            next_cache_index: 0,
            declarations: FxHashSet::default(),
            temp: FxHashMap::default(),
            unique_identifiers,
            synthesized_names: FxHashMap::default(),
            fbt_operands,
            enable_emit_hook_guards,
            output_mode,
            shapes,
            object_methods: FxHashMap::default(),
            enable_name_anonymous_functions,
            jsx_text_temps: FxHashSet::default(),
            jsx_element_temps: FxHashSet::default(),
            assignment_temps: FxHashSet::default(),
            logical_temps: FxHashMap::default(),
            fn_expr_temps: FxHashSet::default(),
            arrow_paren_temps: FxHashSet::default(),
            ternary_temps: FxHashSet::default(),
            inside_optional_chain: false,
            fn_decl_data: FxHashMap::default(),
        }
    }

    /// Check whether an identifier is an fbt operand.
    pub fn is_fbt_operand(&self, id: IdentifierId) -> bool {
        self.fbt_operands.contains(&id)
    }

    /// Allocate the next cache index.
    fn alloc_cache_index(&mut self) -> u32 {
        let idx = self.next_cache_index;
        self.next_cache_index += 1;
        idx
    }

    /// Record that an identifier has been declared.
    fn declare(&mut self, decl_id: DeclarationId) {
        self.declarations.insert(decl_id);
    }

    /// Check whether an identifier has already been declared.
    fn has_declared(&self, decl_id: DeclarationId) -> bool {
        self.declarations.contains(&decl_id)
    }

    /// Synthesize a unique name based on a base name.
    fn synthesize_name(&mut self, name: &str) -> String {
        if let Some(existing) = self.synthesized_names.get(name) {
            return existing.clone();
        }
        let mut validated = name.to_string();
        let mut index = 0u32;
        while self.unique_identifiers.contains(&validated) {
            validated = format!("{name}{index}");
            index += 1;
        }
        self.unique_identifiers.insert(validated.clone());
        self.synthesized_names.insert(name.to_string(), validated.clone());
        validated
    }
}

// =====================================================================================
// Entry point: codegen_function
// =====================================================================================

/// Generate code from a reactive function.
///
/// # Errors
/// Returns a `CompilerError` if code generation fails.
/// Currently always succeeds; error path will be used once invariant checks are added.
#[expect(clippy::unnecessary_wraps)]
pub fn codegen_function(
    reactive_fn: &ReactiveFunction,
    options: CodegenOptions,
) -> Result<CodegenFunction, CompilerError> {
    let mut cx = CodegenContext::new(
        options.unique_identifiers,
        options.fbt_operands,
        options.enable_emit_hook_guards,
        options.output_mode,
        options.shapes,
        options.enable_name_anonymous_functions,
    );

    // Fast Refresh reuses component instances at runtime even as the source of the
    // component changes. The generated code needs to prevent values from one version of
    // the code being reused after a code change.
    // If HMR detection is enabled and we know the source code of the component, assign a
    // cache slot to track the source hash, and later, emit code to check for source
    // changes and reset the cache on source changes.
    let fast_refresh_state = if options.enable_reset_cache_on_source_file_changes {
        options.code.map(|code| {
            let hash = compute_source_hash(&code);
            let cache_index = cx.alloc_cache_index();
            FastRefreshState { cache_index, hash }
        })
    } else {
        None
    };

    // Register function params as declared and as temporaries
    for param in &reactive_fn.params {
        let place = match param {
            crate::hir::ReactiveParam::Place(p) => p,
            crate::hir::ReactiveParam::Spread(s) => &s.place,
        };
        cx.temp.insert(place.identifier.declaration_id, None);
        cx.declare(place.identifier.declaration_id);
    }

    // Generate the function body
    let mut body = codegen_block(&mut cx, &reactive_fn.body);

    // Remove trailing `return undefined` / `return;`
    if matches!(body.last(), Some(CodegenStatement::Return { value: None, .. })) {
        body.pop();
    }

    // Function-level hook guard: wrap the entire body in a try-finally with
    // PushHookGuard/PopHookGuard if enableEmitHookGuards is set and output mode is client.
    if cx.output_mode == CompilerOutputMode::Client {
        if let Some(specifier_name) =
            cx.enable_emit_hook_guards.as_ref().map(|g| g.import_specifier_name.clone())
        {
            let guard_fn_name = cx.synthesize_name(&specifier_name);
            body = vec![create_hook_guard(
                &guard_fn_name,
                body,
                GuardKind::PushHookGuard,
                GuardKind::PopHookGuard,
            )];
        }
    }

    // Count memo blocks/values
    let mut counter = MemoCounter::default();
    visit_reactive_block(&reactive_fn.body, &mut counter);

    // Insert the `const $ = _c(N);` preamble if there are cache slots
    let cache_count = cx.next_cache_index;
    if cache_count > 0 {
        let cache_name = cx.synthesize_name("$");
        let memo_cache_fn = cx.synthesize_name("_c");
        let mut preface: Vec<CodegenStatement> = Vec::new();

        preface.push(CodegenStatement::VariableDeclaration {
            kind: VarKind::Const,
            name: cache_name.clone(),
            init: Some(format!("{memo_cache_fn}({cache_count})")),
        });

        // Emit HMR/Fast Refresh hash check and cache reset
        if let Some(ref state) = fast_refresh_state {
            let index_name = cx.synthesize_name("$i");
            preface.push(CodegenStatement::If {
                test: format!("{cache_name}[{}] !== \"{}\"", state.cache_index, state.hash),
                consequent: vec![
                    CodegenStatement::For {
                        init: Some(format!("let {index_name} = 0")),
                        test: Some(format!("{index_name} < {cache_count}")),
                        update: Some(format!("{index_name} += 1")),
                        body: vec![CodegenStatement::ExpressionStatement(format!(
                            "{cache_name}[{index_name}] = Symbol.for(\"{MEMO_CACHE_SENTINEL}\")"
                        ))],
                    },
                    CodegenStatement::ExpressionStatement(format!(
                        "{cache_name}[{}] = \"{}\"",
                        state.cache_index, state.hash
                    )),
                ],
                alternate: None,
            });
        }

        // Prepend all preface statements to the body
        for (i, stmt) in preface.into_iter().enumerate() {
            body.insert(i, stmt);
        }
    }

    // Emit instrument forget: if the config is set, the function has a name,
    // and the output mode is Client, prepend an instrumentation call at the
    // start of the function body.
    //
    // Port of CodegenReactiveFunction.ts lines 247-307.
    //
    // Generated pattern:
    //   if (globalGating && gatingFn) { instrumentFn("fnName", "filename"); }
    //
    // This uses `.unshift()` in the TS version — i.e., inserted at position 0.
    if let Some(ref instrument_config) = options.enable_emit_instrument_forget
        && options.fn_id.is_some()
        && options.output_mode == CompilerOutputMode::Client
    {
        let fn_name = options.fn_id.as_deref().unwrap_or("");
        let filename = options.filename.as_deref().unwrap_or("");

        // Build the gating condition
        let gating_expr =
            instrument_config.gating.as_ref().map(|g| g.import_specifier_name.clone());
        let global_gating_expr = instrument_config.global_gating.clone();

        let if_test = match (&gating_expr, &global_gating_expr) {
            (Some(gating), Some(global)) => format!("{global} && {gating}"),
            (Some(gating), None) => gating.clone(),
            (None, Some(global)) => global.clone(),
            (None, None) => {
                // This should not happen — validated in validate_environment_config.
                // But if it does, skip instrumentation rather than panic.
                String::new()
            }
        };

        if !if_test.is_empty() {
            let instrument_fn_name = &instrument_config.func.import_specifier_name;
            let instrument_call = format!(
                "{instrument_fn_name}(\"{}\", \"{}\")",
                escape_string(fn_name),
                escape_string(filename),
            );
            let instrument_if = CodegenStatement::If {
                test: if_test,
                consequent: vec![CodegenStatement::ExpressionStatement(instrument_call)],
                alternate: None,
            };
            body.insert(0, instrument_if);
        }
    }

    let params = convert_params(&reactive_fn.params);

    Ok(CodegenFunction {
        id: reactive_fn.id.clone(),
        name_hint: reactive_fn.name_hint.clone(),
        params,
        generator: reactive_fn.generator,
        is_async: reactive_fn.is_async,
        loc: reactive_fn.loc,
        memo_slots_used: cache_count,
        memo_blocks: counter.memo_blocks,
        memo_values: counter.memo_values,
        pruned_memo_blocks: counter.pruned_memo_blocks,
        pruned_memo_values: counter.pruned_memo_values,
        body,
        directives: reactive_fn.directives.clone(),
        outlined: Vec::new(),
    })
}

/// Convert reactive function parameters to formatted string representations.
///
/// Port of `convertParameter()` from `CodegenReactiveFunction.ts` lines 410-418.
fn convert_params(params: &[ReactiveParam]) -> Vec<String> {
    params
        .iter()
        .map(|param| match param {
            ReactiveParam::Place(place) => identifier_name(&place.identifier),
            ReactiveParam::Spread(spread) => {
                format!("...{}", identifier_name(&spread.place.identifier))
            }
        })
        .collect()
}

/// Run the sub-pipeline on an inner HIR function (e.g., FunctionExpression or ObjectMethod)
/// and produce a `CodegenFunction`.
///
/// This mirrors the pattern in the TypeScript where `buildReactiveFunction` is called,
/// followed by pruning passes, `renameVariables`, and then `codegenReactiveFunction`.
///
/// `include_prune_hoisted` controls whether `pruneHoistedContexts` is called.
/// FunctionExpression passes `true`, ObjectMethod passes `false`.
fn codegen_inner_function(
    func: &HIRFunction,
    cx: &CodegenContext,
    include_prune_hoisted: bool,
) -> Result<CodegenFunction, CompilerError> {
    let mut reactive_fn =
        crate::reactive_scopes::build_reactive_function::build_reactive_function(func)?;
    crate::reactive_scopes::prune_unused_labels::prune_unused_labels(&mut reactive_fn);
    crate::reactive_scopes::prune_unused_lvalues::prune_unused_lvalues(&mut reactive_fn);
    if include_prune_hoisted {
        crate::reactive_scopes::prune::prune_hoisted_contexts(&mut reactive_fn);
    }
    // NOTE: We do NOT call rename_variables here, matching the TS reference.
    // In TS, codegenReactiveFunction for FunctionExpression/ObjectMethod
    // passes the parent's cx.uniqueIdentifiers and cx.temp — no separate
    // renameVariables call. The outer rename_variables already visited
    // the inner HIR function (via visitHirFunction), so all identifiers
    // are already renamed with the parent's naming state.
    let options = CodegenOptions {
        unique_identifiers: cx.unique_identifiers.clone(),
        fbt_operands: cx.fbt_operands.clone(),
        enable_reset_cache_on_source_file_changes: false,
        code: None,
        enable_emit_hook_guards: cx.enable_emit_hook_guards.clone(),
        enable_emit_instrument_forget: None,
        fn_id: None,
        filename: None,
        output_mode: cx.output_mode,
        shapes: cx.shapes.clone(),
        enable_name_anonymous_functions: cx.enable_name_anonymous_functions,
    };
    codegen_function(&reactive_fn, options)
}

/// Format `CodegenFunction` body statements as a string for embedding in a function expression.
///
/// This produces the contents between `{` and `}` for function bodies, including directives.
fn format_codegen_body(func: &CodegenFunction) -> String {
    use std::fmt::Write;

    let has_content = !func.directives.is_empty() || !func.body.is_empty();
    if !has_content {
        return String::new();
    }

    let mut body = String::new();
    body.push('\n');
    for directive in &func.directives {
        let _ = writeln!(body, "  \"{directive}\";");
    }
    for stmt in &func.body {
        // Use the StmtAtIndent wrapper to write statements at indent level 1
        let _ = write!(body, "{}", StmtAtIndent(stmt, 1));
    }
    body
}

/// Helper struct to format a `CodegenStatement` at a specific indentation level.
struct StmtAtIndent<'a>(&'a CodegenStatement, usize);

impl std::fmt::Display for StmtAtIndent<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_statement(f, self.0, self.1)
    }
}

/// State for HMR/Fast Refresh cache reset.
struct FastRefreshState {
    /// The cache index allocated for tracking the source hash.
    cache_index: u32,
    /// The hex-encoded HMAC-SHA256 hash of the source code.
    hash: String,
}

/// Compute an HMAC-SHA256 hash of the source code, matching the TypeScript implementation:
/// `createHmac('sha256', code).digest('hex')`
///
/// In Node.js, `createHmac(algo, key)` uses the source code as the HMAC key
/// and digests an empty message.
fn compute_source_hash(code: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    // HMAC accepts keys of any length, so `new_from_slice` is infallible in practice.
    let Ok(mut mac) = HmacSha256::new_from_slice(code.as_bytes()) else {
        return String::new();
    };
    mac.update(b"");
    let result = mac.finalize();
    let bytes = result.into_bytes();
    hex_encode(&bytes[..])
}

/// Encode bytes as a lowercase hexadecimal string.
fn hex_encode(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

// =====================================================================================
// MemoCounter — counts memo blocks/values via visitor
// =====================================================================================

#[derive(Default)]
struct MemoCounter {
    memo_blocks: u32,
    memo_values: u32,
    pruned_memo_blocks: u32,
    pruned_memo_values: u32,
}

impl ReactiveVisitor for MemoCounter {
    fn visit_scope_block(&mut self, scope: &crate::hir::ReactiveScope) {
        self.memo_blocks += 1;
        self.memo_values += u32::try_from(scope.declarations.len()).unwrap_or(u32::MAX);
    }

    fn visit_pruned_scope_block(&mut self, scope: &crate::hir::ReactiveScope) {
        self.pruned_memo_blocks += 1;
        self.pruned_memo_values += u32::try_from(scope.declarations.len()).unwrap_or(u32::MAX);
    }
}

// =====================================================================================
// Hook guard helpers
// =====================================================================================

/// Create a try-finally statement that wraps `stmts` with hook guard calls.
///
/// Produces:
/// ```js
/// try {
///   guardFn(before);
///   ...stmts
/// } finally {
///   guardFn(after);
/// }
/// ```
///
/// Port of `createHookGuard()` from `CodegenReactiveFunction.ts` lines 1343-1362.
fn create_hook_guard(
    guard_fn_name: &str,
    stmts: Vec<CodegenStatement>,
    before: GuardKind,
    after: GuardKind,
) -> CodegenStatement {
    let before_call =
        CodegenStatement::ExpressionStatement(format!("{guard_fn_name}({})", before as u8,));
    let after_call =
        CodegenStatement::ExpressionStatement(format!("{guard_fn_name}({})", after as u8,));

    let mut try_block = Vec::with_capacity(stmts.len() + 1);
    try_block.push(before_call);
    try_block.extend(stmts);

    CodegenStatement::Try {
        block: try_block,
        handler_param: None,
        handler: None,
        finalizer: Some(vec![after_call]),
    }
}

/// Check if an identifier represents a hook by looking up its type in the shape registry.
///
/// Port of `getHookKind()` from `HIR/HIR.ts` lines 1974-1976.
fn get_hook_kind(shapes: &ShapeRegistry, identifier: &crate::hir::Identifier) -> bool {
    let Type::Function(ref ft) = identifier.type_ else {
        return false;
    };
    let Some(ref shape_id) = ft.shape_id else {
        return false;
    };
    let Some(shape) = shapes.get(shape_id) else {
        return false;
    };
    shape.function_type.as_ref().is_some_and(|sig| sig.hook_kind.is_some())
}

/// Create a call expression string, optionally wrapping hook calls in an IIFE with
/// try-finally hook guards.
///
/// When hook guards are enabled, hook calls are wrapped like:
/// ```js
/// (() => {
///   try {
///     guardFn(2); // AllowHook
///     return hookCall(args);
///   } finally {
///     guardFn(3); // DisallowHook
///   }
/// })()
/// ```
///
/// Port of `createCallExpression()` from `CodegenReactiveFunction.ts` lines 1383-1414.
fn create_call_expression_string(
    cx: &mut CodegenContext,
    callee_str: &str,
    args_str: &str,
    is_hook: bool,
) -> String {
    let call_expr = format!("{callee_str}({args_str})");

    if is_hook && cx.output_mode == CompilerOutputMode::Client {
        if let Some(specifier_name) =
            cx.enable_emit_hook_guards.as_ref().map(|g| g.import_specifier_name.clone())
        {
            let guard_fn_name = cx.synthesize_name(&specifier_name);
            // Generate an IIFE with try-finally:
            // (() => { try { guardFn(2); return callExpr; } finally { guardFn(3); } })()
            return format!(
                "(() => {{ try {{ {guard_fn_name}({}); return {call_expr}; }} finally {{ {guard_fn_name}({}); }} }})()",
                GuardKind::AllowHook as u8,
                GuardKind::DisallowHook as u8,
            );
        }
    }

    call_expr
}

// =====================================================================================
// codegen_block — the central dispatch
// =====================================================================================

/// Generate code for a reactive block. Saves and restores temporaries so that
/// temporaries defined inside a block do not leak out to the parent scope.
fn codegen_block(cx: &mut CodegenContext, block: &ReactiveBlock) -> Vec<CodegenStatement> {
    let saved_temp = cx.temp.clone();
    let result = codegen_block_no_reset(cx, block);
    cx.temp = saved_temp;
    result
}

/// Generate code for a reactive block without resetting temporaries.
/// Used for sequence expressions where the final value references temporaries
/// created in preceding instructions.
fn codegen_block_no_reset(cx: &mut CodegenContext, block: &ReactiveBlock) -> Vec<CodegenStatement> {
    let mut statements: Vec<CodegenStatement> = Vec::new();

    for item in block {
        match item {
            ReactiveStatement::Instruction(instr_stmt) => {
                if let Some(stmt) = codegen_instruction_nullable(cx, &instr_stmt.instruction) {
                    statements.push(stmt);
                }
            }
            ReactiveStatement::PrunedScope(pruned) => {
                // Pruned scopes: just emit the instructions without memoization
                let scope_stmts = codegen_block_no_reset(cx, &pruned.instructions);
                statements.extend(scope_stmts);
            }
            ReactiveStatement::Scope(scope_block) => {
                let saved_temp = cx.temp.clone();
                codegen_reactive_scope(
                    cx,
                    &mut statements,
                    &scope_block.scope,
                    &scope_block.instructions,
                );
                cx.temp = saved_temp;
            }
            ReactiveStatement::Terminal(term_stmt) => {
                if let Some(stmt) = codegen_terminal(cx, &term_stmt.terminal) {
                    if let Some(ref label) = term_stmt.label {
                        if label.implicit {
                            // Implicit label: flatten blocks
                            match stmt {
                                CodegenStatement::Block(stmts) => {
                                    statements.extend(stmts);
                                }
                                other => statements.push(other),
                            }
                        } else {
                            // Labeled statement: unwrap single-statement blocks
                            let inner = match stmt {
                                CodegenStatement::Block(ref stmts) if stmts.len() == 1 => {
                                    stmts[0].clone()
                                }
                                other => other,
                            };
                            statements.push(CodegenStatement::Labeled {
                                label: codegen_label(label.id),
                                body: Box::new(inner),
                            });
                        }
                    } else {
                        // No label: flatten blocks
                        match stmt {
                            CodegenStatement::Block(stmts) => {
                                statements.extend(stmts);
                            }
                            other => statements.push(other),
                        }
                    }
                }
            }
        }
    }

    statements
}

// =====================================================================================
// codegen_reactive_scope — generates memoization if/else for a reactive scope
// =====================================================================================

/// A cache load entry for reactive scope codegen.
struct CacheLoad {
    name: String,
    index: u32,
}

/// Generate code for a reactive scope block.
///
/// The output pattern:
/// ```js
/// let decl1, decl2;
/// if ($[idx] !== dep1 || $[idx+1] !== dep2) {
///   // ... compute ...
///   $[idx] = dep1;     // store dependencies
///   $[idx+n] = decl1;  // store declarations
///   $[idx+n+1] = decl2;
/// } else {
///   decl1 = $[idx+n];
///   decl2 = $[idx+n+1];
/// }
/// ```
fn codegen_reactive_scope(
    cx: &mut CodegenContext,
    statements: &mut Vec<CodegenStatement>,
    scope: &ReactiveScope,
    block: &ReactiveBlock,
) {
    let cache_name = cx.synthesize_name("$");

    let mut cache_store_stmts: Vec<CodegenStatement> = Vec::new();
    let mut cache_load_stmts: Vec<CodegenStatement> = Vec::new();
    let mut change_expressions: Vec<String> = Vec::new();

    // Process dependencies: sorted for determinism
    let mut deps: Vec<&ReactiveScopeDependency> = scope.dependencies.iter().collect();
    deps.sort_by(|a, b| compare_scope_dependency(a, b));

    for dep in &deps {
        let index = cx.alloc_cache_index();
        let dep_expr = codegen_dependency(dep);
        let comparison = format!("{cache_name}[{index}] !== {dep_expr}");
        change_expressions.push(comparison);

        // Store dependency value into cache (only in consequent/store)
        cache_store_stmts.push(CodegenStatement::ExpressionStatement(format!(
            "{cache_name}[{index}] = {dep_expr}"
        )));
    }

    // Process declarations: sorted for determinism, deduplicated by declaration_id.
    // After SSA renaming, multiple IdentifierIds may share the same DeclarationId
    // (i.e., the same source-level variable). The TS reference prevents duplicates
    // in PropagateScopeDependenciesHIR by checking declarationId before inserting.
    // We deduplicate here to ensure each source variable gets exactly one cache slot.
    let mut decls: Vec<(&IdentifierId, &ReactiveScopeDeclaration)> =
        scope.declarations.iter().collect();
    decls.sort_by(|(_, a), (_, b)| compare_scope_declaration(a, b));
    let mut seen_declaration_ids: FxHashSet<DeclarationId> = FxHashSet::default();
    decls.retain(|(_, decl)| seen_declaration_ids.insert(decl.identifier.declaration_id));

    let mut first_output_index: Option<u32> = None;
    let mut cache_loads: Vec<CacheLoad> = Vec::new();

    for (_, decl) in &decls {
        let index = cx.alloc_cache_index();
        if first_output_index.is_none() {
            first_output_index = Some(index);
        }

        let name = identifier_name(&decl.identifier);

        // Emit `let name;` before the if-block if not yet declared
        if !cx.has_declared(decl.identifier.declaration_id) {
            statements.push(CodegenStatement::VariableDeclaration {
                kind: VarKind::Let,
                name: name.clone(),
                init: None,
            });
        }
        cache_loads.push(CacheLoad { name: name.clone(), index });
        cx.declare(decl.identifier.declaration_id);
    }

    // Process reassignments — now stores full Identifier objects
    for reassignment_ident in &scope.reassignments {
        let index = cx.alloc_cache_index();
        if first_output_index.is_none() {
            first_output_index = Some(index);
        }
        let name = match &reassignment_ident.name {
            Some(crate::hir::IdentifierName::Named(n)) => n.to_string(),
            Some(crate::hir::IdentifierName::Promoted(n)) => n.clone(),
            None => format!("t{}", reassignment_ident.id.0),
        };
        cache_loads.push(CacheLoad { name, index });
    }

    // Build the test condition
    let test_condition = if change_expressions.is_empty() {
        // No dependencies — use sentinel check on first output
        if let Some(first_idx) = first_output_index {
            format!("{cache_name}[{first_idx}] === Symbol.for(\"{MEMO_CACHE_SENTINEL}\")")
        } else {
            // No deps and no outputs — should not happen, but be safe
            "true".to_string()
        }
    } else {
        change_expressions.join(" || ")
    };

    // Generate the computation block
    let mut computation_stmts = codegen_block(cx, block);

    // Store each output into the cache
    for load in &cache_loads {
        cache_store_stmts.push(CodegenStatement::ExpressionStatement(format!(
            "{cache_name}[{}] = {}",
            load.index, load.name
        )));
    }
    computation_stmts.extend(cache_store_stmts);

    // Load from cache in else branch
    for load in &cache_loads {
        cache_load_stmts.push(CodegenStatement::ExpressionStatement(format!(
            "{} = {cache_name}[{}]",
            load.name, load.index
        )));
    }

    statements.push(CodegenStatement::If {
        test: test_condition,
        consequent: computation_stmts,
        alternate: Some(cache_load_stmts),
    });

    // Handle early return value
    if let Some(ref early_return) = scope.early_return_value {
        let name = identifier_name(&early_return.value);
        statements.push(CodegenStatement::If {
            test: format!("{name} !== Symbol.for(\"{EARLY_RETURN_SENTINEL}\")"),
            consequent: vec![CodegenStatement::Return { value: Some(name), is_conditional: false }],
            alternate: None,
        });
    }
}

// =====================================================================================
// codegen_terminal — generates code for reactive terminals
// =====================================================================================

/// Generate code for a reactive terminal.
fn codegen_terminal(
    cx: &mut CodegenContext,
    terminal: &ReactiveTerminal,
) -> Option<CodegenStatement> {
    match terminal {
        ReactiveTerminal::Break(t) => codegen_break(t),
        ReactiveTerminal::Continue(t) => codegen_continue(t),
        ReactiveTerminal::Return(t) => {
            let value = codegen_place_to_expression(cx, &t.value);
            let is_conditional = cx.arrow_paren_temps.contains(&t.value.identifier.declaration_id);
            if value == "undefined" {
                Some(CodegenStatement::Return { value: None, is_conditional: false })
            } else {
                Some(CodegenStatement::Return { value: Some(value), is_conditional })
            }
        }
        ReactiveTerminal::Throw(t) => {
            let value = codegen_place_to_expression(cx, &t.value);
            Some(CodegenStatement::Throw(value))
        }
        ReactiveTerminal::If(t) => {
            let test = codegen_place_to_expression(cx, &t.test);
            let consequent = codegen_block(cx, &t.consequent);
            let alternate = t.alternate.as_ref().map(|alt| {
                let block = codegen_block(cx, alt);
                if block.is_empty() {
                    return Vec::new();
                }
                block
            });
            // Omit empty alternates
            let alternate = alternate.and_then(|a| if a.is_empty() { None } else { Some(a) });
            Some(CodegenStatement::If { test, consequent, alternate })
        }
        ReactiveTerminal::Switch(t) => {
            let discriminant = codegen_place_to_expression(cx, &t.test);
            let cases = t
                .cases
                .iter()
                .map(|case| {
                    let test = case.test.as_ref().map(|p| codegen_place_to_expression(cx, p));
                    let body = case
                        .block
                        .as_ref()
                        .map(|b| {
                            let stmts = codegen_block(cx, b);
                            if stmts.is_empty() {
                                Vec::new()
                            } else {
                                // Wrap in a block statement like TS does
                                vec![CodegenStatement::Block(stmts)]
                            }
                        })
                        .unwrap_or_default();
                    CodegenSwitchCase { test, body }
                })
                .collect();
            Some(CodegenStatement::Switch { discriminant, cases })
        }
        ReactiveTerminal::For(t) => {
            let init = codegen_for_init(cx, &t.init);
            let test = codegen_reactive_value_to_expression(cx, &t.test);
            let update = t.update.as_ref().map(|u| {
                let s = codegen_reactive_value_to_expression(cx, u);
                // Strip outer parens from the update expression — they're unnecessary
                // in for-loop update position and the reference compiler doesn't emit them.
                if s.starts_with('(') && s.ends_with(')') {
                    s[1..s.len() - 1].to_string()
                } else {
                    s
                }
            });
            let body = codegen_block(cx, &t.r#loop);
            Some(CodegenStatement::For { init: Some(init), test: Some(test), update, body })
        }
        ReactiveTerminal::ForOf(t) => {
            let (kind, left) = codegen_for_of_in_init(cx, &t.init, &t.test);
            let right = codegen_for_of_collection(cx, &t.init);
            let body = codegen_block(cx, &t.r#loop);
            Some(CodegenStatement::ForOf { kind, left, right, body })
        }
        ReactiveTerminal::ForIn(t) => {
            let (kind, left) = codegen_for_in_init(cx, &t.init);
            let right = codegen_for_in_collection(cx, &t.init);
            let body = codegen_block(cx, &t.r#loop);
            Some(CodegenStatement::ForIn { kind, left, right, body })
        }
        ReactiveTerminal::While(t) => {
            let test = codegen_reactive_value_to_expression(cx, &t.test);
            let body = codegen_block(cx, &t.r#loop);
            Some(CodegenStatement::While { test, body })
        }
        ReactiveTerminal::DoWhile(t) => {
            let test = codegen_reactive_value_to_expression(cx, &t.test);
            let body = codegen_block(cx, &t.r#loop);
            Some(CodegenStatement::DoWhile { body, test })
        }
        ReactiveTerminal::Label(t) => {
            let block = codegen_block(cx, &t.block);
            if block.is_empty() { None } else { Some(CodegenStatement::Block(block)) }
        }
        ReactiveTerminal::Try(t) => {
            let block = codegen_block(cx, &t.block);
            let handler_param = t.handler_binding.as_ref().map(|binding| {
                let name = identifier_name(&binding.identifier);
                cx.temp.insert(binding.identifier.declaration_id, None);
                name
            });
            let handler = Some(codegen_block(cx, &t.handler));
            Some(CodegenStatement::Try { block, handler_param, handler, finalizer: None })
        }
    }
}

fn codegen_break(t: &ReactiveBreakTerminal) -> Option<CodegenStatement> {
    if t.target_kind == ReactiveTerminalTargetKind::Implicit {
        return None;
    }
    let label = if t.target_kind == ReactiveTerminalTargetKind::Labeled {
        Some(codegen_label(t.target))
    } else {
        None
    };
    Some(CodegenStatement::Break(label))
}

fn codegen_continue(t: &ReactiveContinueTerminal) -> Option<CodegenStatement> {
    if t.target_kind == ReactiveTerminalTargetKind::Implicit {
        return None;
    }
    let label = if t.target_kind == ReactiveTerminalTargetKind::Labeled {
        Some(codegen_label(t.target))
    } else {
        None
    };
    Some(CodegenStatement::Continue(label))
}

// =====================================================================================
// codegen_instruction_nullable — instruction to statement (may return None)
// =====================================================================================

/// Generate code for a reactive instruction. Returns `None` if the instruction
/// is suppressed (e.g., temporary assignment, memoization markers).
fn codegen_instruction_nullable(
    cx: &mut CodegenContext,
    instr: &ReactiveInstruction,
) -> Option<CodegenStatement> {
    match &instr.value {
        ReactiveValue::Instruction(boxed) => match boxed.as_ref() {
            InstructionValue::StoreLocal(store) => {
                let kind = if cx.has_declared(store.lvalue.place.identifier.declaration_id)
                    && !matches!(
                        store.lvalue.kind,
                        InstructionKind::Function | InstructionKind::HoistedFunction
                    )
                {
                    // Function/HoistedFunction declarations always emit their body as a
                    // function declaration statement, even if the binding was pre-declared
                    // (e.g., as HoistedLet). Never override to Reassign for these kinds.
                    InstructionKind::Reassign
                } else {
                    store.lvalue.kind
                };
                // Use raw (non-paren-wrapping) expression for the value in Reassign
                // context, since chained assignments `x = y = z = 1` should not have
                // intermediate parens. The parens are added at the point of consumption
                // in expression contexts (call args, initializers, etc.).
                let value_expr = if kind == InstructionKind::Reassign {
                    codegen_place_to_expression_raw(cx, &store.value)
                } else {
                    codegen_place_to_expression(cx, &store.value)
                };
                codegen_store_or_declare(cx, instr, kind, &store.lvalue.place, Some(&value_expr), Some(&store.value))
            }
            InstructionValue::StoreContext(store) => {
                let kind = store.lvalue_kind;
                let value_expr = if kind == InstructionKind::Reassign {
                    codegen_place_to_expression_raw(cx, &store.value)
                } else {
                    codegen_place_to_expression(cx, &store.value)
                };
                codegen_store_or_declare(cx, instr, kind, &store.lvalue_place, Some(&value_expr), Some(&store.value))
            }
            InstructionValue::DeclareLocal(decl) => {
                if cx.has_declared(decl.lvalue.place.identifier.declaration_id) {
                    return None;
                }
                codegen_store_or_declare(cx, instr, decl.lvalue.kind, &decl.lvalue.place, None, None)
            }
            InstructionValue::DeclareContext(decl) => {
                if cx.has_declared(decl.lvalue_place.identifier.declaration_id) {
                    return None;
                }
                codegen_store_or_declare(cx, instr, decl.lvalue_kind, &decl.lvalue_place, None, None)
            }
            InstructionValue::Destructure(destr) => {
                let kind = destr.lvalue.kind;
                // Register unnamed pattern places as temporaries
                for place in each_pattern_operand(&destr.lvalue.pattern) {
                    if kind != InstructionKind::Reassign && place.identifier.name.is_none() {
                        cx.temp.insert(place.identifier.declaration_id, None);
                    }
                }
                let value_expr = codegen_place_to_expression(cx, &destr.value);
                let lval = codegen_pattern(cx, &destr.lvalue.pattern);
                codegen_destructure_statement(cx, instr, kind, &lval, &value_expr)
            }
            InstructionValue::StartMemoize(_) | InstructionValue::FinishMemoize(_) => None,
            InstructionValue::Debugger(_) => Some(CodegenStatement::Debugger),
            // Delete expressions are side effects. When their result is not
            // consumed (unnamed lvalue), emit as expression statements so the
            // side effect is not silently dropped. When the result IS consumed
            // (named lvalue), fall through to the default path that produces
            // `let y = delete x.prop`.
            InstructionValue::PropertyDelete(del) => {
                let object = codegen_place_to_expression(cx, &del.object);
                let member = codegen_member_access(cx, &object, &del.property);
                let value_str = format!("delete {member}");
                // Always use codegen_instruction_to_statement so that if there is
                // a lvalue (named or unnamed temp), the result is properly captured.
                // If there is no lvalue, it is emitted as a side-effectful statement.
                codegen_instruction_to_statement(cx, instr, &value_str)
            }
            InstructionValue::ComputedDelete(del) => {
                let object = codegen_place_to_expression(cx, &del.object);
                let property = codegen_place_to_expression(cx, &del.property);
                let value_str = format!("delete {object}[{property}]");
                codegen_instruction_to_statement(cx, instr, &value_str)
            }
            InstructionValue::ObjectMethod(method) => {
                // Store object method for later use by ObjectExpression codegen.
                // Port of CodegenReactiveFunction.ts lines 1168-1174.
                if let Some(lval) = &instr.lvalue {
                    cx.object_methods.insert(lval.identifier.id, method.clone());
                }
                None
            }
            InstructionValue::FunctionExpression(func_expr) => {
                // Handle FunctionExpression separately from the `other` branch so
                // that we can store structured CodegenFunction data for
                // FunctionDeclaration types. This data is later used by
                // codegen_store_or_declare to emit proper `function x() {}`
                // declaration statements instead of expression statements.
                if let Some(lval) = &instr.lvalue {
                    cx.fn_expr_temps.insert(lval.identifier.declaration_id);
                }
                let (value_str, fn_decl) = codegen_function_expression(cx, func_expr);
                if let Some(fn_data) = fn_decl {
                    if let Some(lval) = &instr.lvalue {
                        cx.fn_decl_data.insert(lval.identifier.declaration_id, fn_data);
                    }
                }
                codegen_instruction_to_statement(cx, instr, &value_str)
            }
            other => {
                // Track JSX text/element/function temporaries for proper rendering
                if let Some(lval) = &instr.lvalue {
                    let decl_id = lval.identifier.declaration_id;
                    match other {
                        InstructionValue::JsxText(_) => {
                            cx.jsx_text_temps.insert(decl_id);
                        }
                        InstructionValue::JsxExpression(_) | InstructionValue::JsxFragment(_) => {
                            cx.jsx_element_temps.insert(decl_id);
                        }
                        _ => {}
                    }
                }
                let value_str = codegen_instruction_value(cx, other);
                // Track assignment expressions (PropertyStore, ComputedStore, StoreGlobal)
                // as assignment temps when they are stored as unnamed temporaries.
                // The parens will be added by codegen_place_to_expression when the temp
                // is consumed in expression context (call args, initializers, etc.).
                // For named lvalues, wrap in parens immediately since they'll appear
                // as `let name = (lhs = rhs)` in the output.
                let is_store_instr = matches!(
                    other,
                    InstructionValue::PropertyStore(_)
                        | InstructionValue::ComputedStore(_)
                        | InstructionValue::StoreGlobal(_)
                );
                if is_store_instr {
                    if let Some(lval) = instr.lvalue.as_ref() {
                        if lval.identifier.name.is_none() {
                            // Unnamed temp: store as assignment temp for deferred wrapping
                            cx.temp.insert(lval.identifier.declaration_id, Some(value_str));
                            cx.assignment_temps.insert(lval.identifier.declaration_id);
                            return None;
                        }
                        // Named lval: emit as `t = (lhs = rhs)` — no parens needed since
                        // assignment expressions are right-associative and valid as rvalue
                        // in a `let t = lhs = rhs` statement without parens (matching TS output).
                        return codegen_instruction_to_statement(cx, instr, &value_str);
                    }
                }
                codegen_instruction_to_statement(cx, instr, &value_str)
            }
        },
        ReactiveValue::Logical(logical) => {
            let value_str = codegen_reactive_value_to_expression(cx, &instr.value);
            // Track logical operator on temps so that parent logical expressions
            // can detect when `??` is mixed with `||`/`&&` and add mandatory parens.
            if let Some(lval) = &instr.lvalue {
                if lval.identifier.name.is_none() {
                    cx.logical_temps.insert(lval.identifier.declaration_id, logical.operator);
                }
            }
            codegen_instruction_to_statement(cx, instr, &value_str)
        }
        ReactiveValue::Ternary(_) | ReactiveValue::Sequence(_) | ReactiveValue::OptionalCall(_) => {
            // For Sequence values used as ExpressionStatements (lvalue: None),
            // Prettier formats the comma expression WITHOUT outer parens but WITH
            // parens around individual assignment sub-expressions:
            //   `(x = []), null` instead of `(x = [], null)`
            // In all other contexts (temps, assignment RHS, variable init, while
            // test, etc.), the outer parens from codegen_reactive_value_to_expression
            // are correct.
            let is_sequence_expr_stmt = matches!(&instr.value, ReactiveValue::Sequence(_))
                && instr.lvalue.is_none();
            let value_str = if is_sequence_expr_stmt {
                codegen_sequence_for_expr_stmt(cx, &instr.value)
            } else {
                codegen_reactive_value_to_expression(cx, &instr.value)
            };
            // Track these temps for arrow-body parenthesisation: Babel's printer
            // auto-wraps ConditionalExpression / SequenceExpression in parens when
            // used as a concise arrow body; we must do it explicitly.
            if let Some(lval) = &instr.lvalue {
                if lval.identifier.name.is_none() {
                    cx.arrow_paren_temps.insert(lval.identifier.declaration_id);
                    // Track ternary temps specifically: they need parens when used
                    // as binary expression operands (e.g. `x + (cond ? a : b)`).
                    if matches!(&instr.value, ReactiveValue::Ternary(_)) {
                        cx.ternary_temps.insert(lval.identifier.declaration_id);
                    }
                }
            }
            // For Sequence values, the inner sequence may contain a Logical expression
            // that was stored as a temp. Propagate the logical operator tracking to
            // this instruction's lvalue so parent logical expressions can detect mixing.
            if matches!(&instr.value, ReactiveValue::Sequence(_)) {
                if let Some(lval) = &instr.lvalue {
                    if lval.identifier.name.is_none() {
                        if let Some(op) = find_logical_operator_in_value(&instr.value) {
                            cx.logical_temps.insert(lval.identifier.declaration_id, op);
                        }
                    }
                }
            }
            codegen_instruction_to_statement(cx, instr, &value_str)
        }
    }
}

/// Handle StoreLocal/StoreContext/DeclareLocal/DeclareContext
///
/// `value_place` is the RHS Place (when available) so that for
/// `InstructionKind::Function`/`HoistedFunction` we can look up
/// the structured `CodegenFunction` data and emit a proper function
/// declaration statement.
fn codegen_store_or_declare(
    cx: &mut CodegenContext,
    instr: &ReactiveInstruction,
    kind: InstructionKind,
    lvalue_place: &Place,
    value: Option<&str>,
    value_place: Option<&Place>,
) -> Option<CodegenStatement> {
    let name = identifier_name(&lvalue_place.identifier);

    match kind {
        InstructionKind::Const | InstructionKind::HoistedConst => {
            cx.declare(lvalue_place.identifier.declaration_id);
            Some(CodegenStatement::VariableDeclaration {
                kind: VarKind::Const,
                name,
                init: value.map(String::from),
            })
        }
        InstructionKind::Let | InstructionKind::HoistedLet => {
            cx.declare(lvalue_place.identifier.declaration_id);
            Some(CodegenStatement::VariableDeclaration {
                kind: VarKind::Let,
                name,
                init: value.map(String::from),
            })
        }
        InstructionKind::Function | InstructionKind::HoistedFunction => {
            cx.declare(lvalue_place.identifier.declaration_id);
            // Emit as a proper function declaration statement, matching the TS
            // reference which calls `createFunctionDeclaration` for this kind
            // (CodegenReactiveFunction.ts lines 1086-1108).
            //
            // Look up the structured CodegenFunction data stored when the
            // FunctionExpression instruction was processed. If available, emit
            // a FunctionDeclaration; otherwise fall back to ExpressionStatement.
            if let Some(vp) = value_place {
                if let Some(fn_data) = cx.fn_decl_data.remove(&vp.identifier.declaration_id) {
                    return Some(CodegenStatement::FunctionDeclaration {
                        name,
                        params: fn_data.params,
                        body: fn_data.body,
                        generator: fn_data.generator,
                        is_async: fn_data.is_async,
                    });
                }
            }
            if let Some(val) = value {
                Some(CodegenStatement::ExpressionStatement(val.to_string()))
            } else {
                Some(CodegenStatement::VariableDeclaration {
                    kind: VarKind::Const,
                    name,
                    init: None,
                })
            }
        }
        InstructionKind::Reassign => {
            if let Some(val) = value {
                let assign_expr = format!("{name} = {val}");
                // If there's an lvalue on the instruction (i.e., it's used as an expression),
                // store as temporary and mark it as an assignment expression so it can
                // be wrapped in parens when consumed in expression contexts.
                if let Some(lval) = instr.lvalue.as_ref()
                    && lval.identifier.name.is_none()
                {
                    cx.temp.insert(lval.identifier.declaration_id, Some(assign_expr));
                    cx.assignment_temps.insert(lval.identifier.declaration_id);
                    return None;
                }
                return try_store_as_temporary(cx, None, assign_expr);
            }
            None
        }
        InstructionKind::Catch => Some(CodegenStatement::Empty),
    }
}

/// Handle destructure statements.
fn codegen_destructure_statement(
    cx: &mut CodegenContext,
    instr: &ReactiveInstruction,
    kind: InstructionKind,
    lval: &str,
    value: &str,
) -> Option<CodegenStatement> {
    match kind {
        InstructionKind::Const
        | InstructionKind::HoistedConst
        | InstructionKind::Function
        | InstructionKind::HoistedFunction => Some(CodegenStatement::VariableDeclaration {
            kind: VarKind::Const,
            name: lval.to_string(),
            init: Some(value.to_string()),
        }),
        InstructionKind::Let | InstructionKind::HoistedLet => {
            Some(CodegenStatement::VariableDeclaration {
                kind: VarKind::Let,
                name: lval.to_string(),
                init: Some(value.to_string()),
            })
        }
        InstructionKind::Reassign => {
            // Object destructuring assignments need parens to avoid being
            // parsed as block statements: `({ f: g } = t3)` not `{ f: g } = t3`
            let assign_expr = if lval.starts_with('{') {
                format!("({lval} = {value})")
            } else {
                format!("{lval} = {value}")
            };
            // Mark as assignment temp when stored as temp, so consumers can
            // add parens when needed (e.g., `foo(([x] = obj))`).
            if let Some(lval_place) = instr.lvalue.as_ref()
                && lval_place.identifier.name.is_none()
            {
                cx.temp.insert(lval_place.identifier.declaration_id, Some(assign_expr));
                cx.assignment_temps.insert(lval_place.identifier.declaration_id);
                return None;
            }
            try_store_as_temporary(cx, None, assign_expr)
        }
        InstructionKind::Catch => Some(CodegenStatement::Empty),
    }
}

/// Store an expression as a temporary if the lvalue is unnamed, otherwise emit as expression statement.
fn try_store_as_temporary(
    cx: &mut CodegenContext,
    lvalue: Option<&Place>,
    expr: String,
) -> Option<CodegenStatement> {
    if let Some(lval) = lvalue
        && lval.identifier.name.is_none()
    {
        cx.temp.insert(lval.identifier.declaration_id, Some(expr));
        return None;
    }
    Some(CodegenStatement::ExpressionStatement(expr))
}

/// Convert a codegen value into a statement, handling temporaries and named lvalues.
fn codegen_instruction_to_statement(
    cx: &mut CodegenContext,
    instr: &ReactiveInstruction,
    value: &str,
) -> Option<CodegenStatement> {
    match &instr.lvalue {
        None => Some(CodegenStatement::ExpressionStatement(value.to_string())),
        Some(lval) => {
            if lval.identifier.name.is_none() {
                // Temporary — store for later reference
                cx.temp.insert(lval.identifier.declaration_id, Some(value.to_string()));
                None
            } else {
                let name = identifier_name(&lval.identifier);
                if cx.has_declared(lval.identifier.declaration_id) {
                    Some(CodegenStatement::ExpressionStatement(format!("{name} = {value}")))
                } else {
                    cx.declare(lval.identifier.declaration_id);
                    Some(CodegenStatement::VariableDeclaration {
                        kind: VarKind::Const,
                        name,
                        init: Some(value.to_string()),
                    })
                }
            }
        }
    }
}

// =====================================================================================
// codegen_instruction_value — InstructionValue → expression string
// =====================================================================================

/// Generate an expression string from an InstructionValue.
fn codegen_instruction_value(cx: &mut CodegenContext, value: &InstructionValue) -> String {
    match value {
        InstructionValue::ArrayExpression(arr) => {
            let elements: Vec<String> = arr
                .elements
                .iter()
                .map(|elem| match elem {
                    ArrayExpressionElement::Place(p) => codegen_place_to_expression(cx, p),
                    ArrayExpressionElement::Spread(s) => {
                        format!("...{}", codegen_place_to_expression(cx, &s.place))
                    }
                    ArrayExpressionElement::Hole => String::new(),
                })
                .collect();
            format!("[{}]", elements.join(", "))
        }
        InstructionValue::BinaryExpression(bin) => {
            let left = codegen_place_to_expression(cx, &bin.left);
            let right = codegen_place_to_expression(cx, &bin.right);
            // Ternary expressions have lower precedence than all binary operators.
            // Wrap ternary-valued operands in parens for correct precedence.
            // e.g., `x + cond ? a : b` must be `x + (cond ? a : b)`.
            //
            // We check two conditions:
            // 1. Direct ternary temp (stored when a Ternary reactive value is codegen'd)
            // 2. String heuristic: value contains ` ? ` at top level (not inside parens)
            //    - this handles phi nodes that hold ternary values
            let needs_binary_parens = |expr: &str, decl_id: DeclarationId| -> bool {
                if cx.ternary_temps.contains(&decl_id) {
                    return true;
                }
                // String heuristic: if the substituted value looks like a ternary
                // (contains ` ? ` at top level without being already parenthesized)
                if !expr.starts_with('(') && is_ternary_expression(expr) {
                    return true;
                }
                // NullishCoalescing (`??`) has lower precedence than all arithmetic and
                // comparison operators. Wrap it when used as a binary expression operand.
                // e.g., `(x ?? 0) + 1` must not become `x ?? 0 + 1`.
                if !expr.starts_with('(') && is_nullish_coalescing_expression(expr) {
                    return true;
                }
                // Also check if this temp was recorded as a NullishCoalescing expression.
                if let Some(LogicalOperator::Coalesce) =
                    cx.logical_temps.get(&decl_id).copied()
                {
                    return true;
                }
                false
            };
            let left = if needs_binary_parens(&left, bin.left.identifier.declaration_id) {
                format!("({left})")
            } else {
                left
            };
            let right = if needs_binary_parens(&right, bin.right.identifier.declaration_id) {
                format!("({right})")
            } else {
                right
            };
            format!("{left} {} {right}", bin.operator.as_str())
        }
        InstructionValue::UnaryExpression(unary) => {
            let operand = codegen_place_to_expression(cx, &unary.value);
            let op = unary.operator.as_str();
            // typeof, void, delete need a space; others (+, -, ~, !) don't
            if op.chars().next().is_some_and(char::is_alphabetic) {
                format!("{op} {operand}")
            } else {
                format!("{op}{operand}")
            }
        }
        InstructionValue::Primitive(prim) => codegen_primitive(&prim.value),
        InstructionValue::JsxText(text) => {
            // JSX text: emit as raw text (will be rendered directly in JSX children)
            text.value.clone()
        }
        InstructionValue::CallExpression(call) => {
            let is_hook = get_hook_kind(&cx.shapes, &call.callee.identifier);
            let callee = codegen_place_to_expression(cx, &call.callee);
            let args = codegen_args(cx, &call.args);
            // When the callee is a FunctionExpression or arrow function (IIFE pattern),
            // wrap it in parentheses so it parses as an expression, not a declaration.
            // In the TS reference this is handled automatically by Babel's AST printer.
            // We also check `fn_expr_temps` for cases where the callee is a temp
            // holding a function expression (e.g., arrow functions stored as temps).
            let is_fn_expr = callee.starts_with("function")
                || callee.starts_with("async function")
                || cx.fn_expr_temps.contains(&call.callee.identifier.declaration_id);
            let callee = if is_fn_expr { format!("({callee})") } else { callee };
            create_call_expression_string(cx, &callee, &args, is_hook)
        }
        InstructionValue::MethodCall(method) => {
            let is_hook = get_hook_kind(&cx.shapes, &method.property.identifier);
            // TS: the property Place is a PropertyLoad result (e.g. "console.log"),
            // already a member expression string. Use it directly as the callee.
            let callee = codegen_place_to_expression(cx, &method.property);
            let args = codegen_args(cx, &method.args);
            create_call_expression_string(cx, &callee, &args, is_hook)
        }
        InstructionValue::NewExpression(new) => {
            let callee = codegen_place_to_expression(cx, &new.callee);
            let args = codegen_args(cx, &new.args);
            format!("new {callee}({args})")
        }
        InstructionValue::ObjectExpression(obj) => codegen_object_expression(cx, obj),
        InstructionValue::PropertyLoad(load) => {
            let object = codegen_place_to_expression(cx, &load.object);
            codegen_member_access(cx, &object, &load.property)
        }
        InstructionValue::PropertyStore(store) => {
            let object = codegen_place_to_expression(cx, &store.object);
            let member = codegen_member_access(cx, &object, &store.property);
            let value = codegen_place_to_expression(cx, &store.value);
            format!("{member} = {value}")
        }
        InstructionValue::PropertyDelete(del) => {
            let object = codegen_place_to_expression(cx, &del.object);
            let member = codegen_member_access(cx, &object, &del.property);
            format!("delete {member}")
        }
        InstructionValue::ComputedLoad(load) => {
            let object = codegen_place_to_expression(cx, &load.object);
            let object = maybe_parenthesize_optional_chain(cx, &object);
            let property = codegen_place_to_expression(cx, &load.property);
            format!("{object}[{property}]")
        }
        InstructionValue::ComputedStore(store) => {
            let object = codegen_place_to_expression(cx, &store.object);
            let object = maybe_parenthesize_optional_chain(cx, &object);
            let property = codegen_place_to_expression(cx, &store.property);
            let value = codegen_place_to_expression(cx, &store.value);
            format!("{object}[{property}] = {value}")
        }
        InstructionValue::ComputedDelete(del) => {
            let object = codegen_place_to_expression(cx, &del.object);
            let object = maybe_parenthesize_optional_chain(cx, &object);
            let property = codegen_place_to_expression(cx, &del.property);
            format!("delete {object}[{property}]")
        }
        InstructionValue::LoadLocal(load) => codegen_place_to_expression(cx, &load.place),
        InstructionValue::LoadContext(load) => codegen_place_to_expression(cx, &load.place),
        InstructionValue::LoadGlobal(load) => load.binding.name().to_string(),
        InstructionValue::StoreGlobal(store) => {
            let value = codegen_place_to_expression(cx, &store.value);
            format!("{} = {value}", store.name)
        }
        InstructionValue::FunctionExpression(func_expr) => {
            codegen_function_expression(cx, func_expr).0
        }
        InstructionValue::RegExpLiteral(re) => {
            format!("/{}/{}", re.pattern, re.flags)
        }
        InstructionValue::TemplateLiteral(tmpl) => {
            let mut result = String::from("`");
            for (i, quasi) in tmpl.quasis.iter().enumerate() {
                result.push_str(&quasi.raw);
                if i < tmpl.subexprs.len() {
                    result.push_str("${");
                    result.push_str(&codegen_place_to_expression(cx, &tmpl.subexprs[i]));
                    result.push('}');
                }
            }
            result.push('`');
            result
        }
        InstructionValue::TaggedTemplateExpression(tagged) => {
            let tag = codegen_place_to_expression(cx, &tagged.tag);
            format!("{tag}`{}`", tagged.value.raw)
        }
        InstructionValue::TypeCastExpression(cast) => {
            // Simplified: just emit the value (type annotations are stripped)
            codegen_place_to_expression(cx, &cast.value)
        }
        InstructionValue::JsxExpression(jsx) => {
            let tag_str = match &jsx.tag {
                JsxTag::Place(p) => codegen_place_to_expression(cx, p),
                JsxTag::BuiltIn(b) => b.name.clone(),
            };
            let attrs: Vec<String> =
                jsx.props.iter().map(|attr| codegen_jsx_attribute(cx, attr)).collect();
            let attrs_str =
                if attrs.is_empty() { String::new() } else { format!(" {}", attrs.join(" ")) };
            match &jsx.children {
                None => format!("<{tag_str}{attrs_str} />"),
                Some(children) if children.is_empty() => {
                    format!("<{tag_str}{attrs_str}></{tag_str}>")
                }
                Some(children) => {
                    let children_str: Vec<String> =
                        children.iter().map(|c| codegen_jsx_child(cx, c)).collect();
                    // Check if any child is a JSX text node (not wrapped in { })
                    let has_text_child =
                        children_str.iter().any(|c| !c.starts_with('{') && !c.starts_with('<'));
                    if children_str.len() > 1 && !has_text_child {
                        // Multi-child, all expression containers/elements: emit with newlines
                        let joined = join_jsx_children_multiline(&children_str);
                        format!("<{tag_str}{attrs_str}>\n{joined}\n</{tag_str}>")
                    } else {
                        format!("<{tag_str}{attrs_str}>{}</{tag_str}>", children_str.join(""))
                    }
                }
            }
        }
        InstructionValue::JsxFragment(frag) => {
            let children_str: Vec<String> =
                frag.children.iter().map(|c| codegen_jsx_child(cx, c)).collect();
            if children_str.len() > 1 {
                // Multi-child: emit with newlines matching reference compiler
                let joined = join_jsx_children_multiline(&children_str);
                format!("<>\n{joined}\n</>")
            } else {
                format!("<>{}</>", children_str.join(""))
            }
        }
        InstructionValue::GetIterator(iter) => codegen_place_to_expression(cx, &iter.collection),
        InstructionValue::IteratorNext(iter) => codegen_place_to_expression(cx, &iter.iterator),
        InstructionValue::NextPropertyOf(next) => codegen_place_to_expression(cx, &next.value),
        InstructionValue::PrefixUpdate(update) => {
            let lval = codegen_place_to_expression(cx, &update.lvalue);
            format!("{}{lval}", update.operation.as_str())
        }
        InstructionValue::PostfixUpdate(update) => {
            let lval = codegen_place_to_expression(cx, &update.lvalue);
            format!("{lval}{}", update.operation.as_str())
        }
        InstructionValue::Await(aw) => {
            let value = codegen_place_to_expression(cx, &aw.value);
            format!("await {value}")
        }
        InstructionValue::MetaProperty(meta) => {
            format!("{}.{}", meta.meta, meta.property)
        }
        // StoreLocal in expression context means it's a reassignment used as a value.
        // Wrap in parentheses because assignment expressions have very low precedence
        // and are typically used inside call arguments, conditions, etc.
        // e.g., `foo((x = 1))` not `foo(x = 1)`.
        InstructionValue::StoreLocal(store) => {
            let lval = codegen_place_to_expression(cx, &store.lvalue.place);
            let value = codegen_place_to_expression(cx, &store.value);
            format!("({lval} = {value})")
        }
        InstructionValue::StoreContext(_)
        | InstructionValue::DeclareLocal(_)
        | InstructionValue::DeclareContext(_)
        | InstructionValue::Destructure(_)
        | InstructionValue::StartMemoize(_)
        | InstructionValue::FinishMemoize(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::ObjectMethod(_) => {
            // These are handled in codegen_instruction_nullable
            String::new()
        }
        InstructionValue::UnsupportedNode(_) => "/* unsupported */".to_string(),
    }
}

// =====================================================================================
// codegen_function_expression — recursive codegen for FunctionExpression
// =====================================================================================

/// Generate a function expression string by recursively compiling the inner function.
///
/// Port of `CodegenReactiveFunction.ts` lines 1852-1901 (case 'FunctionExpression').
///
/// This runs the sub-pipeline (build reactive function, prune, rename) on the lowered
/// function, then formats the result based on the expression type (arrow, function expr,
/// function declaration).
///
/// Returns the expression string and, for `FunctionDeclaration` types, the structured
/// `CodegenFunction` data needed to emit a proper function declaration statement.
fn codegen_function_expression(
    cx: &CodegenContext,
    func_expr: &crate::hir::FunctionExpressionValue,
) -> (String, Option<CodegenFunction>) {
    match codegen_inner_function(&func_expr.lowered_func.func, cx, true) {
        Ok(inner_fn) => {
            let params_str = inner_fn.params.join(", ");

            // For FunctionDeclaration types, preserve the structured CodegenFunction
            // so callers can emit a proper `function name(...) { ... }` declaration
            // statement (matching the TS reference's `createFunctionDeclaration`).
            let fn_decl = if func_expr.expression_type == FunctionExpressionType::FunctionDeclaration
            {
                Some(inner_fn.clone())
            } else {
                None
            };

            let mut value = match func_expr.expression_type {
                FunctionExpressionType::ArrowFunctionExpression => {
                    let async_prefix = if inner_fn.is_async { "async " } else { "" };
                    // Check for concise arrow body: single return statement with argument,
                    // and no directives on the lowered function
                    if inner_fn.body.len() == 1 && func_expr.lowered_func.func.directives.is_empty()
                    {
                        if let CodegenStatement::Return { value: Some(expr), is_conditional } =
                            &inner_fn.body[0]
                        {
                            // Babel wraps ConditionalExpression / SequenceExpression
                            // in parens when used as a concise arrow body. We detect
                            // this via the `is_conditional` flag (populated by the
                            // inner function's own codegen pass), and also apply a
                            // string-based heuristic for cross-context cases where
                            // the reactive IR doesn't track the ternary temp.
                            // Also wrap object literals in parens: `() => ({x})` not
                            // `() => {x}` which would be parsed as a block statement.
                            // JSX expressions are wrapped in parens to match Prettier
                            // formatting: `() => (<div />)` not `() => <div />`.
                            let needs_parens =
                                *is_conditional || expr.starts_with('{') || expr.starts_with('<') || {
                                    let s = expr.replace("??", "__");
                                    s.contains(" ? ")
                                };
                            if needs_parens {
                                format!("{async_prefix}({params_str}) => ({expr})")
                            } else {
                                format!("{async_prefix}({params_str}) => {expr}")
                            }
                        } else {
                            let body_str = format_codegen_body(&inner_fn);
                            format!("{async_prefix}({params_str}) => {{{body_str}}}")
                        }
                    } else {
                        let body_str = format_codegen_body(&inner_fn);
                        format!("{async_prefix}({params_str}) => {{{body_str}}}")
                    }
                }
                FunctionExpressionType::FunctionExpression
                | FunctionExpressionType::FunctionDeclaration => {
                    let async_prefix = if inner_fn.is_async { "async " } else { "" };
                    let star = if inner_fn.generator { "*" } else { "" };
                    let name = func_expr.name.as_ref().map_or(String::new(), |n| format!(" {n}"));
                    let body_str = format_codegen_body(&inner_fn);
                    format!("{async_prefix}function{star}{name}({params_str}) {{{body_str}}}")
                }
            };

            // enableNameAnonymousFunctions: wrap anonymous functions in a naming expression
            // Produces: ({"nameHint": <funcExpr>})["nameHint"]
            if cx.enable_name_anonymous_functions
                && func_expr.name.is_none()
                && func_expr.name_hint.is_some()
            {
                let hint = func_expr.name_hint.as_ref().map_or("", String::as_str);
                value = format!("({{\"{hint}\": {value}}})[\"{hint}\"]");
            }

            (value, fn_decl)
        }
        Err(_) => {
            // Fallback for inner function compilation errors
            ("/* error compiling inner function */".to_string(), None)
        }
    }
}

/// Generate an object expression string, handling properties, shorthand, computed keys,
/// method properties, and spread elements.
///
/// Port of `CodegenReactiveFunction.ts` ObjectExpression case (lines ~1580-1698).
///
/// Method properties are compiled recursively via `codegen_object_method_expression`.
fn codegen_object_expression(cx: &CodegenContext, obj: &ObjectExpression) -> String {
    let mut props: Vec<String> = Vec::with_capacity(obj.properties.len());

    for prop in &obj.properties {
        match prop {
            ObjectPatternProperty::Property(p) => {
                let key = codegen_object_property_key(cx, &p.key);
                let is_computed = matches!(p.key, ObjectPropertyKey::Computed(_));

                if p.property_type == ObjectPropertyType::Method {
                    // Look up the ObjectMethod from the context, clone to avoid borrow conflict
                    let method = cx.object_methods.get(&p.place.identifier.id).cloned();
                    if let Some(method) = method {
                        props.push(codegen_object_method_expression(
                            cx,
                            &method,
                            &key,
                            is_computed,
                        ));
                    } else {
                        // Fallback if method not found (should not happen)
                        props.push(format!("{key}() {{}}"));
                    }
                } else {
                    let value = codegen_place_to_expression(cx, &p.place);
                    let is_shorthand =
                        matches!(&p.key, ObjectPropertyKey::Identifier(k) if *k == value);
                    if is_shorthand && !is_computed {
                        props.push(key);
                    } else if is_computed {
                        props.push(format!("[{key}]: {value}"));
                    } else {
                        props.push(format!("{key}: {value}"));
                    }
                }
            }
            ObjectPatternProperty::Spread(s) => {
                props.push(format!("...{}", codegen_place_to_expression(cx, &s.place)));
            }
        }
    }

    if props.is_empty() { "{}".to_string() } else { format!("{{ {} }}", props.join(", ")) }
}

/// Generate an object method expression string by recursively compiling the inner function.
///
/// Port of `CodegenReactiveFunction.ts` lines 1649-1684 (case 'method' in ObjectExpression).
///
/// Object methods do NOT call `pruneHoistedContexts`.
fn codegen_object_method_expression(
    cx: &CodegenContext,
    method: &ObjectMethodValue,
    key: &str,
    is_computed: bool,
) -> String {
    match codegen_inner_function(&method.lowered_func.func, cx, false) {
        Ok(inner_fn) => {
            let params_str = inner_fn.params.join(", ");
            let body_str = format_codegen_body(&inner_fn);
            let async_prefix = if inner_fn.is_async { "async " } else { "" };
            let star = if inner_fn.generator { "*" } else { "" };
            let key_str = if is_computed { format!("[{key}]") } else { key.to_string() };
            format!("{async_prefix}{star}{key_str}({params_str}) {{{body_str}}}")
        }
        Err(_) => {
            format!("{key}() {{ /* error compiling method */ }}")
        }
    }
}

// =====================================================================================
// codegen_reactive_value_to_expression — ReactiveValue → expression string
// =====================================================================================

/// Determine whether a logical operand expression needs to be wrapped in
/// parentheses. This matches Babel's code generator: a `LogicalExpression`
/// whose operator differs from the parent logical operator gets wrapped,
/// as does a ternary expression (lower precedence than any logical operator).
///
/// For `Sequence` values, we look at the inner final `value` since sequences
/// are reduced to their final value expression during codegen. When the operand
/// was stored in a temp (the common case for nested logical expressions), we
/// also check the `logical_temps` map to recover the original logical operator.
fn wrap_logical_operand_if_needed(
    cx: &CodegenContext,
    expr: &str,
    child_value: &ReactiveValue,
    parent_op: LogicalOperator,
) -> String {
    let effective = effective_value(child_value);
    let needs_parens = match effective {
        ReactiveValue::Logical(child_logical) => child_logical.operator != parent_op,
        ReactiveValue::Ternary(_) => true,
        _ => {
            // The operand may be an Instruction (LoadLocal) that reads from a temp
            // which was originally a logical expression. Check the logical_temps map
            // via the final place's declaration_id.
            if let Some(child_op) = resolve_logical_operator_from_value(cx, child_value) {
                child_op != parent_op
            } else {
                false
            }
        }
    };
    if needs_parens { format!("({expr})") } else { expr.to_string() }
}

/// Try to resolve the logical operator from a value chain that ultimately reads
/// from a temp that was a logical expression. This traces through Sequences
/// and LoadLocal instructions to find the final declaration_id and checks
/// `cx.logical_temps`.
fn resolve_logical_operator_from_value(
    cx: &CodegenContext,
    value: &ReactiveValue,
) -> Option<LogicalOperator> {
    match value {
        ReactiveValue::Instruction(iv) => {
            if let InstructionValue::LoadLocal(load) = iv.as_ref() {
                cx.logical_temps.get(&load.place.identifier.declaration_id).copied()
            } else {
                None
            }
        }
        ReactiveValue::Sequence(seq) => {
            // First check the sequence instructions for a Logical value - the ??
            // operator is often represented as an instruction inside the sequence.
            for instr in &seq.instructions {
                match &instr.value {
                    ReactiveValue::Logical(logical) => return Some(logical.operator),
                    ReactiveValue::Sequence(_) => {
                        if let Some(op) = resolve_logical_operator_from_value(cx, &instr.value) {
                            return Some(op);
                        }
                    }
                    _ => {}
                }
            }
            // Fall back to checking the final value (e.g. LoadLocal of a logical temp)
            resolve_logical_operator_from_value(cx, &seq.value)
        }
        _ => None,
    }
}

/// Search a `ReactiveValue` tree (without codegen context) for the top-level
/// logical operator. This looks through Sequences and their instructions to
/// find a `ReactiveValue::Logical` that represents the "effective" expression.
fn find_logical_operator_in_value(value: &ReactiveValue) -> Option<LogicalOperator> {
    match value {
        ReactiveValue::Logical(logical) => Some(logical.operator),
        ReactiveValue::Sequence(seq) => {
            // Check if the last instruction in the sequence is/contains a Logical
            if let Some(last_instr) = seq.instructions.last() {
                if let Some(op) = find_logical_operator_in_value(&last_instr.value) {
                    return Some(op);
                }
            }
            find_logical_operator_in_value(&seq.value)
        }
        _ => None,
    }
}

/// Unwrap `Sequence` wrappers to find the effective value for precedence checks.
/// Sequences wrap a final value with preceding instructions; the emitted expression
/// string corresponds to that final value.
///
/// When a sequence contains instructions that produce a `Logical` expression and
/// the final value is just a `LoadLocal` of that result temp, we treat the
/// `Logical` as the effective value. This is needed because Babel's AST-based
/// codegen inserts parens automatically for nested logical expressions, while our
/// string-based codegen needs to detect the precedence relationship explicitly.
fn effective_value(value: &ReactiveValue) -> &ReactiveValue {
    match value {
        ReactiveValue::Sequence(seq) => {
            // If the sequence's final value is a plain instruction (LoadLocal) and
            // the last instruction in the sequence produced a Logical/Ternary value,
            // use that instruction's value for precedence instead of the LoadLocal.
            if let ReactiveValue::Instruction(_) = &*seq.value {
                if let Some(last_instr) = seq.instructions.last() {
                    match &last_instr.value {
                        ReactiveValue::Logical(_) | ReactiveValue::Ternary(_) => {
                            return &last_instr.value;
                        }
                        // The last instruction's value may itself be a Sequence
                        // (e.g., when `??` is nested inside `||`, the `??` result
                        // appears as a Sequence-wrapped value in the `||`'s test
                        // block). Recursively resolve it to find the effective value.
                        ReactiveValue::Sequence(_) => {
                            let inner = effective_value(&last_instr.value);
                            if matches!(
                                inner,
                                ReactiveValue::Logical(_) | ReactiveValue::Ternary(_)
                            ) {
                                return inner;
                            }
                        }
                        _ => {}
                    }
                }
            }
            effective_value(&seq.value)
        }
        other => other,
    }
}

/// Convert a `ReactiveValue` to an expression string.
fn codegen_reactive_value_to_expression(cx: &mut CodegenContext, value: &ReactiveValue) -> String {
    match value {
        ReactiveValue::Instruction(boxed) => codegen_instruction_value(cx, boxed),
        ReactiveValue::Logical(logical) => {
            let left = codegen_reactive_value_to_expression(cx, &logical.left);
            let right = codegen_reactive_value_to_expression(cx, &logical.right);
            // Wrap operands in parens when they are logical expressions with a
            // different operator or ternary expressions. This matches Babel's
            // code generator behaviour and is required for correctness when
            // lower-precedence operators are nested inside higher-precedence
            // ones (e.g. `||` inside `&&`), and is mandatory when `??` is
            // mixed with `||`/`&&` (a JavaScript syntax error without parens).
            let left = wrap_logical_operand_if_needed(cx, &left, &logical.left, logical.operator);
            let right =
                wrap_logical_operand_if_needed(cx, &right, &logical.right, logical.operator);
            format!("{left} {} {right}", logical.operator.as_str())
        }
        ReactiveValue::Ternary(ternary) => {
            let test = codegen_reactive_value_to_expression(cx, &ternary.test);
            let consequent = codegen_reactive_value_to_expression(cx, &ternary.consequent);
            let alternate = codegen_reactive_value_to_expression(cx, &ternary.alternate);
            // Wrap subexpressions in parens to match Babel's code generator.
            // Helper to check if a value is/contains a NullishCoalescing (??).
            // We check both structural (effective_value) and via resolve_logical_operator_from_value,
            // since the ?? may be in a Sequence's instructions or a LoadLocal of a logical temp.
            let is_coalesce = |val: &ReactiveValue| -> bool {
                let eff = effective_value(val);
                match eff {
                    ReactiveValue::Logical(l) if l.operator == LogicalOperator::Coalesce => true,
                    _ => {
                        resolve_logical_operator_from_value(cx, val)
                            == Some(LogicalOperator::Coalesce)
                    }
                }
            };
            // Helper to check if a value is/contains a nested ternary.
            let is_ternary = |val: &ReactiveValue| -> bool {
                if matches!(effective_value(val), ReactiveValue::Ternary(_)) {
                    return true;
                }
                // Also check if the value is a Sequence that contains a Ternary instruction
                // (same pattern as resolve_logical_operator_from_value for ?? detection).
                // This handles the case where: Sequence { value: Sequence/Instruction,
                //   instructions: [{ value: Ternary(...) }] }
                if let ReactiveValue::Sequence(seq) = val {
                    for instr in &seq.instructions {
                        match &instr.value {
                            ReactiveValue::Ternary(_) => return true,
                            ReactiveValue::Sequence(_) => {
                                // Recursively check nested sequences
                                if matches!(
                                    effective_value(&instr.value),
                                    ReactiveValue::Ternary(_)
                                ) {
                                    return true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                false
            };
            // Test: wrap nested ternary or NullishCoalescing.
            let test = if is_ternary(&ternary.test)
                || (is_coalesce(&ternary.test) && !test.starts_with('('))
            {
                format!("({test})")
            } else {
                test
            };
            // Consequent: wrap nested ternary or NullishCoalescing.
            let consequent = if is_ternary(&ternary.consequent)
                || (is_coalesce(&ternary.consequent) && !consequent.starts_with('('))
            {
                format!("({consequent})")
            } else {
                consequent
            };
            // Alternate: wrap NullishCoalescing or nested ternary.
            let alternate = if is_coalesce(&ternary.alternate) && !alternate.starts_with('(') {
                format!("({alternate})")
            } else {
                alternate
            };
            format!("{test} ? {consequent} : {alternate}")
        }
        ReactiveValue::Sequence(seq) => {
            // Process sequence instructions, matching the TS reference's
            // `codegenBlockNoReset` + `SequenceExpression` handling.
            //
            // After `prune_unused_lvalues` runs, instructions whose unnamed
            // temp results are never consumed have `lvalue: None`. When
            // `codegen_instruction_nullable` processes these, it produces an
            // `ExpressionStatement` (the side-effect expression). Instructions
            // whose results ARE consumed retain `lvalue: Some(unnamed temp)`
            // and are stored in `cx.temp` (returning None).
            //
            // We collect the ExpressionStatements into a sequence expression
            // `(expr1, expr2, ..., finalValue)` to preserve evaluation order.
            // If there are no expression statements (all instructions are
            // consumed temps), we just emit the final value directly.
            let mut expressions: Vec<String> = Vec::new();

            for instr in &seq.instructions {
                let stmt = codegen_instruction_nullable(cx, instr);

                if let Some(CodegenStatement::ExpressionStatement(expr)) = stmt {
                    expressions.push(expr);
                }
                // VariableDeclarations in value blocks are an error in the TS
                // reference but we silently skip them for now.
            }

            let final_value = codegen_reactive_value_to_expression(cx, &seq.value);
            if expressions.is_empty() {
                final_value
            } else {
                expressions.push(final_value);
                format!("({})", expressions.join(", "))
            }
        }
        ReactiveValue::OptionalCall(optional) => {
            // While generating the body of an optional-chain sequence, member and
            // computed accesses on intermediate optional-chain temps are PART of the
            // chain (e.g. the `.c` in `props?.b.c`), so they must NOT be wrapped in
            // parentheses.  Set the flag before recursing and restore it afterwards.
            let prev_inside_optional_chain = cx.inside_optional_chain;
            cx.inside_optional_chain = true;
            let value = codegen_reactive_value_to_expression(cx, &optional.value);
            cx.inside_optional_chain = prev_inside_optional_chain;
            if optional.optional {
                // The inner value is a complete call expression like `foo.bar(args)` or
                // member expression like `foo.bar`. We need to insert `?` before the last
                // `.` or `(` to produce `foo?.bar(args)` or `foo?.bar`.
                //
                // Strategy: find the boundary between the receiver and the access/call,
                // then insert `?` there.
                if let Some(paren_idx) = find_optional_insertion_point(&value) {
                    let (before, after) = value.split_at(paren_idx);
                    // When the insertion point is a `(` (call), use `?.(`
                    // syntax instead of `?(` which is invalid.
                    // `?.` is used for optional member access (`foo?.bar`)
                    // `?.(` is used for optional calls (`foo?.()`)
                    if after.starts_with('(') || after.starts_with('[') {
                        format!("{before}?.{after}")
                    } else {
                        format!("{before}?{after}")
                    }
                } else {
                    // Fallback: just append ?.
                    format!("{value}?.()")
                }
            } else {
                value
            }
        }
    }
}

// =====================================================================================
// Helper functions
// =====================================================================================

/// Join multi-child JSX children with appropriate newline separators.
///
/// The reference compiler emits expression-container children (`{expr}`) on separate
/// lines, but JSX text nodes remain adjacent to neighboring children. For example:
///
/// ```jsx
/// <>
///   {t3}
///   {t5}
/// </>
/// ```
///
/// But text nodes stay inline:
/// ```jsx
/// <div>
///   {t1}-{t2}-{t3}
/// </div>
/// ```
fn join_jsx_children_multiline(children: &[String]) -> String {
    let mut result = String::new();
    for (i, child) in children.iter().enumerate() {
        if i > 0 {
            let prev = &children[i - 1];
            // Add newline between children when BOTH are expression containers,
            // or when transitioning between a text node and an expression container
            // that is not glued to it. Text nodes (not wrapped in {}) stay adjacent.
            let prev_is_expr = prev.starts_with('{') || prev.starts_with('<');
            let curr_is_expr = child.starts_with('{') || child.starts_with('<');
            if prev_is_expr && curr_is_expr {
                result.push('\n');
            }
            // Text nodes are adjacent — no separator
        }
        result.push_str(child);
    }
    result
}

/// Generate a JSX attribute string.
///
/// Handles namespaced attributes (e.g., `xmlns:xlink`) and string value escaping.
fn codegen_jsx_attribute(cx: &CodegenContext, attr: &JsxAttribute) -> String {
    match attr {
        JsxAttribute::Attribute { name, place } => {
            let value = codegen_place_to_expression(cx, place);
            // Check if value is a simple string literal that can be used directly
            let is_double_quoted = value.starts_with('"') && value.ends_with('"');
            let is_single_quoted = value.starts_with('\'') && value.ends_with('\'');
            if is_double_quoted || is_single_quoted {
                let inner = &value[1..value.len() - 1];
                // If the string contains special chars, wrap in expression container
                if string_requires_expr_container(inner) {
                    format!("{name}={{{value}}}")
                } else {
                    format!("{name}={value}")
                }
            } else {
                format!("{name}={{{value}}}")
            }
        }
        JsxAttribute::Spread { argument } => {
            format!("{{...{}}}", codegen_place_to_expression(cx, argument))
        }
    }
}

/// Check if a string literal value requires wrapping in a JSX expression container.
///
/// Matches the TS `STRING_REQUIRES_EXPR_CONTAINER_PATTERN` regex:
/// `/[\u{0000}-\u{001F}\u{007F}\u{0080}-\u{FFFF}\u{010000}-\u{10FFFF}]|"|\\/u`
fn string_requires_expr_container(s: &str) -> bool {
    for c in s.chars() {
        let code = c as u32;
        // Control characters, DEL, or non-ASCII
        if code <= 0x1F || code == 0x7F || code >= 0x80 {
            return true;
        }
        // Double quote or backslash
        if c == '"' || c == '\\' {
            return true;
        }
    }
    false
}

/// Render a JSX child element. Matches TS `codegenJsxElement` which:
/// - JSX text → raw text (or expression container if contains `<>&{}`)
/// - JSX element/fragment → pass through as-is
/// - Other expressions → wrap in `{...}` expression container
fn codegen_jsx_child(cx: &CodegenContext, place: &Place) -> String {
    let decl_id = place.identifier.declaration_id;
    if cx.jsx_text_temps.contains(&decl_id) {
        let text = codegen_place_to_expression(cx, place);
        // If text contains JSX-special chars, wrap in expression container with string literal
        if text.contains(['<', '>', '&', '{', '}']) {
            format!("{{\"{}\"}}", escape_string(&text))
        } else {
            text
        }
    } else if cx.jsx_element_temps.contains(&decl_id) {
        // JSX elements render directly without wrapper, BUT only if the place
        // resolves to an inline expression (not a promoted variable name).
        // When a temp is promoted/named (e.g. after scope caching), it's just
        // a variable reference like `t0` and needs `{ t0 }` wrapping.
        let expr = codegen_place_to_expression(cx, place);
        if expr.starts_with('<') { expr } else { format!("{{{expr}}}") }
    } else {
        // All other expressions get wrapped in expression containers
        format!("{{{}}}", codegen_place_to_expression(cx, place))
    }
}

/// Convert a Place to an expression string.
///
/// When the place references a temporary that holds an assignment expression
/// (e.g., `x = value`), the result is automatically wrapped in parentheses
/// because assignment expressions have very low precedence and need parens
/// when used as sub-expressions (call args, initializers, etc.).
fn codegen_place_to_expression(cx: &CodegenContext, place: &Place) -> String {
    let decl_id = place.identifier.declaration_id;
    // Check if this place is a temporary with a stored expression
    if let Some(Some(expr)) = cx.temp.get(&decl_id) {
        // Wrap assignment-expression temps in parentheses for correct precedence.
        if cx.assignment_temps.contains(&decl_id) {
            return format!("({expr})");
        }
        return expr.clone();
    }

    // Must be a named identifier
    identifier_name(&place.identifier)
}

/// Convert a Place to an expression string WITHOUT wrapping assignment expressions.
/// Used in contexts where assignment expressions are being chained (e.g., `x = y = z = 1`)
/// and parens would be superfluous since `=` is right-associative.
fn codegen_place_to_expression_raw(cx: &CodegenContext, place: &Place) -> String {
    let decl_id = place.identifier.declaration_id;
    if let Some(Some(expr)) = cx.temp.get(&decl_id) {
        return expr.clone();
    }
    identifier_name(&place.identifier)
}

/// Get the string name of an identifier.
fn identifier_name(identifier: &crate::hir::Identifier) -> String {
    match &identifier.name {
        Some(IdentifierName::Named(name) | IdentifierName::Promoted(name)) => name.clone(),
        None => format!("t${}", identifier.id.0),
    }
}

/// Generate a sequence expression string for ExpressionStatement context.
///
/// Unlike `codegen_reactive_value_to_expression` which wraps the entire sequence
/// in outer parens `(expr1, expr2, ..., value)`, this produces the Prettier-style
/// format used in ExpressionStatement context: no outer parens, but individual
/// assignment sub-expressions are wrapped in parens.
///
/// e.g., `(x = []), null` instead of `(x = [], null)`
///
/// This matches Prettier's formatting: in an ExpressionStatement, SequenceExpression
/// does not need outer parens (the semicolon terminates the statement), but
/// individual AssignmentExpressions within the sequence are parenthesized for clarity.
fn codegen_sequence_for_expr_stmt(cx: &mut CodegenContext, value: &ReactiveValue) -> String {
    let ReactiveValue::Sequence(seq) = value else {
        return codegen_reactive_value_to_expression(cx, value);
    };

    let mut expressions: Vec<String> = Vec::new();

    for instr in &seq.instructions {
        let stmt = codegen_instruction_nullable(cx, instr);

        if let Some(CodegenStatement::ExpressionStatement(expr)) = stmt {
            if is_top_level_assignment(&expr) {
                expressions.push(format!("({expr})"));
            } else {
                expressions.push(expr);
            }
        }
    }

    let final_value = codegen_reactive_value_to_expression(cx, &seq.value);
    if expressions.is_empty() {
        final_value
    } else {
        expressions.push(final_value);
        expressions.join(", ")
    }
}

/// Check if an expression string contains a top-level assignment operator (`=`).
///
/// Used by the Sequence expression handler to wrap assignment sub-expressions
/// in parens, matching Prettier's formatting of `SequenceExpression` members.
/// e.g., in the sequence `(x = []), null`, the assignment `x = []` is wrapped.
///
/// Returns `true` if the string contains `=` at the top level (not inside
/// parens/brackets/braces/strings) that is not part of `==`, `===`, `!=`,
/// `!==`, `<=`, `>=`, or `=>`.
fn is_top_level_assignment(expr: &str) -> bool {
    let bytes = expr.as_bytes();
    let len = bytes.len();
    let mut depth: i32 = 0;
    let mut i = 0;
    while i < len {
        match bytes[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            b'\'' | b'"' | b'`' => {
                let quote = bytes[i];
                i += 1;
                while i < len && bytes[i] != quote {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'=' if depth == 0 => {
                let prev = if i > 0 { bytes[i - 1] } else { 0 };
                let next = if i + 1 < len { bytes[i + 1] } else { 0 };
                // Skip ==, ===, !=, !==, <=, >=, =>
                if next != b'=' && next != b'>' && prev != b'!' && prev != b'<' && prev != b'>' && prev != b'=' {
                    return true;
                }
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// Check if an expression string is a ternary (conditional) expression at the top level.
///
/// Returns `true` if the string contains ` ? ` that is not inside any parentheses,
/// brackets, or braces. This is used to detect when a substituted temp value is a
/// ternary expression that needs wrapping in parens inside binary expressions.
///
/// e.g., `cond ? a : b` → true
///       `(cond ? a : b)` → false (already parenthesized)
///       `foo(cond ? a : b)` → false (the `?` is inside function call parens)
fn is_ternary_expression(expr: &str) -> bool {
    let bytes = expr.as_bytes();
    let len = bytes.len();
    let mut depth: i32 = 0;
    let mut i = 0;
    while i < len {
        match bytes[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b'\'' | b'"' | b'`' => {
                // Skip over string literals
                let quote = bytes[i];
                i += 1;
                while i < len && bytes[i] != quote {
                    if bytes[i] == b'\\' {
                        i += 1; // skip escaped char
                    }
                    i += 1;
                }
            }
            b'?' if depth == 0 => {
                // Check if this is ` ? ` (with spaces) - ternary, not optional chain
                if i > 0 && i + 1 < len && bytes[i - 1] == b' ' && bytes[i + 1] == b' ' {
                    return true;
                }
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// Check if a string expression contains ` ?? ` (nullish coalescing) at the top level
/// (not inside parens/brackets/braces/strings).
///
/// Used to detect when a `??` expression is used as an operand in a binary expression
/// and needs to be wrapped in parentheses for correct operator precedence.
/// `??` has lower precedence than `+`, `-`, `*`, `/`, `%`, etc.
fn is_nullish_coalescing_expression(expr: &str) -> bool {
    let bytes = expr.as_bytes();
    let len = bytes.len();
    let mut depth: i32 = 0;
    let mut i = 0;
    while i < len {
        match bytes[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b'\'' | b'"' | b'`' => {
                // Skip over string literals
                let quote = bytes[i];
                i += 1;
                while i < len && bytes[i] != quote {
                    if bytes[i] == b'\\' {
                        i += 1; // skip escaped char
                    }
                    i += 1;
                }
            }
            b'?' if depth == 0 => {
                // Check if this is ` ?? ` (with spaces and double `?`)
                if i + 1 < len && bytes[i + 1] == b'?' {
                    // It's `??` - check it's surrounded by spaces to be an operator
                    if i > 0 && i + 2 < len && bytes[i - 1] == b' ' && bytes[i + 2] == b' ' {
                        return true;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// Find the point in a codegen'd expression string where `?` should be inserted
/// to convert a call/member expression to an optional chain.
///
/// For `foo.bar(args)` returns the index of `.` → produces `foo?.bar(args)`
/// For `foo.bar` returns the index of `.bar` → produces `foo?.bar`
/// For `foo(args)` returns the index of `(` → produces `foo?.(args)`
fn find_optional_insertion_point(expr: &str) -> Option<usize> {
    let bytes = expr.as_bytes();
    let mut depth_paren: i32 = 0;
    let mut depth_bracket: i32 = 0;
    let mut last_access_point: Option<usize> = None;

    // Scan from the end to find the rightmost top-level call/access operator.
    //
    // The goal: given a string like `callee(args)` or `callee?.(args)(more_args)`,
    // find the position where we should insert `?.` to make the outermost call
    // optional.  For `callee(args)` that's the `(`.  For `callee?.(args)(more_args)`
    // that's the second `(` (the last top-level call), NOT the `.` in `?.`.
    //
    // Algorithm:
    // 1. Scanning right-to-left, find the FIRST `(` or `[` encountered at depth 0
    //    (that is, the rightmost top-level call/computed access).
    //    Record it in `last_access_point` but do NOT immediately return — keep
    //    scanning to determine whether a `.` member access comes immediately before it.
    // 2. If we subsequently encounter a `.` at depth 0:
    //    - If `last_access_point` is already set, the `(` is the boundary point
    //      (not the `.`), so return `last_access_point`.
    //    - Otherwise return the `.` position (the `.` IS the boundary for cases
    //      like `foo.bar` where there's no preceding call).
    // 3. If we hit any other character at depth 0 after registering `last_access_point`,
    //    the call/bracket IS the insertion boundary — return it.
    for i in (0..bytes.len()).rev() {
        match bytes[i] {
            b')' => depth_paren += 1,
            b'(' => {
                depth_paren -= 1;
                if depth_paren == 0 && depth_bracket == 0 {
                    // Record only the FIRST (rightmost) `(` we encounter at depth 0.
                    // Do NOT overwrite a previously found call: the rightmost one is
                    // the correct insertion boundary.
                    if last_access_point.is_none() {
                        last_access_point = Some(i);
                    }
                    // Keep scanning to see if a `.` precedes this call.
                    continue;
                }
            }
            b']' => depth_bracket += 1,
            b'[' => {
                depth_bracket -= 1;
                if depth_paren == 0 && depth_bracket == 0 {
                    if last_access_point.is_none() {
                        last_access_point = Some(i);
                    }
                    continue;
                }
            }
            b'.' if depth_paren == 0 && depth_bracket == 0 => {
                // If we already found a `(` or `[` to the right of this `.`, that
                // call/bracket is the rightmost top-level operation and should be the
                // insertion point (not this `.`).  This prevents inserting into the
                // middle of an existing `?.` sequence.
                if last_access_point.is_some() {
                    return last_access_point;
                }
                return Some(i);
            }
            _ => {
                if depth_paren == 0 && depth_bracket == 0 && last_access_point.is_some() {
                    return last_access_point;
                }
            }
        }
    }
    last_access_point
}

/// Generate a label string from a BlockId.
fn codegen_label(id: crate::hir::BlockId) -> String {
    format!("bb{}", id.0)
}

/// Generate a primitive value expression string.
fn codegen_primitive(value: &PrimitiveValueKind) -> String {
    match value {
        PrimitiveValueKind::Number(n) => {
            // Negative zero: JS `(-0).toString()` returns "0", and Babel's codegen
            // emits `0` for the value `-0`. Match that behavior.
            if *n == 0.0 && n.is_sign_negative() {
                "0".to_string()
            } else if *n < 0.0 {
                format!("-{}", -n)
            } else {
                format!("{n}")
            }
        }
        PrimitiveValueKind::Boolean(b) => format!("{b}"),
        PrimitiveValueKind::String(s) => {
            if s.contains('"') {
                // Use single quotes when the string contains double quotes
                format!("'{}'", escape_string_single_quote(s))
            } else {
                format!("\"{}\"", escape_string(s))
            }
        }
        PrimitiveValueKind::Null => "null".to_string(),
        PrimitiveValueKind::Undefined => "undefined".to_string(),
    }
}

/// Escape special characters in a double-quoted string literal.
///
/// Handles all standard JavaScript escape sequences so that decoded string values
/// (where the parser has already resolved escape sequences like `\b` into their
/// actual character values) are correctly re-encoded for output.
fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\u{0008}' => result.push_str("\\b"),
            '\u{000C}' => result.push_str("\\f"),
            '\u{000B}' => result.push_str("\\v"),
            '\0' => result.push_str("\\0"),
            c if c.is_control() => {
                // Escape other control characters as \uXXXX
                for unit in c.encode_utf16(&mut [0; 2]) {
                    result.push_str(&format!("\\u{unit:04x}"));
                }
            }
            _ => result.push(c),
        }
    }
    result
}

/// Escape special characters in a single-quoted string literal.
/// Does not escape double quotes but escapes single quotes.
fn escape_string_single_quote(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\'' => result.push_str("\\'"),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\u{0008}' => result.push_str("\\b"),
            '\u{000C}' => result.push_str("\\f"),
            '\u{000B}' => result.push_str("\\v"),
            '\0' => result.push_str("\\0"),
            c if c.is_control() => {
                for unit in c.encode_utf16(&mut [0; 2]) {
                    result.push_str(&format!("\\u{unit:04x}"));
                }
            }
            _ => result.push(c),
        }
    }
    result
}

/// Returns `true` if `expr` is an optional-chain expression that would absorb
/// a following `.prop` or `[prop]` access into the chain, changing semantics.
///
/// Specifically, this detects `?.` at nesting depth 0 (not inside `()`, `[]`,
/// or template literal `${}`).  When such an expression is used as the object
/// of a *non-optional* member access we must wrap it in parentheses, e.g.
///
///   `props?.a`    + `.b`  → `(props?.a).b`   (not `props?.a.b`)
///   `a?.b?.c`     + `.d`  → `(a?.b?.c).d`
///   `foo(x?.y)`   + `.z`  → `foo(x?.y).z`    (no wrap needed; `?.` is nested)
fn is_optional_chain_at_top_level(expr: &str) -> bool {
    let bytes = expr.as_bytes();
    let mut depth_paren: i32 = 0;
    let mut depth_bracket: i32 = 0;
    // Simple template-literal depth (counts opening backtick vs closing)
    let mut depth_template: i32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth_paren += 1,
            b')' => depth_paren -= 1,
            b'[' => depth_bracket += 1,
            b']' => depth_bracket -= 1,
            b'`' => {
                // Toggle template literal depth (simplified: no nested templates)
                if depth_template > 0 {
                    depth_template -= 1;
                } else {
                    depth_template += 1;
                }
            }
            b'?' if depth_paren == 0 && depth_bracket == 0 && depth_template == 0 => {
                // Check for `?.` (not `??`)
                if i + 1 < bytes.len() && bytes[i + 1] == b'.' {
                    return true;
                }
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// Wrap `object` in parentheses if it is an optional-chain expression that
/// would absorb a following `.prop` / `[prop]` into the chain, AND we are not
/// currently inside an optional-chain Sequence (where the `.prop` access is
/// intentionally part of the chain itself).
///
/// # When parens are needed
/// `(props?.a).b` — the `.b` is an unconditional access on the result of the
/// chain, not part of the chain itself.  Without parens the output would be
/// `props?.a.b`, which short-circuits `.b` as well.
///
/// # When parens are NOT needed
/// `props?.b.c` — the `.c` is *inside* the chain (optional=false in the
/// OptionalMemberExpression).  Here the object temp holds `"props?.b"` during
/// sequence processing, but the outer `OptionalCall` will later insert `?.`
/// at the right position so the string must remain `props.b.c` (the `?.` will
/// be injected to produce `props?.b.c`).
fn maybe_parenthesize_optional_chain(cx: &CodegenContext, object: &str) -> String {
    // Inside an optional-chain Sequence the access is part of the chain, so no
    // wrapping is required (and would in fact be wrong).
    if cx.inside_optional_chain {
        return object.to_string();
    }
    if is_optional_chain_at_top_level(object) { format!("({object})") } else { object.to_string() }
}

/// Generate a member access expression.
fn codegen_member_access(
    cx: &CodegenContext,
    object: &str,
    property: &crate::hir::types::PropertyLiteral,
) -> String {
    // If `object` is an optional-chain expression (e.g. `props?.a`), a following
    // non-optional access like `.b` would be parsed as part of the chain, producing
    // `props?.a.b` instead of `(props?.a).b`.  Wrap with parens to prevent this.
    // The wrapping is skipped when we are inside an optional-chain Sequence because
    // in that context the access IS part of the chain (e.g. the `.c` in `props?.b.c`).
    let object = maybe_parenthesize_optional_chain(cx, object);
    match property {
        crate::hir::types::PropertyLiteral::String(name) => {
            format!("{object}.{name}")
        }
        crate::hir::types::PropertyLiteral::Number(n) => {
            format!("{object}[{n}]")
        }
    }
}

/// Check if a string is a valid JavaScript identifier name that can be used
/// unquoted as an object property key. This includes keywords since they are
/// valid as property keys in ES5+.
fn is_valid_identifier_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' && first != '$' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Generate an object property key string.
fn codegen_object_property_key(cx: &CodegenContext, key: &ObjectPropertyKey) -> String {
    match key {
        ObjectPropertyKey::String(s) => {
            // If the string is a valid identifier name, emit it unquoted.
            // This matches the reference compiler behavior where `{['foo']: v}`
            // is lowered to `{foo: v}` rather than `{"foo": v}`.
            if is_valid_identifier_name(s) {
                s.clone()
            } else {
                format!("\"{}\"", escape_string(s))
            }
        }
        ObjectPropertyKey::Identifier(name) => name.clone(),
        ObjectPropertyKey::Computed(place) => codegen_place_to_expression(cx, place),
        ObjectPropertyKey::Number(n) => format!("{n}"),
    }
}

/// Generate call arguments string.
fn codegen_args(cx: &CodegenContext, args: &[CallArg]) -> String {
    args.iter()
        .map(|arg| match arg {
            CallArg::Place(p) => codegen_place_to_expression(cx, p),
            CallArg::Spread(s) => {
                format!("...{}", codegen_place_to_expression(cx, &s.place))
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Generate a destructure pattern string.
fn codegen_pattern(cx: &CodegenContext, pattern: &Pattern) -> String {
    match pattern {
        Pattern::Array(arr) => {
            let elements: Vec<String> = arr
                .items
                .iter()
                .map(|item| match item {
                    ArrayPatternElement::Place(p) => codegen_place_to_expression(cx, p),
                    ArrayPatternElement::Spread(s) => {
                        format!("...{}", codegen_place_to_expression(cx, &s.place))
                    }
                    ArrayPatternElement::Hole => String::new(),
                })
                .collect();
            format!("[{}]", elements.join(", "))
        }
        Pattern::Object(obj) => {
            let props: Vec<String> = obj
                .properties
                .iter()
                .map(|prop| match prop {
                    ObjectPatternProperty::Property(p) => {
                        let key = codegen_object_property_key(cx, &p.key);
                        let value = codegen_place_to_expression(cx, &p.place);
                        let is_computed = matches!(p.key, ObjectPropertyKey::Computed(_));
                        let is_shorthand =
                            matches!(&p.key, ObjectPropertyKey::Identifier(k) if *k == value);
                        if is_shorthand && !is_computed {
                            key
                        } else if is_computed {
                            format!("[{key}]: {value}")
                        } else {
                            format!("{key}: {value}")
                        }
                    }
                    ObjectPatternProperty::Spread(s) => {
                        format!("...{}", codegen_place_to_expression(cx, &s.place))
                    }
                })
                .collect();
            if props.is_empty() { "{}".to_string() } else { format!("{{ {} }}", props.join(", ")) }
        }
    }
}

/// Iterate over all Place operands in a pattern.
fn each_pattern_operand(pattern: &Pattern) -> Vec<&Place> {
    let mut places = Vec::new();
    collect_pattern_operands(pattern, &mut places);
    places
}

fn collect_pattern_operands<'a>(pattern: &'a Pattern, places: &mut Vec<&'a Place>) {
    match pattern {
        Pattern::Array(arr) => {
            for item in &arr.items {
                match item {
                    ArrayPatternElement::Place(p) => places.push(p),
                    ArrayPatternElement::Spread(s) => places.push(&s.place),
                    ArrayPatternElement::Hole => {}
                }
            }
        }
        Pattern::Object(obj) => {
            for prop in &obj.properties {
                match prop {
                    ObjectPatternProperty::Property(p) => places.push(&p.place),
                    ObjectPatternProperty::Spread(s) => places.push(&s.place),
                }
            }
        }
    }
}

/// Generate a dependency expression string.
fn codegen_dependency(dep: &ReactiveScopeDependency) -> String {
    let mut object = identifier_name(&dep.identifier);
    for entry in &dep.path {
        match &entry.property {
            crate::hir::types::PropertyLiteral::String(name) => {
                if entry.optional {
                    object = format!("{object}?.{name}");
                } else {
                    object = format!("{object}.{name}");
                }
            }
            crate::hir::types::PropertyLiteral::Number(n) => {
                if entry.optional {
                    object = format!("{object}?.[{n}]");
                } else {
                    object = format!("{object}[{n}]");
                }
            }
        }
    }
    object
}

/// Generate the init part of a for statement.
fn codegen_for_init(cx: &mut CodegenContext, init: &ReactiveValue) -> String {
    match init {
        ReactiveValue::Sequence(seq) => {
            // Process sequence instructions to build variable declarations.
            // In a for-init like `let i = 0, length = items.length`, only the
            // first declaration gets the `let`/`const` keyword; subsequent ones
            // in the same comma-separated list omit it.
            let mut parts: Vec<String> = Vec::new();
            let mut first_var_kind: Option<VarKind> = None;
            for instr in &seq.instructions {
                if let Some(stmt) = codegen_instruction_nullable(cx, instr) {
                    match stmt {
                        CodegenStatement::VariableDeclaration { kind, name, init } => {
                            let init_str = init.map_or(String::new(), |i| format!(" = {i}"));
                            if first_var_kind.is_none() {
                                first_var_kind = Some(kind);
                                parts.push(format!("{} {name}{init_str}", kind.as_str()));
                            } else {
                                // Subsequent declarations in the same for-init
                                // omit the keyword (comma-separated declarators).
                                parts.push(format!("{name}{init_str}"));
                            }
                        }
                        CodegenStatement::ExpressionStatement(expr) => {
                            parts.push(expr);
                        }
                        _ => {}
                    }
                }
            }
            parts.join(", ")
        }
        other => codegen_reactive_value_to_expression(cx, other),
    }
}

/// Extract left and right for for-of from reactive init/test values.
fn codegen_for_of_in_init(
    cx: &mut CodegenContext,
    _init: &ReactiveValue,
    test: &ReactiveValue,
) -> (VarKind, String) {
    // The test value for for-of contains the iterator item assignment.
    // Search all instructions in reverse for the StoreLocal/Destructure that
    // defines the loop variable. This is more robust than checking a fixed index
    // because the position may vary depending on how many instructions precede the
    // assignment (e.g. IteratorNext may be followed by Destructure and/or StoreLocal).
    if let ReactiveValue::Sequence(seq) = test {
        for item_instr in seq.instructions.iter().rev() {
            if let ReactiveValue::Instruction(boxed) = &item_instr.value {
                match boxed.as_ref() {
                    InstructionValue::StoreLocal(store) => {
                        // Only use this StoreLocal if it has a named binding (not a temp)
                        if store.lvalue.place.identifier.name.is_some() {
                            let kind = match store.lvalue.kind {
                                InstructionKind::Const | InstructionKind::HoistedConst => {
                                    VarKind::Const
                                }
                                _ => VarKind::Let,
                            };
                            let name = identifier_name(&store.lvalue.place.identifier);
                            cx.declare(store.lvalue.place.identifier.declaration_id);
                            return (kind, name);
                        }
                    }
                    InstructionValue::Destructure(destr) => {
                        let kind = match destr.lvalue.kind {
                            InstructionKind::Const | InstructionKind::HoistedConst => {
                                VarKind::Const
                            }
                            _ => VarKind::Let,
                        };
                        let pattern = codegen_pattern(cx, &destr.lvalue.pattern);
                        return (kind, pattern);
                    }
                    _ => {}
                }
            }
        }
    }
    (VarKind::Const, "item".to_string())
}

/// Extract the collection expression for for-of from the init value.
fn codegen_for_of_collection(cx: &mut CodegenContext, init: &ReactiveValue) -> String {
    if let ReactiveValue::Sequence(seq) = init
        && let Some(first) = seq.instructions.first()
    {
        if let ReactiveValue::Instruction(boxed) = &first.value {
            if let InstructionValue::GetIterator(iter) = boxed.as_ref() {
                return codegen_place_to_expression(cx, &iter.collection);
            }
        }
    }
    codegen_reactive_value_to_expression(cx, init)
}

/// Extract left and collection for for-in from the init value.
fn codegen_for_in_init(cx: &mut CodegenContext, init: &ReactiveValue) -> (VarKind, String) {
    if let ReactiveValue::Sequence(seq) = init {
        // Search all instructions (not just index 1) for the StoreLocal/Destructure
        // that defines the loop variable. The position varies depending on how many
        // instructions are in the init block (e.g., phi nodes may be present).
        for item_instr in seq.instructions.iter().rev() {
            if let ReactiveValue::Instruction(boxed) = &item_instr.value {
                match boxed.as_ref() {
                    InstructionValue::StoreLocal(store) => {
                        let kind = match store.lvalue.kind {
                            InstructionKind::Const | InstructionKind::HoistedConst => {
                                VarKind::Const
                            }
                            _ => VarKind::Let,
                        };
                        let name = identifier_name(&store.lvalue.place.identifier);
                        cx.declare(store.lvalue.place.identifier.declaration_id);
                        return (kind, name);
                    }
                    InstructionValue::Destructure(destr) => {
                        let kind = match destr.lvalue.kind {
                            InstructionKind::Const | InstructionKind::HoistedConst => {
                                VarKind::Const
                            }
                            _ => VarKind::Let,
                        };
                        let pattern = codegen_pattern(cx, &destr.lvalue.pattern);
                        return (kind, pattern);
                    }
                    _ => {}
                }
            }
        }
    }
    (VarKind::Const, "key".to_string())
}

/// Extract the collection expression for for-in from the init value.
fn codegen_for_in_collection(cx: &mut CodegenContext, init: &ReactiveValue) -> String {
    if let ReactiveValue::Sequence(seq) = init
        && let Some(first) = seq.instructions.first()
    {
        return codegen_reactive_value_to_expression(cx, &first.value);
    }
    codegen_reactive_value_to_expression(cx, init)
}

/// Compare two scope dependencies for deterministic ordering.
///
/// Port of `compareScopeDependency()` from `CodegenReactiveFunction.ts` lines 2426-2448.
/// Builds a sort key from the identifier name joined with path entries (with optional markers).
fn compare_scope_dependency(
    a: &ReactiveScopeDependency,
    b: &ReactiveScopeDependency,
) -> std::cmp::Ordering {
    let a_key = build_dependency_sort_key(a);
    let b_key = build_dependency_sort_key(b);
    a_key.cmp(&b_key)
}

/// Build a sort key for a dependency, matching the TypeScript:
/// ```js
/// [identifier.name.value, ...path.map(entry => `${entry.optional ? '?' : ''}${entry.property}`)].join('.')
/// ```
fn build_dependency_sort_key(dep: &ReactiveScopeDependency) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(1 + dep.path.len());
    parts.push(identifier_name(&dep.identifier));
    for entry in &dep.path {
        let optional_prefix = if entry.optional { "?" } else { "" };
        match &entry.property {
            crate::hir::types::PropertyLiteral::String(name) => {
                parts.push(format!("{optional_prefix}{name}"));
            }
            crate::hir::types::PropertyLiteral::Number(n) => {
                parts.push(format!("{optional_prefix}{n}"));
            }
        }
    }
    parts.join(".")
}

/// Compare two scope declarations for deterministic ordering.
fn compare_scope_declaration(
    a: &ReactiveScopeDeclaration,
    b: &ReactiveScopeDeclaration,
) -> std::cmp::Ordering {
    let a_name = identifier_name(&a.identifier);
    let b_name = identifier_name(&b.identifier);
    a_name.cmp(&b_name)
}

// =====================================================================================
// CodegenStatement formatting (for display/debug)
// =====================================================================================

impl std::fmt::Display for CodegenStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_statement(f, self, 0)
    }
}

/// Format a value string that may contain newlines, wrapping in parentheses with
/// proper indentation. Matches the TS reference compiler's behavior for multi-line
/// JSX expressions.
///
/// If the value contains newlines, it becomes:
/// ```text
/// (
///   <indented lines of value>
/// )
/// ```
/// Otherwise the value is returned as-is.
fn write_statement(
    f: &mut std::fmt::Formatter<'_>,
    stmt: &CodegenStatement,
    indent: usize,
) -> std::fmt::Result {
    let pad = "  ".repeat(indent);
    match stmt {
        CodegenStatement::VariableDeclaration { kind, name, init } => match init {
            Some(val) => {
                writeln!(f, "{pad}{} {name} = {val};", kind.as_str())
            }
            None => writeln!(f, "{pad}{} {name};", kind.as_str()),
        },
        CodegenStatement::ExpressionStatement(expr) => {
            writeln!(f, "{pad}{expr};")
        }
        CodegenStatement::Block(stmts) => {
            writeln!(f, "{pad}{{")?;
            for s in stmts {
                write_statement(f, s, indent + 1)?;
            }
            writeln!(f, "{pad}}}")
        }
        CodegenStatement::If { test, consequent, alternate } => {
            writeln!(f, "{pad}if ({test}) {{")?;
            for s in consequent {
                write_statement(f, s, indent + 1)?;
            }
            if let Some(alt) = alternate {
                writeln!(f, "{pad}}} else {{")?;
                for s in alt {
                    write_statement(f, s, indent + 1)?;
                }
            }
            writeln!(f, "{pad}}}")
        }
        CodegenStatement::Return { value, .. } => match value {
            Some(val) => writeln!(f, "{pad}return {val};"),
            None => writeln!(f, "{pad}return;"),
        },
        CodegenStatement::For { init, test, update, body } => {
            let init_str = init.as_deref().unwrap_or("");
            let test_str = test.as_deref().unwrap_or("");
            let update_str = update.as_deref().unwrap_or("");
            if body.is_empty() {
                writeln!(f, "{pad}for ({init_str}; {test_str}; {update_str}) {{}}")
            } else {
                writeln!(f, "{pad}for ({init_str}; {test_str}; {update_str}) {{")?;
                for s in body {
                    write_statement(f, s, indent + 1)?;
                }
                writeln!(f, "{pad}}}")
            }
        }
        CodegenStatement::ForOf { kind, left, right, body } => {
            if body.is_empty() {
                writeln!(f, "{pad}for ({} {left} of {right}) {{}}", kind.as_str())
            } else {
                writeln!(f, "{pad}for ({} {left} of {right}) {{", kind.as_str())?;
                for s in body {
                    write_statement(f, s, indent + 1)?;
                }
                writeln!(f, "{pad}}}")
            }
        }
        CodegenStatement::ForIn { kind, left, right, body } => {
            if body.is_empty() {
                writeln!(f, "{pad}for ({} {left} in {right}) {{}}", kind.as_str())
            } else {
                writeln!(f, "{pad}for ({} {left} in {right}) {{", kind.as_str())?;
                for s in body {
                    write_statement(f, s, indent + 1)?;
                }
                writeln!(f, "{pad}}}")
            }
        }
        CodegenStatement::While { test, body } => {
            if body.is_empty() {
                writeln!(f, "{pad}while ({test}) {{}}")
            } else {
                writeln!(f, "{pad}while ({test}) {{")?;
                for s in body {
                    write_statement(f, s, indent + 1)?;
                }
                writeln!(f, "{pad}}}")
            }
        }
        CodegenStatement::DoWhile { body, test } => {
            writeln!(f, "{pad}do {{")?;
            for s in body {
                write_statement(f, s, indent + 1)?;
            }
            writeln!(f, "{pad}}} while ({test});")
        }
        CodegenStatement::Switch { discriminant, cases } => {
            writeln!(f, "{pad}switch ({discriminant}) {{")?;
            for case in cases {
                match &case.test {
                    Some(t) => writeln!(f, "{pad}  case {t}:")?,
                    None => writeln!(f, "{pad}  default:")?,
                }
                for s in &case.body {
                    write_statement(f, s, indent + 2)?;
                }
            }
            writeln!(f, "{pad}}}")
        }
        CodegenStatement::Break(label) => match label {
            Some(l) => writeln!(f, "{pad}break {l};"),
            None => writeln!(f, "{pad}break;"),
        },
        CodegenStatement::Continue(label) => match label {
            Some(l) => writeln!(f, "{pad}continue {l};"),
            None => writeln!(f, "{pad}continue;"),
        },
        CodegenStatement::Try { block, handler_param, handler, finalizer } => {
            // Collapse empty try block onto one line
            if block.is_empty() {
                write!(f, "{pad}try {{}}")?;
            } else {
                writeln!(f, "{pad}try {{")?;
                for s in block {
                    write_statement(f, s, indent + 1)?;
                }
                write!(f, "{pad}}}")?;
            }
            if let Some(handler_stmts) = handler {
                // Collapse empty catch block onto same line
                if handler_stmts.is_empty() {
                    match handler_param {
                        Some(param) => write!(f, " catch ({param}) {{}}")?,
                        None => write!(f, " catch {{}}")?,
                    }
                } else {
                    match handler_param {
                        Some(param) => writeln!(f, " catch ({param}) {{")?,
                        None => writeln!(f, " catch {{")?,
                    }
                    for s in handler_stmts {
                        write_statement(f, s, indent + 1)?;
                    }
                    write!(f, "{pad}}}")?;
                }
            }
            if let Some(finalizer_stmts) = finalizer {
                if finalizer_stmts.is_empty() {
                    write!(f, " finally {{}}")?;
                } else {
                    writeln!(f, " finally {{")?;
                    for s in finalizer_stmts {
                        write_statement(f, s, indent + 1)?;
                    }
                    write!(f, "{pad}}}")?;
                }
            }
            writeln!(f)
        }
        CodegenStatement::Throw(expr) => {
            writeln!(f, "{pad}throw {expr};")
        }
        CodegenStatement::Labeled { label, body } => {
            write!(f, "{pad}{label}: ")?;
            write_statement(f, body, indent)
        }
        CodegenStatement::FunctionDeclaration { name, params, body, generator, is_async } => {
            let async_prefix = if *is_async { "async " } else { "" };
            let star = if *generator { "*" } else { "" };
            writeln!(f, "{pad}{async_prefix}function{star} {name}({}) {{", params.join(", "))?;
            for s in body {
                write_statement(f, s, indent + 1)?;
            }
            writeln!(f, "{pad}}}")
        }
        CodegenStatement::Debugger => writeln!(f, "{pad}debugger;"),
        CodegenStatement::Empty => Ok(()),
    }
}

impl std::fmt::Display for CodegenFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for directive in &self.directives {
            writeln!(f, "\"{directive}\";")?;
        }
        for stmt in &self.body {
            write_statement(f, stmt, 0)?;
        }
        Ok(())
    }
}

// =====================================================================================
// Backward-compatible exports (used by count_memo_slots in old code)
// =====================================================================================

/// Count the number of memo slots needed for the reactive function.
/// This is kept for compatibility but the actual counting is now done
/// via the MemoCounter visitor in codegen_function.
pub fn count_memo_slots(
    block: &ReactiveBlock,
    memo_blocks: &mut u32,
    memo_values: &mut u32,
    pruned_blocks: &mut u32,
    pruned_values: &mut u32,
) {
    for stmt in block {
        match stmt {
            ReactiveStatement::Scope(scope) => {
                *memo_blocks += 1;
                *memo_values += u32::try_from(scope.scope.declarations.len()).unwrap_or(u32::MAX);
                *memo_values += 1;
                count_memo_slots(
                    &scope.instructions,
                    memo_blocks,
                    memo_values,
                    pruned_blocks,
                    pruned_values,
                );
            }
            ReactiveStatement::PrunedScope(scope) => {
                *pruned_blocks += 1;
                *pruned_values += u32::try_from(scope.scope.declarations.len()).unwrap_or(u32::MAX);
                count_memo_slots(
                    &scope.instructions,
                    memo_blocks,
                    memo_values,
                    pruned_blocks,
                    pruned_values,
                );
            }
            ReactiveStatement::Terminal(term) => {
                count_terminal_memo_slots(
                    &term.terminal,
                    memo_blocks,
                    memo_values,
                    pruned_blocks,
                    pruned_values,
                );
            }
            ReactiveStatement::Instruction(_) => {}
        }
    }
}

fn count_terminal_memo_slots(
    terminal: &ReactiveTerminal,
    memo_blocks: &mut u32,
    memo_values: &mut u32,
    pruned_blocks: &mut u32,
    pruned_values: &mut u32,
) {
    match terminal {
        ReactiveTerminal::If(t) => {
            count_memo_slots(&t.consequent, memo_blocks, memo_values, pruned_blocks, pruned_values);
            if let Some(alt) = &t.alternate {
                count_memo_slots(alt, memo_blocks, memo_values, pruned_blocks, pruned_values);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &t.cases {
                if let Some(block) = &case.block {
                    count_memo_slots(block, memo_blocks, memo_values, pruned_blocks, pruned_values);
                }
            }
        }
        ReactiveTerminal::While(t) => {
            count_memo_slots(&t.r#loop, memo_blocks, memo_values, pruned_blocks, pruned_values);
        }
        ReactiveTerminal::DoWhile(t) => {
            count_memo_slots(&t.r#loop, memo_blocks, memo_values, pruned_blocks, pruned_values);
        }
        ReactiveTerminal::For(t) => {
            count_memo_slots(&t.r#loop, memo_blocks, memo_values, pruned_blocks, pruned_values);
        }
        ReactiveTerminal::ForOf(t) => {
            count_memo_slots(&t.r#loop, memo_blocks, memo_values, pruned_blocks, pruned_values);
        }
        ReactiveTerminal::ForIn(t) => {
            count_memo_slots(&t.r#loop, memo_blocks, memo_values, pruned_blocks, pruned_values);
        }
        ReactiveTerminal::Label(t) => {
            count_memo_slots(&t.block, memo_blocks, memo_values, pruned_blocks, pruned_values);
        }
        ReactiveTerminal::Try(t) => {
            count_memo_slots(&t.block, memo_blocks, memo_values, pruned_blocks, pruned_values);
            count_memo_slots(&t.handler, memo_blocks, memo_values, pruned_blocks, pruned_values);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}
