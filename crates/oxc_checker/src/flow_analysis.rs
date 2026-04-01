//! Backward walk over the flow graph for type narrowing.
//!
//! Given a variable reference and its declared type, walks backward through
//! the flow graph to compute the narrowed type at that point, considering
//! conditions (typeof, truthiness, equality) and assignments.

use oxc_ast::ast::Expression;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};
use oxc_syntax::symbol::SymbolId;
use oxc_types::{TypeData, TypeFlags, TypeId};

use crate::flow::{CacheState, FlowGraph, FlowNodeId, FlowNodeKind};
use crate::Checker;

/// Maximum recursion depth for the backward walk.
const MAX_FLOW_DEPTH: u32 = 2000;

impl Checker<'_> {
    /// Get the narrowed type of a reference at a specific point in the flow graph.
    ///
    /// Walks backward through the flow graph from the identifier's position,
    /// applying narrowing from conditions and assignment resets.
    /// Returns `declared_type` if no narrowing applies or no flow graph is active.
    pub fn get_flow_type_of_reference(
        &mut self,
        ident_node_id: oxc_syntax::node::NodeId,
        symbol_id: SymbolId,
        declared_type: TypeId,
    ) -> TypeId {
        // Swap the flow graph out of self so we can pass &FlowGraph independently
        // of &mut self through the recursive backward walk. This is the single
        // swap/put-back point — the entire walk below uses zero-copy &FlowGraph.
        let flow_graph = std::mem::replace(
            &mut self.current_flow_graph,
            FlowGraph::empty(),
        );

        let Some(flow_node_id) = flow_graph.get_flow_for_node(ident_node_id) else {
            self.current_flow_graph = flow_graph;
            return declared_type;
        };

        // Only narrow union types or types that include null/undefined.
        // Simple types like `string` or `number` can't be narrowed further.
        let flags = self.type_arena.get_flags(declared_type);
        if !flags.intersects(
            TypeFlags::Union
                | TypeFlags::Null
                | TypeFlags::Undefined
                | TypeFlags::Any
                | TypeFlags::Unknown,
        ) {
            self.current_flow_graph = flow_graph;
            return declared_type;
        }

        let result =
            self.get_type_at_flow_node(&flow_graph, flow_node_id, symbol_id, declared_type, 0);

        self.current_flow_graph = flow_graph;
        result
    }

    /// Core backward walk: resolve the type at a given flow node.
    /// Uses a loop for linear chains (assignments, single-antecedent nodes)
    /// and recurses only at branching points (labels, conditions).
    fn get_type_at_flow_node(
        &mut self,
        flow_graph: &FlowGraph,
        flow_id: FlowNodeId,
        symbol_id: SymbolId,
        declared_type: TypeId,
        depth: u32,
    ) -> TypeId {
        let mut current_id = flow_id;
        let mut current_depth = depth;

        loop {
            if current_depth > MAX_FLOW_DEPTH {
                return declared_type;
            }

            let entry = flow_graph.get(current_id);
            let is_shared = entry.cache_state == CacheState::Shared;

            // Check cache. Always check for loop labels (sentinel prevents
            // infinite recursion through back-edges) and shared nodes
            // (avoids redundant work when a node has multiple successors).
            let is_loop = matches!(entry.kind, FlowNodeKind::LoopLabel { .. });
            if is_shared || is_loop {
                if let Some(&cached) = self.flow_type_cache.get(&(current_id, symbol_id)) {
                    return cached;
                }
            }

            match &entry.kind {
                FlowNodeKind::Start => {
                    return declared_type;
                }

                FlowNodeKind::Unreachable => {
                    return self.never_type;
                }

                FlowNodeKind::Assignment { symbol_id: assign_sym, antecedent, .. } => {
                    if *assign_sym == symbol_id {
                        // Assignment to our reference — type resets to declared type.
                        // TODO: infer the type of the assigned expression for more precision.
                        return declared_type;
                    }
                    // Assignment to a different variable — continue backward (loop).
                    current_id = *antecedent;
                    current_depth += 1;
                    continue;
                }

                FlowNodeKind::TrueCondition { node_id, antecedent } => {
                    let node_id = *node_id;
                    let antecedent = *antecedent;
                    let result = self.handle_condition_flow(
                        flow_graph, antecedent, node_id, true,
                        symbol_id, declared_type, current_depth,
                    );
                    if is_shared {
                        self.flow_type_cache.insert((current_id, symbol_id), result);
                    }
                    return result;
                }

                FlowNodeKind::FalseCondition { node_id, antecedent } => {
                    let node_id = *node_id;
                    let antecedent = *antecedent;
                    let result = self.handle_condition_flow(
                        flow_graph, antecedent, node_id, false,
                        symbol_id, declared_type, current_depth,
                    );
                    if is_shared {
                        self.flow_type_cache.insert((current_id, symbol_id), result);
                    }
                    return result;
                }

                FlowNodeKind::BranchLabel { antecedents } => {
                    let antecedents = antecedents.clone();
                    let result = self.handle_label_flow(
                        flow_graph, &antecedents, symbol_id, declared_type, current_depth,
                    );
                    if is_shared {
                        self.flow_type_cache.insert((current_id, symbol_id), result);
                    }
                    return result;
                }

                FlowNodeKind::LoopLabel { antecedents } => {
                    let antecedents = antecedents.clone();
                    // Insert a sentinel (declared_type) before recursing so that
                    // back-edges that revisit this loop label hit the cache instead
                    // of recursing indefinitely. This mirrors tsgo's approach of
                    // using the declared type as the initial assumption for
                    // loop-carried types.
                    self.flow_type_cache.insert((current_id, symbol_id), declared_type);
                    let result = self.handle_label_flow(
                        flow_graph, &antecedents, symbol_id, declared_type, current_depth,
                    );
                    self.flow_type_cache.insert((current_id, symbol_id), result);
                    return result;
                }
            }
        }
    }

    fn handle_condition_flow(
        &mut self,
        flow_graph: &FlowGraph,
        antecedent: FlowNodeId,
        condition_node_id: oxc_syntax::node::NodeId,
        assume_true: bool,
        symbol_id: SymbolId,
        declared_type: TypeId,
        depth: u32,
    ) -> TypeId {
        let type_before =
            self.get_type_at_flow_node(flow_graph, antecedent, symbol_id, declared_type, depth + 1);

        // Look up the condition expression from the AST using the NodeId.
        self.narrow_type_by_flow_condition(type_before, condition_node_id, symbol_id, assume_true)
    }

    fn handle_label_flow(
        &mut self,
        flow_graph: &FlowGraph,
        antecedents: &[FlowNodeId],
        symbol_id: SymbolId,
        declared_type: TypeId,
        depth: u32,
    ) -> TypeId {
        if antecedents.is_empty() {
            return self.never_type;
        }

        // Merge types from all antecedents into a union.
        let mut result_types = Vec::new();
        for &ant in antecedents {
            let t =
                self.get_type_at_flow_node(flow_graph, ant, symbol_id, declared_type, depth + 1);
            if !result_types.contains(&t) {
                result_types.push(t);
            }
        }

        if result_types.len() == 1 {
            return result_types[0];
        }

        self.get_or_create_union_type(result_types)
    }

    // ── Narrowing by condition ─────────────────────────────────────────

    /// Narrow a type based on a condition expression at a given AST node.
    fn narrow_type_by_flow_condition(
        &mut self,
        type_id: TypeId,
        condition_node_id: oxc_syntax::node::NodeId,
        symbol_id: SymbolId,
        assume_true: bool,
    ) -> TypeId {
        // Get the AST node for the condition.
        let node = self.semantic().nodes().get_node(condition_node_id);
        let kind = node.kind();

        match kind {
            oxc_ast::AstKind::IdentifierReference(ident) => {
                // Truthiness narrowing: `if (x)` — narrow x.
                let Some(ref_id) = ident.reference_id.get() else {
                    return type_id;
                };
                let reference = self.semantic().scoping().get_reference(ref_id);
                let Some(ref_symbol) = reference.symbol_id() else {
                    return type_id;
                };
                if ref_symbol != symbol_id {
                    return type_id;
                }
                self.narrow_by_truthiness(type_id, assume_true)
            }

            oxc_ast::AstKind::BinaryExpression(bin) => {
                self.narrow_type_by_binary_expression(type_id, bin, symbol_id, assume_true)
            }

            oxc_ast::AstKind::UnaryExpression(unary) => {
                if unary.operator == UnaryOperator::LogicalNot {
                    // !expr — flip assume_true.
                    let inner_node_id = self.get_inner_expression_node_id(&unary.argument);
                    self.narrow_type_by_flow_condition(
                        type_id,
                        inner_node_id,
                        symbol_id,
                        !assume_true,
                    )
                } else {
                    type_id
                }
            }

            _ => type_id,
        }
    }

    fn narrow_type_by_binary_expression(
        &mut self,
        type_id: TypeId,
        bin: &oxc_ast::ast::BinaryExpression<'_>,
        symbol_id: SymbolId,
        assume_true: bool,
    ) -> TypeId {
        match bin.operator {
            BinaryOperator::StrictEquality | BinaryOperator::StrictInequality => {
                let is_eq = bin.operator == BinaryOperator::StrictEquality;
                let assume_eq = if assume_true { is_eq } else { !is_eq };

                // Check for `typeof x === "string"` pattern.
                if let Some(narrowed) =
                    self.try_narrow_by_typeof(type_id, &bin.left, &bin.right, symbol_id, assume_eq)
                {
                    return narrowed;
                }
                if let Some(narrowed) =
                    self.try_narrow_by_typeof(type_id, &bin.right, &bin.left, symbol_id, assume_eq)
                {
                    return narrowed;
                }

                // Check for `x === null` or `x === undefined` pattern.
                if let Some(narrowed) = self.try_narrow_by_equality(
                    type_id,
                    &bin.left,
                    &bin.right,
                    symbol_id,
                    assume_eq,
                    true, // strict
                ) {
                    return narrowed;
                }
                if let Some(narrowed) = self.try_narrow_by_equality(
                    type_id,
                    &bin.right,
                    &bin.left,
                    symbol_id,
                    assume_eq,
                    true,
                ) {
                    return narrowed;
                }

                type_id
            }

            BinaryOperator::Equality | BinaryOperator::Inequality => {
                let is_eq = bin.operator == BinaryOperator::Equality;
                let assume_eq = if assume_true { is_eq } else { !is_eq };

                // Loose equality with null: `x == null` narrows to null | undefined.
                if let Some(narrowed) = self.try_narrow_by_equality(
                    type_id,
                    &bin.left,
                    &bin.right,
                    symbol_id,
                    assume_eq,
                    false, // loose
                ) {
                    return narrowed;
                }
                if let Some(narrowed) = self.try_narrow_by_equality(
                    type_id,
                    &bin.right,
                    &bin.left,
                    symbol_id,
                    assume_eq,
                    false,
                ) {
                    return narrowed;
                }

                type_id
            }

            _ => type_id,
        }
    }

    // ── Typeof narrowing ───────────────────────────────────────────────

    /// Try to narrow via `typeof expr === "typeString"`.
    /// `ref_expr` is the potential `typeof x`, `value_expr` is the string literal.
    fn try_narrow_by_typeof(
        &mut self,
        type_id: TypeId,
        ref_expr: &Expression<'_>,
        value_expr: &Expression<'_>,
        symbol_id: SymbolId,
        assume_eq: bool,
    ) -> Option<TypeId> {
        // ref_expr must be `typeof x` where x matches our symbol.
        let Expression::UnaryExpression(unary) = ref_expr else {
            return None;
        };
        if unary.operator != UnaryOperator::Typeof {
            return None;
        }
        let Expression::Identifier(ident) = &unary.argument else {
            return None;
        };
        let ref_id = ident.reference_id.get()?;
        let reference = self.semantic().scoping().get_reference(ref_id);
        let ref_symbol = reference.symbol_id()?;
        if ref_symbol != symbol_id {
            return None;
        }

        // value_expr must be a string literal.
        let Expression::StringLiteral(lit) = value_expr else {
            return None;
        };

        Some(self.narrow_by_typeof(type_id, lit.value.as_str(), assume_eq))
    }

    /// Narrow a type by keeping only members that satisfy a predicate.
    /// For unions: filters constituents. For non-unions: keeps or discards the whole type.
    pub(crate) fn narrow_type_by_predicate(
        &mut self,
        type_id: TypeId,
        keep: impl Fn(&Self, TypeId) -> bool,
    ) -> TypeId {
        let flags = self.type_arena.get_flags(type_id);

        if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(type_id) {
                let filtered: Vec<TypeId> =
                    u.types.iter().copied().filter(|&t| keep(self, t)).collect();

                if filtered.is_empty() {
                    return self.never_type;
                }
                return self.get_or_create_union_type(filtered);
            }
        }

        if keep(self, type_id) { type_id } else { self.never_type }
    }

    /// Narrow a type based on a typeof check.
    fn narrow_by_typeof(
        &mut self,
        type_id: TypeId,
        type_string: &str,
        assume_eq: bool,
    ) -> TypeId {
        self.narrow_type_by_predicate(type_id, |checker, t| {
            let matches = checker.type_matches_typeof(t, type_string);
            if assume_eq { matches } else { !matches }
        })
    }

    /// Check if a type matches a typeof string (e.g., "string", "number").
    fn type_matches_typeof(&self, type_id: TypeId, type_string: &str) -> bool {
        let flags = self.type_arena.get_flags(type_id);
        match type_string {
            "string" => flags.intersects(TypeFlags::StringLike),
            "number" => flags.intersects(TypeFlags::NumberLike),
            "bigint" => flags.intersects(TypeFlags::BigIntLike),
            "boolean" => flags.intersects(TypeFlags::BooleanLike),
            "symbol" => flags.intersects(TypeFlags::ESSymbolLike),
            "undefined" => flags.intersects(TypeFlags::Undefined | TypeFlags::Void),
            "object" => {
                flags.intersects(
                    TypeFlags::Object | TypeFlags::Null | TypeFlags::NonPrimitive,
                )
            }
            "function" => {
                // TODO: check for callable types
                flags.intersects(TypeFlags::Object)
                    && matches!(
                        self.type_arena.get_data(type_id),
                        TypeData::Function(_)
                    )
            }
            _ => false,
        }
    }

    // ── Truthiness narrowing ───────────────────────────────────────────

    /// Narrow a type by truthiness.
    /// If `assume_true`: remove null, undefined, false, 0, "", 0n from unions.
    /// If `!assume_true`: keep only nullable/falsy constituents.
    fn narrow_by_truthiness(&mut self, type_id: TypeId, assume_true: bool) -> TypeId {
        self.narrow_type_by_predicate(type_id, |checker, t| {
            let is_falsy = checker.is_falsy_type(t);
            if assume_true { !is_falsy } else { is_falsy }
        })
    }

    /// Check if a type is always falsy (null, undefined, false, void).
    fn is_falsy_type(&self, type_id: TypeId) -> bool {
        let flags = self.type_arena.get_flags(type_id);
        if flags.intersects(TypeFlags::Null | TypeFlags::Undefined | TypeFlags::Void) {
            return true;
        }
        // Check for `false` literal.
        if flags.intersects(TypeFlags::BooleanLiteral) {
            if let TypeData::Literal(oxc_types::LiteralType::Boolean(false)) =
                self.type_arena.get_data(type_id)
            {
                return true;
            }
        }
        // Check for numeric 0.
        if flags.intersects(TypeFlags::NumberLiteral) {
            if let TypeData::Literal(oxc_types::LiteralType::Number(n)) =
                self.type_arena.get_data(type_id)
            {
                return *n == 0.0;
            }
        }
        // Check for empty string.
        if flags.intersects(TypeFlags::StringLiteral) {
            if let TypeData::Literal(oxc_types::LiteralType::String(s)) =
                self.type_arena.get_data(type_id)
            {
                return s.is_empty();
            }
        }
        false
    }

    // ── Equality narrowing ─────────────────────────────────────────────

    /// Try to narrow via `expr === null` / `expr !== undefined`.
    fn try_narrow_by_equality(
        &mut self,
        type_id: TypeId,
        ref_expr: &Expression<'_>,
        value_expr: &Expression<'_>,
        symbol_id: SymbolId,
        assume_eq: bool,
        is_strict: bool,
    ) -> Option<TypeId> {
        // ref_expr must be an identifier matching our symbol.
        let Expression::Identifier(ident) = ref_expr else {
            return None;
        };
        let ref_id = ident.reference_id.get()?;
        let reference = self.semantic().scoping().get_reference(ref_id);
        let ref_symbol = reference.symbol_id()?;
        if ref_symbol != symbol_id {
            return None;
        }

        // value_expr must be null or undefined.
        let is_null = matches!(value_expr, Expression::NullLiteral(_));
        let is_undefined = matches!(value_expr, Expression::Identifier(id) if id.name == "undefined");

        if !is_null && !is_undefined {
            return None;
        }

        Some(self.narrow_by_null_undefined(type_id, is_null, assume_eq, is_strict))
    }

    /// Narrow a type by null/undefined equality check.
    fn narrow_by_null_undefined(
        &mut self,
        type_id: TypeId,
        is_null: bool,
        assume_eq: bool,
        is_strict: bool,
    ) -> TypeId {
        self.narrow_type_by_predicate(type_id, |checker, t| {
            let t_flags = checker.type_arena.get_flags(t);
            let is_nullable = if is_strict {
                if is_null {
                    t_flags.intersects(TypeFlags::Null)
                } else {
                    t_flags.intersects(TypeFlags::Undefined)
                }
            } else {
                t_flags.intersects(TypeFlags::Null | TypeFlags::Undefined)
            };
            if assume_eq { is_nullable } else { !is_nullable }
        })
    }

    // ── Helpers ────────────────────────────────────────────────────────

    /// Get the NodeId of an expression, unwrapping parentheses.
    fn get_inner_expression_node_id(&self, expr: &Expression<'_>) -> oxc_syntax::node::NodeId {
        match expr {
            Expression::ParenthesizedExpression(paren) => {
                self.get_inner_expression_node_id(&paren.expression)
            }
            Expression::Identifier(id) => id.node_id.get(),
            Expression::BinaryExpression(bin) => bin.node_id.get(),
            Expression::UnaryExpression(unary) => unary.node_id.get(),
            Expression::LogicalExpression(logical) => logical.node_id.get(),
            Expression::CallExpression(call) => call.node_id.get(),
            _ => {
                debug_assert!(false, "get_inner_expression_node_id called on unexpected expression");
                oxc_syntax::node::NodeId::DUMMY
            }
        }
    }
}
