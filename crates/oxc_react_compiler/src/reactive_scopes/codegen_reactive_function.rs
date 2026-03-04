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
use oxc_allocator::{CloneIn, Vec as AVec};
use oxc_ast::AstBuilder;
use oxc_ast::NONE;
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_syntax::operator::{
    AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
};
use rustc_hash::{FxHashMap, FxHashSet};
use sha2::Sha256;

use crate::{
    compiler_error::{CompilerError, SourceLocation},
    hir::{
        ArrayExpressionElement as HirArrayExpressionElement,
        ArrayPatternElement, CallArg, DeclarationId,
        FunctionExpressionType, HIRFunction, IdentifierId,
        IdentifierName as HirIdentifierName, InstructionKind,
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
// CodegenOutput — the final output of code generation
// =====================================================================================

/// Result of code generation — produces oxc_ast nodes directly.
#[derive(Debug)]
pub struct CodegenOutput<'a> {
    pub id: Option<String>,
    pub name_hint: Option<String>,
    pub params: Vec<String>,
    pub generator: bool,
    pub is_async: bool,
    pub loc: SourceLocation,
    pub memo_slots_used: u32,
    pub memo_blocks: u32,
    pub memo_values: u32,
    pub pruned_memo_blocks: u32,
    pub pruned_memo_values: u32,
    pub body: AVec<'a, Statement<'a>>,
    pub directives: Vec<String>,
    pub outlined: Vec<OutlinedOutput<'a>>,
}

#[derive(Debug)]
pub struct OutlinedOutput<'a> {
    pub fn_: CodegenOutput<'a>,
    pub fn_type: Option<crate::hir::ReactFunctionType>,
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
pub struct CodegenContext<'a> {
    /// The AST builder (Copy — wraps `&'a Allocator`).
    pub ast: AstBuilder<'a>,
    /// Next cache slot index to allocate.
    pub next_cache_index: u32,
    /// Tracks which declarations have been emitted, keyed by DeclarationId.
    declarations: FxHashSet<DeclarationId>,
    /// Maps temporary DeclarationId to expression (or None if declared but no value yet).
    pub temp: FxHashMap<DeclarationId, Option<Expression<'a>>>,
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
    /// Structured `CodegenOutput` data for function expression temporaries
    /// that have `FunctionDeclaration` type.
    fn_decl_data: FxHashMap<DeclarationId, CodegenOutput<'a>>,
    /// Accumulated invariant errors during codegen.
    codegen_errors: std::cell::RefCell<Vec<CompilerError>>,
}

impl<'a> CodegenContext<'a> {
    fn new(
        ast: AstBuilder<'a>,
        unique_identifiers: FxHashSet<String>,
        fbt_operands: FxHashSet<IdentifierId>,
        enable_emit_hook_guards: Option<ExternalFunction>,
        output_mode: CompilerOutputMode,
        shapes: ShapeRegistry,
        enable_name_anonymous_functions: bool,
    ) -> Self {
        Self {
            ast,
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
            fn_decl_data: FxHashMap::default(),
            codegen_errors: std::cell::RefCell::new(Vec::new()),
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
// AST helper functions
// =====================================================================================

/// Create an identifier reference expression.
fn make_id<'a>(cx: &CodegenContext<'a>, name: &str) -> Expression<'a> {
    cx.ast.expression_identifier(SPAN, cx.ast.atom(name))
}

/// Create a string literal expression.
fn make_string<'a>(cx: &CodegenContext<'a>, value: &str) -> Expression<'a> {
    cx.ast.expression_string_literal(SPAN, cx.ast.atom(value), None)
}

/// Create a numeric literal expression.
fn make_number<'a>(cx: &CodegenContext<'a>, value: f64) -> Expression<'a> {
    cx.ast.expression_numeric_literal(SPAN, value, None, NumberBase::Decimal)
}

/// Create a boolean literal expression.
fn make_bool<'a>(cx: &CodegenContext<'a>, value: bool) -> Expression<'a> {
    cx.ast.expression_boolean_literal(SPAN, value)
}

/// Create a null literal expression.
fn make_null<'a>(cx: &CodegenContext<'a>) -> Expression<'a> {
    cx.ast.expression_null_literal(SPAN)
}

/// Create undefined (identifier reference to "undefined").
fn make_undefined<'a>(cx: &CodegenContext<'a>) -> Expression<'a> {
    make_id(cx, "undefined")
}

/// Create an expression statement.
fn make_expr_stmt<'a>(cx: &CodegenContext<'a>, expr: Expression<'a>) -> Statement<'a> {
    cx.ast.statement_expression(SPAN, expr)
}

/// Create a variable declaration statement.
fn make_var_decl<'a>(
    cx: &CodegenContext<'a>,
    kind: VariableDeclarationKind,
    name: &str,
    init: Option<Expression<'a>>,
) -> Statement<'a> {
    let binding = cx.ast.binding_pattern_binding_identifier(SPAN, cx.ast.atom(name));
    let declarator = cx.ast.variable_declarator(SPAN, kind, binding, NONE, init, false);
    let declarators = cx.ast.vec1(declarator);
    let decl = cx.ast.variable_declaration(SPAN, kind, declarators, false);
    Statement::VariableDeclaration(cx.ast.alloc(decl))
}

/// Create a return statement.
fn make_return<'a>(cx: &CodegenContext<'a>, value: Option<Expression<'a>>) -> Statement<'a> {
    cx.ast.statement_return(SPAN, value)
}

/// Create a member expression: `object.property`
fn make_member<'a>(
    cx: &CodegenContext<'a>,
    object: Expression<'a>,
    property: &str,
) -> Expression<'a> {
    let prop = cx.ast.identifier_name(SPAN, cx.ast.atom(property));
    Expression::from(cx.ast.member_expression_static(SPAN, object, prop, false))
}

/// Create a computed member: `object[property]`
fn make_computed_member<'a>(
    cx: &CodegenContext<'a>,
    object: Expression<'a>,
    property: Expression<'a>,
) -> Expression<'a> {
    Expression::from(cx.ast.member_expression_computed(SPAN, object, property, false))
}

/// Create a call expression.
fn make_call<'a>(
    cx: &CodegenContext<'a>,
    callee: Expression<'a>,
    args: AVec<'a, Argument<'a>>,
) -> Expression<'a> {
    cx.ast.expression_call(SPAN, callee, NONE, args, false)
}

/// Create an assignment expression: `left = right`
fn make_assignment<'a>(
    cx: &CodegenContext<'a>,
    target: AssignmentTarget<'a>,
    value: Expression<'a>,
) -> Expression<'a> {
    cx.ast.expression_assignment(SPAN, AssignmentOperator::Assign, target, value)
}

/// Create a simple assignment target from a name.
fn make_simple_target<'a>(cx: &CodegenContext<'a>, name: &str) -> AssignmentTarget<'a> {
    let target =
        cx.ast.simple_assignment_target_assignment_target_identifier(SPAN, cx.ast.atom(name));
    AssignmentTarget::from(target)
}

/// Create an assignment target from a static member expression: `object.property`
fn make_member_assignment_target<'a>(
    cx: &CodegenContext<'a>,
    object: Expression<'a>,
    property: &str,
) -> AssignmentTarget<'a> {
    let prop = cx.ast.identifier_name(SPAN, cx.ast.atom(property));
    let member = cx.ast.alloc_static_member_expression(SPAN, object, prop, false);
    AssignmentTarget::from(SimpleAssignmentTarget::StaticMemberExpression(member))
}

/// Create an assignment target from a computed member expression: `object[property]`
fn make_computed_member_assignment_target<'a>(
    cx: &CodegenContext<'a>,
    object: Expression<'a>,
    property: Expression<'a>,
) -> AssignmentTarget<'a> {
    let member = cx.ast.alloc_computed_member_expression(SPAN, object, property, false);
    AssignmentTarget::from(SimpleAssignmentTarget::ComputedMemberExpression(member))
}

/// Create an assignment target from a property literal (string or number).
fn make_property_assignment_target<'a>(
    cx: &CodegenContext<'a>,
    object: Expression<'a>,
    property: &crate::hir::types::PropertyLiteral,
) -> AssignmentTarget<'a> {
    match property {
        crate::hir::types::PropertyLiteral::String(name) => {
            make_member_assignment_target(cx, object, name)
        }
        crate::hir::types::PropertyLiteral::Number(n) => {
            make_computed_member_assignment_target(cx, object, make_number(cx, *n as f64))
        }
    }
}

/// Create a `SimpleAssignmentTarget` from an identifier name (for update expressions).
fn make_simple_assignment_target_id<'a>(
    cx: &CodegenContext<'a>,
    name: &str,
) -> SimpleAssignmentTarget<'a> {
    cx.ast.simple_assignment_target_assignment_target_identifier(SPAN, cx.ast.atom(name))
}

/// Create a binary expression.
fn make_binary<'a>(
    cx: &CodegenContext<'a>,
    left: Expression<'a>,
    op: BinaryOperator,
    right: Expression<'a>,
) -> Expression<'a> {
    cx.ast.expression_binary(SPAN, left, op, right)
}

/// Create a logical expression.
fn make_logical<'a>(
    cx: &CodegenContext<'a>,
    left: Expression<'a>,
    op: LogicalOperator,
    right: Expression<'a>,
) -> Expression<'a> {
    cx.ast.expression_logical(SPAN, left, op, right)
}

/// Create a unary expression.
fn make_unary<'a>(
    cx: &CodegenContext<'a>,
    op: UnaryOperator,
    operand: Expression<'a>,
) -> Expression<'a> {
    cx.ast.expression_unary(SPAN, op, operand)
}

/// Create a sequence expression.
fn make_sequence<'a>(
    cx: &CodegenContext<'a>,
    expressions: AVec<'a, Expression<'a>>,
) -> Expression<'a> {
    cx.ast.expression_sequence(SPAN, expressions)
}

/// Clone an expression from the temp map (arena-allocated copy).
fn clone_expr<'a>(cx: &CodegenContext<'a>, expr: &Expression<'a>) -> Expression<'a> {
    expr.clone_in(cx.ast.allocator)
}

/// Resolve a temp map entry — clone the expression if present.
fn resolve_temp<'a>(cx: &CodegenContext<'a>, decl_id: DeclarationId) -> Option<Expression<'a>> {
    cx.temp.get(&decl_id).and_then(|opt| opt.as_ref().map(|expr| clone_expr(cx, expr)))
}

/// Clone the temp map (deep-cloning all expressions into the arena).
fn clone_temp_map<'a>(
    cx: &CodegenContext<'a>,
) -> FxHashMap<DeclarationId, Option<Expression<'a>>> {
    cx.temp
        .iter()
        .map(|(k, v)| (*k, v.as_ref().map(|expr| expr.clone_in(cx.ast.allocator))))
        .collect()
}

/// Convert statements into a boxed block statement.
fn stmts_to_block_body<'a>(
    cx: &CodegenContext<'a>,
    stmts: AVec<'a, Statement<'a>>,
) -> oxc_allocator::Box<'a, BlockStatement<'a>> {
    cx.ast.alloc_block_statement(SPAN, stmts)
}

// =====================================================================================
// Entry point: codegen_function
// =====================================================================================

/// Generate code from a reactive function.
///
/// # Errors
/// Returns a `CompilerError` if code generation fails.
pub fn codegen_function<'a>(
    reactive_fn: &ReactiveFunction,
    options: CodegenOptions,
    ast: AstBuilder<'a>,
) -> Result<CodegenOutput<'a>, CompilerError> {
    let mut cx = CodegenContext::new(
        ast,
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
    if let Some(Statement::ReturnStatement(ret)) = body.last() {
        if ret.argument.is_none() {
            body.pop();
        }
    }

    // TODO: Function-level hook guard: wrap the entire body in a try-finally with
    // PushHookGuard/PopHookGuard if enableEmitHookGuards is set and output mode is client.

    // Count memo blocks/values
    let mut counter = MemoCounter::default();
    visit_reactive_block(&reactive_fn.body, &mut counter);

    // TODO: Insert the `const $ = _c(N);` preamble if there are cache slots
    // TODO: Emit HMR/Fast Refresh hash check and cache reset
    // TODO: Emit instrument forget

    let cache_count = cx.next_cache_index;

    let params = convert_params(&reactive_fn.params);

    // Check for invariant errors accumulated during codegen.
    if let Some(first_error) = cx.codegen_errors.into_inner().into_iter().next() {
        return Err(first_error);
    }

    Ok(CodegenOutput {
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

/// Run the sub-pipeline on an inner HIR function and produce a `CodegenOutput`.
fn codegen_inner_function<'a>(
    func: &HIRFunction,
    cx: &CodegenContext<'a>,
    include_prune_hoisted: bool,
) -> Result<CodegenOutput<'a>, CompilerError> {
    let mut reactive_fn =
        crate::reactive_scopes::build_reactive_function::build_reactive_function(func)?;
    crate::reactive_scopes::prune_unused_labels::prune_unused_labels(&mut reactive_fn);
    crate::reactive_scopes::prune_unused_lvalues::prune_unused_lvalues(&mut reactive_fn);
    if include_prune_hoisted {
        crate::reactive_scopes::prune::prune_hoisted_contexts(&mut reactive_fn)?;
    }
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
    codegen_function(&reactive_fn, options, cx.ast)
}

/// State for HMR/Fast Refresh cache reset.
struct FastRefreshState {
    /// The cache index allocated for tracking the source hash.
    cache_index: u32,
    /// The hex-encoded HMAC-SHA256 hash of the source code.
    hash: String,
}

/// Compute an HMAC-SHA256 hash of the source code.
fn compute_source_hash(code: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
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
fn create_hook_guard<'a>(
    cx: &CodegenContext<'a>,
    guard_fn_name: &str,
    stmts: AVec<'a, Statement<'a>>,
    before: GuardKind,
    after: GuardKind,
) -> Statement<'a> {
    // Build: guardFn(before)
    let before_args = cx.ast.vec1(Argument::from(make_number(cx, before as u8 as f64)));
    let before_call = make_call(cx, make_id(cx, guard_fn_name), before_args);
    let before_stmt = make_expr_stmt(cx, before_call);

    // Build try block: { guardFn(before); ...stmts }
    let mut try_stmts = cx.ast.vec_with_capacity(1 + stmts.len());
    try_stmts.push(before_stmt);
    try_stmts.extend(stmts);
    let try_block = stmts_to_block_body(cx, try_stmts);

    // Build: guardFn(after)
    let after_args = cx.ast.vec1(Argument::from(make_number(cx, after as u8 as f64)));
    let after_call = make_call(cx, make_id(cx, guard_fn_name), after_args);
    let after_stmt = make_expr_stmt(cx, after_call);

    // Build finally block: { guardFn(after); }
    let finally_stmts = cx.ast.vec1(after_stmt);
    let finally_block = stmts_to_block_body(cx, finally_stmts);

    // Build try-finally statement (no catch handler)
    cx.ast.statement_try(
        SPAN,
        try_block,
        None::<oxc_allocator::Box<'_, CatchClause<'_>>>,
        Some(finally_block),
    )
}

/// Check if an identifier represents a hook by looking up its type in the shape registry.
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

/// Create a call expression, optionally wrapping hook calls in an IIFE with
/// try-finally hook guards.
///
/// When hook guards are enabled and this is a hook call, wraps like:
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
fn create_call_expression<'a>(
    cx: &mut CodegenContext<'a>,
    callee: Expression<'a>,
    args: AVec<'a, Argument<'a>>,
    is_hook: bool,
) -> Expression<'a> {
    let call = make_call(cx, callee, args);

    // If no hook guard needed, return the call directly
    let Some(ref guard_fn) = cx.enable_emit_hook_guards else {
        return call;
    };
    if !is_hook {
        return call;
    }

    // Hook guard wrapping: (() => { try { guardFn(2); return callExpr; } finally { guardFn(3); } })()
    let guard_name = cx.synthesize_name(&guard_fn.import_specifier_name.clone());

    // Build: guardFn(2)
    let before_args =
        cx.ast.vec1(Argument::from(make_number(cx, GuardKind::AllowHook as u8 as f64)));
    let before_call = make_call(cx, make_id(cx, &guard_name), before_args);
    let before_stmt = make_expr_stmt(cx, before_call);

    // Build: return callExpr
    let return_stmt = make_return(cx, Some(call));

    // Build try block: { guardFn(2); return callExpr; }
    let try_stmts = cx.ast.vec_from_array([before_stmt, return_stmt]);
    let try_block = stmts_to_block_body(cx, try_stmts);

    // Build: guardFn(3)
    let after_args =
        cx.ast.vec1(Argument::from(make_number(cx, GuardKind::DisallowHook as u8 as f64)));
    let after_call = make_call(cx, make_id(cx, &guard_name), after_args);
    let after_stmt = make_expr_stmt(cx, after_call);

    // Build finally block: { guardFn(3); }
    let finally_stmts = cx.ast.vec1(after_stmt);
    let finally_block = stmts_to_block_body(cx, finally_stmts);

    // Build try-finally statement
    let try_stmt = cx.ast.statement_try(
        SPAN,
        try_block,
        None::<oxc_allocator::Box<'_, CatchClause<'_>>>,
        Some(finally_block),
    );

    // Build arrow function body
    let arrow_body_stmts = cx.ast.vec1(try_stmt);
    let arrow_body = cx.ast.alloc_function_body(SPAN, cx.ast.vec(), arrow_body_stmts);

    // Build arrow: () => { try { ... } finally { ... } }
    let arrow = cx.ast.expression_arrow_function(
        SPAN,
        false,
        false,
        NONE,
        cx.ast.formal_parameters(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            cx.ast.vec(),
            NONE,
        ),
        NONE,
        arrow_body,
    );

    // Build IIFE: (() => { ... })()
    let wrapped = cx.ast.expression_parenthesized(SPAN, arrow);
    make_call(cx, wrapped, cx.ast.vec())
}

// =====================================================================================
// codegen_block — the central dispatch
// =====================================================================================

/// Generate code for a reactive block. Saves and restores temporaries so that
/// temporaries defined inside a block do not leak out to the parent scope.
fn codegen_block<'a>(
    cx: &mut CodegenContext<'a>,
    block: &ReactiveBlock,
) -> AVec<'a, Statement<'a>> {
    let saved_temp = clone_temp_map(cx);
    let result = codegen_block_no_reset(cx, block);
    cx.temp = saved_temp;
    result
}

/// Generate code for a reactive block without resetting temporaries.
fn codegen_block_no_reset<'a>(
    cx: &mut CodegenContext<'a>,
    block: &ReactiveBlock,
) -> AVec<'a, Statement<'a>> {
    let mut statements: AVec<'a, Statement<'a>> = cx.ast.vec();

    for item in block {
        match item {
            ReactiveStatement::Instruction(instr_stmt) => {
                codegen_instruction_nullable(cx, &instr_stmt.instruction, &mut statements);
            }
            ReactiveStatement::PrunedScope(pruned) => {
                // Pruned scopes: just emit the instructions without memoization
                let scope_stmts = codegen_block_no_reset(cx, &pruned.instructions);
                statements.extend(scope_stmts);
            }
            ReactiveStatement::Scope(scope_block) => {
                let saved_temp = clone_temp_map(cx);
                codegen_reactive_scope(
                    cx,
                    &mut statements,
                    &scope_block.scope,
                    &scope_block.instructions,
                );
                cx.temp = saved_temp;
            }
            ReactiveStatement::Terminal(term_stmt) => {
                codegen_terminal(cx, &term_stmt.terminal, &mut statements);
                // TODO: handle labels (implicit/explicit)
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
fn codegen_reactive_scope<'a>(
    cx: &mut CodegenContext<'a>,
    statements: &mut AVec<'a, Statement<'a>>,
    scope: &ReactiveScope,
    block: &ReactiveBlock,
) {
    todo!()
}

// =====================================================================================
// codegen_terminal — generates code for reactive terminals
// =====================================================================================

/// Generate code for a reactive terminal. Pushes statements directly.
fn codegen_terminal<'a>(
    cx: &mut CodegenContext<'a>,
    terminal: &ReactiveTerminal,
    stmts: &mut AVec<'a, Statement<'a>>,
) {
    todo!()
}

fn codegen_break<'a>(
    cx: &CodegenContext<'a>,
    t: &ReactiveBreakTerminal,
) -> Option<Statement<'a>> {
    todo!()
}

fn codegen_continue<'a>(
    cx: &CodegenContext<'a>,
    t: &ReactiveContinueTerminal,
) -> Option<Statement<'a>> {
    todo!()
}

// =====================================================================================
// codegen_instruction_nullable — instruction to statement (may push or not)
// =====================================================================================

/// Generate code for a reactive instruction. Pushes to `stmts` if the instruction
/// produces a statement; does nothing if the instruction is suppressed.
fn codegen_instruction_nullable<'a>(
    cx: &mut CodegenContext<'a>,
    instr: &ReactiveInstruction,
    stmts: &mut AVec<'a, Statement<'a>>,
) {
    todo!()
}

/// Handle StoreLocal/StoreContext/DeclareLocal/DeclareContext
fn codegen_store_or_declare<'a>(
    cx: &mut CodegenContext<'a>,
    instr: &ReactiveInstruction,
    kind: InstructionKind,
    lvalue_place: &Place,
    value: Option<Expression<'a>>,
    value_place: Option<&Place>,
    stmts: &mut AVec<'a, Statement<'a>>,
) {
    todo!()
}

/// Handle destructure statements.
fn codegen_destructure_statement<'a>(
    cx: &mut CodegenContext<'a>,
    instr: &ReactiveInstruction,
    kind: InstructionKind,
    lval: BindingPattern<'a>,
    value: Expression<'a>,
    stmts: &mut AVec<'a, Statement<'a>>,
) {
    todo!()
}

/// Store an expression as a temporary if the lvalue is unnamed, otherwise emit as expression statement.
fn try_store_as_temporary<'a>(
    cx: &mut CodegenContext<'a>,
    lvalue: Option<&Place>,
    expr: Expression<'a>,
    stmts: &mut AVec<'a, Statement<'a>>,
) {
    todo!()
}

/// Convert a codegen value into a statement, handling temporaries and named lvalues.
fn codegen_instruction_to_statement<'a>(
    cx: &mut CodegenContext<'a>,
    instr: &ReactiveInstruction,
    value: Expression<'a>,
    stmts: &mut AVec<'a, Statement<'a>>,
) {
    todo!()
}

// =====================================================================================
// codegen_instruction_value — InstructionValue → Expression
// =====================================================================================

/// Generate an expression from an InstructionValue.
fn codegen_instruction_value<'a>(
    cx: &mut CodegenContext<'a>,
    value: &InstructionValue,
) -> Expression<'a> {
    match value {
        InstructionValue::ArrayExpression(arr) => {
            let mut elements = cx.ast.vec_with_capacity(arr.elements.len());
            for elem in &arr.elements {
                match elem {
                    HirArrayExpressionElement::Place(p) => {
                        let expr = codegen_place_to_expression(cx, p);
                        elements.push(ArrayExpressionElement::from(expr));
                    }
                    HirArrayExpressionElement::Spread(s) => {
                        let expr = codegen_place_to_expression(cx, &s.place);
                        elements.push(
                            cx.ast.array_expression_element_spread_element(SPAN, expr),
                        );
                    }
                    HirArrayExpressionElement::Hole => {
                        elements.push(cx.ast.array_expression_element_elision(SPAN));
                    }
                }
            }
            cx.ast.expression_array(SPAN, elements)
        }
        InstructionValue::BinaryExpression(bin) => {
            let left = codegen_place_to_expression(cx, &bin.left);
            let right = codegen_place_to_expression(cx, &bin.right);
            make_binary(cx, left, bin.operator, right)
        }
        InstructionValue::UnaryExpression(unary) => {
            let operand = codegen_place_to_expression(cx, &unary.value);
            make_unary(cx, unary.operator, operand)
        }
        InstructionValue::Primitive(prim) => codegen_primitive(cx, &prim.value),
        InstructionValue::JsxText(text) => make_string(cx, &text.value),
        InstructionValue::CallExpression(call) => {
            let is_hook = get_hook_kind(&cx.shapes, &call.callee.identifier);
            let callee = codegen_place_to_expression(cx, &call.callee);
            let args = codegen_args(cx, &call.args);
            create_call_expression(cx, callee, args, is_hook)
        }
        InstructionValue::MethodCall(method) => {
            let is_hook = get_hook_kind(&cx.shapes, &method.property.identifier);
            let callee = codegen_place_to_expression(cx, &method.property);
            let property_decl_id = method.property.identifier.declaration_id;
            let is_member_expr = cx.temp.get(&property_decl_id).is_some_and(|v| v.is_some());
            if !is_member_expr {
                cx.codegen_errors.borrow_mut().push(CompilerError::invariant(
                    "[Codegen] Internal error: MethodCall::property must be an unpromoted + unmemoized MemberExpression",
                    Some("Got: 'Identifier'"),
                    method.loc,
                ));
            }
            let args = codegen_args(cx, &method.args);
            create_call_expression(cx, callee, args, is_hook)
        }
        InstructionValue::NewExpression(new) => {
            let callee = codegen_place_to_expression(cx, &new.callee);
            let args = codegen_args(cx, &new.args);
            cx.ast.expression_new(SPAN, callee, NONE, args)
        }
        InstructionValue::ObjectExpression(obj) => codegen_object_expression(cx, obj),
        InstructionValue::PropertyLoad(load) => {
            let object = codegen_place_to_expression(cx, &load.object);
            codegen_member_access(cx, object, &load.property)
        }
        InstructionValue::PropertyStore(store) => {
            let object = codegen_place_to_expression(cx, &store.object);
            let target = make_property_assignment_target(cx, object, &store.property);
            let value = codegen_place_to_expression(cx, &store.value);
            make_assignment(cx, target, value)
        }
        InstructionValue::PropertyDelete(del) => {
            let object = codegen_place_to_expression(cx, &del.object);
            let member = codegen_member_access(cx, object, &del.property);
            make_unary(cx, UnaryOperator::Delete, member)
        }
        InstructionValue::ComputedLoad(load) => {
            let object = codegen_place_to_expression(cx, &load.object);
            let property = codegen_place_to_expression(cx, &load.property);
            make_computed_member(cx, object, property)
        }
        InstructionValue::ComputedStore(store) => {
            let object = codegen_place_to_expression(cx, &store.object);
            let property = codegen_place_to_expression(cx, &store.property);
            let target = make_computed_member_assignment_target(cx, object, property);
            let value = codegen_place_to_expression(cx, &store.value);
            make_assignment(cx, target, value)
        }
        InstructionValue::ComputedDelete(del) => {
            let object = codegen_place_to_expression(cx, &del.object);
            let property = codegen_place_to_expression(cx, &del.property);
            let computed = make_computed_member(cx, object, property);
            make_unary(cx, UnaryOperator::Delete, computed)
        }
        InstructionValue::LoadLocal(load) => codegen_place_to_expression(cx, &load.place),
        InstructionValue::LoadContext(load) => codegen_place_to_expression(cx, &load.place),
        InstructionValue::LoadGlobal(load) => make_id(cx, &load.binding.name()),
        InstructionValue::StoreGlobal(store) => {
            let value = codegen_place_to_expression(cx, &store.value);
            make_assignment(cx, make_simple_target(cx, &store.name), value)
        }
        InstructionValue::FunctionExpression(func_expr) => {
            codegen_function_expression(cx, func_expr).0
        }
        InstructionValue::RegExpLiteral(re) => {
            let mut flags = RegExpFlags::empty();
            for c in re.flags.chars() {
                if let Ok(f) = RegExpFlags::try_from(c) {
                    flags |= f;
                }
            }
            let pattern = RegExpPattern {
                text: cx.ast.atom(&re.pattern),
                pattern: None,
            };
            let regex = RegExp { pattern, flags };
            let raw = cx.ast.atom(&format!("/{}/{}", re.pattern, re.flags));
            cx.ast.expression_reg_exp_literal(SPAN, regex, Some(raw))
        }
        InstructionValue::TemplateLiteral(tmpl) => {
            let mut quasis = cx.ast.vec_with_capacity(tmpl.quasis.len());
            let mut expressions = cx.ast.vec_with_capacity(tmpl.subexprs.len());
            for (i, quasi) in tmpl.quasis.iter().enumerate() {
                let tail = i == tmpl.quasis.len() - 1;
                let value = TemplateElementValue {
                    raw: cx.ast.atom(&quasi.raw),
                    cooked: quasi.cooked.as_ref().map(|c| cx.ast.atom(c.as_str())),
                };
                quasis.push(cx.ast.template_element(SPAN, value, tail, false));
                if i < tmpl.subexprs.len() {
                    expressions.push(codegen_place_to_expression(cx, &tmpl.subexprs[i]));
                }
            }
            cx.ast.expression_template_literal(SPAN, quasis, expressions)
        }
        InstructionValue::TaggedTemplateExpression(tagged) => {
            let tag = codegen_place_to_expression(cx, &tagged.tag);
            let value = TemplateElementValue {
                raw: cx.ast.atom(&tagged.value.raw),
                cooked: tagged.value.cooked.as_ref().map(|c| cx.ast.atom(c.as_str())),
            };
            let quasi_elem = cx.ast.template_element(SPAN, value, true, false);
            let quasis = cx.ast.vec1(quasi_elem);
            let expressions = cx.ast.vec();
            let quasi = cx.ast.template_literal(SPAN, quasis, expressions);
            cx.ast.expression_tagged_template(SPAN, tag, NONE, quasi)
        }
        InstructionValue::TypeCastExpression(cast) => {
            codegen_place_to_expression(cx, &cast.value)
        }
        InstructionValue::JsxExpression(jsx) => {
            let tag_name = codegen_jsx_tag_to_element_name(cx, &jsx.tag);
            let mut attrs = cx.ast.vec_with_capacity(jsx.props.len());
            for attr in &jsx.props {
                attrs.push(codegen_jsx_attribute(cx, attr));
            }
            let opening = cx.ast.jsx_opening_element(SPAN, tag_name, NONE, attrs);
            match &jsx.children {
                None => {
                    // Self-closing: <Tag ... />
                    let children = cx.ast.vec();
                    cx.ast.expression_jsx_element(SPAN, opening, children, NONE)
                }
                Some(children) => {
                    let mut child_nodes = cx.ast.vec_with_capacity(children.len());
                    for c in children {
                        child_nodes.push(codegen_jsx_child(cx, c));
                    }
                    let closing_name = codegen_jsx_tag_to_element_name(cx, &jsx.tag);
                    let closing = cx.ast.jsx_closing_element(SPAN, closing_name);
                    cx.ast.expression_jsx_element(SPAN, opening, child_nodes, Some(closing))
                }
            }
        }
        InstructionValue::JsxFragment(frag) => {
            let opening = cx.ast.jsx_opening_fragment(SPAN);
            let mut children = cx.ast.vec_with_capacity(frag.children.len());
            for c in &frag.children {
                children.push(codegen_jsx_child(cx, c));
            }
            let closing = cx.ast.jsx_closing_fragment(SPAN);
            cx.ast.expression_jsx_fragment(SPAN, opening, children, closing)
        }
        InstructionValue::GetIterator(iter) => {
            codegen_place_to_expression(cx, &iter.collection)
        }
        InstructionValue::IteratorNext(iter) => {
            codegen_place_to_expression(cx, &iter.iterator)
        }
        InstructionValue::NextPropertyOf(next) => {
            codegen_place_to_expression(cx, &next.value)
        }
        InstructionValue::PrefixUpdate(update) => {
            let name = identifier_name(&update.lvalue.identifier);
            let target = make_simple_assignment_target_id(cx, &name);
            cx.ast.expression_update(SPAN, update.operation, true, target)
        }
        InstructionValue::PostfixUpdate(update) => {
            let name = identifier_name(&update.lvalue.identifier);
            let target = make_simple_assignment_target_id(cx, &name);
            cx.ast.expression_update(SPAN, update.operation, false, target)
        }
        InstructionValue::Await(aw) => {
            let value = codegen_place_to_expression(cx, &aw.value);
            cx.ast.expression_await(SPAN, value)
        }
        InstructionValue::MetaProperty(meta) => {
            let meta_ident = cx.ast.identifier_name(SPAN, cx.ast.atom(&meta.meta));
            let property_ident = cx.ast.identifier_name(SPAN, cx.ast.atom(&meta.property));
            cx.ast.expression_meta_property(SPAN, meta_ident, property_ident)
        }
        // StoreLocal in expression context: assignment expression
        InstructionValue::StoreLocal(store) => {
            let lval_name = identifier_name(&store.lvalue.place.identifier);
            let target = make_simple_target(cx, &lval_name);
            let value = codegen_place_to_expression(cx, &store.value);
            make_assignment(cx, target, value)
        }
        // These are handled in codegen_instruction_nullable, not here.
        InstructionValue::StoreContext(_)
        | InstructionValue::DeclareLocal(_)
        | InstructionValue::DeclareContext(_)
        | InstructionValue::Destructure(_)
        | InstructionValue::StartMemoize(_)
        | InstructionValue::FinishMemoize(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::ObjectMethod(_) => make_undefined(cx),
        InstructionValue::UnsupportedNode(_) => {
            make_string(cx, "/* unsupported */")
        }
    }
}

/// Convert a JSX tag to a `JSXElementName`.
fn codegen_jsx_tag_to_element_name<'a>(
    cx: &CodegenContext<'a>,
    tag: &JsxTag,
) -> JSXElementName<'a> {
    match tag {
        JsxTag::BuiltIn(builtin) => {
            // Built-in HTML tags (lowercase) use JSXIdentifier
            cx.ast.jsx_element_name_identifier(SPAN, cx.ast.atom(&builtin.name))
        }
        JsxTag::Place(place) => {
            // Component tags resolve through the temp map to expressions.
            // Convert the expression to a JSXElementName.
            let name = identifier_name(&place.identifier);
            // Check if this is a member expression stored in temp
            if let Some(Some(expr)) = cx.temp.get(&place.identifier.declaration_id) {
                return expression_to_jsx_element_name(cx, expr);
            }
            // Simple identifier — use IdentifierReference for component names
            cx.ast.jsx_element_name_identifier_reference(SPAN, cx.ast.atom(&name))
        }
    }
}

/// Convert an `Expression` to a `JSXElementName`.
/// Handles identifier references and static member expressions (e.g., `Foo.Bar`).
fn expression_to_jsx_element_name<'a>(
    cx: &CodegenContext<'a>,
    expr: &Expression<'a>,
) -> JSXElementName<'a> {
    match expr {
        Expression::Identifier(ident) => {
            cx.ast.jsx_element_name_identifier_reference(SPAN, cx.ast.atom(&ident.name))
        }
        Expression::StaticMemberExpression(member) => {
            let property = cx.ast.jsx_identifier(SPAN, cx.ast.atom(&member.property.name));
            let object = expression_to_jsx_member_object(cx, &member.object);
            let jsx_member = cx.ast.alloc(cx.ast.jsx_member_expression(SPAN, object, property));
            JSXElementName::MemberExpression(jsx_member)
        }
        _ => {
            // Fallback: use the expression as an identifier name.
            // This shouldn't normally happen in well-formed code.
            cx.ast.jsx_element_name_identifier(SPAN, cx.ast.atom("unknown"))
        }
    }
}

/// Convert an `Expression` to a `JSXMemberExpressionObject`.
fn expression_to_jsx_member_object<'a>(
    cx: &CodegenContext<'a>,
    expr: &Expression<'a>,
) -> JSXMemberExpressionObject<'a> {
    match expr {
        Expression::Identifier(ident) => {
            let id_ref = cx.ast.alloc_identifier_reference(SPAN, cx.ast.atom(&ident.name));
            JSXMemberExpressionObject::IdentifierReference(id_ref)
        }
        Expression::StaticMemberExpression(member) => {
            let property = cx.ast.jsx_identifier(SPAN, cx.ast.atom(&member.property.name));
            let object = expression_to_jsx_member_object(cx, &member.object);
            let jsx_member = cx.ast.alloc(cx.ast.jsx_member_expression(SPAN, object, property));
            JSXMemberExpressionObject::MemberExpression(jsx_member)
        }
        Expression::ThisExpression(_) => {
            let this = cx.ast.alloc_this_expression(SPAN);
            JSXMemberExpressionObject::ThisExpression(this)
        }
        _ => {
            // Fallback
            let id_ref = cx.ast.alloc_identifier_reference(SPAN, cx.ast.atom("unknown"));
            JSXMemberExpressionObject::IdentifierReference(id_ref)
        }
    }
}

// =====================================================================================
// codegen_function_expression — recursive codegen for FunctionExpression
// =====================================================================================

/// Generate a function expression by recursively compiling the inner function.
///
/// Returns the expression and, for `FunctionDeclaration` types, the structured
/// `CodegenOutput` data needed to emit a proper function declaration statement.
fn codegen_function_expression<'a>(
    cx: &CodegenContext<'a>,
    func_expr: &crate::hir::FunctionExpressionValue,
) -> (Expression<'a>, Option<CodegenOutput<'a>>) {
    todo!()
}

/// Generate an object expression.
fn codegen_object_expression<'a>(
    cx: &CodegenContext<'a>,
    obj: &ObjectExpression,
) -> Expression<'a> {
    todo!()
}

/// Generate an object method expression by recursively compiling the inner function.
fn codegen_object_method_expression<'a>(
    cx: &CodegenContext<'a>,
    method: &ObjectMethodValue,
    key: PropertyKey<'a>,
    is_computed: bool,
) -> Expression<'a> {
    todo!()
}

// =====================================================================================
// codegen_reactive_value_to_expression — ReactiveValue → Expression
// =====================================================================================

/// Convert a `ReactiveValue` to an expression.
fn codegen_reactive_value_to_expression<'a>(
    cx: &mut CodegenContext<'a>,
    value: &ReactiveValue,
) -> Expression<'a> {
    todo!()
}

// =====================================================================================
// Helper functions
// =====================================================================================

/// Join multi-child JSX children with appropriate newline separators.
fn join_jsx_children_multiline(children: &[String]) -> String {
    let mut result = String::new();
    for (i, child) in children.iter().enumerate() {
        if i > 0 {
            let prev = &children[i - 1];
            let prev_is_expr = prev.starts_with('{') || prev.starts_with('<');
            let curr_is_expr = child.starts_with('{') || child.starts_with('<');
            if prev_is_expr && curr_is_expr {
                result.push('\n');
            }
        }
        result.push_str(child);
    }
    result
}

/// Generate a JSX attribute.
fn codegen_jsx_attribute<'a>(
    cx: &CodegenContext<'a>,
    attr: &JsxAttribute,
) -> JSXAttributeItem<'a> {
    todo!()
}

/// Check if a string literal value requires wrapping in a JSX expression container.
fn string_requires_expr_container(s: &str) -> bool {
    for c in s.chars() {
        let code = c as u32;
        if code <= 0x1F || code == 0x7F || code >= 0x80 {
            return true;
        }
        if c == '"' || c == '\\' {
            return true;
        }
    }
    false
}

/// Render a JSX child element.
fn codegen_jsx_child<'a>(cx: &CodegenContext<'a>, place: &Place) -> JSXChild<'a> {
    todo!()
}

/// Convert a Place to an expression.
fn codegen_place_to_expression<'a>(
    cx: &CodegenContext<'a>,
    place: &Place,
) -> Expression<'a> {
    let decl_id = place.identifier.declaration_id;
    if let Some(expr) = resolve_temp(cx, decl_id) {
        return expr;
    }

    // TS invariant: convertIdentifier checks that the identifier has a name.
    // If unnamed, this is an invariant violation indicating the identifier
    // was not promoted by an earlier pass (PromoteUsedTemporaries).
    if place.identifier.name.is_none() {
        cx.codegen_errors.borrow_mut().push(CompilerError::invariant(
            "Expected temporaries to be promoted to named identifiers in an earlier pass",
            Some(&format!("identifier {} is unnamed.", place.identifier.id.0)),
            crate::compiler_error::GENERATED_SOURCE,
        ));
    }
    let name = identifier_name(&place.identifier);
    make_id(cx, &name)
}

/// Convert a Place to an expression WITHOUT wrapping assignment expressions.
/// With AST-based codegen, there is no string-level paren wrapping needed,
/// so this is identical to `codegen_place_to_expression`.
fn codegen_place_to_expression_raw<'a>(
    cx: &CodegenContext<'a>,
    place: &Place,
) -> Expression<'a> {
    codegen_place_to_expression(cx, place)
}

/// Get the string name of an identifier.
fn identifier_name(identifier: &crate::hir::Identifier) -> String {
    match &identifier.name {
        Some(HirIdentifierName::Named(name) | HirIdentifierName::Promoted(name)) => name.clone(),
        None => format!("t${}", identifier.id.0),
    }
}

/// Generate a sequence expression for ExpressionStatement context.
/// With AST-based codegen, parenthesization decisions are handled by oxc_codegen,
/// so this delegates directly to `codegen_reactive_value_to_expression`.
fn codegen_sequence_for_expr_stmt<'a>(
    cx: &mut CodegenContext<'a>,
    value: &ReactiveValue,
) -> Expression<'a> {
    codegen_reactive_value_to_expression(cx, value)
}

/// Generate a label string from a BlockId.
fn codegen_label(id: crate::hir::BlockId) -> String {
    format!("bb{}", id.0)
}

/// Generate a primitive value expression.
fn codegen_primitive<'a>(
    cx: &CodegenContext<'a>,
    value: &PrimitiveValueKind,
) -> Expression<'a> {
    match value {
        PrimitiveValueKind::Number(n) => {
            let n = *n;
            if n == 0.0 && n.is_sign_negative() {
                // -0 => just 0 (matching the old codegen behavior)
                make_number(cx, 0.0)
            } else if n < 0.0 {
                make_unary(cx, UnaryOperator::UnaryNegation, make_number(cx, -n))
            } else {
                make_number(cx, n)
            }
        }
        PrimitiveValueKind::Boolean(b) => make_bool(cx, *b),
        PrimitiveValueKind::String(s) => make_string(cx, s),
        PrimitiveValueKind::Null => make_null(cx),
        PrimitiveValueKind::Undefined => make_undefined(cx),
    }
}

/// Escape special characters in a double-quoted string literal.
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

/// Generate a member access expression.
fn codegen_member_access<'a>(
    cx: &CodegenContext<'a>,
    object: Expression<'a>,
    property: &crate::hir::types::PropertyLiteral,
) -> Expression<'a> {
    match property {
        crate::hir::types::PropertyLiteral::String(name) => {
            make_member(cx, object, name)
        }
        crate::hir::types::PropertyLiteral::Number(n) => {
            make_computed_member(cx, object, make_number(cx, *n as f64))
        }
    }
}

/// Check if a string is a valid JavaScript identifier name.
fn is_valid_identifier_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    if let Some(first) = chars.next() {
        if !first.is_ascii_alphabetic() && first != '_' && first != '$' {
            return false;
        }
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Generate an object property key.
fn codegen_object_property_key<'a>(
    cx: &CodegenContext<'a>,
    key: &ObjectPropertyKey,
) -> PropertyKey<'a> {
    todo!()
}

/// Generate call arguments.
fn codegen_args<'a>(
    cx: &CodegenContext<'a>,
    args: &[CallArg],
) -> AVec<'a, Argument<'a>> {
    let mut result = cx.ast.vec_with_capacity(args.len());
    for arg in args {
        match arg {
            CallArg::Place(p) => {
                result.push(Argument::from(codegen_place_to_expression(cx, p)));
            }
            CallArg::Spread(s) => {
                let expr = codegen_place_to_expression(cx, &s.place);
                let spread = cx.ast.alloc(cx.ast.spread_element(SPAN, expr));
                result.push(Argument::SpreadElement(spread));
            }
        }
    }
    result
}

/// Generate a destructure pattern.
fn codegen_pattern<'a>(
    cx: &CodegenContext<'a>,
    pattern: &Pattern,
) -> BindingPattern<'a> {
    todo!()
}

/// Iterate over all Place operands in a pattern.
fn each_pattern_operand(pattern: &Pattern) -> Vec<&Place> {
    let mut places = Vec::new();
    collect_pattern_operands(pattern, &mut places);
    places
}

fn collect_pattern_operands<'b>(pattern: &'b Pattern, places: &mut Vec<&'b Place>) {
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

/// Generate a dependency expression.
fn codegen_dependency<'a>(
    cx: &CodegenContext<'a>,
    dep: &ReactiveScopeDependency,
) -> Expression<'a> {
    let name = identifier_name(&dep.identifier);
    let mut result: Expression<'a> = make_id(cx, &name);
    for entry in &dep.path {
        match &entry.property {
            crate::hir::types::PropertyLiteral::String(name) => {
                if entry.optional {
                    // object?.property - optional member expression
                    let ident = cx.ast.identifier_name(SPAN, cx.ast.atom(name.as_str()));
                    result = Expression::from(
                        cx.ast.member_expression_static(SPAN, result, ident, true),
                    );
                } else {
                    result = make_member(cx, result, name);
                }
            }
            crate::hir::types::PropertyLiteral::Number(n) => {
                let index = make_number(cx, *n as f64);
                if entry.optional {
                    result = Expression::from(
                        cx.ast.member_expression_computed(SPAN, result, index, true),
                    );
                } else {
                    result = make_computed_member(cx, result, index);
                }
            }
        }
    }
    result
}

/// Generate the init part of a for statement.
fn codegen_for_init<'a>(
    cx: &mut CodegenContext<'a>,
    init: &ReactiveValue,
) -> Expression<'a> {
    todo!()
}

/// Extract left and right for for-of from reactive init/test values.
fn codegen_for_of_in_init<'a>(
    cx: &mut CodegenContext<'a>,
    init: &ReactiveValue,
    test: &ReactiveValue,
) -> (VariableDeclarationKind, BindingPattern<'a>) {
    todo!()
}

/// Extract the collection expression for for-of from the init value.
fn codegen_for_of_collection<'a>(
    cx: &mut CodegenContext<'a>,
    init: &ReactiveValue,
) -> Expression<'a> {
    if let ReactiveValue::Sequence(seq) = init
        && let Some(first) = seq.instructions.first()
        && let ReactiveValue::Instruction(boxed) = &first.value
        && let InstructionValue::GetIterator(iter) = boxed.as_ref()
    {
        return codegen_place_to_expression(cx, &iter.collection);
    }
    codegen_reactive_value_to_expression(cx, init)
}

/// Extract left and collection for for-in from the init value.
fn codegen_for_in_init<'a>(
    cx: &mut CodegenContext<'a>,
    init: &ReactiveValue,
) -> (VariableDeclarationKind, BindingPattern<'a>) {
    todo!()
}

/// Extract the collection expression for for-in from the init value.
fn codegen_for_in_collection<'a>(
    cx: &mut CodegenContext<'a>,
    init: &ReactiveValue,
) -> Expression<'a> {
    if let ReactiveValue::Sequence(seq) = init
        && let Some(first) = seq.instructions.first()
    {
        return codegen_reactive_value_to_expression(cx, &first.value);
    }
    codegen_reactive_value_to_expression(cx, init)
}

/// Compare two scope dependencies for deterministic ordering.
fn compare_scope_dependency(
    a: &ReactiveScopeDependency,
    b: &ReactiveScopeDependency,
) -> std::cmp::Ordering {
    let a_key = build_dependency_sort_key(a);
    let b_key = build_dependency_sort_key(b);
    a_key.cmp(&b_key)
}

/// Build a sort key for a dependency.
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
// Backward-compatible exports (used by count_memo_slots in old code)
// =====================================================================================

/// Count the number of memo slots needed for the reactive function.
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
