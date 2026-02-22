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
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{CompilerError, SourceLocation},
    hir::{
        ArrayExpressionElement, ArrayPatternElement, CallArg, DeclarationId, IdentifierId,
        IdentifierName, InstructionKind, InstructionValue, JsxAttribute, JsxTag,
        ObjectPatternProperty, ObjectPropertyKey, ObjectPropertyType, Pattern, Place,
        PrimitiveValueKind, ReactiveBlock, ReactiveBreakTerminal, ReactiveContinueTerminal,
        ReactiveFunction, ReactiveInstruction, ReactiveScope, ReactiveScopeDeclaration,
        ReactiveScopeDependency, ReactiveStatement, ReactiveTerminal, ReactiveTerminalTargetKind,
        ReactiveValue,
    },
};

use super::visitors::{ReactiveVisitor, visit_reactive_block};

/// Sentinel values used in the output code.
pub const MEMO_CACHE_SENTINEL: &str = "react.memo_cache_sentinel";
pub const EARLY_RETURN_SENTINEL: &str = "react.early_return_sentinel";

// =====================================================================================
// CodegenFunction — the final output of code generation
// =====================================================================================

/// Result of code generation.
#[derive(Debug)]
pub struct CodegenFunction {
    /// Name of the function (if it had one).
    pub id: Option<String>,
    /// Name hint for anonymous functions.
    pub name_hint: Option<String>,
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
#[derive(Debug)]
pub struct OutlinedFunction {
    pub fn_: CodegenFunction,
    pub fn_type: crate::hir::ReactFunctionType,
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
    Return(Option<String>),
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
    /// A try statement: `try { block } catch (param) { handler }`
    Try {
        block: Vec<CodegenStatement>,
        handler_param: Option<String>,
        handler: Vec<CodegenStatement>,
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
}

impl CodegenContext {
    fn new(unique_identifiers: FxHashSet<String>, fbt_operands: FxHashSet<IdentifierId>) -> Self {
        Self {
            next_cache_index: 0,
            declarations: FxHashSet::default(),
            temp: FxHashMap::default(),
            unique_identifiers,
            synthesized_names: FxHashMap::default(),
            fbt_operands,
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
    let mut cx = CodegenContext::new(options.unique_identifiers, options.fbt_operands);

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
    if matches!(body.last(), Some(CodegenStatement::Return(None))) {
        body.pop();
    }

    // Count memo blocks/values
    let mut counter = MemoCounter::default();
    visit_reactive_block(&reactive_fn.body, &mut counter);

    // Insert the `const $ = _c(N);` preamble if there are cache slots
    let cache_count = cx.next_cache_index;
    if cache_count > 0 {
        let cache_name = cx.synthesize_name("$");
        let init_stmt = CodegenStatement::VariableDeclaration {
            kind: VarKind::Const,
            name: cache_name,
            init: Some(format!("_c({cache_count})")),
        };
        body.insert(0, init_stmt);
    }

    Ok(CodegenFunction {
        id: reactive_fn.id.clone(),
        name_hint: reactive_fn.name_hint.clone(),
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

    // Process declarations: sorted for determinism
    let mut decls: Vec<(&IdentifierId, &ReactiveScopeDeclaration)> =
        scope.declarations.iter().collect();
    decls.sort_by(|(_, a), (_, b)| compare_scope_declaration(a, b));

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

    // Process reassignments
    for reassignment_id in &scope.reassignments {
        let index = cx.alloc_cache_index();
        if first_output_index.is_none() {
            first_output_index = Some(index);
        }
        // reassignment_id is an IdentifierId — we need to find the name
        // For reassignments, we just use the id as a fallback name
        let name = format!("t{}", reassignment_id.0);
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
            consequent: vec![CodegenStatement::Return(Some(name))],
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
            if value == "undefined" {
                Some(CodegenStatement::Return(None))
            } else {
                Some(CodegenStatement::Return(Some(value)))
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
            let update = t.update.as_ref().map(|u| codegen_reactive_value_to_expression(cx, u));
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
            Some(CodegenStatement::Block(block))
        }
        ReactiveTerminal::Try(t) => {
            let block = codegen_block(cx, &t.block);
            let handler_param = t.handler_binding.as_ref().map(|binding| {
                let name = identifier_name(&binding.identifier);
                cx.temp.insert(binding.identifier.declaration_id, None);
                name
            });
            let handler = codegen_block(cx, &t.handler);
            Some(CodegenStatement::Try { block, handler_param, handler })
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
                let kind = if cx.has_declared(store.lvalue.place.identifier.declaration_id) {
                    InstructionKind::Reassign
                } else {
                    store.lvalue.kind
                };
                let value_expr = codegen_place_to_expression(cx, &store.value);
                codegen_store_or_declare(cx, instr, kind, &store.lvalue.place, Some(&value_expr))
            }
            InstructionValue::StoreContext(store) => {
                let kind = store.lvalue_kind;
                let value_expr = codegen_place_to_expression(cx, &store.value);
                codegen_store_or_declare(cx, instr, kind, &store.lvalue_place, Some(&value_expr))
            }
            InstructionValue::DeclareLocal(decl) => {
                if cx.has_declared(decl.lvalue.place.identifier.declaration_id) {
                    return None;
                }
                codegen_store_or_declare(cx, instr, decl.lvalue.kind, &decl.lvalue.place, None)
            }
            InstructionValue::DeclareContext(decl) => {
                if cx.has_declared(decl.lvalue_place.identifier.declaration_id) {
                    return None;
                }
                codegen_store_or_declare(cx, instr, decl.lvalue_kind, &decl.lvalue_place, None)
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
            InstructionValue::ObjectMethod(_) => {
                // Object methods are stored for later use by ObjectExpression codegen
                // No statement emitted here
                None
            }
            other => {
                let value_str = codegen_instruction_value(cx, other);
                codegen_instruction_to_statement(cx, instr, &value_str)
            }
        },
        ReactiveValue::Logical(_)
        | ReactiveValue::Ternary(_)
        | ReactiveValue::Sequence(_)
        | ReactiveValue::OptionalCall(_) => {
            let value_str = codegen_reactive_value_to_expression(cx, &instr.value);
            codegen_instruction_to_statement(cx, instr, &value_str)
        }
    }
}

/// Handle StoreLocal/StoreContext/DeclareLocal/DeclareContext
fn codegen_store_or_declare(
    cx: &mut CodegenContext,
    instr: &ReactiveInstruction,
    kind: InstructionKind,
    lvalue_place: &Place,
    value: Option<&str>,
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
            // Emit as const for function declarations in codegen IR
            Some(CodegenStatement::VariableDeclaration {
                kind: VarKind::Const,
                name,
                init: value.map(String::from),
            })
        }
        InstructionKind::Reassign => {
            if let Some(val) = value {
                let assign_expr = format!("{name} = {val}");
                // If there's an lvalue on the instruction (i.e., it's used as an expression),
                // store as temporary
                return try_store_as_temporary(cx, instr.lvalue.as_ref(), assign_expr);
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
            let assign_expr = format!("{lval} = {value}");
            try_store_as_temporary(cx, instr.lvalue.as_ref(), assign_expr)
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
fn codegen_instruction_value(cx: &CodegenContext, value: &InstructionValue) -> String {
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
            // JSX text: emit as string literal
            format!("\"{}\"", escape_string(&text.value))
        }
        InstructionValue::CallExpression(call) => {
            let callee = codegen_place_to_expression(cx, &call.callee);
            let args = codegen_args(cx, &call.args);
            format!("{callee}({args})")
        }
        InstructionValue::MethodCall(method) => {
            let property_expr = codegen_place_to_expression(cx, &method.property);
            let args = codegen_args(cx, &method.args);
            format!("{property_expr}({args})")
        }
        InstructionValue::NewExpression(new) => {
            let callee = codegen_place_to_expression(cx, &new.callee);
            let args = codegen_args(cx, &new.args);
            format!("new {callee}({args})")
        }
        InstructionValue::ObjectExpression(obj) => {
            let props: Vec<String> = obj
                .properties
                .iter()
                .map(|prop| match prop {
                    ObjectPatternProperty::Property(p) => {
                        let key = codegen_object_property_key(&p.key);
                        let value = codegen_place_to_expression(cx, &p.place);
                        let is_computed = matches!(p.key, ObjectPropertyKey::Computed(_));
                        // Check for shorthand: key matches value
                        let is_shorthand =
                            matches!(&p.key, ObjectPropertyKey::Identifier(k) if *k == value);
                        if p.property_type == ObjectPropertyType::Method {
                            // Method shorthand
                            format!("{key}(...)")
                        } else if is_shorthand && !is_computed {
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
            format!("{{{}}}", props.join(", "))
        }
        InstructionValue::PropertyLoad(load) => {
            let object = codegen_place_to_expression(cx, &load.object);
            codegen_member_access(&object, &load.property)
        }
        InstructionValue::PropertyStore(store) => {
            let object = codegen_place_to_expression(cx, &store.object);
            let member = codegen_member_access(&object, &store.property);
            let value = codegen_place_to_expression(cx, &store.value);
            format!("{member} = {value}")
        }
        InstructionValue::PropertyDelete(del) => {
            let object = codegen_place_to_expression(cx, &del.object);
            let member = codegen_member_access(&object, &del.property);
            format!("delete {member}")
        }
        InstructionValue::ComputedLoad(load) => {
            let object = codegen_place_to_expression(cx, &load.object);
            let property = codegen_place_to_expression(cx, &load.property);
            format!("{object}[{property}]")
        }
        InstructionValue::ComputedStore(store) => {
            let object = codegen_place_to_expression(cx, &store.object);
            let property = codegen_place_to_expression(cx, &store.property);
            let value = codegen_place_to_expression(cx, &store.value);
            format!("{object}[{property}] = {value}")
        }
        InstructionValue::ComputedDelete(del) => {
            let object = codegen_place_to_expression(cx, &del.object);
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
            // Simplified: emit as a function expression string
            let name = func_expr.name.as_deref().unwrap_or("");
            let is_async = if func_expr.lowered_func.func.is_async { "async " } else { "" };
            let star = if func_expr.lowered_func.func.generator { "*" } else { "" };
            match func_expr.expression_type {
                crate::hir::FunctionExpressionType::ArrowFunctionExpression => {
                    format!("{is_async}(...) => {{}}")
                }
                crate::hir::FunctionExpressionType::FunctionExpression
                | crate::hir::FunctionExpressionType::FunctionDeclaration => {
                    format!("{is_async}function{star} {name}(...) {{}}")
                }
            }
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
            format!("`{tag}`{}", tagged.value.raw)
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
            let attrs: Vec<String> = jsx
                .props
                .iter()
                .map(|attr| match attr {
                    JsxAttribute::Attribute { name, place } => {
                        let value = codegen_place_to_expression(cx, place);
                        format!("{name}={{{value}}}")
                    }
                    JsxAttribute::Spread { argument } => {
                        format!("{{...{}}}", codegen_place_to_expression(cx, argument))
                    }
                })
                .collect();
            let attrs_str =
                if attrs.is_empty() { String::new() } else { format!(" {}", attrs.join(" ")) };
            match &jsx.children {
                None => format!("<{tag_str}{attrs_str} />"),
                Some(children) if children.is_empty() => {
                    format!("<{tag_str}{attrs_str}></{tag_str}>")
                }
                Some(children) => {
                    let children_str: Vec<String> = children
                        .iter()
                        .map(|c| format!("{{{}}}", codegen_place_to_expression(cx, c)))
                        .collect();
                    format!("<{tag_str}{attrs_str}>{}</{tag_str}>", children_str.join(""))
                }
            }
        }
        InstructionValue::JsxFragment(frag) => {
            let children_str: Vec<String> = frag
                .children
                .iter()
                .map(|c| format!("{{{}}}", codegen_place_to_expression(cx, c)))
                .collect();
            format!("<>{}</>", children_str.join(""))
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
        // These should not appear in codegen_instruction_value
        InstructionValue::StoreLocal(store) => {
            // StoreLocal in expression context means it's a reassignment
            let lval = codegen_place_to_expression(cx, &store.lvalue.place);
            let value = codegen_place_to_expression(cx, &store.value);
            format!("{lval} = {value}")
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
// codegen_reactive_value_to_expression — ReactiveValue → expression string
// =====================================================================================

/// Convert a `ReactiveValue` to an expression string.
fn codegen_reactive_value_to_expression(cx: &mut CodegenContext, value: &ReactiveValue) -> String {
    match value {
        ReactiveValue::Instruction(boxed) => codegen_instruction_value(cx, boxed),
        ReactiveValue::Logical(logical) => {
            let left = codegen_reactive_value_to_expression(cx, &logical.left);
            let right = codegen_reactive_value_to_expression(cx, &logical.right);
            format!("{left} {} {right}", logical.operator.as_str())
        }
        ReactiveValue::Ternary(ternary) => {
            let test = codegen_reactive_value_to_expression(cx, &ternary.test);
            let consequent = codegen_reactive_value_to_expression(cx, &ternary.consequent);
            let alternate = codegen_reactive_value_to_expression(cx, &ternary.alternate);
            format!("{test} ? {consequent} : {alternate}")
        }
        ReactiveValue::Sequence(seq) => {
            // Process sequence instructions (they may create temporaries)
            let stmts: Vec<CodegenStatement> = seq
                .instructions
                .iter()
                .filter_map(|instr| codegen_instruction_nullable(cx, instr))
                .collect();

            let expressions: Vec<String> = stmts
                .into_iter()
                .filter_map(|stmt| match stmt {
                    CodegenStatement::ExpressionStatement(expr) => Some(expr),
                    _ => None,
                })
                .collect();

            let final_value = codegen_reactive_value_to_expression(cx, &seq.value);
            if expressions.is_empty() {
                final_value
            } else {
                let mut all = expressions;
                all.push(final_value);
                format!("({})", all.join(", "))
            }
        }
        ReactiveValue::OptionalCall(optional) => {
            let value = codegen_reactive_value_to_expression(cx, &optional.value);
            if optional.optional { format!("{value}?.") } else { value }
        }
    }
}

// =====================================================================================
// Helper functions
// =====================================================================================

/// Convert a Place to an expression string.
fn codegen_place_to_expression(cx: &CodegenContext, place: &Place) -> String {
    // Check if this place is a temporary with a stored expression
    if let Some(Some(expr)) = cx.temp.get(&place.identifier.declaration_id) {
        return expr.clone();
    }

    // Must be a named identifier
    identifier_name(&place.identifier)
}

/// Get the string name of an identifier.
fn identifier_name(identifier: &crate::hir::Identifier) -> String {
    match &identifier.name {
        Some(IdentifierName::Named(name) | IdentifierName::Promoted(name)) => name.clone(),
        None => format!("t${}", identifier.id.0),
    }
}

/// Generate a label string from a BlockId.
fn codegen_label(id: crate::hir::BlockId) -> String {
    format!("bb{}", id.0)
}

/// Generate a primitive value expression string.
fn codegen_primitive(value: &PrimitiveValueKind) -> String {
    match value {
        PrimitiveValueKind::Number(n) => {
            if *n < 0.0 {
                format!("-{}", -n)
            } else {
                format!("{n}")
            }
        }
        PrimitiveValueKind::Boolean(b) => format!("{b}"),
        PrimitiveValueKind::String(s) => format!("\"{}\"", escape_string(s)),
        PrimitiveValueKind::Null => "null".to_string(),
        PrimitiveValueKind::Undefined => "undefined".to_string(),
    }
}

/// Escape special characters in a string literal.
fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result
}

/// Generate a member access expression.
fn codegen_member_access(object: &str, property: &crate::hir::types::PropertyLiteral) -> String {
    match property {
        crate::hir::types::PropertyLiteral::String(name) => {
            format!("{object}.{name}")
        }
        crate::hir::types::PropertyLiteral::Number(n) => {
            format!("{object}[{n}]")
        }
    }
}

/// Generate an object property key string.
fn codegen_object_property_key(key: &ObjectPropertyKey) -> String {
    match key {
        ObjectPropertyKey::String(s) => format!("\"{}\"", escape_string(s)),
        ObjectPropertyKey::Identifier(name) => name.clone(),
        ObjectPropertyKey::Computed(place) => {
            // Computed keys: we'd need a context here, but for codegen IR
            // we represent it as the place name
            format!("t${}", place.identifier.id.0)
        }
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
                        let key = codegen_object_property_key(&p.key);
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
            format!("{{{}}}", props.join(", "))
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
    let mut object = format!("t${}", dep.identifier_id.0);
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
            // Process sequence instructions to build variable declarations
            let mut parts: Vec<String> = Vec::new();
            for instr in &seq.instructions {
                if let Some(stmt) = codegen_instruction_nullable(cx, instr) {
                    match stmt {
                        CodegenStatement::VariableDeclaration { kind, name, init } => {
                            let init_str = init.map_or(String::new(), |i| format!(" = {i}"));
                            parts.push(format!("{} {name}{init_str}", kind.as_str()));
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
    // The test value for for-of contains the iterator item assignment
    if let ReactiveValue::Sequence(seq) = test
        && let Some(item_instr) = seq.instructions.get(1)
        && let ReactiveValue::Instruction(boxed) = &item_instr.value
    {
        match boxed.as_ref() {
            InstructionValue::StoreLocal(store) => {
                let kind = match store.lvalue.kind {
                    InstructionKind::Const | InstructionKind::HoistedConst => VarKind::Const,
                    _ => VarKind::Let,
                };
                let name = identifier_name(&store.lvalue.place.identifier);
                cx.declare(store.lvalue.place.identifier.declaration_id);
                return (kind, name);
            }
            InstructionValue::Destructure(destr) => {
                let kind = match destr.lvalue.kind {
                    InstructionKind::Const | InstructionKind::HoistedConst => VarKind::Const,
                    _ => VarKind::Let,
                };
                let pattern = codegen_pattern(cx, &destr.lvalue.pattern);
                return (kind, pattern);
            }
            _ => {}
        }
    }
    (VarKind::Const, "item".to_string())
}

/// Extract the collection expression for for-of from the init value.
fn codegen_for_of_collection(cx: &mut CodegenContext, init: &ReactiveValue) -> String {
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
fn codegen_for_in_init(cx: &mut CodegenContext, init: &ReactiveValue) -> (VarKind, String) {
    if let ReactiveValue::Sequence(seq) = init
        && let Some(item_instr) = seq.instructions.get(1)
        && let ReactiveValue::Instruction(boxed) = &item_instr.value
    {
        match boxed.as_ref() {
            InstructionValue::StoreLocal(store) => {
                let kind = match store.lvalue.kind {
                    InstructionKind::Const | InstructionKind::HoistedConst => VarKind::Const,
                    _ => VarKind::Let,
                };
                let name = identifier_name(&store.lvalue.place.identifier);
                cx.declare(store.lvalue.place.identifier.declaration_id);
                return (kind, name);
            }
            InstructionValue::Destructure(destr) => {
                let kind = match destr.lvalue.kind {
                    InstructionKind::Const | InstructionKind::HoistedConst => VarKind::Const,
                    _ => VarKind::Let,
                };
                let pattern = codegen_pattern(cx, &destr.lvalue.pattern);
                return (kind, pattern);
            }
            _ => {}
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
fn compare_scope_dependency(
    a: &ReactiveScopeDependency,
    b: &ReactiveScopeDependency,
) -> std::cmp::Ordering {
    let a_name = format!("t${}", a.identifier_id.0);
    let b_name = format!("t${}", b.identifier_id.0);
    a_name.cmp(&b_name)
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

fn write_statement(
    f: &mut std::fmt::Formatter<'_>,
    stmt: &CodegenStatement,
    indent: usize,
) -> std::fmt::Result {
    let pad = "  ".repeat(indent);
    match stmt {
        CodegenStatement::VariableDeclaration { kind, name, init } => match init {
            Some(val) => writeln!(f, "{pad}{} {name} = {val};", kind.as_str()),
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
        CodegenStatement::Return(arg) => match arg {
            Some(val) => writeln!(f, "{pad}return {val};"),
            None => writeln!(f, "{pad}return;"),
        },
        CodegenStatement::For { init, test, update, body } => {
            let init_str = init.as_deref().unwrap_or("");
            let test_str = test.as_deref().unwrap_or("");
            let update_str = update.as_deref().unwrap_or("");
            writeln!(f, "{pad}for ({init_str}; {test_str}; {update_str}) {{")?;
            for s in body {
                write_statement(f, s, indent + 1)?;
            }
            writeln!(f, "{pad}}}")
        }
        CodegenStatement::ForOf { kind, left, right, body } => {
            writeln!(f, "{pad}for ({} {left} of {right}) {{", kind.as_str())?;
            for s in body {
                write_statement(f, s, indent + 1)?;
            }
            writeln!(f, "{pad}}}")
        }
        CodegenStatement::ForIn { kind, left, right, body } => {
            writeln!(f, "{pad}for ({} {left} in {right}) {{", kind.as_str())?;
            for s in body {
                write_statement(f, s, indent + 1)?;
            }
            writeln!(f, "{pad}}}")
        }
        CodegenStatement::While { test, body } => {
            writeln!(f, "{pad}while ({test}) {{")?;
            for s in body {
                write_statement(f, s, indent + 1)?;
            }
            writeln!(f, "{pad}}}")
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
        CodegenStatement::Try { block, handler_param, handler } => {
            writeln!(f, "{pad}try {{")?;
            for s in block {
                write_statement(f, s, indent + 1)?;
            }
            match handler_param {
                Some(param) => writeln!(f, "{pad}}} catch ({param}) {{")?,
                None => writeln!(f, "{pad}}} catch {{")?,
            }
            for s in handler {
                write_statement(f, s, indent + 1)?;
            }
            writeln!(f, "{pad}}}")
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
