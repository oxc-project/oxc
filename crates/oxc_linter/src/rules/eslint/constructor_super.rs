use std::collections::VecDeque;

use rustc_hash::FxHashMap;

use oxc_ast::{AstKind, ast::*};
use oxc_cfg::{
    BlockNodeId, ControlFlowGraph, EdgeType,
    graph::{
        Direction,
        visit::{EdgeRef, neighbors_filtered_by_edge_weight},
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn missing_super_all(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected to call `super()`.")
        .with_help("Add a `super()` call to the constructor")
        .with_label(span)
}

fn missing_super_some(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Lacked a call of `super()` in some code paths.")
        .with_help("Ensure `super()` is called in all code paths")
        .with_label(span)
}

fn duplicate_super(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected duplicate `super()`.")
        .with_help("Remove the duplicate `super()` call")
        .with_label(span)
}

fn bad_super(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected `super()` because `super` is not a constructor.")
        .with_help("Remove the `super()` call or check the class declaration")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ConstructorSuper;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires `super()` calls in constructors of derived classes and disallows `super()` calls
    /// in constructors of non-derived classes.
    ///
    /// This rule can be disabled for TypeScript code, as the TypeScript compiler
    /// enforces this check.
    ///
    /// ### Why is this bad?
    ///
    /// In JavaScript, calling `super()` in the constructor of a derived class (a class that extends
    /// another class) is required. Failing to do so will result in a ReferenceError at runtime.
    /// Conversely, calling `super()` in a non-derived class is a syntax error.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // Missing super() call
    /// class A extends B {
    ///     constructor() { }
    /// }
    ///
    /// // super() in non-derived class
    /// class A {
    ///     constructor() {
    ///         super();
    ///     }
    /// }
    ///
    /// // super() only in some code paths
    /// class C extends D {
    ///     constructor() {
    ///         if (condition) {
    ///             super();
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// // Proper super() call in derived class
    /// class A extends B {
    ///     constructor() {
    ///         super();
    ///     }
    /// }
    ///
    /// // No super() in non-derived class
    /// class A {
    ///     constructor() { }
    /// }
    ///
    /// // super() in all code paths
    /// class C extends D {
    ///     constructor() {
    ///         if (condition) {
    ///             super();
    ///         } else {
    ///             super();
    ///         }
    ///     }
    /// }
    /// ```
    ConstructorSuper,
    eslint,
    correctness,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SuperClassType {
    None,    // No extends clause
    Null,    // extends null
    Invalid, // extends <literal or invalid expression>
    Valid,   // extends <potentially valid class expression>
}

/// Result of analyzing control flow paths for super() calls
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathResult {
    /// super() was not called in this path
    NoSuper,
    /// super() was called exactly once in this path
    CalledOnce,
    /// super() was called multiple times in this path (duplicate)
    CalledMultiple,
    /// Path exited early (return/throw) without calling super()
    ExitedWithoutSuper,
}

/// Abstract state representing super() call status in dataflow analysis.
///
/// This enum forms a lattice used for iterative dataflow analysis:
/// - `Unreached`: Block has not been reached by any execution path
/// - `Never`: All paths have 0 super() calls
/// - `Once`: All paths have exactly 1 super() call
/// - `Multiple`: At least one path has 2+ super() calls (may include valid paths)
/// - `Mixed`: Some paths have 0 calls, some have 1 (represents {0, 1})
///
/// Note: This lattice doesn't distinguish `{1, 2+}` from `{2+}`. When `Mixed + 1`
/// produces `{1, 2}`, we map to `Multiple` which may over-report duplicates but
/// won't miss them. A more precise lattice would need additional states.
///
/// The `merge` operation computes the least upper bound (join) of two states,
/// and `add_super_calls` is the transfer function for executing super() calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SuperCallState {
    Unreached,
    Never,
    Once,
    Multiple,
    Mixed,
}

impl SuperCallState {
    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Unreached, s) | (s, Self::Unreached) => s,
            (Self::Never, Self::Never) => Self::Never,
            (Self::Once, Self::Once) => Self::Once,
            (Self::Multiple, _) | (_, Self::Multiple) => Self::Multiple,
            (Self::Mixed, _)
            | (_, Self::Mixed)
            | (Self::Never, Self::Once)
            | (Self::Once, Self::Never) => Self::Mixed,
        }
    }

    /// Transfer function: compute state after executing `count` super() calls.
    ///
    /// Note: When `Mixed` (representing paths with {0, 1} calls) gains a call,
    /// it becomes `Multiple` (representing {1, 2+}). This is a deliberate
    /// conservative choice: we report potential duplicates rather than track
    /// the full set of possible counts. The alternative of staying `Mixed`
    /// would incorrectly represent {1, 2+} as {0, 1}, missing valid paths.
    fn add_super_calls(self, count: usize) -> Self {
        match (self, count) {
            (Self::Unreached, _) => Self::Unreached,
            (Self::Never, 0) => Self::Never,
            (Self::Never, 1) | (Self::Once, 0) => Self::Once,
            (Self::Mixed, 0) => Self::Mixed,
            // Mixed + 1 = {0,1} + 1 = {1,2} -> Multiple (has path with 2+ calls)
            // Once + 1 = {1} + 1 = {2} -> Multiple
            // Any state + 2+ calls -> Multiple
            _ => Self::Multiple,
        }
    }
}

impl Rule for ConstructorSuper {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class) = node.kind() else { return };

        let super_class_type = Self::classify_super_class(class.super_class.as_ref());

        let Some(constructor) = class.body.body.iter().find_map(|elem| {
            if let ClassElement::MethodDefinition(method) = elem
                && matches!(method.kind, MethodDefinitionKind::Constructor)
            {
                return Some(method);
            }
            None
        }) else {
            return;
        };

        let Some(body) = &constructor.value.body else { return };

        let cfg = ctx.cfg();

        // Find constructor's CFG entry block
        // TODO: this is expensive, we should have a direct mapping from function AST to node id (and then to CFG block)
        let constructor_func_node = ctx.nodes().iter().find(
            |n| matches!(n.kind(), AstKind::Function(func) if func.span == constructor.value.span),
        );

        let Some(constructor_func_node) = constructor_func_node else { return };
        let constructor_block_id = ctx.nodes().cfg_id(constructor_func_node.id());

        let (super_call_counts, super_call_spans) =
            Self::find_super_calls_in_cfg(cfg, constructor_block_id, node.id(), ctx);

        match super_class_type {
            SuperClassType::None | SuperClassType::Invalid => {
                for &span in &super_call_spans {
                    ctx.diagnostic(bad_super(span));
                }
            }
            SuperClassType::Null => {
                // extends null: must return or will error, but super() is invalid
                if super_call_counts.is_empty() {
                    let has_return_with_value = Self::has_return_with_value(&body.statements);
                    if !has_return_with_value {
                        ctx.diagnostic(missing_super_all(constructor.span));
                    }
                } else {
                    for &span in &super_call_spans {
                        ctx.diagnostic(bad_super(span));
                    }
                }
            }
            SuperClassType::Valid => {
                if super_call_counts.is_empty() {
                    ctx.diagnostic(missing_super_all(constructor.span));
                    return;
                }

                let path_results =
                    Self::analyze_super_paths(cfg, constructor_block_id, &super_call_counts);

                if path_results.is_empty() {
                    // Treat as single path
                    if super_call_spans.len() > 1 {
                        let mut sorted_spans = super_call_spans;
                        sorted_spans.sort_by_key(|s| s.start);

                        for &span in sorted_spans.iter().skip(1) {
                            ctx.diagnostic(duplicate_super(span));
                        }
                    }
                } else {
                    // Check if super() was called on any path (once or multiple times)
                    let has_super_on_any_path = path_results
                        .iter()
                        .any(|r| matches!(r, PathResult::CalledOnce | PathResult::CalledMultiple));

                    // Check if super() is missing on some paths
                    let some_missing =
                        path_results.iter().any(|r| matches!(r, PathResult::NoSuper));

                    // Check if ALL paths exit early (no normal completion)
                    let all_paths_exit =
                        path_results.iter().all(|r| matches!(r, PathResult::ExitedWithoutSuper));

                    // Check for duplicate super() calls
                    let has_duplicate =
                        path_results.iter().any(|r| matches!(r, PathResult::CalledMultiple));

                    if !has_super_on_any_path && all_paths_exit {
                        // super() was never called and all paths exit early (like "return; super();")
                        ctx.diagnostic(missing_super_all(constructor.span));
                    } else if some_missing {
                        // super() was called on some paths but missing on others
                        ctx.diagnostic(missing_super_some(constructor.span));
                    }

                    if has_duplicate && super_call_spans.len() > 1 {
                        let mut sorted_spans = super_call_spans;
                        sorted_spans.sort_by_key(|s| s.start);

                        for &span in sorted_spans.iter().skip(1) {
                            ctx.diagnostic(duplicate_super(span));
                        }
                    }
                }
            }
        }
    }
}

impl ConstructorSuper {
    /// Classify the superclass expression to determine if super() is needed/valid
    fn classify_super_class(super_class: Option<&Expression>) -> SuperClassType {
        match super_class {
            None => SuperClassType::None,
            Some(Expression::NullLiteral(_)) => SuperClassType::Null,
            Some(expr) if Self::is_invalid_super_class(expr) => SuperClassType::Invalid,
            Some(_) => SuperClassType::Valid,
        }
    }

    /// Find all super() calls in CFG reachable from constructor
    /// Returns (block counts, all spans)
    fn find_super_calls_in_cfg(
        cfg: &ControlFlowGraph,
        constructor_block: BlockNodeId,
        class_node_id: NodeId,
        ctx: &LintContext,
    ) -> (FxHashMap<BlockNodeId, usize>, Vec<Span>) {
        let mut super_call_counts = FxHashMap::default();
        let mut super_call_spans = Vec::new();

        neighbors_filtered_by_edge_weight(
            &cfg.graph,
            constructor_block,
            &|edge| if matches!(edge, EdgeType::NewFunction) { Some(()) } else { None },
            &mut |block_id, (): ()| {
                let block = cfg.basic_block(*block_id);

                // Skip unreachable blocks (e.g., code after return/throw)
                if block.is_unreachable() {
                    return ((), true);
                }

                // Track if we've hit a return/throw - anything after is unreachable
                let mut hit_exit = false;

                for instruction in block.instructions() {
                    // Check if this instruction is an exit (return/throw)
                    if matches!(
                        instruction.kind,
                        oxc_cfg::InstructionKind::Return(_) | oxc_cfg::InstructionKind::Throw
                    ) {
                        hit_exit = true;
                        continue;
                    }

                    // Skip processing instructions after an exit
                    if hit_exit {
                        continue;
                    }

                    let Some(node_id) = instruction.node_id else { continue };

                    if Self::is_in_nested_scope_cfg(node_id, ctx, class_node_id) {
                        continue;
                    }

                    let node = ctx.nodes().get_node(node_id);

                    let mut record_super = |span: Span| {
                        *super_call_counts.entry(*block_id).or_insert(0) += 1;
                        super_call_spans.push(span);
                    };

                    match node.kind() {
                        AstKind::CallExpression(call) => {
                            if matches!(&call.callee, Expression::Super(_)) {
                                record_super(call.span);
                            }
                        }
                        AstKind::ExpressionStatement(expr_stmt) => match &expr_stmt.expression {
                            Expression::CallExpression(call) => {
                                if matches!(&call.callee, Expression::Super(_)) {
                                    record_super(call.span);
                                }
                            }
                            // Ternary: both branches in same block but only one executes
                            Expression::ConditionalExpression(cond) => {
                                let check_super = |expr: &Expression| -> Option<Span> {
                                    if let Expression::CallExpression(call) = expr
                                        && matches!(&call.callee, Expression::Super(_))
                                    {
                                        Some(call.span)
                                    } else {
                                        None
                                    }
                                };

                                if let Some(span) = check_super(&cond.consequent)
                                    .or_else(|| check_super(&cond.alternate))
                                {
                                    record_super(span);
                                }
                            }
                            // Special case: `super() || super()` - report both as duplicate.
                            //
                            // Technically, `super()` returns the constructed instance (truthy),
                            // so the RHS of `||` won't execute at runtime. However:
                            // 1. ESLint reports this as duplicate for compatibility
                            // 2. This code pattern is almost certainly a mistake
                            // 3. The CFG doesn't catch this because it sees only LHS as reachable
                            //
                            // We intentionally match ESLint's behavior here.
                            Expression::LogicalExpression(logical)
                                if matches!(logical.operator, LogicalOperator::Or) =>
                            {
                                let check_super = |expr: &Expression| -> Option<Span> {
                                    if let Expression::CallExpression(call) = expr
                                        && matches!(&call.callee, Expression::Super(_))
                                    {
                                        Some(call.span)
                                    } else {
                                        None
                                    }
                                };

                                let left_span = check_super(&logical.left);
                                let right_span = check_super(&logical.right);

                                // Report both super() calls as duplicates if both exist
                                if left_span.is_some() && right_span.is_some() {
                                    if let Some(span) = left_span {
                                        record_super(span);
                                    }
                                    if let Some(span) = right_span {
                                        record_super(span);
                                    }
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }

                ((), true)
            },
        );

        (super_call_counts, super_call_spans)
    }

    /// Analyzes control flow paths to determine super() call patterns using iterative dataflow analysis.
    ///
    /// Uses a worklist algorithm to propagate abstract states through the CFG. States represent
    /// whether super() has been called: Never, Once, Multiple, Mixed, or Unreached. The algorithm
    /// handles loops conservatively by not following backedges and marking any loop containing super()
    /// as potentially violating the "exactly once" requirement.
    ///
    /// Returns path results indicating super() call patterns for different execution paths.
    fn analyze_super_paths(
        cfg: &ControlFlowGraph,
        constructor_block: BlockNodeId,
        super_call_counts: &FxHashMap<BlockNodeId, usize>,
    ) -> Vec<PathResult> {
        let mut block_states: FxHashMap<BlockNodeId, SuperCallState> = FxHashMap::default();
        let mut loop_with_super = false;
        let mut worklist: VecDeque<BlockNodeId> = VecDeque::new();
        let mut exit_results: Vec<(SuperCallState, bool)> = Vec::new();

        block_states.insert(constructor_block, SuperCallState::Never);
        worklist.push_back(constructor_block);

        while let Some(block_id) = worklist.pop_front() {
            let current_state =
                block_states.get(&block_id).copied().unwrap_or(SuperCallState::Unreached);

            if current_state == SuperCallState::Unreached {
                continue;
            }

            let block = cfg.basic_block(block_id);

            if block.is_unreachable() {
                continue;
            }

            let block_super_count = super_call_counts.get(&block_id).copied().unwrap_or(0);
            let state_after_block = current_state.add_super_calls(block_super_count);

            let has_exit = block.instructions().iter().any(|inst| {
                matches!(
                    inst.kind,
                    oxc_cfg::InstructionKind::Return(_)
                        | oxc_cfg::InstructionKind::Throw
                        | oxc_cfg::InstructionKind::ImplicitReturn
                )
            });

            if has_exit {
                let is_acceptable_exit = block.instructions().iter().any(|inst| {
                    matches!(
                        inst.kind,
                        oxc_cfg::InstructionKind::Throw
                            | oxc_cfg::InstructionKind::Return(
                                oxc_cfg::ReturnInstructionKind::NotImplicitUndefined
                            )
                    )
                });
                exit_results.push((state_after_block, is_acceptable_exit));
                // Exit blocks are terminal - don't propagate to successors
                continue;
            }

            for edge in cfg.graph.edges_directed(block_id, Direction::Outgoing) {
                let target = edge.target();

                match edge.weight() {
                    EdgeType::Backedge => {
                        // Check if super() was called within the loop body.
                        // We need to detect super() calls in ANY block within the loop,
                        // including intermediate blocks (e.g., inside nested if statements).
                        //
                        // We check:
                        // 1. If the backedge source or loop header directly contains super()
                        // 2. If the dataflow state indicates super() was called within the
                        //    loop body by comparing the state at the loop header entry with
                        //    the state at the backedge. If the state "increased" (e.g., from
                        //    Never to Once/Mixed/Multiple), then super() was called in the loop.
                        let current_block_has_super = super_call_counts.contains_key(&block_id);
                        let target_has_super = super_call_counts.contains_key(&target);

                        // Compare loop header state with backedge state to detect super() in loop body.
                        // If the loop header already has state Once/Multiple (from before the loop),
                        // then state_after_block being Once doesn't mean super() is IN the loop.
                        // But if state_after_block is Mixed or Multiple when header was Once,
                        // or if state_after_block is Once/Mixed/Multiple when header was Never,
                        // then super() must be in the loop body.
                        let loop_header_state =
                            block_states.get(&target).copied().unwrap_or(SuperCallState::Unreached);

                        // Detect super() in loop body by checking if state increased from
                        // loop header to backedge:
                        // - Header Never -> any super state means super() called in loop
                        // - Header Once -> Multiple/Mixed means additional super() in loop
                        // - Header Mixed -> Multiple means super() called on more paths
                        let super_in_loop_body = matches!(
                            (loop_header_state, state_after_block),
                            (
                                SuperCallState::Never,
                                SuperCallState::Once
                                    | SuperCallState::Multiple
                                    | SuperCallState::Mixed
                            ) | (
                                SuperCallState::Once,
                                SuperCallState::Multiple | SuperCallState::Mixed
                            ) | (SuperCallState::Mixed, SuperCallState::Multiple)
                        );

                        if current_block_has_super || target_has_super || super_in_loop_body {
                            loop_with_super = true;
                        }
                    }

                    EdgeType::Error(oxc_cfg::ErrorEdgeKind::Explicit) => {
                        // For explicit error edges (try/catch), use pre-transfer state because
                        // exceptions can occur before any statements in the block execute.
                        // This is conservative in the "report more errors" direction, which is
                        // correct for a linter: if super() is only in a try block, we report
                        // that some paths may not call it (if an exception occurs first).
                        let old_state =
                            block_states.get(&target).copied().unwrap_or(SuperCallState::Unreached);
                        let new_state = old_state.merge(current_state);
                        if new_state != old_state {
                            block_states.insert(target, new_state);
                            worklist.push_back(target);
                        }
                    }

                    EdgeType::NewFunction
                    | EdgeType::Unreachable
                    | EdgeType::Error(oxc_cfg::ErrorEdgeKind::Implicit) => {}

                    EdgeType::Jump | EdgeType::Normal | EdgeType::Join | EdgeType::Finalize => {
                        let old_state =
                            block_states.get(&target).copied().unwrap_or(SuperCallState::Unreached);
                        let new_state = old_state.merge(state_after_block);
                        if new_state != old_state {
                            block_states.insert(target, new_state);
                            worklist.push_back(target);
                        }
                    }
                }
            }
        }

        let mut path_results = Vec::new();

        if loop_with_super {
            // A loop containing super() can execute 0 times (NoSuper) or multiple times (CalledMultiple).
            // Report both to trigger appropriate diagnostics.
            path_results.push(PathResult::NoSuper);
            path_results.push(PathResult::CalledMultiple);
        }

        for (state, is_acceptable_exit) in exit_results {
            match state {
                SuperCallState::Unreached => {}
                SuperCallState::Never => {
                    if is_acceptable_exit {
                        path_results.push(PathResult::ExitedWithoutSuper);
                    } else {
                        path_results.push(PathResult::NoSuper);
                    }
                }
                SuperCallState::Once => {
                    path_results.push(PathResult::CalledOnce);
                }
                SuperCallState::Multiple => {
                    path_results.push(PathResult::CalledMultiple);
                }
                SuperCallState::Mixed => {
                    path_results.push(PathResult::CalledOnce);
                    path_results.push(PathResult::NoSuper);
                }
            }
        }

        path_results
    }

    /// Checks if an expression is definitely an invalid superclass.
    ///
    /// Returns true for expressions that are guaranteed to be invalid as a superclass,
    /// such as literals (numbers, strings, booleans), binary expressions, and certain
    /// assignment operations. Returns false for expressions that might be valid (e.g.,
    /// identifiers, function calls, or short-circuit expressions that could evaluate
    /// to a valid class).
    fn is_invalid_super_class(expr: &Expression) -> bool {
        match expr {
            Expression::ParenthesizedExpression(paren) => {
                Self::is_invalid_super_class(&paren.expression)
            }

            Expression::AssignmentExpression(assign) => match assign.operator {
                AssignmentOperator::Assign | AssignmentOperator::LogicalAnd => {
                    Self::is_invalid_super_class(&assign.right)
                }

                AssignmentOperator::Addition
                | AssignmentOperator::Subtraction
                | AssignmentOperator::Multiplication
                | AssignmentOperator::Division
                | AssignmentOperator::Remainder
                | AssignmentOperator::ShiftLeft
                | AssignmentOperator::ShiftRight
                | AssignmentOperator::ShiftRightZeroFill
                | AssignmentOperator::BitwiseOR
                | AssignmentOperator::BitwiseXOR
                | AssignmentOperator::BitwiseAnd
                | AssignmentOperator::Exponential => true,

                AssignmentOperator::LogicalOr | AssignmentOperator::LogicalNullish => false,
            },

            Expression::LogicalExpression(logical) => match logical.operator {
                LogicalOperator::And => Self::is_invalid_super_class(&logical.right),
                LogicalOperator::Or | LogicalOperator::Coalesce => false,
            },

            Expression::ConditionalExpression(cond) => {
                Self::is_invalid_super_class(&cond.consequent)
                    && Self::is_invalid_super_class(&cond.alternate)
            }

            // Sequence: result is last expression
            Expression::SequenceExpression(seq) => {
                seq.expressions.last().is_none_or(|e| Self::is_invalid_super_class(e))
            }

            // Literals and binary expressions are invalid
            Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::BinaryExpression(_) => true,

            _ => false,
        }
    }

    /// Check if a node is inside a nested function or class
    fn is_in_nested_scope_cfg(node_id: NodeId, ctx: &LintContext, class_node_id: NodeId) -> bool {
        for ancestor in ctx.nodes().ancestors(node_id) {
            if ancestor.id() == class_node_id {
                return false;
            }

            match ancestor.kind() {
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                    // Skip if this function is the constructor itself
                    if let Some(parent) =
                        ctx.nodes().parent_node(ancestor.id()).kind().as_method_definition()
                        && matches!(parent.kind, MethodDefinitionKind::Constructor)
                    {
                        continue;
                    }
                    return true;
                }
                AstKind::Class(_) if ancestor.id() != class_node_id => return true,
                _ => {}
            }
        }
        false
    }

    /// Check if statements contain a return with value
    fn has_return_with_value(statements: &[Statement]) -> bool {
        statements.iter().any(|stmt| Self::statement_returns_value(stmt))
    }

    /// Recursively check if statement contains return with value
    fn statement_returns_value(stmt: &Statement) -> bool {
        match stmt {
            Statement::ReturnStatement(ret) => ret.argument.is_some(),
            Statement::BlockStatement(block) => Self::has_return_with_value(&block.body),
            Statement::IfStatement(if_stmt) => {
                Self::statement_returns_value(&if_stmt.consequent)
                    || if_stmt
                        .alternate
                        .as_ref()
                        .is_some_and(|alt| Self::statement_returns_value(alt))
            }
            Statement::SwitchStatement(switch) => {
                switch.cases.iter().any(|case| Self::has_return_with_value(&case.consequent))
            }
            Statement::TryStatement(try_stmt) => {
                Self::has_return_with_value(&try_stmt.block.body)
                    || try_stmt
                        .handler
                        .as_ref()
                        .is_some_and(|handler| Self::has_return_with_value(&handler.body.body))
                    || try_stmt
                        .finalizer
                        .as_ref()
                        .is_some_and(|finalizer| Self::has_return_with_value(&finalizer.body))
            }
            Statement::WhileStatement(s) => Self::statement_returns_value(&s.body),
            Statement::DoWhileStatement(s) => Self::statement_returns_value(&s.body),
            Statement::ForStatement(s) => Self::statement_returns_value(&s.body),
            Statement::ForInStatement(s) => Self::statement_returns_value(&s.body),
            Statement::ForOfStatement(s) => Self::statement_returns_value(&s.body),
            _ => false,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class A { }",
        "class A { constructor() { } }",
        "class A extends null { }",
        "class A extends B { }",
        "class A extends B { constructor() { super(); } }",
        "class A extends B { constructor() { if (true) { super(); } else { super(); } } }",
        "class A extends (class B {}) { constructor() { super(); } }",
        "class A extends (B = C) { constructor() { super(); } }",
        "class A extends (B &&= C) { constructor() { super(); } }",
        "class A extends (B ||= C) { constructor() { super(); } }",
        "class A extends (B ??= C) { constructor() { super(); } }",
        "class A extends (B ||= 5) { constructor() { super(); } }",
        "class A extends (B ??= 5) { constructor() { super(); } }",
        "class A extends (B || C) { constructor() { super(); } }",
        "class A extends (5 && B) { constructor() { super(); } }",
        "class A extends (false && B) { constructor() { super(); } }",
        "class A extends (B || 5) { constructor() { super(); } }",
        "class A extends (B ?? 5) { constructor() { super(); } }",
        "class A extends (a ? B : C) { constructor() { super(); } }",
        "class A extends (B, C) { constructor() { super(); } }",
        "class A { constructor() { class B extends C { constructor() { super(); } } } }",
        "class A extends B { constructor() { super(); class C extends D { constructor() { super(); } } } }",
        "class A extends B { constructor() { super(); class C { constructor() { } } } }",
        "class A extends B { constructor() { a ? super() : super(); } }",
        "class A extends B { constructor() { if (a) super(); else super(); } }",
        "class A extends B { constructor() { switch (a) { case 0: super(); break; default: super(); } } }",
        "class A extends B { constructor() { try {} finally { super(); } } }",
        "class A extends B { constructor() { if (a) throw Error(); super(); } }",
        "class A extends B { constructor() { if (true) return a; super(); } }",
        "class A extends null { constructor() { return a; } }",
        "class A { constructor() { return a; } }",
        "class A extends B { constructor(a) { super(); for (const b of a) { this.a(); } } }",
        "class A extends B { constructor(a) { super(); for (b in a) ( foo(b) ); } }",
        "class Foo extends Object { constructor(method) { super(); this.method = method || function() {}; } }",
        "class A extends Object {
                constructor() {
                    super();
                    for (let i = 0; i < 0; i++);
                }
            }
            ",
        "class A extends Object {
                constructor() {
                    super();
                    for (; i < 0; i++);
                }
            }
            ",
        "class A extends Object {
                constructor() {
                    super();
                    for (let i = 0;; i++) {
                        if (foo) break;
                    }
                }
            }
            ",
        "class A extends Object {
                constructor() {
                    super();
                    for (let i = 0; i < 0;);
                }
            }
            ",
        "class A extends Object {
                constructor() {
                    super();
                    for (let i = 0;;) {
                        if (foo) break;
                    }
                }
            }
            ",
        "
                        class A extends B {
                            constructor(props) {
                                super(props);

                                try {
                                    let arr = [];
                                    for (let a of arr) {
                                    }
                                } catch (err) {
                                }
                            }
                        }
                    ",
        "class A extends obj?.prop { constructor() { super(); } }",
        "
                        class A extends Base {
                            constructor(list) {
                                for (const a of list) {
                                    if (a.foo) {
                                        super(a);
                                        return;
                                    }
                                }
                                super();
                            }
                        }
                    ",
        // super() before potential throw - should pass since super() is called before any exception
        "class A extends B { constructor() { super(); try { throw new Error(); } catch (e) {} } }",
        // super() in finally always executes
        "class A extends B { constructor() { try { mayThrow(); } catch (e) {} finally { super(); } } }",
    ];

    let fail = vec![
        "class A extends null { constructor() { super(); } }",
        "class A extends null { constructor() { } }",
        "class A extends 100 { constructor() { super(); } }",
        "class A extends 'test' { constructor() { super(); } }",
        "class A extends (B = 5) { constructor() { super(); } }",
        "class A extends (B && 5) { constructor() { super(); } }",
        "class A extends (B &&= 5) { constructor() { super(); } }",
        "class A extends (B += C) { constructor() { super(); } }",
        "class A extends (B -= C) { constructor() { super(); } }",
        "class A extends (B **= C) { constructor() { super(); } }",
        "class A extends (B |= C) { constructor() { super(); } }",
        "class A extends (B &= C) { constructor() { super(); } }",
        "class A extends B { constructor() { } }",
        "class A extends B { constructor() { for (var a of b) super.foo(); } }",
        "class A extends B { constructor() { for (var i = 1; i < 10; i++) super.foo(); } }",
        "class A extends B { constructor() { var c = class extends D { constructor() { super(); } } } }",
        "class A extends B { constructor() { var c = () => super(); } }",
        "class A extends B { constructor() { class C extends D { constructor() { super(); } } } }",
        "class A extends B { constructor() { var C = class extends D { constructor() { super(); } } } }",
        "class A extends B { constructor() { super(); class C extends D { constructor() { } } } }",
        "class A extends B { constructor() { super(); var C = class extends D { constructor() { } } } }",
        "class A extends B { constructor() { if (a) super(); } }",
        "class A extends B { constructor() { if (a); else super(); } }",
        "class A extends B { constructor() { a && super(); } }",
        "class A extends B { constructor() { switch (a) { case 0: super(); } } }",
        "class A extends B { constructor() { switch (a) { case 0: break; default: super(); } } }",
        "class A extends B { constructor() { try { super(); } catch (err) {} } }",
        "class A extends B { constructor() { try { a; } catch (err) { super(); } } }",
        "class A extends B { constructor() { if (a) return; super(); } }",
        "class A extends B { constructor() { super(); super(); } }",
        "class A extends B { constructor() { super() || super(); } }",
        "class A extends B { constructor() { if (a) super(); super(); } }",
        "class A extends B { constructor() { switch (a) { case 0: super(); default: super(); } } }",
        "class A extends B { constructor(a) { while (a) super(); } }",
        "class A extends B { constructor() { return; super(); } }",
        "class Foo extends Bar {
                            constructor() {
                                for (a in b) for (c in d);
                            }
                        }",
        "class C extends D {

                            constructor() {
                                do {
                                    something();
                                } while (foo);
                            }

                        }",
        "class C extends D {

                            constructor() {
                                for (let i = 1;;i++) {
                                    if (bar) {
                                        break;
                                    }
                                }
                            }

                        }",
        "class C extends D {

                            constructor() {
                                do {
                                    super();
                                } while (foo);
                            }

                        }",
        "class C extends D {

                            constructor() {
                                while (foo) {
                                    if (bar) {
                                        super();
                                        break;
                                    }
                                }
                            }

                        }",
        // Loop with super() in intermediate block (not in backedge source or loop header)
        "class A extends B {
            constructor() {
                while (condition) {
                    if (x) { super(); }
                }
            }
        }",
        // For loop with super() in nested if
        "class A extends B {
            constructor() {
                for (let i = 0; i < 10; i++) {
                    if (i === 5) { super(); }
                }
            }
        }",
        // Do-while with super() in nested block
        "class A extends B {
            constructor() {
                do {
                    if (x) { super(); }
                } while (condition);
            }
        }",
    ];

    Tester::new(ConstructorSuper::NAME, ConstructorSuper::PLUGIN, pass, fail).test_and_snapshot();
}
