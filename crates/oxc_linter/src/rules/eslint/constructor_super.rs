use oxc_ast::{
    AstKind,
    ast::{
        AssignmentOperator, ClassElement, Expression, LogicalOperator, MethodDefinitionKind,
        Statement,
    },
};
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
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{AstNode, context::LintContext, rule::Rule};

fn missing_super_all(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected to call 'super()'.")
        .with_help("Add a 'super()' call to the constructor")
        .with_label(span)
}

fn missing_super_some(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Lacked a call of 'super()' in some code paths.")
        .with_help("Ensure 'super()' is called in all code paths")
        .with_label(span)
}

fn duplicate_super(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected duplicate 'super()'.")
        .with_help("Remove the duplicate 'super()' call")
        .with_label(span)
}

fn bad_super(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected 'super()' because 'super' is not a constructor.")
        .with_help("Remove the 'super()' call or check the class declaration")
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
    nursery,
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
        let constructor_func_node = ctx.nodes().iter().find(
            |n| matches!(n.kind(), AstKind::Function(func) if func.span == constructor.value.span),
        );

        let Some(constructor_func_node) = constructor_func_node else { return };
        let constructor_block_id = ctx.nodes().cfg_id(constructor_func_node.id());

        // Check AST for LogicalExpression with super() (CFG doesn't handle these properly)
        let has_conditional_super =
            Self::has_logical_expression_super(&body.statements, node.id(), ctx);

        let (super_call_counts, super_call_spans, has_conditional_super_from_cfg) =
            Self::find_super_calls_in_cfg(cfg, constructor_block_id, node.id(), ctx);

        let has_conditional_super = has_conditional_super || has_conditional_super_from_cfg;
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
                // Conditional super() (from LogicalExpression) can't be in all paths
                if has_conditional_super {
                    ctx.diagnostic(missing_super_some(constructor.span));
                    return;
                }

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
                    // Valid paths: CalledOnce or ExitedWithoutSuper
                    let all_paths_valid = path_results.iter().all(|r| {
                        matches!(r, PathResult::CalledOnce | PathResult::ExitedWithoutSuper)
                    });

                    let some_missing =
                        path_results.iter().any(|r| matches!(r, PathResult::NoSuper));

                    if !all_paths_valid && some_missing {
                        ctx.diagnostic(missing_super_some(constructor.span));
                    }

                    if path_results.iter().any(|r| matches!(r, PathResult::CalledMultiple))
                        && super_call_spans.len() > 1
                    {
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
    /// Check if constructor body contains a LogicalExpression with super()
    /// CFG doesn't create proper instructions for logical expressions, so we check AST
    fn has_logical_expression_super(
        statements: &[Statement],
        _class_node_id: NodeId,
        _ctx: &LintContext,
    ) -> bool {
        fn check_expression(expr: &Expression) -> bool {
            match expr {
                Expression::LogicalExpression(logical) => {
                    let has_left_super = if let Expression::CallExpression(call) = &logical.left {
                        matches!(&call.callee, Expression::Super(_))
                    } else {
                        check_expression(&logical.left)
                    };

                    let has_right_super = if let Expression::CallExpression(call) = &logical.right {
                        matches!(&call.callee, Expression::Super(_))
                    } else {
                        check_expression(&logical.right)
                    };

                    has_left_super || has_right_super
                }
                Expression::ConditionalExpression(cond) => {
                    check_expression(&cond.test)
                        || check_expression(&cond.consequent)
                        || check_expression(&cond.alternate)
                }
                Expression::ParenthesizedExpression(paren) => check_expression(&paren.expression),
                _ => false,
            }
        }

        fn check_statement(stmt: &Statement) -> bool {
            match stmt {
                Statement::ExpressionStatement(expr_stmt) => {
                    check_expression(&expr_stmt.expression)
                }
                Statement::BlockStatement(block) => block.body.iter().any(|s| check_statement(s)),
                Statement::IfStatement(if_stmt) => {
                    check_statement(&if_stmt.consequent)
                        || if_stmt.alternate.as_ref().is_some_and(|alt| check_statement(alt))
                }
                _ => false,
            }
        }

        statements.iter().any(|stmt| check_statement(stmt))
    }

    /// Classify the superclass expression to determine if super() is needed/valid
    fn classify_super_class(super_class: Option<&Expression>) -> SuperClassType {
        match super_class {
            None => SuperClassType::None,
            Some(Expression::NullLiteral(_)) => SuperClassType::Null,
            Some(expr) => {
                if Self::is_invalid_super_class(expr) {
                    SuperClassType::Invalid
                } else {
                    SuperClassType::Valid
                }
            }
        }
    }

    /// Find all super() calls in CFG reachable from constructor
    /// Returns (block counts, all spans, has conditional super)
    fn find_super_calls_in_cfg(
        cfg: &ControlFlowGraph,
        constructor_block: BlockNodeId,
        class_node_id: NodeId,
        ctx: &LintContext,
    ) -> (FxHashMap<BlockNodeId, usize>, Vec<Span>, bool) {
        let mut super_call_counts = FxHashMap::default();
        let mut super_call_spans = Vec::new();
        let mut has_conditional_super = false;

        neighbors_filtered_by_edge_weight(
            &cfg.graph,
            constructor_block,
            &|edge| if matches!(edge, EdgeType::NewFunction) { Some(()) } else { None },
            &mut |block_id, (): ()| {
                let block = cfg.basic_block(*block_id);

                for instruction in block.instructions() {
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
                        AstKind::ExpressionStatement(expr_stmt) => {
                            match &expr_stmt.expression {
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
                                // Logical expressions need AST detection for proper path analysis
                                Expression::LogicalExpression(logical) => {
                                    let check_super = |expr: &Expression| -> Option<Span> {
                                        if let Expression::CallExpression(call) = expr
                                            && matches!(&call.callee, Expression::Super(_))
                                        {
                                            Some(call.span)
                                        } else {
                                            None
                                        }
                                    };

                                    if let Some(span) = check_super(&logical.left)
                                        .or_else(|| check_super(&logical.right))
                                    {
                                        has_conditional_super = true;
                                        record_super(span);
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }

                ((), true)
            },
        );

        (super_call_counts, super_call_spans, has_conditional_super)
    }

    /// Analyze CFG paths to determine super() call patterns
    fn analyze_super_paths(
        cfg: &ControlFlowGraph,
        constructor_block: BlockNodeId,
        super_call_counts: &FxHashMap<BlockNodeId, usize>,
    ) -> Vec<PathResult> {
        let mut path_results = Vec::new();
        let mut visited_in_path = FxHashSet::default();

        Self::dfs_analyze_paths(
            cfg,
            constructor_block,
            super_call_counts,
            &mut visited_in_path,
            0,
            &mut path_results,
        );

        path_results
    }

    /// DFS through CFG to track super() calls per path
    fn dfs_analyze_paths(
        cfg: &ControlFlowGraph,
        block_id: BlockNodeId,
        super_call_counts: &FxHashMap<BlockNodeId, usize>,
        visited_in_path: &mut FxHashSet<BlockNodeId>,
        super_count: usize,
        path_results: &mut Vec<PathResult>,
    ) {
        // Avoid cycles
        if visited_in_path.contains(&block_id) {
            return;
        }

        visited_in_path.insert(block_id);

        let block = cfg.basic_block(block_id);

        if block.is_unreachable() {
            visited_in_path.remove(&block_id);
            return;
        }

        let block_super_count = super_call_counts.get(&block_id).copied().unwrap_or(0);
        let new_count = super_count + block_super_count;

        let has_exit = block.instructions().iter().any(|inst| {
            matches!(
                inst.kind,
                oxc_cfg::InstructionKind::Return(_)
                    | oxc_cfg::InstructionKind::Throw
                    | oxc_cfg::InstructionKind::ImplicitReturn
            )
        });

        if has_exit {
            // Acceptable exits: throw or return with value (not implicit undefined)
            let is_acceptable_exit = block.instructions().iter().any(|inst| {
                matches!(
                    inst.kind,
                    oxc_cfg::InstructionKind::Throw
                        | oxc_cfg::InstructionKind::Return(
                            oxc_cfg::ReturnInstructionKind::NotImplicitUndefined
                        )
                )
            });

            let result = match new_count {
                0 if is_acceptable_exit => PathResult::ExitedWithoutSuper,
                0 => PathResult::NoSuper,
                1 => PathResult::CalledOnce,
                _ => PathResult::CalledMultiple,
            };
            path_results.push(result);
            visited_in_path.remove(&block_id);
            return;
        }

        let mut has_outgoing_edges = false;
        for edge in cfg.graph.edges_directed(block_id, Direction::Outgoing) {
            has_outgoing_edges = true;
            match edge.weight() {
                // Backedge: super() in loops violates "exactly once" (0 or multiple times)
                EdgeType::Backedge => {
                    let target = edge.target();
                    let current_block_has_super = super_call_counts.contains_key(&block_id);
                    let target_has_super = super_call_counts.contains_key(&target);
                    let loop_contains_super = current_block_has_super || target_has_super;

                    if loop_contains_super {
                        path_results.push(PathResult::NoSuper);
                    }
                }
                // Explicit error edges (try/catch) represent real execution paths
                EdgeType::Error(oxc_cfg::ErrorEdgeKind::Explicit) => {
                    // Use super_count from before this block (exception skips rest of try)
                    Self::dfs_analyze_paths(
                        cfg,
                        edge.target(),
                        super_call_counts,
                        visited_in_path,
                        super_count,
                        path_results,
                    );
                }
                // Don't follow these edges
                EdgeType::NewFunction
                | EdgeType::Unreachable
                | EdgeType::Error(oxc_cfg::ErrorEdgeKind::Implicit) => {}
                // Follow normal edges with accumulated count
                EdgeType::Jump | EdgeType::Normal | EdgeType::Join | EdgeType::Finalize => {
                    Self::dfs_analyze_paths(
                        cfg,
                        edge.target(),
                        super_call_counts,
                        visited_in_path,
                        new_count,
                        path_results,
                    );
                }
            }
        }

        // Dead-end block (shouldn't happen in valid CFG, but handle it)
        if !has_outgoing_edges {
            let result = match new_count {
                0 => PathResult::NoSuper,
                1 => PathResult::CalledOnce,
                _ => PathResult::CalledMultiple,
            };
            path_results.push(result);
        }

        visited_in_path.remove(&block_id);
    }

    /// Check if an expression is definitely an invalid superclass
    fn is_invalid_super_class(expr: &Expression) -> bool {
        match expr {
            Expression::ParenthesizedExpression(paren) => {
                Self::is_invalid_super_class(&paren.expression)
            }

            Expression::AssignmentExpression(assign) => {
                match assign.operator {
                    // = and &&= invalid if right side is invalid
                    AssignmentOperator::Assign | AssignmentOperator::LogicalAnd => {
                        Self::is_invalid_super_class(&assign.right)
                    }

                    // Arithmetic/bitwise assignments are invalid
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

                    // ||= and ??= valid (short-circuit to left)
                    AssignmentOperator::LogicalOr | AssignmentOperator::LogicalNullish => false,
                }
            }

            Expression::LogicalExpression(logical) => {
                match logical.operator {
                    // extends (A && B): invalid if B is invalid
                    LogicalOperator::And => Self::is_invalid_super_class(&logical.right),
                    // extends (B || 5) or (B ?? 5): valid if left could be valid
                    LogicalOperator::Or | LogicalOperator::Coalesce => false,
                }
            }

            // Conditional: valid if either branch could be valid
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
    ];

    Tester::new(ConstructorSuper::NAME, ConstructorSuper::PLUGIN, pass, fail).test_and_snapshot();
}
