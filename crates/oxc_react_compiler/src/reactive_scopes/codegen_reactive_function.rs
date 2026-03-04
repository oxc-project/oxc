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
        ArrayExpressionElement as HirArrayExpressionElement, ArrayPatternElement, CallArg,
        DeclarationId, FunctionExpressionType, HIRFunction, IdentifierId,
        IdentifierName as HirIdentifierName, InstructionKind, InstructionValue, JsxAttribute,
        JsxTag, ObjectExpression, ObjectMethodValue, ObjectPatternProperty, ObjectPropertyKey,
        ObjectPropertyType, Pattern, Place, PrimitiveValueKind, ReactiveBlock,
        ReactiveBreakTerminal, ReactiveContinueTerminal, ReactiveFunction, ReactiveInstruction,
        ReactiveInstructionStatement, ReactiveParam, ReactiveScope, ReactiveScopeDeclaration,
        ReactiveScopeDependency, ReactiveStatement, ReactiveTerminal, ReactiveTerminalTargetKind,
        ReactiveValue,
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
    /// Tracks which DeclarationIds originated from `InstructionValue::JsxText`.
    /// In the TS reference, the temp map stores `t.Expression | t.JSXText` (a union),
    /// so `codegenJsxElement` can check `value.type === 'JSXText'` directly.
    /// In Rust, both JsxText and Primitive::String produce `Expression::StringLiteral`,
    /// so we use this set to distinguish the two cases in `codegen_jsx_child`.
    jsx_text_ids: FxHashSet<DeclarationId>,
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
            jsx_text_ids: FxHashSet::default(),
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

/// Convert a `BindingPattern` to an `AssignmentTarget`.
///
/// Used for destructuring reassignments where the pattern was built as a
/// `BindingPattern` but needs to be an `AssignmentTarget` for the assignment expression.
fn binding_pattern_to_assignment_target<'a>(
    cx: &CodegenContext<'a>,
    pattern: BindingPattern<'a>,
) -> AssignmentTarget<'a> {
    match pattern {
        BindingPattern::BindingIdentifier(ident) => {
            let ident = ident.unbox();
            make_simple_target(cx, ident.name.as_str())
        }
        BindingPattern::ObjectPattern(obj) => {
            let obj = obj.unbox();
            let mut properties = cx.ast.vec_with_capacity(obj.properties.len());
            for prop in obj.properties.into_iter() {
                let target = binding_pattern_to_assignment_target(cx, prop.value);
                let binding: AssignmentTargetMaybeDefault<'a> = target.into();
                let property = cx.ast.assignment_target_property_property(
                    SPAN,
                    prop.key,
                    binding,
                    prop.computed,
                );
                properties.push(AssignmentTargetProperty::AssignmentTargetPropertyProperty(
                    cx.ast.alloc(property),
                ));
            }
            let rest: Option<oxc_allocator::Box<'a, AssignmentTargetRest<'a>>> =
                obj.rest.map(|rest| {
                    let rest: BindingRestElement<'a> = rest.unbox();
                    let target = binding_pattern_to_assignment_target(cx, rest.argument);
                    cx.ast.alloc(cx.ast.assignment_target_rest(SPAN, target))
                });
            let obj_target = cx.ast.object_assignment_target(SPAN, properties, rest);
            AssignmentTarget::from(AssignmentTargetPattern::ObjectAssignmentTarget(
                cx.ast.alloc(obj_target),
            ))
        }
        BindingPattern::ArrayPattern(arr) => {
            let arr = arr.unbox();
            let mut elements: AVec<'a, Option<AssignmentTargetMaybeDefault<'a>>> =
                cx.ast.vec_with_capacity(arr.elements.len());
            for elem in arr.elements.into_iter() {
                match elem {
                    Some(pat) => {
                        let target = binding_pattern_to_assignment_target(cx, pat);
                        let maybe_default: AssignmentTargetMaybeDefault<'a> = target.into();
                        elements.push(Some(maybe_default));
                    }
                    None => elements.push(None),
                }
            }
            let rest: Option<oxc_allocator::Box<'a, AssignmentTargetRest<'a>>> =
                arr.rest.map(|rest| {
                    let rest: BindingRestElement<'a> = rest.unbox();
                    let target = binding_pattern_to_assignment_target(cx, rest.argument);
                    cx.ast.alloc(cx.ast.assignment_target_rest(SPAN, target))
                });
            let arr_target = cx.ast.array_assignment_target(SPAN, elements, rest);
            AssignmentTarget::from(AssignmentTargetPattern::ArrayAssignmentTarget(
                cx.ast.alloc(arr_target),
            ))
        }
        BindingPattern::AssignmentPattern(_) => {
            // Assignment patterns with defaults should not appear in reassignment context
            make_simple_target(cx, "unknown")
        }
    }
}

/// Build `FormalParameters` from param name strings and an optional rest param.
///
/// Each param string becomes a simple `BindingIdentifier` parameter.
/// If the last param string starts with "...", it becomes a rest element.
fn build_formal_params<'a>(
    cx: &CodegenContext<'a>,
    params: &[String],
) -> oxc_allocator::Box<'a, FormalParameters<'a>> {
    let mut items = cx.ast.vec_with_capacity(params.len());
    let mut rest: Option<oxc_allocator::Box<'a, FormalParameterRest<'a>>> = None;

    for param_str in params {
        if let Some(rest_name) = param_str.strip_prefix("...") {
            // Rest parameter
            let binding = cx.ast.binding_pattern_binding_identifier(SPAN, cx.ast.atom(rest_name));
            let rest_elem = cx.ast.binding_rest_element(SPAN, binding);
            let fp_rest = cx.ast.formal_parameter_rest(SPAN, cx.ast.vec(), rest_elem, NONE);
            rest = Some(oxc_allocator::Box::new_in(fp_rest, cx.ast.allocator));
        } else {
            let binding =
                cx.ast.binding_pattern_binding_identifier(SPAN, cx.ast.atom(param_str.as_str()));
            let decorators = cx.ast.vec();
            let fp = cx
                .ast
                .formal_parameter(SPAN, decorators, binding, NONE, NONE, false, None, false, false);
            items.push(fp);
        }
    }

    cx.ast.alloc_formal_parameters(SPAN, FormalParameterKind::FormalParameter, items, rest)
}

/// Build a `FunctionBody` from directives and body statements.
fn build_function_body<'a>(
    cx: &CodegenContext<'a>,
    directives: &[String],
    body: AVec<'a, Statement<'a>>,
) -> oxc_allocator::Box<'a, FunctionBody<'a>> {
    let mut directive_nodes = cx.ast.vec_with_capacity(directives.len());
    for d in directives {
        let str_lit = cx.ast.string_literal(SPAN, cx.ast.atom(d.as_str()), None);
        let directive_node = cx.ast.directive(SPAN, str_lit, cx.ast.atom(d.as_str()));
        directive_nodes.push(directive_node);
    }
    cx.ast.alloc_function_body(SPAN, directive_nodes, body)
}

/// Convert an expression into an optional `ChainElement`.
///
/// Takes a CallExpression, StaticMemberExpression, ComputedMemberExpression,
/// or ChainExpression and produces a `ChainElement` with `optional: true`.
/// Used for building `ChainExpression` nodes for optional chains (`?.`).
/// Convert an `Expression` to a `ChainElement` without changing the optional flag.
///
/// This is used when wrapping an optional chain: the expression already has the
/// correct `optional` flag set, and we just need to convert it to a `ChainElement`
/// so it can be wrapped in a `ChainExpression`.
fn expression_to_chain_element(expr: Expression<'_>) -> Option<ChainElement<'_>> {
    match expr {
        Expression::CallExpression(call) => Some(ChainElement::CallExpression(call)),
        Expression::StaticMemberExpression(m) => Some(ChainElement::StaticMemberExpression(m)),
        Expression::ComputedMemberExpression(m) => Some(ChainElement::ComputedMemberExpression(m)),
        Expression::PrivateFieldExpression(f) => Some(ChainElement::PrivateFieldExpression(f)),
        _ => None,
    }
}

/// Check if an expression (or any of its nested object/callee sub-expressions)
/// contains an optional member access or optional call (`optional=true`).
///
/// This is used by the `OptionalCall(optional=false)` handler to determine whether
/// the inner expression is a non-optional continuation of an optional chain and
/// therefore needs to be wrapped in a `ChainExpression`.
fn contains_optional_access(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::StaticMemberExpression(m) => m.optional || contains_optional_access(&m.object),
        Expression::ComputedMemberExpression(m) => {
            m.optional || contains_optional_access(&m.object)
        }
        Expression::CallExpression(c) => c.optional || contains_optional_access(&c.callee),
        Expression::ChainExpression(_) => true,
        _ => false,
    }
}

/// Set the `optional` flag to `true` on a `CallExpression`, `StaticMemberExpression`,
/// or `ComputedMemberExpression`, returning the modified expression.
///
/// In ESTree/OXC AST, optional chain segments (`?.`) are represented by setting the
/// `optional` flag on the member/call expression, NOT by wrapping in `ChainExpression`.
/// The `ChainExpression` wrapper is added only once around the entire chain.
fn set_optional_flag<'a>(
    cx: &CodegenContext<'a>,
    expr: Expression<'a>,
    loc: crate::compiler_error::SourceLocation,
) -> Expression<'a> {
    match expr {
        Expression::CallExpression(call) => {
            let call = call.unbox();
            cx.ast.expression_call(SPAN, call.callee, NONE, call.arguments, true)
        }
        Expression::StaticMemberExpression(member) => {
            let member = member.unbox();
            Expression::from(cx.ast.member_expression_static(
                SPAN,
                member.object,
                member.property,
                true,
            ))
        }
        Expression::ComputedMemberExpression(member) => {
            let member = member.unbox();
            Expression::from(cx.ast.member_expression_computed(
                SPAN,
                member.object,
                member.expression,
                true,
            ))
        }
        _ => {
            cx.codegen_errors.borrow_mut().push(CompilerError::invariant(
                "Expected an optional value to resolve to a call expression or member expression",
                None,
                loc,
            ));
            expr
        }
    }
}

/// Wrap an expression in a `ChainExpression` if it (or any of its nested sub-expressions)
/// contains an optional access. This is the single wrapping point for optional chains.
fn wrap_in_chain_if_needed<'a>(cx: &CodegenContext<'a>, expr: Expression<'a>) -> Expression<'a> {
    if !contains_optional_access(&expr) {
        return expr;
    }
    // The expression contains optional accesses. Wrap the outermost
    // member/call in a ChainExpression. If the expression is not a
    // convertible type (shouldn't happen for well-formed optional chains),
    // return it as-is.
    match expression_to_chain_element(expr) {
        Some(elem) => cx.ast.expression_chain(SPAN, elem),
        None => make_undefined(cx),
    }
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
fn clone_temp_map<'a>(cx: &CodegenContext<'a>) -> FxHashMap<DeclarationId, Option<Expression<'a>>> {
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

    // TODO: Emit HMR/Fast Refresh hash check and cache reset
    // TODO: Emit instrument forget

    let cache_count = cx.next_cache_index;

    // Insert `const $ = _c(N);` preamble if there are cache slots
    if cache_count > 0 {
        let call = make_call(
            &cx,
            make_id(&cx, "_c"),
            cx.ast.vec1(Argument::from(make_number(&cx, f64::from(cache_count)))),
        );
        let preamble = make_var_decl(&cx, VariableDeclarationKind::Const, "$", Some(call));
        body.insert(0, preamble);
    }

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
                // Generate terminal statements into a temporary buffer
                let mut term_stmts: AVec<'a, Statement<'a>> = cx.ast.vec();
                codegen_terminal(cx, &term_stmt.terminal, &mut term_stmts);

                if term_stmts.is_empty() {
                    // Terminal produced nothing — skip
                } else if let Some(ref label) = term_stmt.label {
                    if label.implicit {
                        // Implicit label: flatten statements
                        statements.extend(term_stmts);
                    } else {
                        // Explicit label: wrap in LabeledStatement
                        let label_name = codegen_label(label.id);
                        let label_id = cx.ast.label_identifier(SPAN, cx.ast.atom(&label_name));
                        // Combine terminal output into a single statement
                        let body = if term_stmts.len() == 1 {
                            term_stmts.pop().unwrap_or_else(|| cx.ast.statement_empty(SPAN))
                        } else {
                            let block = cx.ast.alloc_block_statement(SPAN, term_stmts);
                            Statement::BlockStatement(block)
                        };
                        // If the body is a BlockStatement with exactly one statement, unwrap it
                        // (matches TS: statement.type === 'BlockStatement' && statement.body.length === 1)
                        let body = match body {
                            Statement::BlockStatement(block_alloc)
                                if block_alloc.body.len() == 1 =>
                            {
                                let mut block = block_alloc.unbox();
                                block.body.pop().unwrap_or_else(|| cx.ast.statement_empty(SPAN))
                            }
                            other => other,
                        };
                        statements.push(cx.ast.statement_labeled(SPAN, label_id, body));
                    }
                } else {
                    // No label: flatten BlockStatement body, otherwise push directly
                    // (matches TS: statement.type === 'BlockStatement' ? ...statement.body : statement)
                    if term_stmts.len() == 1 {
                        let stmt = term_stmts.pop().unwrap_or_else(|| cx.ast.statement_empty(SPAN));
                        match stmt {
                            Statement::BlockStatement(block_alloc) => {
                                statements.extend(block_alloc.unbox().body);
                            }
                            other => {
                                statements.push(other);
                            }
                        }
                    } else {
                        statements.extend(term_stmts);
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
/// Produces the memoization if/else structure:
/// ```js
/// let decl;
/// if ($[idx] !== dep || ...) {  // or $[idx] === Symbol.for("react.memo_cache_sentinel")
///   // computation
///   $[idx] = dep;
///   $[idx] = decl;
/// } else {
///   decl = $[idx];
/// }
/// if (decl !== Symbol.for("react.early_return_sentinel")) { return decl; }
/// ```
fn codegen_reactive_scope<'a>(
    cx: &mut CodegenContext<'a>,
    statements: &mut AVec<'a, Statement<'a>>,
    scope: &ReactiveScope,
    block: &ReactiveBlock,
) {
    let cache_name = cx.synthesize_name("$");

    let mut cache_store_stmts: AVec<'a, Statement<'a>> = cx.ast.vec();
    let mut cache_load_stmts: AVec<'a, Statement<'a>> = cx.ast.vec();
    let mut change_expressions: Vec<Expression<'a>> = Vec::new();

    // Process dependencies: sorted for determinism
    let mut deps: Vec<&ReactiveScopeDependency> = scope.dependencies.iter().collect();
    deps.sort_by(|a, b| compare_scope_dependency(a, b));

    for dep in &deps {
        let index = cx.alloc_cache_index();

        // Build change test: $[idx] !== dep_expr
        let cache_access =
            make_computed_member(cx, make_id(cx, &cache_name), make_number(cx, f64::from(index)));
        let dep_expr = codegen_dependency(cx, dep);
        let comparison = make_binary(cx, cache_access, BinaryOperator::StrictInequality, dep_expr);
        change_expressions.push(comparison);

        // Build cache store: $[idx] = dep_expr (for consequent block)
        let store_target = make_computed_member_assignment_target(
            cx,
            make_id(cx, &cache_name),
            make_number(cx, f64::from(index)),
        );
        let dep_val = codegen_dependency(cx, dep);
        let store_assign = make_assignment(cx, store_target, dep_val);
        cache_store_stmts.push(make_expr_stmt(cx, store_assign));
    }

    // Process declarations: sorted for determinism, deduplicated by declaration_id.
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
            statements.push(make_var_decl(cx, VariableDeclarationKind::Let, &name, None));
        }
        cache_loads.push(CacheLoad { name: name.clone(), index });
        cx.declare(decl.identifier.declaration_id);
    }

    // Process reassignments
    for reassignment_ident in &scope.reassignments {
        let index = cx.alloc_cache_index();
        if first_output_index.is_none() {
            first_output_index = Some(index);
        }
        let name = match &reassignment_ident.name {
            Some(
                crate::hir::IdentifierName::Named(n) | crate::hir::IdentifierName::Promoted(n),
            ) => n.clone(),
            None => format!("t{}", reassignment_ident.id.0),
        };
        cache_loads.push(CacheLoad { name, index });
    }

    // Build the test condition
    let test_condition = if change_expressions.is_empty() {
        // No dependencies — use sentinel check on first output:
        // $[first_output_idx] === Symbol.for("react.memo_cache_sentinel")
        if let Some(first_idx) = first_output_index {
            let cache_access = make_computed_member(
                cx,
                make_id(cx, &cache_name),
                make_number(cx, f64::from(first_idx)),
            );
            let sentinel = make_sentinel_call(cx, MEMO_CACHE_SENTINEL);
            make_binary(cx, cache_access, BinaryOperator::StrictEquality, sentinel)
        } else {
            // No deps and no outputs — should not happen, but be safe
            make_bool(cx, true)
        }
    } else {
        // Join change expressions with || (LogicalExpression)
        let mut iter = change_expressions.into_iter();
        let first = iter.next().unwrap_or_else(|| make_bool(cx, true));
        iter.fold(first, |acc, expr| make_logical(cx, acc, LogicalOperator::Or, expr))
    };

    // Generate the computation block
    let mut computation_stmts = codegen_block(cx, block);

    // Store each output into the cache: $[idx] = name
    for load in &cache_loads {
        let target = make_computed_member_assignment_target(
            cx,
            make_id(cx, &cache_name),
            make_number(cx, f64::from(load.index)),
        );
        let value = make_id(cx, &load.name);
        let assign = make_assignment(cx, target, value);
        cache_store_stmts.push(make_expr_stmt(cx, assign));
    }
    computation_stmts.extend(cache_store_stmts);

    // Load from cache in else branch: name = $[idx]
    for load in &cache_loads {
        let target = make_simple_target(cx, &load.name);
        let cache_access = make_computed_member(
            cx,
            make_id(cx, &cache_name),
            make_number(cx, f64::from(load.index)),
        );
        let assign = make_assignment(cx, target, cache_access);
        cache_load_stmts.push(make_expr_stmt(cx, assign));
    }

    // Build: if (test) { computation + stores } else { loads }
    let consequent = Statement::BlockStatement(stmts_to_block_body(cx, computation_stmts));
    let alternate = Statement::BlockStatement(stmts_to_block_body(cx, cache_load_stmts));
    statements.push(cx.ast.statement_if(SPAN, test_condition, consequent, Some(alternate)));

    // Handle early return value:
    // if (name !== Symbol.for("react.early_return_sentinel")) { return name; }
    if let Some(ref early_return) = scope.early_return_value {
        let name = identifier_name(&early_return.value);
        let test = make_binary(
            cx,
            make_id(cx, &name),
            BinaryOperator::StrictInequality,
            make_sentinel_call(cx, EARLY_RETURN_SENTINEL),
        );
        let return_stmt = make_return(cx, Some(make_id(cx, &name)));
        let return_block =
            Statement::BlockStatement(stmts_to_block_body(cx, cx.ast.vec1(return_stmt)));
        statements.push(cx.ast.statement_if(SPAN, test, return_block, None));
    }
}

/// Build `Symbol.for("sentinel_string")` call expression.
fn make_sentinel_call<'a>(cx: &CodegenContext<'a>, sentinel: &str) -> Expression<'a> {
    let symbol_for = make_member(cx, make_id(cx, "Symbol"), "for");
    let args = cx.ast.vec1(Argument::from(make_string(cx, sentinel)));
    make_call(cx, symbol_for, args)
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
    match terminal {
        ReactiveTerminal::Break(t) => {
            if let Some(stmt) = codegen_break(cx, t) {
                stmts.push(stmt);
            }
        }
        ReactiveTerminal::Continue(t) => {
            if let Some(stmt) = codegen_continue(cx, t) {
                stmts.push(stmt);
            }
        }
        ReactiveTerminal::Return(t) => {
            let value = codegen_place_to_expression(cx, &t.value);
            // If the return value is just `undefined`, emit bare `return;`
            let ret = if matches!(&value, Expression::Identifier(id) if id.name == "undefined") {
                make_return(cx, None)
            } else {
                make_return(cx, Some(value))
            };
            stmts.push(ret);
        }
        ReactiveTerminal::Throw(t) => {
            let value = codegen_place_to_expression(cx, &t.value);
            stmts.push(cx.ast.statement_throw(SPAN, value));
        }
        ReactiveTerminal::If(t) => {
            let test = codegen_place_to_expression(cx, &t.test);
            let consequent_stmts = codegen_block(cx, &t.consequent);
            let consequent = Statement::BlockStatement(stmts_to_block_body(cx, consequent_stmts));

            let alternate = t.alternate.as_ref().and_then(|alt| {
                let alt_stmts = codegen_block(cx, alt);
                if alt_stmts.is_empty() {
                    None
                } else {
                    Some(Statement::BlockStatement(stmts_to_block_body(cx, alt_stmts)))
                }
            });

            stmts.push(cx.ast.statement_if(SPAN, test, consequent, alternate));
        }
        ReactiveTerminal::Switch(t) => {
            let discriminant = codegen_place_to_expression(cx, &t.test);
            let mut cases: AVec<'a, SwitchCase<'a>> = cx.ast.vec();
            for case in &t.cases {
                let test = case.test.as_ref().map(|p| codegen_place_to_expression(cx, p));
                let consequent: AVec<'a, Statement<'a>> = case
                    .block
                    .as_ref()
                    .map(|b| {
                        let block_stmts = codegen_block(cx, b);
                        if block_stmts.is_empty() {
                            cx.ast.vec()
                        } else {
                            // Wrap in a BlockStatement like TS does
                            let block =
                                Statement::BlockStatement(stmts_to_block_body(cx, block_stmts));
                            cx.ast.vec1(block)
                        }
                    })
                    .unwrap_or_else(|| cx.ast.vec());
                cases.push(cx.ast.switch_case(SPAN, test, consequent));
            }
            stmts.push(cx.ast.statement_switch(SPAN, discriminant, cases));
        }
        ReactiveTerminal::For(t) => {
            let init = Some(codegen_for_init(cx, &t.init));
            let test = Some(codegen_reactive_value_to_expression(cx, &t.test));
            let update = t.update.as_ref().map(|u| codegen_reactive_value_to_expression(cx, u));
            let body_stmts = codegen_block(cx, &t.r#loop);
            let body = Statement::BlockStatement(stmts_to_block_body(cx, body_stmts));
            stmts.push(cx.ast.statement_for(SPAN, init, test, update, body));
        }
        ReactiveTerminal::ForOf(t) => {
            let (kind, binding) = codegen_for_of_in_init(cx, &t.init, &t.test);
            let declarator = cx.ast.variable_declarator(SPAN, kind, binding, NONE, None, false);
            let decl = cx.ast.variable_declaration(SPAN, kind, cx.ast.vec1(declarator), false);
            let left = ForStatementLeft::VariableDeclaration(cx.ast.alloc(decl));
            let right = codegen_for_of_collection(cx, &t.init);
            let body_stmts = codegen_block(cx, &t.r#loop);
            let body = Statement::BlockStatement(stmts_to_block_body(cx, body_stmts));
            stmts.push(cx.ast.statement_for_of(SPAN, false, left, right, body));
        }
        ReactiveTerminal::ForIn(t) => {
            let (kind, binding) = codegen_for_in_init(cx, &t.init);
            let declarator = cx.ast.variable_declarator(SPAN, kind, binding, NONE, None, false);
            let decl = cx.ast.variable_declaration(SPAN, kind, cx.ast.vec1(declarator), false);
            let left = ForStatementLeft::VariableDeclaration(cx.ast.alloc(decl));
            let right = codegen_for_in_collection(cx, &t.init);
            let body_stmts = codegen_block(cx, &t.r#loop);
            let body = Statement::BlockStatement(stmts_to_block_body(cx, body_stmts));
            stmts.push(cx.ast.statement_for_in(SPAN, left, right, body));
        }
        ReactiveTerminal::While(t) => {
            let test = codegen_reactive_value_to_expression(cx, &t.test);
            let body_stmts = codegen_block(cx, &t.r#loop);
            let body = Statement::BlockStatement(stmts_to_block_body(cx, body_stmts));
            stmts.push(cx.ast.statement_while(SPAN, test, body));
        }
        ReactiveTerminal::DoWhile(t) => {
            let body_stmts = codegen_block(cx, &t.r#loop);
            let body = Statement::BlockStatement(stmts_to_block_body(cx, body_stmts));
            let test = codegen_reactive_value_to_expression(cx, &t.test);
            stmts.push(cx.ast.statement_do_while(SPAN, body, test));
        }
        ReactiveTerminal::Label(t) => {
            let block_stmts = codegen_block(cx, &t.block);
            if !block_stmts.is_empty() {
                stmts.push(Statement::BlockStatement(stmts_to_block_body(cx, block_stmts)));
            }
        }
        ReactiveTerminal::Try(t) => {
            let block_stmts = codegen_block(cx, &t.block);
            let block = stmts_to_block_body(cx, block_stmts);

            let handler_stmts = codegen_block(cx, &t.handler);
            let handler = {
                let handler_body = stmts_to_block_body(cx, handler_stmts);
                let param = t.handler_binding.as_ref().map(|binding| {
                    let name = identifier_name(&binding.identifier);
                    cx.temp.insert(binding.identifier.declaration_id, None);
                    let binding_pattern =
                        cx.ast.binding_pattern_binding_identifier(SPAN, cx.ast.atom(&name));
                    cx.ast.catch_parameter(SPAN, binding_pattern, NONE)
                });
                Some(cx.ast.alloc_catch_clause(SPAN, param, handler_body))
            };

            stmts.push(cx.ast.statement_try(
                SPAN,
                block,
                handler,
                None::<oxc_allocator::Box<'_, BlockStatement<'_>>>,
            ));
        }
    }
}

fn codegen_break<'a>(cx: &CodegenContext<'a>, t: &ReactiveBreakTerminal) -> Option<Statement<'a>> {
    if t.target_kind == ReactiveTerminalTargetKind::Implicit {
        return None;
    }
    let label = if t.target_kind == ReactiveTerminalTargetKind::Labeled {
        Some(cx.ast.label_identifier(SPAN, cx.ast.atom(&codegen_label(t.target))))
    } else {
        None
    };
    Some(cx.ast.statement_break(SPAN, label))
}

fn codegen_continue<'a>(
    cx: &CodegenContext<'a>,
    t: &ReactiveContinueTerminal,
) -> Option<Statement<'a>> {
    if t.target_kind == ReactiveTerminalTargetKind::Implicit {
        return None;
    }
    let label = if t.target_kind == ReactiveTerminalTargetKind::Labeled {
        Some(cx.ast.label_identifier(SPAN, cx.ast.atom(&codegen_label(t.target))))
    } else {
        None
    };
    Some(cx.ast.statement_continue(SPAN, label))
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
    match &instr.value {
        ReactiveValue::Instruction(boxed) => match boxed.as_ref() {
            InstructionValue::StoreLocal(store) => {
                let kind = if cx.has_declared(store.lvalue.place.identifier.declaration_id)
                    && !matches!(
                        store.lvalue.kind,
                        InstructionKind::Function | InstructionKind::HoistedFunction
                    ) {
                    InstructionKind::Reassign
                } else {
                    store.lvalue.kind
                };
                let value_expr = codegen_place_to_expression(cx, &store.value);
                codegen_store_or_declare(
                    cx,
                    instr,
                    kind,
                    &store.lvalue.place,
                    Some(value_expr),
                    Some(&store.value),
                    stmts,
                );
            }
            InstructionValue::StoreContext(store) => {
                let kind = store.lvalue_kind;
                let value_expr = codegen_place_to_expression(cx, &store.value);
                codegen_store_or_declare(
                    cx,
                    instr,
                    kind,
                    &store.lvalue_place,
                    Some(value_expr),
                    Some(&store.value),
                    stmts,
                );
            }
            InstructionValue::DeclareLocal(decl) => {
                if cx.has_declared(decl.lvalue.place.identifier.declaration_id) {
                    return;
                }
                codegen_store_or_declare(
                    cx,
                    instr,
                    decl.lvalue.kind,
                    &decl.lvalue.place,
                    None,
                    None,
                    stmts,
                );
            }
            InstructionValue::DeclareContext(decl) => {
                if cx.has_declared(decl.lvalue_place.identifier.declaration_id) {
                    return;
                }
                codegen_store_or_declare(
                    cx,
                    instr,
                    decl.lvalue_kind,
                    &decl.lvalue_place,
                    None,
                    None,
                    stmts,
                );
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
                codegen_destructure_statement(cx, instr, kind, lval, value_expr, stmts);
            }
            InstructionValue::StartMemoize(_) | InstructionValue::FinishMemoize(_) => {
                // Memoization markers are consumed by codegen_reactive_scope
            }
            InstructionValue::Debugger(_) => {
                stmts.push(cx.ast.statement_debugger(SPAN));
            }
            InstructionValue::ObjectMethod(method) => {
                // Store object method for later use by ObjectExpression codegen.
                if let Some(lval) = &instr.lvalue {
                    cx.object_methods.insert(lval.identifier.id, method.clone());
                }
            }
            InstructionValue::FunctionExpression(func_expr) => {
                // Handle FunctionExpression separately so we can store structured
                // CodegenOutput data for FunctionDeclaration types.
                let (value_expr, fn_decl) = codegen_function_expression(cx, func_expr);
                if let Some(fn_data) = fn_decl {
                    if let Some(lval) = &instr.lvalue {
                        cx.fn_decl_data.insert(lval.identifier.declaration_id, fn_data);
                    }
                }
                codegen_instruction_to_statement(cx, instr, value_expr, stmts);
            }
            other => {
                // Track JsxText-origin declarations so codegen_jsx_child can
                // distinguish them from Primitive::String (both produce StringLiteral).
                if matches!(other, InstructionValue::JsxText(_)) {
                    if let Some(lval) = &instr.lvalue {
                        cx.jsx_text_ids.insert(lval.identifier.declaration_id);
                    }
                }
                let value_expr = codegen_instruction_value(cx, other);
                codegen_instruction_to_statement(cx, instr, value_expr, stmts);
            }
        },
        ReactiveValue::Logical(_)
        | ReactiveValue::Ternary(_)
        | ReactiveValue::Sequence(_)
        | ReactiveValue::OptionalCall(_) => {
            let value_expr = codegen_reactive_value_to_expression(cx, &instr.value);
            codegen_instruction_to_statement(cx, instr, value_expr, stmts);
        }
    }
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
    let name = identifier_name(&lvalue_place.identifier);
    let decl_id = lvalue_place.identifier.declaration_id;

    // Wrap optional chains in ChainExpression at the final usage point.
    let value = value.map(|v| wrap_in_chain_if_needed(cx, v));

    match kind {
        InstructionKind::Const | InstructionKind::HoistedConst => {
            // TS: CompilerError.invariant(instr.lvalue === null, ...)
            if kind == InstructionKind::Const && instr.lvalue.is_some() {
                cx.codegen_errors.borrow_mut().push(CompilerError::invariant(
                    "Const declaration cannot be referenced as an expression",
                    Some(&format!("this is {kind:?}")),
                    instr.loc,
                ));
            }
            cx.declare(decl_id);
            stmts.push(make_var_decl(cx, VariableDeclarationKind::Const, &name, value));
        }
        InstructionKind::Let | InstructionKind::HoistedLet => {
            if kind == InstructionKind::Let && instr.lvalue.is_some() {
                cx.codegen_errors.borrow_mut().push(CompilerError::invariant(
                    "Const declaration cannot be referenced as an expression",
                    Some(&format!("this is {kind:?}")),
                    instr.loc,
                ));
            }
            cx.declare(decl_id);
            stmts.push(make_var_decl(cx, VariableDeclarationKind::Let, &name, value));
        }
        InstructionKind::Function | InstructionKind::HoistedFunction => {
            cx.declare(decl_id);
            // Try to emit a proper function declaration statement.
            // Look up the structured CodegenOutput data stored when the
            // FunctionExpression instruction was processed.
            if let Some(vp) = value_place {
                if let Some(fn_data) = cx.fn_decl_data.remove(&vp.identifier.declaration_id) {
                    let fn_name = fn_data
                        .id
                        .as_ref()
                        .map(|n| cx.ast.binding_identifier(SPAN, cx.ast.atom(n.as_str())));
                    let params = build_formal_params(cx, &fn_data.params);
                    let body = build_function_body(cx, &fn_data.directives, fn_data.body);
                    let decl = cx.ast.function(
                        SPAN,
                        FunctionType::FunctionDeclaration,
                        fn_name,
                        fn_data.generator,
                        fn_data.is_async,
                        false, // declare
                        NONE,  // type_parameters
                        NONE,  // this_param
                        params,
                        NONE, // return_type
                        Some(body),
                    );
                    stmts.push(Statement::FunctionDeclaration(cx.ast.alloc(decl)));
                    return;
                }
            }
            // Fallback: emit as const declaration or expression statement
            if let Some(val) = value {
                stmts.push(make_var_decl(cx, VariableDeclarationKind::Const, &name, Some(val)));
            } else {
                stmts.push(make_var_decl(cx, VariableDeclarationKind::Const, &name, None));
            }
        }
        InstructionKind::Reassign => {
            if let Some(val) = value {
                // If there's an lvalue on the instruction (i.e., it's used as an expression),
                // store as temporary.
                if let Some(lval) = instr.lvalue.as_ref() {
                    if lval.identifier.name.is_none() {
                        let target = make_simple_target(cx, &name);
                        let assign = make_assignment(cx, target, val);
                        cx.temp.insert(lval.identifier.declaration_id, Some(assign));
                        return;
                    }
                }
                // Named reassignment: emit `name = value` as expression statement
                let target = make_simple_target(cx, &name);
                let assign = make_assignment(cx, target, val);
                try_store_as_temporary(cx, None, assign, stmts);
            }
        }
        InstructionKind::Catch => {
            // Catch bindings: emit empty statement
            stmts.push(cx.ast.statement_empty(SPAN));
        }
    }
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
    match kind {
        InstructionKind::Const
        | InstructionKind::HoistedConst
        | InstructionKind::Function
        | InstructionKind::HoistedFunction => {
            if kind == InstructionKind::Const && instr.lvalue.is_some() {
                cx.codegen_errors.borrow_mut().push(CompilerError::invariant(
                    "Const declaration cannot be referenced as an expression",
                    Some(&format!("this is {kind:?}")),
                    instr.loc,
                ));
            }
            let var_kind = VariableDeclarationKind::Const;
            let declarator =
                cx.ast.variable_declarator(SPAN, var_kind, lval, NONE, Some(value), false);
            let declarators = cx.ast.vec1(declarator);
            let decl = cx.ast.variable_declaration(SPAN, var_kind, declarators, false);
            stmts.push(Statement::VariableDeclaration(cx.ast.alloc(decl)));
        }
        InstructionKind::Let | InstructionKind::HoistedLet => {
            if kind == InstructionKind::Let && instr.lvalue.is_some() {
                cx.codegen_errors.borrow_mut().push(CompilerError::invariant(
                    "Const declaration cannot be referenced as an expression",
                    Some(&format!("this is {kind:?}")),
                    instr.loc,
                ));
            }
            let var_kind = VariableDeclarationKind::Let;
            let declarator =
                cx.ast.variable_declarator(SPAN, var_kind, lval, NONE, Some(value), false);
            let declarators = cx.ast.vec1(declarator);
            let decl = cx.ast.variable_declaration(SPAN, var_kind, declarators, false);
            stmts.push(Statement::VariableDeclaration(cx.ast.alloc(decl)));
        }
        InstructionKind::Reassign => {
            // Convert BindingPattern to AssignmentTarget for destructuring assignment
            let target = binding_pattern_to_assignment_target(cx, lval);
            let assign = make_assignment(cx, target, value);
            // If there's an lvalue on the instruction used as expression, store as temp
            if let Some(lval_place) = instr.lvalue.as_ref() {
                if lval_place.identifier.name.is_none() {
                    cx.temp.insert(lval_place.identifier.declaration_id, Some(assign));
                    return;
                }
            }
            stmts.push(make_expr_stmt(cx, assign));
        }
        InstructionKind::Catch => {
            stmts.push(cx.ast.statement_empty(SPAN));
        }
    }
}

/// Store an expression as a temporary if the lvalue is unnamed, otherwise emit as expression statement.
fn try_store_as_temporary<'a>(
    cx: &mut CodegenContext<'a>,
    lvalue: Option<&Place>,
    expr: Expression<'a>,
    stmts: &mut AVec<'a, Statement<'a>>,
) {
    if let Some(lval) = lvalue {
        let name = identifier_name(&lval.identifier);
        if name.is_empty() || lval.identifier.name.is_none() {
            // Unnamed temp — store for inline substitution.
            // Do NOT wrap in ChainExpression here (see codegen_instruction_to_statement).
            cx.temp.insert(lval.identifier.declaration_id, Some(expr));
            return;
        }
    }
    // Final usage — wrap optional chains.
    let expr = wrap_in_chain_if_needed(cx, expr);
    stmts.push(make_expr_stmt(cx, expr));
}

/// Convert a codegen value into a statement, handling temporaries and named lvalues.
fn codegen_instruction_to_statement<'a>(
    cx: &mut CodegenContext<'a>,
    instr: &ReactiveInstruction,
    value: Expression<'a>,
    stmts: &mut AVec<'a, Statement<'a>>,
) {
    let lvalue = instr.lvalue.as_ref();
    match lvalue {
        None => {
            // No lvalue — just emit expression statement.
            // Wrap optional chains: expressions with optional flags need a
            // ChainExpression wrapper when used as a standalone expression.
            let value = wrap_in_chain_if_needed(cx, value);
            stmts.push(make_expr_stmt(cx, value));
        }
        Some(lval) => {
            if lval.identifier.name.is_none() {
                // Unnamed temp — store for inline substitution.
                // Do NOT wrap in ChainExpression here: the temp may be used as
                // an object in a subsequent PropertyLoad (optional chain
                // continuation), and ChainExpression as an object would cause
                // extra parentheses.
                cx.temp.insert(lval.identifier.declaration_id, Some(value));
            } else {
                // Named variable — this is a final usage point. Wrap optional
                // chains in ChainExpression for correct output.
                let value = wrap_in_chain_if_needed(cx, value);
                let name = identifier_name(&lval.identifier);
                if cx.has_declared(lval.identifier.declaration_id) {
                    // Already declared — reassignment
                    let target = make_simple_target(cx, &name);
                    let assign = make_assignment(cx, target, value);
                    stmts.push(make_expr_stmt(cx, assign));
                } else {
                    // First declaration
                    cx.declare(lval.identifier.declaration_id);
                    stmts.push(make_var_decl(
                        cx,
                        VariableDeclarationKind::Const,
                        &name,
                        Some(value),
                    ));
                }
            }
        }
    }
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
                        elements.push(cx.ast.array_expression_element_spread_element(SPAN, expr));
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
            let pattern = RegExpPattern { text: cx.ast.atom(&re.pattern), pattern: None };
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
        InstructionValue::TypeCastExpression(cast) => codegen_place_to_expression(cx, &cast.value),
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
        InstructionValue::GetIterator(iter) => codegen_place_to_expression(cx, &iter.collection),
        InstructionValue::IteratorNext(iter) => codegen_place_to_expression(cx, &iter.iterator),
        InstructionValue::NextPropertyOf(next) => codegen_place_to_expression(cx, &next.value),
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
        InstructionValue::UnsupportedNode(_) => make_string(cx, "/* unsupported */"),
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
/// Port of `CodegenReactiveFunction.ts` FunctionExpression case (lines 1861-1909).
///
/// Returns the expression and, for `FunctionDeclaration` types, the structured
/// `CodegenOutput` data needed to emit a proper function declaration statement.
fn codegen_function_expression<'a>(
    cx: &CodegenContext<'a>,
    func_expr: &crate::hir::FunctionExpressionValue,
) -> (Expression<'a>, Option<CodegenOutput<'a>>) {
    match codegen_inner_function(&func_expr.lowered_func.func, cx, true) {
        Ok(inner_fn) => {
            // For FunctionDeclaration types, preserve the CodegenOutput
            // so callers can emit a proper function declaration statement.
            let fn_decl =
                if func_expr.expression_type == FunctionExpressionType::FunctionDeclaration {
                    // Clone the inner_fn for the declaration data.
                    // We need to clone the body since we move inner_fn below.
                    let decl_body = inner_fn.body.clone_in(cx.ast.allocator);
                    Some(CodegenOutput {
                        id: inner_fn.id.clone(),
                        name_hint: inner_fn.name_hint.clone(),
                        params: inner_fn.params.clone(),
                        generator: inner_fn.generator,
                        is_async: inner_fn.is_async,
                        loc: inner_fn.loc,
                        memo_slots_used: inner_fn.memo_slots_used,
                        memo_blocks: inner_fn.memo_blocks,
                        memo_values: inner_fn.memo_values,
                        pruned_memo_blocks: inner_fn.pruned_memo_blocks,
                        pruned_memo_values: inner_fn.pruned_memo_values,
                        body: decl_body,
                        directives: inner_fn.directives.clone(),
                        outlined: Vec::new(),
                    })
                } else {
                    None
                };

            let value = match func_expr.expression_type {
                FunctionExpressionType::ArrowFunctionExpression => {
                    let params = build_formal_params(cx, &inner_fn.params);
                    let body_stmts = inner_fn.body;

                    // Check for concise arrow body: single return statement with argument,
                    // and no directives on the lowered function.
                    // In the AST world, we check if the body has exactly one ReturnStatement
                    // with an argument, and there are no directives.
                    let concise = if body_stmts.len() == 1
                        && func_expr.lowered_func.func.directives.is_empty()
                    {
                        if let Statement::ReturnStatement(ret) = &body_stmts[0] {
                            ret.argument.as_ref().map(|arg| arg.clone_in(cx.ast.allocator))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some(expr_body) = concise {
                        // Concise arrow: () => expr
                        // Build a function body containing just the expression.
                        // The `expression: true` flag on ArrowFunctionExpression tells
                        // the AST that this is a concise body.
                        let expr_stmt = make_expr_stmt(cx, expr_body);
                        let stmts = cx.ast.vec1(expr_stmt);
                        let fb = build_function_body(cx, &inner_fn.directives, stmts);
                        cx.ast.expression_arrow_function(
                            SPAN,
                            true, // expression (concise body)
                            inner_fn.is_async,
                            NONE, // type_parameters
                            params,
                            NONE, // return_type
                            fb,
                        )
                    } else {
                        let fb = build_function_body(cx, &inner_fn.directives, body_stmts);
                        cx.ast.expression_arrow_function(
                            SPAN,
                            false, // expression (block body)
                            inner_fn.is_async,
                            NONE, // type_parameters
                            params,
                            NONE, // return_type
                            fb,
                        )
                    }
                }
                FunctionExpressionType::FunctionExpression
                | FunctionExpressionType::FunctionDeclaration => {
                    let params = build_formal_params(cx, &inner_fn.params);
                    let body = build_function_body(cx, &inner_fn.directives, inner_fn.body);
                    let id = func_expr
                        .name
                        .as_ref()
                        .map(|n| cx.ast.binding_identifier(SPAN, cx.ast.atom(n.as_str())));

                    cx.ast.expression_function(
                        SPAN,
                        FunctionType::FunctionExpression,
                        id,
                        inner_fn.generator,
                        inner_fn.is_async,
                        false, // declare
                        NONE,  // type_parameters
                        NONE,  // this_param
                        params,
                        NONE, // return_type
                        Some(body),
                    )
                }
            };

            // enableNameAnonymousFunctions: wrap anonymous functions in a naming expression.
            // Produces: {"nameHint": <funcExpr>}["nameHint"]
            // TS reference lines 1896-1908
            let value = if cx.enable_name_anonymous_functions
                && func_expr.name.is_none()
                && func_expr.name_hint.is_some()
            {
                let hint = func_expr.name_hint.as_ref().map_or("", String::as_str);
                // Build: {hint: value}[hint]
                let key = PropertyKey::StringLiteral(cx.ast.alloc(cx.ast.string_literal(
                    SPAN,
                    cx.ast.atom(hint),
                    None,
                )));
                let obj_prop = cx.ast.object_property(
                    SPAN,
                    PropertyKind::Init,
                    key,
                    value,
                    false, // method
                    false, // shorthand
                    false, // computed
                );
                let props = cx.ast.vec1(ObjectPropertyKind::ObjectProperty(cx.ast.alloc(obj_prop)));
                let obj = cx.ast.expression_object(SPAN, props);
                let index = make_string(cx, hint);
                make_computed_member(cx, obj, index)
            } else {
                value
            };

            (value, fn_decl)
        }
        Err(e) => {
            // Propagate inner function compilation errors to the outer context
            cx.codegen_errors.borrow_mut().push(e);
            (make_undefined(cx), None)
        }
    }
}

/// Generate an object expression.
///
/// Port of `CodegenReactiveFunction.ts` ObjectExpression case (lines 1637-1708).
///
/// For each property:
/// - Regular properties: build `ObjectProperty` with key + value
/// - Method properties: compile inner function, build `ObjectProperty` with `method: true`
/// - Spread properties: build `SpreadElement`
fn codegen_object_expression<'a>(
    cx: &CodegenContext<'a>,
    obj: &ObjectExpression,
) -> Expression<'a> {
    let mut properties = cx.ast.vec_with_capacity(obj.properties.len());

    for prop in &obj.properties {
        match prop {
            ObjectPatternProperty::Property(p) => {
                let key = codegen_object_property_key(cx, &p.key);
                let is_computed = matches!(p.key, ObjectPropertyKey::Computed(_));

                if p.property_type == ObjectPropertyType::Method {
                    // Look up the ObjectMethod from the context
                    let method = cx.object_methods.get(&p.place.identifier.id).cloned();
                    if let Some(method) = method {
                        let method_key = codegen_object_property_key(cx, &p.key);
                        let method_expr =
                            codegen_object_method_expression(cx, &method, method_key, is_computed);
                        // Build ObjectProperty with method: true, kind: Init
                        let obj_prop = cx.ast.object_property(
                            SPAN,
                            PropertyKind::Init,
                            key,
                            method_expr,
                            true,  // method
                            false, // shorthand
                            is_computed,
                        );
                        properties.push(ObjectPropertyKind::ObjectProperty(cx.ast.alloc(obj_prop)));
                    } else {
                        // Fallback if method not found (should not happen)
                        cx.codegen_errors.borrow_mut().push(CompilerError::invariant(
                            "Expected ObjectMethod instruction",
                            None,
                            crate::compiler_error::GENERATED_SOURCE,
                        ));
                    }
                } else {
                    let value = codegen_place_to_expression(cx, &p.place);

                    // Check if shorthand: key is an identifier and value resolves to the same name
                    let is_shorthand = if !is_computed {
                        if let ObjectPropertyKey::Identifier(key_name) = &p.key {
                            // The value expression should be an identifier with the same name
                            let val_name = identifier_name(&p.place.identifier);
                            *key_name == val_name
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    let obj_prop = cx.ast.object_property(
                        SPAN,
                        PropertyKind::Init,
                        key,
                        value,
                        false, // method
                        is_shorthand,
                        is_computed,
                    );
                    properties.push(ObjectPropertyKind::ObjectProperty(cx.ast.alloc(obj_prop)));
                }
            }
            ObjectPatternProperty::Spread(s) => {
                let expr = codegen_place_to_expression(cx, &s.place);
                let spread = cx.ast.spread_element(SPAN, expr);
                properties.push(ObjectPropertyKind::SpreadProperty(cx.ast.alloc(spread)));
            }
        }
    }

    cx.ast.expression_object(SPAN, properties)
}

/// Generate an object method expression by recursively compiling the inner function.
///
/// Port of `CodegenReactiveFunction.ts` ObjectMethod case (lines 1658-1693).
///
/// Object methods do NOT call `pruneHoistedContexts` (the `include_prune_hoisted`
/// flag is false for `codegen_inner_function`).
///
/// Returns a `FunctionExpression` that will be used as the `value` of the
/// `ObjectProperty` with `method: true`.
fn codegen_object_method_expression<'a>(
    cx: &CodegenContext<'a>,
    method: &ObjectMethodValue,
    _key: PropertyKey<'a>,
    _is_computed: bool,
) -> Expression<'a> {
    match codegen_inner_function(&method.lowered_func.func, cx, false) {
        Ok(inner_fn) => {
            let params = build_formal_params(cx, &inner_fn.params);
            let body = build_function_body(cx, &inner_fn.directives, inner_fn.body);

            cx.ast.expression_function(
                SPAN,
                FunctionType::FunctionExpression,
                None, // id (methods don't need a name here)
                inner_fn.generator,
                inner_fn.is_async,
                false, // declare
                NONE,  // type_parameters
                NONE,  // this_param
                params,
                NONE, // return_type
                Some(body),
            )
        }
        Err(e) => {
            cx.codegen_errors.borrow_mut().push(e);
            make_undefined(cx)
        }
    }
}

// =====================================================================================
// codegen_reactive_value_to_expression — ReactiveValue → Expression
// =====================================================================================

/// Convert a `ReactiveValue` to an expression.
///
/// Port of `CodegenReactiveFunction.ts` `codegenInstructionValueToExpression` +
/// the reactive value handling (LogicalExpression, ConditionalExpression,
/// SequenceExpression, OptionalExpression).
///
/// With AST-based codegen, precedence and parenthesization are handled automatically
/// by oxc_codegen, so we don't need `wrap_logical_operand_if_needed` or similar helpers.
fn codegen_reactive_value_to_expression<'a>(
    cx: &mut CodegenContext<'a>,
    value: &ReactiveValue,
) -> Expression<'a> {
    match value {
        ReactiveValue::Instruction(boxed) => codegen_instruction_value(cx, boxed),

        ReactiveValue::Logical(logical) => {
            // TS reference: case 'LogicalExpression' (lines 1940-1947)
            // Build left/right expressions, then LogicalExpression node.
            // oxc_codegen handles precedence and parenthesization automatically.
            let left = codegen_reactive_value_to_expression(cx, &logical.left);
            let right = codegen_reactive_value_to_expression(cx, &logical.right);
            make_logical(cx, left, logical.operator, right)
        }

        ReactiveValue::Ternary(ternary) => {
            // TS reference: case 'ConditionalExpression' (lines 1949-1957)
            // Build test/consequent/alternate, then ConditionalExpression node.
            // oxc_codegen handles precedence and parenthesization automatically.
            let test = codegen_reactive_value_to_expression(cx, &ternary.test);
            let consequent = codegen_reactive_value_to_expression(cx, &ternary.consequent);
            let alternate = codegen_reactive_value_to_expression(cx, &ternary.alternate);
            cx.ast.expression_conditional(SPAN, test, consequent, alternate)
        }

        ReactiveValue::Sequence(seq) => {
            // TS reference: case 'SequenceExpression' (lines 1958-2004)
            //
            // Process sequence instructions via `codegen_instruction_nullable`,
            // collecting any ExpressionStatements into a sequence expression.
            //
            // Instructions whose results are consumed temps retain `lvalue: Some(unnamed)`
            // and are stored in `cx.temp` (producing no statements).
            // Instructions whose results are NOT consumed produce ExpressionStatements.
            //
            // We collect expression statements, then combine with the final value
            // into a SequenceExpression if needed.
            let mut temp_stmts: AVec<'a, Statement<'a>> = cx.ast.vec();

            for instr in &seq.instructions {
                codegen_instruction_nullable(cx, instr, &mut temp_stmts);
            }

            // Extract expressions from any ExpressionStatements
            let mut expressions: AVec<'a, Expression<'a>> = cx.ast.vec();
            for stmt in temp_stmts {
                if let Statement::ExpressionStatement(expr_stmt) = stmt {
                    expressions.push(expr_stmt.unbox().expression);
                }
                // VariableDeclarations in value blocks are logged as errors in the TS
                // reference but we silently skip them (they shouldn't occur after pruning).
            }

            let final_value = codegen_reactive_value_to_expression(cx, &seq.value);

            if expressions.is_empty() {
                final_value
            } else {
                expressions.push(final_value);
                make_sequence(cx, expressions)
            }
        }

        ReactiveValue::OptionalCall(optional) => {
            // TS reference: case 'OptionalExpression' (lines 1550-1593)
            //
            // In Babel AST, `OptionalMemberExpression` / `OptionalCallExpression` are
            // self-contained node types (no wrapper needed). In ESTree/OXC AST, there
            // must be exactly ONE `ChainExpression` wrapping the ENTIRE optional chain,
            // with inner member/call nodes using `optional` flags.
            //
            // Strategy:
            // - `optional=true`: Set the `optional` flag on the inner expression.
            //   Do NOT wrap in `ChainExpression` — the wrapper is deferred to the
            //   final usage point (variable declaration, return value, etc.).
            // - `optional=false`: Non-optional continuation of an optional chain
            //   (e.g., `.c` in `a?.b.c`). Return the inner expression as-is; the
            //   optional flags on sub-expressions are already set correctly.
            //
            // The single `ChainExpression` wrapper is added by `wrap_in_chain_if_needed`
            // at the point where the value is stored to a named variable or used
            // directly (see `codegen_instruction_to_statement` and scope dependency
            // codegen).
            let inner = codegen_reactive_value_to_expression(cx, &optional.value);

            if optional.optional {
                // Set the optional flag on the call/member expression.
                set_optional_flag(cx, inner, optional.loc)
            } else {
                // Non-optional continuation — return as-is.
                inner
            }
        }
    }
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
///
/// Handles namespaced attributes (e.g., `xmlns:xlink`) and string value escaping.
/// Matches TS `codegenJsxAttribute`: StringLiteral values that don't need
/// escaping are emitted directly as JSX attribute strings; all other values
/// are wrapped in expression containers.
fn codegen_jsx_attribute<'a>(cx: &CodegenContext<'a>, attr: &JsxAttribute) -> JSXAttributeItem<'a> {
    match attr {
        JsxAttribute::Attribute { name, place } => {
            // Build attribute name, handling namespaced names like xmlns:xlink
            let attr_name = if let Some(colon_pos) = name.find(':') {
                let namespace = &name[..colon_pos];
                let local = &name[colon_pos + 1..];
                let ns_ident = cx.ast.jsx_identifier(SPAN, cx.ast.atom(namespace));
                let name_ident = cx.ast.jsx_identifier(SPAN, cx.ast.atom(local));
                cx.ast.jsx_attribute_name_namespaced_name(SPAN, ns_ident, name_ident)
            } else {
                cx.ast.jsx_attribute_name_identifier(SPAN, cx.ast.atom(name.as_str()))
            };

            let inner_value = codegen_place_to_expression(cx, place);

            // TS: if the value is a StringLiteral, use it directly unless it
            // contains characters that require wrapping in an expression container.
            // fbt operands are exempt from the wrapping check.
            let value = match &inner_value {
                Expression::StringLiteral(lit)
                    if !string_requires_expr_container(&lit.value)
                        || cx.is_fbt_operand(place.identifier.id) =>
                {
                    // Use string literal directly as JSX attribute value
                    let Expression::StringLiteral(lit) = inner_value else { unreachable!() };
                    JSXAttributeValue::StringLiteral(lit)
                }
                _ => {
                    // Wrap in expression container (handles StringLiteral with
                    // special chars, JSXFragment, and all other expression types)
                    cx.ast.jsx_attribute_value_expression_container(
                        SPAN,
                        JSXExpression::from(inner_value),
                    )
                }
            };

            cx.ast.jsx_attribute_item_attribute(SPAN, attr_name, Some(value))
        }
        JsxAttribute::Spread { argument } => {
            let expr = codegen_place_to_expression(cx, argument);
            cx.ast.jsx_attribute_item_spread_attribute(SPAN, expr)
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
        if code <= 0x1F || code == 0x7F || code >= 0x80 {
            return true;
        }
        if c == '"' || c == '\\' {
            return true;
        }
    }
    false
}

/// Check if a JSX text child requires wrapping in an expression container.
///
/// Matches the TS `JSX_TEXT_CHILD_REQUIRES_EXPR_CONTAINER_PATTERN` regex: `/[<>&{}]/`
/// These characters have special meaning in JSX and cannot appear as raw text.
fn jsx_text_child_requires_expr_container(s: &str) -> bool {
    s.contains(['<', '>', '&', '{', '}'])
}

/// Render a JSX child element.
///
/// Matches TS `codegenJsxElement`: JSX text values that don't contain
/// special chars (`<>&{}`) become `JSXText` nodes; JSX elements and
/// fragments pass through directly; everything else is wrapped in a
/// `JSXExpressionContainer`.
fn codegen_jsx_child<'a>(cx: &CodegenContext<'a>, place: &Place) -> JSXChild<'a> {
    let is_jsx_text = cx.jsx_text_ids.contains(&place.identifier.declaration_id);
    let value = codegen_place_to_expression(cx, place);
    match value {
        Expression::StringLiteral(lit) => {
            // TS reference: the temp map stores `t.Expression | t.JSXText`. In
            // `codegenJsxElement`, `case 'JSXText'` applies the special-char
            // heuristic while `default` always wraps in ExpressionContainer.
            // We emulate this by checking `jsx_text_ids`: JsxText-origin strings
            // use the heuristic; Primitive::String-origin strings always wrap.
            if is_jsx_text {
                if jsx_text_child_requires_expr_container(&lit.value) {
                    let container =
                        cx.ast.jsx_expression_container(SPAN, JSXExpression::StringLiteral(lit));
                    JSXChild::ExpressionContainer(cx.ast.alloc(container))
                } else {
                    let text = cx.ast.jsx_text(SPAN, lit.value.clone(), None);
                    JSXChild::Text(cx.ast.alloc(text))
                }
            } else {
                let container =
                    cx.ast.jsx_expression_container(SPAN, JSXExpression::StringLiteral(lit));
                JSXChild::ExpressionContainer(cx.ast.alloc(container))
            }
        }
        Expression::JSXElement(elem) => JSXChild::Element(elem),
        Expression::JSXFragment(frag) => JSXChild::Fragment(frag),
        _ => {
            let container = cx.ast.jsx_expression_container(SPAN, JSXExpression::from(value));
            JSXChild::ExpressionContainer(cx.ast.alloc(container))
        }
    }
}

/// Convert a Place to an expression.
fn codegen_place_to_expression<'a>(cx: &CodegenContext<'a>, place: &Place) -> Expression<'a> {
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
fn codegen_place_to_expression_raw<'a>(cx: &CodegenContext<'a>, place: &Place) -> Expression<'a> {
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
fn codegen_primitive<'a>(cx: &CodegenContext<'a>, value: &PrimitiveValueKind) -> Expression<'a> {
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
        crate::hir::types::PropertyLiteral::String(name) => make_member(cx, object, name),
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
///
/// Port of `CodegenReactiveFunction.ts` `codegenObjectPropertyKey` (lines 2257-2280).
///
/// - `Identifier(name)` -> `PropertyKey::StaticIdentifier`
/// - `String(s)` -> `PropertyKey::StringLiteral`
/// - `Computed(place)` -> `PropertyKey` from the expression (computed)
/// - `Number(n)` -> `PropertyKey::NumericLiteral`
fn codegen_object_property_key<'a>(
    cx: &CodegenContext<'a>,
    key: &ObjectPropertyKey,
) -> PropertyKey<'a> {
    match key {
        ObjectPropertyKey::Identifier(name) => PropertyKey::StaticIdentifier(
            cx.ast.alloc(cx.ast.identifier_name(SPAN, cx.ast.atom(name))),
        ),
        ObjectPropertyKey::String(s) => PropertyKey::StringLiteral(
            cx.ast.alloc(cx.ast.string_literal(SPAN, cx.ast.atom(s), None)),
        ),
        ObjectPropertyKey::Computed(place) => {
            let expr = codegen_place_to_expression(cx, place);
            PropertyKey::from(expr)
        }
        ObjectPropertyKey::Number(n) => PropertyKey::NumericLiteral(
            cx.ast.alloc(cx.ast.numeric_literal(SPAN, *n, None, NumberBase::Decimal)),
        ),
    }
}

/// Generate call arguments.
fn codegen_args<'a>(cx: &CodegenContext<'a>, args: &[CallArg]) -> AVec<'a, Argument<'a>> {
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

/// Generate a destructure pattern as a `BindingPattern`.
///
/// Port of `codegen_pattern` — builds `BindingPattern` from HIR `Pattern`.
/// - Array: `ArrayPattern` with elements (BindingPattern or None for holes) and optional rest
/// - Object: `ObjectPattern` with properties (BindingProperty with key/value) and optional rest
fn codegen_pattern<'a>(cx: &CodegenContext<'a>, pattern: &Pattern) -> BindingPattern<'a> {
    match pattern {
        Pattern::Array(arr) => {
            let mut elements: AVec<'a, Option<BindingPattern<'a>>> =
                cx.ast.vec_with_capacity(arr.items.len());
            let mut rest: Option<oxc_allocator::Box<'a, BindingRestElement<'a>>> = None;
            for item in &arr.items {
                match item {
                    ArrayPatternElement::Place(p) => {
                        let name = identifier_name(&p.identifier);
                        let binding =
                            cx.ast.binding_pattern_binding_identifier(SPAN, cx.ast.atom(&name));
                        elements.push(Some(binding));
                    }
                    ArrayPatternElement::Spread(s) => {
                        let name = identifier_name(&s.place.identifier);
                        let binding =
                            cx.ast.binding_pattern_binding_identifier(SPAN, cx.ast.atom(&name));
                        let rest_elem = cx.ast.binding_rest_element(SPAN, binding);
                        rest = Some(cx.ast.alloc(rest_elem));
                    }
                    ArrayPatternElement::Hole => {
                        elements.push(None);
                    }
                }
            }
            let arr_pat = cx.ast.array_pattern(SPAN, elements, rest);
            BindingPattern::ArrayPattern(cx.ast.alloc(arr_pat))
        }
        Pattern::Object(obj) => {
            let mut properties: AVec<'a, BindingProperty<'a>> =
                cx.ast.vec_with_capacity(obj.properties.len());
            let mut rest: Option<oxc_allocator::Box<'a, BindingRestElement<'a>>> = None;
            for prop in &obj.properties {
                match prop {
                    ObjectPatternProperty::Property(p) => {
                        let key = codegen_object_property_key(cx, &p.key);
                        let name = identifier_name(&p.place.identifier);
                        let value =
                            cx.ast.binding_pattern_binding_identifier(SPAN, cx.ast.atom(&name));
                        let is_computed = matches!(p.key, ObjectPropertyKey::Computed(_));
                        // Check if shorthand: key is an identifier and the value has the same name
                        let is_shorthand = if !is_computed {
                            matches!(&p.key, ObjectPropertyKey::Identifier(k) if *k == name)
                        } else {
                            false
                        };
                        let binding_prop =
                            cx.ast.binding_property(SPAN, key, value, is_shorthand, is_computed);
                        properties.push(binding_prop);
                    }
                    ObjectPatternProperty::Spread(s) => {
                        let name = identifier_name(&s.place.identifier);
                        let binding =
                            cx.ast.binding_pattern_binding_identifier(SPAN, cx.ast.atom(&name));
                        let rest_elem = cx.ast.binding_rest_element(SPAN, binding);
                        rest = Some(cx.ast.alloc(rest_elem));
                    }
                }
            }
            let obj_pat = cx.ast.object_pattern(SPAN, properties, rest);
            BindingPattern::ObjectPattern(cx.ast.alloc(obj_pat))
        }
    }
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
///
/// Port of `codegenDependency` from `CodegenReactiveFunction.ts` lines 1246-1272.
///
/// When any path entry is optional, ALL entries use member expressions with the
/// `optional` flag (matching Babel's `optionalMemberExpression`). The final result
/// is wrapped in a single `ChainExpression` per ESTree convention.
fn codegen_dependency<'a>(
    cx: &CodegenContext<'a>,
    dep: &ReactiveScopeDependency,
) -> Expression<'a> {
    let name = identifier_name(&dep.identifier);
    let mut result: Expression<'a> = make_id(cx, &name);
    let has_optional = dep.path.iter().any(|e| e.optional);
    for entry in &dep.path {
        match &entry.property {
            crate::hir::types::PropertyLiteral::String(name) => {
                if has_optional {
                    // When any entry is optional, all entries use optional member expressions
                    // with their respective optional flags. This matches TS behavior where
                    // `t.optionalMemberExpression(object, property, computed, path.optional)`
                    // is called for ALL entries.
                    let ident = cx.ast.identifier_name(SPAN, cx.ast.atom(name.as_str()));
                    result = Expression::from(cx.ast.member_expression_static(
                        SPAN,
                        result,
                        ident,
                        entry.optional,
                    ));
                } else {
                    result = make_member(cx, result, name);
                }
            }
            crate::hir::types::PropertyLiteral::Number(n) => {
                let index = make_number(cx, *n as f64);
                if has_optional {
                    result = Expression::from(cx.ast.member_expression_computed(
                        SPAN,
                        result,
                        index,
                        entry.optional,
                    ));
                } else {
                    result = make_computed_member(cx, result, index);
                }
            }
        }
    }
    // If any entry was optional, wrap the entire chain in a single ChainExpression
    if has_optional {
        match expression_to_chain_element(result) {
            Some(elem) => cx.ast.expression_chain(SPAN, elem),
            None => {
                // Shouldn't happen: the loop above always produces member expressions
                make_id(cx, &name)
            }
        }
    } else {
        result
    }
}

/// Generate the init part of a for statement.
///
/// For a for-loop init like `let i = 0, length = items.length`, this processes
/// the sequence of instructions into a single `VariableDeclaration` with merged
/// declarators. For non-sequence inits, delegates to expression codegen.
///
/// Port of `codegenForInit` from `CodegenReactiveFunction.ts` lines 1194-1244.
fn codegen_for_init<'a>(cx: &mut CodegenContext<'a>, init: &ReactiveValue) -> ForStatementInit<'a> {
    if let ReactiveValue::Sequence(seq) = init {
        // Convert each instruction in the sequence to statements via codegen_block.
        // This mirrors the TS: codegenBlock(cx, init.instructions.map(i => ({kind:'instruction', instruction:i})))
        let block: Vec<ReactiveStatement> = seq
            .instructions
            .iter()
            .map(|instr| {
                ReactiveStatement::Instruction(ReactiveInstructionStatement {
                    instruction: instr.clone(),
                })
            })
            .collect();
        let body = codegen_block(cx, &block);

        let mut declarators: AVec<'a, VariableDeclarator<'a>> = cx.ast.vec();
        let mut kind = VariableDeclarationKind::Const;

        for stmt in body.into_iter() {
            // Check if this is an ExpressionStatement with an AssignmentExpression
            // that should be merged into the last declarator (handles `let i; i = 0;` pattern)
            let merged = match &stmt {
                Statement::ExpressionStatement(expr_stmt) => {
                    if let Expression::AssignmentExpression(assign) = &expr_stmt.expression {
                        if assign.operator == AssignmentOperator::Assign {
                            if let AssignmentTarget::AssignmentTargetIdentifier(left_id) =
                                &assign.left
                            {
                                // Check if last declarator has the same name and no init
                                let should_merge = declarators
                                    .last()
                                    .map(|top| {
                                        if let BindingPattern::BindingIdentifier(top_id) = &top.id {
                                            top_id.name.as_str() == left_id.name.as_str()
                                                && top.init.is_none()
                                        } else {
                                            false
                                        }
                                    })
                                    .unwrap_or(false);
                                should_merge
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if merged {
                // Extract the right-hand side of the assignment and set it as the init
                // of the last declarator. We need to destructure the statement to take ownership.
                if let Statement::ExpressionStatement(expr_stmt) = stmt {
                    let expr = expr_stmt.unbox().expression;
                    if let Expression::AssignmentExpression(assign) = expr {
                        let assign = assign.unbox();
                        if let Some(top) = declarators.last_mut() {
                            top.init = Some(assign.right);
                        }
                    }
                }
            } else {
                // Must be a VariableDeclaration with let or const
                match stmt {
                    Statement::VariableDeclaration(var_decl) => {
                        let var_decl = var_decl.unbox();
                        if var_decl.kind == VariableDeclarationKind::Let {
                            kind = VariableDeclarationKind::Let;
                        }
                        declarators.extend(var_decl.declarations);
                    }
                    _ => {
                        // Invariant: expected a variable declaration
                        // In error cases, skip this statement
                    }
                }
            }
        }

        if declarators.is_empty() {
            // Fallback: if no declarators were found, return as expression
            return ForStatementInit::from(codegen_reactive_value_to_expression(cx, init));
        }

        // Update declarator kinds to match the final kind
        for decl in declarators.iter_mut() {
            decl.kind = kind;
        }

        let decl = cx.ast.variable_declaration(SPAN, kind, declarators, false);
        ForStatementInit::VariableDeclaration(cx.ast.alloc(decl))
    } else {
        ForStatementInit::from(codegen_reactive_value_to_expression(cx, init))
    }
}

/// Extract the loop variable declaration kind and binding pattern for for-of.
///
/// Searches the test value's sequence instructions in reverse for the
/// StoreLocal/Destructure that defines the loop variable.
fn codegen_for_of_in_init<'a>(
    cx: &mut CodegenContext<'a>,
    _init: &ReactiveValue,
    test: &ReactiveValue,
) -> (VariableDeclarationKind, BindingPattern<'a>) {
    if let ReactiveValue::Sequence(seq) = test {
        for item_instr in seq.instructions.iter().rev() {
            if let ReactiveValue::Instruction(boxed) = &item_instr.value {
                match boxed.as_ref() {
                    InstructionValue::StoreLocal(store) => {
                        // Only use this StoreLocal if it has a named binding (not a temp)
                        if store.lvalue.place.identifier.name.is_some() {
                            let kind = match store.lvalue.kind {
                                InstructionKind::Const | InstructionKind::HoistedConst => {
                                    VariableDeclarationKind::Const
                                }
                                _ => VariableDeclarationKind::Let,
                            };
                            let name = identifier_name(&store.lvalue.place.identifier);
                            cx.declare(store.lvalue.place.identifier.declaration_id);
                            let binding =
                                cx.ast.binding_pattern_binding_identifier(SPAN, cx.ast.atom(&name));
                            return (kind, binding);
                        }
                    }
                    InstructionValue::Destructure(destr) => {
                        let kind = match destr.lvalue.kind {
                            InstructionKind::Const | InstructionKind::HoistedConst => {
                                VariableDeclarationKind::Const
                            }
                            _ => VariableDeclarationKind::Let,
                        };
                        let pattern = codegen_pattern(cx, &destr.lvalue.pattern);
                        return (kind, pattern);
                    }
                    _ => {}
                }
            }
        }
    }
    // Fallback
    let binding = cx.ast.binding_pattern_binding_identifier(SPAN, cx.ast.atom("item"));
    (VariableDeclarationKind::Const, binding)
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

/// Extract the loop variable declaration kind and binding pattern for for-in.
///
/// Searches the init value's sequence instructions in reverse for the
/// StoreLocal/Destructure that defines the loop variable.
fn codegen_for_in_init<'a>(
    cx: &mut CodegenContext<'a>,
    init: &ReactiveValue,
) -> (VariableDeclarationKind, BindingPattern<'a>) {
    if let ReactiveValue::Sequence(seq) = init {
        for item_instr in seq.instructions.iter().rev() {
            if let ReactiveValue::Instruction(boxed) = &item_instr.value {
                match boxed.as_ref() {
                    InstructionValue::StoreLocal(store) => {
                        let kind = match store.lvalue.kind {
                            InstructionKind::Const | InstructionKind::HoistedConst => {
                                VariableDeclarationKind::Const
                            }
                            _ => VariableDeclarationKind::Let,
                        };
                        let name = identifier_name(&store.lvalue.place.identifier);
                        cx.declare(store.lvalue.place.identifier.declaration_id);
                        let binding =
                            cx.ast.binding_pattern_binding_identifier(SPAN, cx.ast.atom(&name));
                        return (kind, binding);
                    }
                    InstructionValue::Destructure(destr) => {
                        let kind = match destr.lvalue.kind {
                            InstructionKind::Const | InstructionKind::HoistedConst => {
                                VariableDeclarationKind::Const
                            }
                            _ => VariableDeclarationKind::Let,
                        };
                        let pattern = codegen_pattern(cx, &destr.lvalue.pattern);
                        return (kind, pattern);
                    }
                    _ => {}
                }
            }
        }
    }
    // Fallback
    let binding = cx.ast.binding_pattern_binding_identifier(SPAN, cx.ast.atom("key"));
    (VariableDeclarationKind::Const, binding)
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
