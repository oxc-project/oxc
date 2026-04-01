//! Flow graph construction from AST.
//!
//! Walks a function body (or program top-level) and emits flow nodes for
//! conditions, assignments, branches, and loops. The resulting `FlowGraph`
//! is consumed by the backward walk in `flow_analysis.rs`.

use oxc_ast::ast::{
    AssignmentTarget, Expression, Statement,
};
use oxc_index::IndexVec;
use rustc_hash::FxHashMap;
use oxc_syntax::node::NodeId;
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator};
use oxc_syntax::symbol::SymbolId;
use smallvec::SmallVec;

use crate::flow::{CacheState, FlowEntry, FlowGraph, FlowNodeId, FlowNodeKind};

/// Builds a flow graph from an AST function body or program statements.
pub struct FlowGraphBuilder<'a, 'b> {
    /// Flow node storage.
    nodes: IndexVec<FlowNodeId, FlowEntry>,
    /// Maps AST NodeId → FlowNodeId for identifier references.
    node_flow_map: FxHashMap<NodeId, FlowNodeId>,
    /// Current flow position (advances as we walk statements).
    current_flow: FlowNodeId,
    /// The start node.
    start: FlowNodeId,
    /// The unreachable sentinel.
    unreachable: FlowNodeId,
    /// Break target stack (for loops and switch).
    break_targets: Vec<FlowNodeId>,
    /// Continue target stack (for loops).
    continue_targets: Vec<FlowNodeId>,
    /// Semantic analysis for resolving symbol IDs from references.
    semantic: &'a oxc_semantic::Semantic<'b>,
}

impl<'a, 'b> FlowGraphBuilder<'a, 'b> {
    /// Build a flow graph for a list of statements (program top-level or function body).
    pub fn build(
        stmts: &[Statement<'_>],
        semantic: &'a oxc_semantic::Semantic<'b>,
    ) -> FlowGraph {
        let mut builder = Self::new(semantic);
        builder.visit_statements(stmts);
        builder.finish()
    }

    fn new(semantic: &'a oxc_semantic::Semantic<'b>) -> Self {
        let mut nodes = IndexVec::new();

        let start = nodes.push(FlowEntry {
            cache_state: CacheState::None,
            kind: FlowNodeKind::Start,
        });
        let unreachable = nodes.push(FlowEntry {
            cache_state: CacheState::None,
            kind: FlowNodeKind::Unreachable,
        });

        Self {
            nodes,
            node_flow_map: FxHashMap::default(),
            current_flow: start,
            start,
            unreachable,
            break_targets: Vec::new(),
            continue_targets: Vec::new(),
            semantic,
        }
    }

    fn finish(self) -> FlowGraph {
        FlowGraph {
            nodes: self.nodes,
            node_flow_map: self.node_flow_map,
            start: self.start,
            unreachable: self.unreachable,
            end_of_flow: self.current_flow,
        }
    }

    // ── Node creation helpers ──────────────────────────────────────────

    fn push_node(&mut self, kind: FlowNodeKind) -> FlowNodeId {
        self.nodes.push(FlowEntry {
            cache_state: CacheState::None,
            kind,
        })
    }

    /// Create a branch label (non-looping merge point).
    fn new_branch_label(&mut self) -> FlowNodeId {
        self.push_node(FlowNodeKind::BranchLabel {
            antecedents: SmallVec::new(),
        })
    }

    /// Create a loop label (looping merge point — back-edge target).
    fn new_loop_label(&mut self) -> FlowNodeId {
        self.push_node(FlowNodeKind::LoopLabel {
            antecedents: SmallVec::new(),
        })
    }

    /// Add an antecedent to a label node (branch or loop).
    fn add_antecedent(&mut self, label: FlowNodeId, flow: FlowNodeId) {
        // Don't add unreachable antecedents.
        if matches!(self.nodes[flow].kind, FlowNodeKind::Unreachable) {
            return;
        }
        // Track whether this flow node is shared (used as antecedent 2+ times).
        // This drives the caching heuristic in the backward walk.
        let flow_entry = &mut self.nodes[flow];
        match flow_entry.cache_state {
            CacheState::None => flow_entry.cache_state = CacheState::Referenced,
            CacheState::Referenced => flow_entry.cache_state = CacheState::Shared,
            CacheState::Shared => {}
        }
        let antecedents = match &mut self.nodes[label].kind {
            FlowNodeKind::BranchLabel { antecedents } | FlowNodeKind::LoopLabel { antecedents } => antecedents,
            _ => unreachable!("add_antecedent called on non-label node"),
        };
        // Avoid duplicates.
        if !antecedents.contains(&flow) {
            antecedents.push(flow);
        }
    }

    /// Finalize a label: if it has 0 antecedents return unreachable,
    /// if 1 return that antecedent directly, otherwise return the label itself.
    fn finish_label(&self, label: FlowNodeId) -> FlowNodeId {
        let antecedents = match &self.nodes[label].kind {
            FlowNodeKind::BranchLabel { antecedents } | FlowNodeKind::LoopLabel { antecedents } => antecedents,
            _ => unreachable!("finish_label called on non-label node"),
        };
        match antecedents.len() {
            0 => self.unreachable,
            1 => antecedents[0],
            _ => label,
        }
    }

    /// Create TrueCondition/FalseCondition pair if the expression is narrowing,
    /// otherwise return the antecedent for both branches.
    fn maybe_create_condition_pair(
        &mut self,
        expr: &Expression<'_>,
        antecedent: FlowNodeId,
    ) -> (FlowNodeId, FlowNodeId) {
        if self.is_narrowing_expression(expr) {
            let node_id = self.get_expression_node_id(expr);
            let true_node = self.push_node(FlowNodeKind::TrueCondition {
                node_id,
                antecedent,
            });
            let false_node = self.push_node(FlowNodeKind::FalseCondition {
                node_id,
                antecedent,
            });
            (true_node, false_node)
        } else {
            (antecedent, antecedent)
        }
    }

    /// Returns true if the current flow is reachable.
    fn is_reachable(&self) -> bool {
        !matches!(self.nodes[self.current_flow].kind, FlowNodeKind::Unreachable)
    }

    /// Record the current flow position for an AST node (identifier reference).
    fn set_node_flow(&mut self, node_id: NodeId) {
        self.node_flow_map.insert(node_id, self.current_flow);
    }

    // ── Statement visitors ─────────────────────────────────────────────

    fn visit_statements(&mut self, stmts: &[Statement<'_>]) {
        for stmt in stmts {
            self.visit_statement(stmt);
        }
    }

    fn visit_statement(&mut self, stmt: &Statement<'_>) {
        match stmt {
            Statement::VariableDeclaration(decl) => {
                for declarator in &decl.declarations {
                    // Visit the initializer for any narrowing expressions
                    if let Some(init) = &declarator.init {
                        self.visit_expression_for_flow(init);
                    }
                    // Record the assignment to this variable
                    if let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &declarator.id {
                        if let Some(symbol_id) = id.symbol_id.get() {
                            self.current_flow = self.push_node(FlowNodeKind::Assignment {
                                node_id: id.node_id.get(),
                                symbol_id,
                                antecedent: self.current_flow,
                            });
                        }
                    }
                }
            }

            Statement::ExpressionStatement(expr_stmt) => {
                self.visit_expression_for_flow(&expr_stmt.expression);
            }

            Statement::IfStatement(if_stmt) => {
                self.visit_if_statement(if_stmt);
            }

            Statement::WhileStatement(while_stmt) => {
                self.visit_while_statement(while_stmt);
            }

            Statement::ForStatement(for_stmt) => {
                self.visit_for_statement(for_stmt);
            }

            Statement::DoWhileStatement(do_while) => {
                self.visit_do_while_statement(do_while);
            }

            Statement::SwitchStatement(switch_stmt) => {
                self.visit_switch_statement(switch_stmt);
            }

            Statement::BlockStatement(block) => {
                self.visit_statements(&block.body);
            }

            Statement::ReturnStatement(ret) => {
                if let Some(arg) = &ret.argument {
                    self.visit_expression_for_flow(arg);
                }
                // After return, flow is unreachable.
                self.current_flow = self.unreachable;
            }

            Statement::ThrowStatement(throw) => {
                self.visit_expression_for_flow(&throw.argument);
                self.current_flow = self.unreachable;
            }

            Statement::BreakStatement(_) => {
                if let Some(&target) = self.break_targets.last() {
                    self.add_antecedent(target, self.current_flow);
                }
                self.current_flow = self.unreachable;
            }

            Statement::ContinueStatement(_) => {
                if let Some(&target) = self.continue_targets.last() {
                    self.add_antecedent(target, self.current_flow);
                }
                self.current_flow = self.unreachable;
            }

            Statement::LabeledStatement(labeled) => {
                self.visit_statement(&labeled.body);
            }

            Statement::TryStatement(try_stmt) => {
                // Simplified: treat try/catch as a branch label merge.
                let pre_try = self.current_flow;
                self.visit_statements(&try_stmt.block.body);
                let post_try = self.current_flow;

                if let Some(handler) = &try_stmt.handler {
                    self.current_flow = pre_try;
                    self.visit_statements(&handler.body.body);
                    let post_catch = self.current_flow;
                    // Merge try and catch paths.
                    let merge = self.new_branch_label();
                    self.add_antecedent(merge, post_try);
                    self.add_antecedent(merge, post_catch);
                    self.current_flow = self.finish_label(merge);
                } else {
                    self.current_flow = post_try;
                }

                if let Some(finalizer) = &try_stmt.finalizer {
                    self.visit_statements(&finalizer.body);
                }
            }

            Statement::FunctionDeclaration(_) => {
                // Function declarations don't affect control flow of the enclosing scope.
                // Their bodies are analyzed separately.
            }

            Statement::ClassDeclaration(_)
            | Statement::TSEnumDeclaration(_)
            | Statement::TSTypeAliasDeclaration(_)
            | Statement::TSInterfaceDeclaration(_) => {
                // Type declarations don't affect control flow.
            }

            // Import/export declarations don't affect control flow.
            Statement::ImportDeclaration(_)
            | Statement::ExportNamedDeclaration(_)
            | Statement::ExportDefaultDeclaration(_)
            | Statement::ExportAllDeclaration(_)
            | Statement::TSExportAssignment(_)
            | Statement::TSNamespaceExportDeclaration(_)
            | Statement::TSImportEqualsDeclaration(_) => {}

            // Module declarations, debugger, empty, with.
            Statement::TSModuleDeclaration(_)
            | Statement::TSGlobalDeclaration(_)
            | Statement::DebuggerStatement(_)
            | Statement::EmptyStatement(_)
            | Statement::WithStatement(_) => {}

            Statement::ForInStatement(for_in) => {
                self.visit_for_in_of_statement(&for_in.body);
            }
            Statement::ForOfStatement(for_of) => {
                self.visit_for_in_of_statement(&for_of.body);
            }
        }
    }

    fn visit_if_statement(&mut self, if_stmt: &oxc_ast::ast::IfStatement<'_>) {
        // Visit condition expression — records flow for identifiers used in condition.
        self.visit_expression_for_flow(&if_stmt.test);

        let pre_condition = self.current_flow;
        let (true_flow, false_flow) =
            self.maybe_create_condition_pair(&if_stmt.test, pre_condition);

        // True branch (consequent).
        self.current_flow = true_flow;
        self.visit_statement(&if_stmt.consequent);
        let post_true = self.current_flow;

        // False branch (alternate).
        self.current_flow = false_flow;
        if let Some(alt) = &if_stmt.alternate {
            self.visit_statement(alt);
        }
        let post_false = self.current_flow;

        // Merge both branches.
        let merge = self.new_branch_label();
        self.add_antecedent(merge, post_true);
        self.add_antecedent(merge, post_false);
        self.current_flow = self.finish_label(merge);
    }

    fn visit_while_statement(&mut self, while_stmt: &oxc_ast::ast::WhileStatement<'_>) {
        // Create loop label as the loop entry point.
        let loop_label = self.new_loop_label();
        self.add_antecedent(loop_label, self.current_flow);

        // Break target: a branch label after the loop.
        let break_label = self.new_branch_label();
        self.break_targets.push(break_label);
        self.continue_targets.push(loop_label);

        // Set current flow to loop label.
        self.current_flow = loop_label;

        // Visit condition.
        self.visit_expression_for_flow(&while_stmt.test);
        let pre_condition = self.current_flow;
        let (true_flow, false_flow) =
            self.maybe_create_condition_pair(&while_stmt.test, pre_condition);

        // Body runs in true condition.
        self.current_flow = true_flow;
        self.visit_statement(&while_stmt.body);

        // Back-edge: body flows back to loop label.
        self.add_antecedent(loop_label, self.current_flow);

        // After loop: false condition + break targets.
        self.add_antecedent(break_label, false_flow);

        self.continue_targets.pop();
        self.break_targets.pop();

        self.current_flow = self.finish_label(break_label);
    }

    fn visit_for_statement(&mut self, for_stmt: &oxc_ast::ast::ForStatement<'_>) {
        // Visit initializer.
        if let Some(init) = &for_stmt.init {
            use oxc_ast::ast::ForStatementInit;
            match init {
                ForStatementInit::VariableDeclaration(decl) => {
                    for declarator in &decl.declarations {
                        if let Some(init) = &declarator.init {
                            self.visit_expression_for_flow(init);
                        }
                    }
                }
                _ => {
                    // Expression initializer
                    if let Some(expr) = init.as_expression() {
                        self.visit_expression_for_flow(expr);
                    }
                }
            }
        }

        let loop_label = self.new_loop_label();
        self.add_antecedent(loop_label, self.current_flow);

        let break_label = self.new_branch_label();
        self.break_targets.push(break_label);
        self.continue_targets.push(loop_label);

        self.current_flow = loop_label;

        // Condition (if present).
        if let Some(test) = &for_stmt.test {
            self.visit_expression_for_flow(test);
            let pre_condition = self.current_flow;
            let (true_flow, false_flow) =
                self.maybe_create_condition_pair(test, pre_condition);

            self.current_flow = true_flow;
            self.visit_statement(&for_stmt.body);

            // Visit update expression.
            if let Some(update) = &for_stmt.update {
                self.visit_expression_for_flow(update);
            }

            self.add_antecedent(loop_label, self.current_flow);
            self.add_antecedent(break_label, false_flow);
        } else {
            // No condition — infinite loop, body always runs.
            self.visit_statement(&for_stmt.body);
            if let Some(update) = &for_stmt.update {
                self.visit_expression_for_flow(update);
            }
            self.add_antecedent(loop_label, self.current_flow);
        }

        self.continue_targets.pop();
        self.break_targets.pop();

        self.current_flow = self.finish_label(break_label);
    }

    fn visit_do_while_statement(&mut self, do_while: &oxc_ast::ast::DoWhileStatement<'_>) {
        let loop_label = self.new_loop_label();
        self.add_antecedent(loop_label, self.current_flow);

        let break_label = self.new_branch_label();
        self.break_targets.push(break_label);
        self.continue_targets.push(loop_label);

        self.current_flow = loop_label;

        // Body runs first.
        self.visit_statement(&do_while.body);

        // Then condition.
        self.visit_expression_for_flow(&do_while.test);

        // If condition is narrowing, true branch loops back, false branch exits.
        let pre_condition = self.current_flow;
        let (true_flow, false_flow) =
            self.maybe_create_condition_pair(&do_while.test, pre_condition);
        self.add_antecedent(loop_label, true_flow);
        self.add_antecedent(break_label, false_flow);

        self.continue_targets.pop();
        self.break_targets.pop();

        self.current_flow = self.finish_label(break_label);
    }

    fn visit_for_in_of_statement(&mut self, body: &Statement<'_>) {
        // Simplified: for-in/of treated as loop.
        let loop_label = self.new_loop_label();
        self.add_antecedent(loop_label, self.current_flow);

        let break_label = self.new_branch_label();
        self.break_targets.push(break_label);
        self.continue_targets.push(loop_label);

        self.current_flow = loop_label;
        self.visit_statement(body);
        self.add_antecedent(loop_label, self.current_flow);
        // Loop may not execute at all.
        self.add_antecedent(break_label, self.current_flow);

        self.continue_targets.pop();
        self.break_targets.pop();

        self.current_flow = self.finish_label(break_label);
    }

    fn visit_switch_statement(&mut self, switch_stmt: &oxc_ast::ast::SwitchStatement<'_>) {
        self.visit_expression_for_flow(&switch_stmt.discriminant);

        let break_label = self.new_branch_label();
        self.break_targets.push(break_label);

        let pre_switch = self.current_flow;
        let mut has_default = false;

        for case in &switch_stmt.cases {
            // Each case starts from the switch discriminant flow.
            self.current_flow = pre_switch;
            if case.test.is_none() {
                has_default = true;
            }
            self.visit_statements(&case.consequent);
            // Fall-through: current flow carries to next case.
        }

        // If no default, the switch can be skipped entirely.
        if !has_default {
            self.add_antecedent(break_label, pre_switch);
        }
        self.add_antecedent(break_label, self.current_flow);

        self.break_targets.pop();
        self.current_flow = self.finish_label(break_label);
    }

    // ── Expression visitors ────────────────────────────────────────────

    /// Walk an expression to record flow positions for identifiers and
    /// handle assignments that affect the flow graph.
    fn visit_expression_for_flow(&mut self, expr: &Expression<'_>) {
        if !self.is_reachable() {
            return;
        }

        match expr {
            Expression::Identifier(ident) => {
                // Record flow position for this identifier reference.
                self.set_node_flow(ident.node_id.get());
            }

            Expression::AssignmentExpression(assign) => {
                // Visit the right-hand side first.
                self.visit_expression_for_flow(&assign.right);

                // Record assignment flow node if LHS is a simple identifier.
                if assign.operator == AssignmentOperator::Assign {
                    if let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign.left {
                        if let Some(symbol_id) = self.resolve_identifier_symbol(ident) {
                            self.current_flow = self.push_node(FlowNodeKind::Assignment {
                                node_id: ident.node_id.get(),
                                symbol_id,
                                antecedent: self.current_flow,
                            });
                        }
                    }
                }
            }

            Expression::LogicalExpression(logical) => {
                self.visit_logical_expression(logical);
            }

            Expression::ConditionalExpression(cond) => {
                // Visit condition.
                self.visit_expression_for_flow(&cond.test);
                let pre_condition = self.current_flow;

                // True branch.
                self.current_flow = pre_condition;
                self.visit_expression_for_flow(&cond.consequent);
                let post_true = self.current_flow;

                // False branch.
                self.current_flow = pre_condition;
                self.visit_expression_for_flow(&cond.alternate);
                let post_false = self.current_flow;

                // Merge.
                let merge = self.new_branch_label();
                self.add_antecedent(merge, post_true);
                self.add_antecedent(merge, post_false);
                self.current_flow = self.finish_label(merge);
            }

            Expression::BinaryExpression(bin) => {
                self.visit_expression_for_flow(&bin.left);
                self.visit_expression_for_flow(&bin.right);
            }

            Expression::UnaryExpression(unary) => {
                self.visit_expression_for_flow(&unary.argument);
            }

            Expression::CallExpression(call) => {
                self.visit_expression_for_flow(&call.callee);
                for arg in &call.arguments {
                    if let Some(expr) = arg.as_expression() {
                        self.visit_expression_for_flow(expr);
                    }
                }
            }

            Expression::NewExpression(new_expr) => {
                self.visit_expression_for_flow(&new_expr.callee);
                for arg in &new_expr.arguments {
                    if let Some(expr) = arg.as_expression() {
                        self.visit_expression_for_flow(expr);
                    }
                }
            }

            Expression::ParenthesizedExpression(paren) => {
                self.visit_expression_for_flow(&paren.expression);
            }

            Expression::SequenceExpression(seq) => {
                for expr in &seq.expressions {
                    self.visit_expression_for_flow(expr);
                }
            }

            // Member access: visit object, record the overall expression.
            Expression::StaticMemberExpression(member) => {
                self.visit_expression_for_flow(&member.object);
            }
            Expression::ComputedMemberExpression(member) => {
                self.visit_expression_for_flow(&member.object);
                self.visit_expression_for_flow(&member.expression);
            }

            Expression::ArrayExpression(arr) => {
                for elem in &arr.elements {
                    if let Some(expr) = elem.as_expression() {
                        self.visit_expression_for_flow(expr);
                    }
                }
            }

            Expression::ObjectExpression(obj) => {
                for prop in &obj.properties {
                    if let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(p) = prop {
                        self.visit_expression_for_flow(&p.value);
                    }
                }
            }

            Expression::TemplateLiteral(tmpl) => {
                for expr in &tmpl.expressions {
                    self.visit_expression_for_flow(expr);
                }
            }

            Expression::TaggedTemplateExpression(tagged) => {
                self.visit_expression_for_flow(&tagged.tag);
            }

            // Arrow/function expressions create a new scope — don't walk into body.
            Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_) => {}

            // Literals, this, class — no flow effect.
            _ => {}
        }
    }

    /// Handle logical expressions (&&, ||, ??) — they create branches.
    fn visit_logical_expression(&mut self, logical: &oxc_ast::ast::LogicalExpression<'_>) {
        self.visit_expression_for_flow(&logical.left);
        let pre_left = self.current_flow;

        match logical.operator {
            LogicalOperator::And => {
                // `a && b`: if `a` is truthy, evaluate `b`.
                if self.is_narrowing_expression(&logical.left) {
                    let true_node = self.push_node(FlowNodeKind::TrueCondition {
                        node_id: self.get_expression_node_id(&logical.left),
                        antecedent: pre_left,
                    });
                    // `b` is evaluated only when `a` is truthy.
                    self.current_flow = true_node;
                }
                self.visit_expression_for_flow(&logical.right);
            }
            LogicalOperator::Or => {
                // `a || b`: if `a` is falsy, evaluate `b`.
                if self.is_narrowing_expression(&logical.left) {
                    let false_node = self.push_node(FlowNodeKind::FalseCondition {
                        node_id: self.get_expression_node_id(&logical.left),
                        antecedent: pre_left,
                    });
                    self.current_flow = false_node;
                }
                self.visit_expression_for_flow(&logical.right);
            }
            LogicalOperator::Coalesce => {
                // `a ?? b`: if `a` is null/undefined, evaluate `b`.
                // TODO: narrow `a` to exclude null/undefined
                self.visit_expression_for_flow(&logical.right);
            }
        }
    }

    // ── Narrowing detection ────────────────────────────────────────────

    /// Determines if a condition expression can narrow types.
    /// Mirrors tsgo's `isNarrowingExpression`.
    fn is_narrowing_expression(&self, expr: &Expression<'_>) -> bool {
        match expr {
            // Identifier — truthiness narrowing (e.g., `if (x)`)
            Expression::Identifier(_) => true,

            // !expr — if inner is narrowing
            Expression::UnaryExpression(unary) => {
                unary.operator == UnaryOperator::LogicalNot
                    && self.is_narrowing_expression(&unary.argument)
            }

            // typeof x === "string" — always narrowing
            Expression::BinaryExpression(bin) => self.is_narrowing_binary_expression(bin),

            // Parenthesized
            Expression::ParenthesizedExpression(paren) => {
                self.is_narrowing_expression(&paren.expression)
            }

            _ => false,
        }
    }

    fn is_narrowing_binary_expression(
        &self,
        bin: &oxc_ast::ast::BinaryExpression<'_>,
    ) -> bool {
        match bin.operator {
            // Equality operators: narrowing if one side is a narrowable reference
            BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality => {
                self.contains_narrowable_reference(&bin.left)
                    || self.contains_narrowable_reference(&bin.right)
            }

            // instanceof
            BinaryOperator::Instanceof => {
                self.contains_narrowable_reference(&bin.left)
            }

            // in operator
            BinaryOperator::In => {
                self.contains_narrowable_reference(&bin.right)
            }

            _ => false,
        }
    }

    /// Check if an expression contains a narrowable reference
    /// (identifier, typeof, or property access on identifier).
    fn contains_narrowable_reference(&self, expr: &Expression<'_>) -> bool {
        match expr {
            Expression::Identifier(_) => true,
            Expression::UnaryExpression(unary) => {
                // typeof x
                unary.operator == UnaryOperator::Typeof
                    && matches!(&unary.argument, Expression::Identifier(_))
            }
            // x.kind — property access on identifier (for discriminated unions)
            Expression::StaticMemberExpression(member) => {
                matches!(&member.object, Expression::Identifier(_))
            }
            Expression::ParenthesizedExpression(paren) => {
                self.contains_narrowable_reference(&paren.expression)
            }
            _ => false,
        }
    }

    // ── Utility ────────────────────────────────────────────────────────

    /// Get the NodeId of an expression (for use in flow condition nodes).
    fn get_expression_node_id(&self, expr: &Expression<'_>) -> NodeId {
        match expr {
            Expression::Identifier(id) => id.node_id.get(),
            Expression::BinaryExpression(bin) => bin.node_id.get(),
            Expression::UnaryExpression(unary) => unary.node_id.get(),
            Expression::LogicalExpression(logical) => logical.node_id.get(),
            Expression::StaticMemberExpression(member) => member.node_id.get(),
            Expression::ParenthesizedExpression(paren) => {
                self.get_expression_node_id(&paren.expression)
            }
            _ => {
                debug_assert!(false, "get_expression_node_id called on non-narrowing expression");
                NodeId::DUMMY
            }
        }
    }

    /// Resolve an identifier reference to its symbol ID.
    fn resolve_identifier_symbol(
        &self,
        ident: &oxc_ast::ast::IdentifierReference<'_>,
    ) -> Option<SymbolId> {
        let reference_id = ident.reference_id.get()?;
        let reference = self.semantic.scoping().get_reference(reference_id);
        reference.symbol_id()
    }
}
