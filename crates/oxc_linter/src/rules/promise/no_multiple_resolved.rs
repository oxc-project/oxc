use std::hash::Hash;

use oxc_ast::{
    AstKind,
    ast::{BindingIdentifier, CallExpression, Expression},
};
use oxc_ast_visit::Visit;
use oxc_cfg::{
    BlockNodeId, ControlFlowGraph, EdgeType, ErrorEdgeKind, InstructionKind,
    graph::{
        Direction,
        visit::{Control, DfsEvent, EdgeRef, set_depth_first_search},
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{Scoping, SymbolId};
use oxc_span::Span;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    AstNode, context::LintContext, rule::Rule, utils::get_promise_constructor_inline_executor,
};

fn already_resolved_diagnostic(line: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Promise should not be resolved multiple times. Promise is already resolved on line {line}."
    ))
    .with_label(span)
}

fn potentially_already_resolved_diagnostic(line: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Promise should not be resolved multiple times. Promise is potentially resolved on line {line}.")).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoMultipleResolved;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule warns of paths that resolve multiple times in executor functions that Promise constructors.
    ///
    /// ### Why is this bad?
    ///
    /// Multiple resolve/reject calls:
    /// - Violate the Promise/A+ specification
    /// - Have no effect on the Promise's behavior
    /// - Make the code's intent unclear
    /// - May indicate logical errors in the implementation
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// new Promise((resolve, reject) => {
    ///   fn((error, value) => {
    ///     if (error) {
    ///       reject(error)
    ///     }
    ///
    ///     resolve(value) // Both `reject` and `resolve` may be called.
    ///   })
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// new Promise((resolve, reject) => {
    ///   fn((error, value) => {
    ///     if (error) {
    ///       reject(error)
    ///     } else {
    ///       resolve(value)
    ///     }
    ///   })
    /// })
    /// ```
    NoMultipleResolved,
    promise,
    suspicious
);

impl Rule for NoMultipleResolved {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(new_expr) = node.kind().as_new_expression() else { return };
        let Some(executor_expr) = get_promise_constructor_inline_executor(new_expr) else { return };
        let (resolve_symbol_id, reject_symbol_id) = get_resolve_symbol_id(executor_expr);
        let resolve_findler =
            ResolveFinder::new(ctx.scoping(), resolve_symbol_id, reject_symbol_id);
        let mut multiple_resolved_checker = MultipleResolvedChecker::new(ctx, resolve_findler);
        let Some(inline_executor_cfg_id) =
            next_new_function_node_id(ctx.cfg(), ctx.nodes().cfg_id(node.id()))
        else {
            return;
        };
        multiple_resolved_checker.check(inline_executor_cfg_id);
    }
}

fn next_new_function_node_id(cfg: &ControlFlowGraph, block_id: BlockNodeId) -> Option<BlockNodeId> {
    cfg.graph()
        .edges_directed(block_id, Direction::Outgoing)
        .find(|edge| matches!(edge.weight(), EdgeType::NewFunction))
        .map(|edge| edge.target())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ResolvedKind {
    /// resolve or reject has been called on some path
    Certain,
    /// resolve or reject may be called on some path
    Potential,
    /// resolve or reject has not been called on any path
    None,
}

#[derive(Debug)]
struct BlockResolvedInfo<'a> {
    resolved: Option<&'a CallExpression<'a>>,
    kind: ResolvedKind,
    throwable_after_resolved: bool,
}

impl<'a> BlockResolvedInfo<'a> {
    fn new(
        resolved: Option<&'a CallExpression<'a>>,
        kind: ResolvedKind,
        throwable_after_resolved: bool,
    ) -> Self {
        Self { resolved, kind, throwable_after_resolved }
    }
}

struct FunctionResolvedInfo<'a> {
    resolved_infos: FxHashMap<BlockNodeId, BlockResolvedInfo<'a>>,
    // The block_ids contains all id of blocks in the current function, It's in topological order.
    block_ids: Vec<BlockNodeId>,
    // The immediate dominator for each block
    dominators: FxHashMap<BlockNodeId, BlockNodeId>,
}

impl FunctionResolvedInfo<'_> {
    fn new() -> Self {
        Self {
            resolved_infos: FxHashMap::default(),
            block_ids: vec![],
            dominators: FxHashMap::default(),
        }
    }
}

fn sets_intersection<T>(sets: &[&FxHashSet<T>]) -> FxHashSet<T>
where
    T: Eq + Hash + Copy,
{
    if let Some((&first, rest)) = sets.split_first() {
        if rest.is_empty() {
            first.clone()
        } else {
            rest.iter().fold(first.clone(), |acc, &set| {
                acc.intersection(set).copied().collect::<FxHashSet<_>>()
            })
        }
    } else {
        FxHashSet::default()
    }
}

struct MultipleResolvedChecker<'a, 'b: 'a> {
    ctx: &'a LintContext<'b>,
    resolve_finder: ResolveFinder<'a>,
    // Stack tracking resolved information for nested functions.
    func_resolved_info_stack: Vec<FunctionResolvedInfo<'a>>,
}

impl<'a, 'b> MultipleResolvedChecker<'a, 'b> {
    fn new(ctx: &'a LintContext<'b>, resolve_finder: ResolveFinder<'a>) -> Self {
        Self { ctx, resolve_finder, func_resolved_info_stack: vec![] }
    }

    #[inline]
    fn current_func_resolved_info(&self) -> Option<&FunctionResolvedInfo<'a>> {
        self.func_resolved_info_stack.last()
    }

    #[inline]
    fn current_func_resolved_info_mut(&mut self) -> Option<&mut FunctionResolvedInfo<'a>> {
        self.func_resolved_info_stack.last_mut()
    }

    #[inline]
    fn push_func_resolved_info(&mut self, info: FunctionResolvedInfo<'a>) {
        self.func_resolved_info_stack.push(info);
    }

    #[inline]
    fn pop_func_resolved_info(&mut self) -> Option<FunctionResolvedInfo<'a>> {
        self.func_resolved_info_stack.pop()
    }

    #[inline]
    fn resolved_infos(&self) -> Option<&FxHashMap<BlockNodeId, BlockResolvedInfo<'a>>> {
        Some(&self.current_func_resolved_info()?.resolved_infos)
    }

    #[inline]
    fn block_resolved_info(&self, block_id: BlockNodeId) -> Option<&BlockResolvedInfo<'a>> {
        self.resolved_infos()?.get(&block_id)
    }

    fn insert_block_resolved_info(&mut self, block_id: BlockNodeId, info: BlockResolvedInfo<'a>) {
        let Some(resolved_infos) = self.resolved_infos_mut() else { return };
        resolved_infos.insert(block_id, info);
    }

    #[inline]
    fn resolved_infos_mut(&mut self) -> Option<&mut FxHashMap<BlockNodeId, BlockResolvedInfo<'a>>> {
        Some(&mut self.current_func_resolved_info_mut()?.resolved_infos)
    }

    #[inline]
    fn block_resolved_info_mut(
        &mut self,
        block_id: BlockNodeId,
    ) -> Option<&mut BlockResolvedInfo<'a>> {
        self.resolved_infos_mut()?.get_mut(&block_id)
    }

    #[inline]
    fn block_ids(&self) -> Option<&Vec<BlockNodeId>> {
        Some(&self.current_func_resolved_info()?.block_ids)
    }

    #[inline]
    fn set_block_ids(&mut self, block_ids: Vec<BlockNodeId>) {
        if let Some(current_func_resolved_info) = self.current_func_resolved_info_mut() {
            current_func_resolved_info.block_ids = block_ids;
        }
    }

    #[inline]
    fn dominators(&self) -> Option<&FxHashMap<BlockNodeId, BlockNodeId>> {
        Some(&self.current_func_resolved_info()?.dominators)
    }

    #[inline]
    fn block_dominator(&self, block_id: BlockNodeId) -> Option<BlockNodeId> {
        self.dominators()?.get(&block_id).copied()
    }

    #[inline]
    fn set_dominators(&mut self, dominators: FxHashMap<BlockNodeId, BlockNodeId>) {
        if let Some(current_func_resolved_info) = self.current_func_resolved_info_mut() {
            current_func_resolved_info.dominators = dominators;
        }
    }

    fn compute_block_node_dominator(&self) -> FxHashMap<BlockNodeId, BlockNodeId> {
        let Some(block_ids) = self.block_ids() else {
            return FxHashMap::default();
        };
        let mut block_nodes_dominators: FxHashMap<BlockNodeId, FxHashSet<BlockNodeId>> = block_ids
            .iter()
            .map(|&block_id| (block_id, FxHashSet::from_iter([block_id])))
            .collect();

        for &block_id in block_ids {
            let in_coming_block_ids = self.in_coming_block_ids(block_id);
            let prev_block_nodes_dominators: Vec<_> = in_coming_block_ids
                .iter()
                .filter_map(|block_id| block_nodes_dominators.get(block_id))
                .collect();
            if prev_block_nodes_dominators.is_empty() {
                continue;
            }
            let prev_dominators = sets_intersection(&prev_block_nodes_dominators);
            block_nodes_dominators.entry(block_id).or_default().extend(prev_dominators);
        }
        let block_nodes_sort: FxHashMap<_, _> =
            block_ids.iter().enumerate().map(|(i, &block_id)| (block_id, i)).collect();
        let mut block_nodes_immediate_dominator: FxHashMap<BlockNodeId, BlockNodeId> =
            FxHashMap::default();
        for (block_node_id, dominators) in &block_nodes_dominators {
            if dominators.len() <= 1 {
                continue;
            }
            let immediate_dominator = dominators
                .iter()
                .filter(|&dominator_node_id| *dominator_node_id != *block_node_id)
                .max_by_key(|&dominator_node_id| block_nodes_sort[dominator_node_id]);
            if let Some(immediate_dominator) = immediate_dominator {
                block_nodes_immediate_dominator.insert(*block_node_id, *immediate_dominator);
            }
        }
        block_nodes_immediate_dominator
    }

    fn block_is_condition(&self, block_id: BlockNodeId) -> bool {
        let instructions = self.ctx.cfg().basic_block(block_id).instructions();
        !instructions.is_empty()
            && instructions
                .iter()
                .all(|instruction| matches!(instruction.kind, InstructionKind::Condition))
    }

    fn backedge_next_target(&self, block_id: BlockNodeId) -> Option<BlockNodeId> {
        let graph = self.ctx.cfg().graph();
        let mut source = block_id;
        'outer: loop {
            let next_edges = graph.edges_directed(source, Direction::Outgoing);
            for edge in next_edges {
                match edge.weight() {
                    EdgeType::Backedge if !self.block_is_condition(edge.source()) => {
                        source = edge.target();
                        continue 'outer;
                    }
                    EdgeType::Normal if source != block_id => {
                        return Some(edge.target());
                    }
                    _ => {}
                }
            }
            return None;
        }
    }

    fn prev_backedge_source(&self, block_id: BlockNodeId) -> Option<BlockNodeId> {
        let graph = self.ctx.cfg().graph();
        let mut source: Option<BlockNodeId> = None;
        loop {
            let back_edge = graph
                .edges_directed(source.unwrap_or(block_id), Direction::Incoming)
                .find(|edge| matches!(edge.weight(), EdgeType::Backedge));
            if let Some(back_edge) = back_edge
                && !self.block_is_condition(back_edge.source())
            {
                source = Some(back_edge.source());
                continue;
            }
            break;
        }
        source
    }

    fn out_going_block_ids(&self, block_id: BlockNodeId) -> Vec<BlockNodeId> {
        let graph = self.ctx.cfg().graph();
        let mut outgoming_block_ids: Vec<BlockNodeId> = Vec::new();
        graph.edges_directed(block_id, Direction::Outgoing).for_each(|edge| match edge.weight() {
            EdgeType::Normal
            | EdgeType::Jump
            | EdgeType::Join
            | EdgeType::Error(ErrorEdgeKind::Explicit)
            | EdgeType::Finalize => {
                outgoming_block_ids.push(edge.target());
            }
            EdgeType::Backedge => {
                if let Some(next_target) = self.backedge_next_target(edge.source()) {
                    outgoming_block_ids.push(next_target);
                }
            }
            _ => {}
        });
        outgoming_block_ids.reverse();
        outgoming_block_ids
    }

    fn in_coming_block_ids(&self, block_id: BlockNodeId) -> Vec<BlockNodeId> {
        let graph = self.ctx.cfg().graph();
        let mut incoming_block_ids: Vec<BlockNodeId> = Vec::new();
        graph.edges_directed(block_id, Direction::Incoming).for_each(|edge| match edge.weight() {
            EdgeType::Normal => {
                let source = edge.source();
                if let Some(prev_source) = self.prev_backedge_source(source) {
                    incoming_block_ids.push(prev_source);
                }
                incoming_block_ids.push(source);
            }
            EdgeType::Jump
            | EdgeType::Join
            | EdgeType::Error(ErrorEdgeKind::Explicit)
            | EdgeType::Finalize => {
                incoming_block_ids.push(edge.source());
            }
            EdgeType::Backedge => {
                let source = edge.source();
                if graph
                    .edges_directed(block_id, Direction::Outgoing)
                    .any(|edge| matches!(edge.weight(), EdgeType::Backedge))
                {
                    incoming_block_ids.push(source);
                }
            }
            _ => {}
        });
        incoming_block_ids.reverse();
        incoming_block_ids
    }

    fn is_catch_start_block(&self, block_id: BlockNodeId) -> bool {
        let graph = self.ctx.cfg().graph();
        graph
            .edges_directed(block_id, Direction::Incoming)
            .all(|edge| matches!(edge.weight(), EdgeType::Error(ErrorEdgeKind::Explicit)))
    }

    fn prev_resolved_info(
        &self,
        block_id: BlockNodeId,
    ) -> (Option<&'a CallExpression<'a>>, ResolvedKind) {
        let incoming_block_ids = self.in_coming_block_ids(block_id);
        let is_catch_start_block = self.is_catch_start_block(block_id);
        let mut certain_resolved_num = 0;
        let mut first_certain_resolved: Option<&CallExpression<'_>> = None;
        let mut first_potential_resolved: Option<&CallExpression<'_>> = None;
        for &block_id in &incoming_block_ids {
            if let Some(resolved_info) = self.block_resolved_info(block_id) {
                if is_catch_start_block && !resolved_info.throwable_after_resolved {
                    continue;
                }
                match resolved_info.kind {
                    ResolvedKind::Certain => {
                        certain_resolved_num += 1;
                        if first_certain_resolved.is_none() {
                            first_certain_resolved = resolved_info.resolved;
                        }
                    }
                    ResolvedKind::Potential => {
                        if first_potential_resolved.is_none()
                            && self.out_going_block_ids(block_id).len() == 1
                        {
                            first_potential_resolved = resolved_info.resolved;
                        }
                    }
                    ResolvedKind::None => {}
                }
            }
        }
        // If all paths reaching this node are Certain, then the current node is Certain
        if certain_resolved_num == incoming_block_ids.len() {
            (first_certain_resolved, ResolvedKind::Certain)
        }
        // If some paths reaching this node are Certain and others are not, then the current node is Potential
        else if first_certain_resolved.is_some() {
            (first_certain_resolved, ResolvedKind::Potential)
        }
        // If any source node is Potential and this node has no other target nodes, then the current node is Potential
        else if first_potential_resolved.is_some() {
            (first_potential_resolved, ResolvedKind::Potential)
        } else {
            // If no source node is Potential and this node has multiple source nodes, then the status of current node  is the same as its immediate dominator's status
            if incoming_block_ids.len() > 1
                && let Some(block_node_dominator) = self.block_dominator(block_id)
            {
                if let Some(resolved_infos) = self.block_resolved_info(block_node_dominator) {
                    (resolved_infos.resolved, resolved_infos.kind)
                } else {
                    (None, ResolvedKind::None)
                }
            } else {
                (None, ResolvedKind::None)
            }
        }
    }

    fn traverse_blocks(&mut self, start_block_id: BlockNodeId) {
        let graph = self.ctx.cfg().graph();
        let mut block_ids: Vec<BlockNodeId> = vec![];
        set_depth_first_search::<_, _, _, Control<()>, _>(graph, Some(start_block_id), |event| {
            match event {
                DfsEvent::TreeEdge(a, b) => {
                    let edges = graph.edges_connecting(a, b).collect::<Vec<_>>();
                    for edge in &edges {
                        if matches!(edge.weight(), EdgeType::NewFunction) {
                            self.check(edge.target());
                            return Control::Prune;
                        }
                    }
                    if edges.iter().any(|edge| {
                        matches!(edge.weight(), EdgeType::Backedge)
                            && graph
                                .edges_directed(edge.target(), Direction::Outgoing)
                                .any(|edge| matches!(edge.weight(), EdgeType::Backedge))
                    }) {
                        return Control::Prune;
                    }
                    Control::Continue
                }
                DfsEvent::Finish(block_id, _) => {
                    block_ids.push(block_id);
                    Control::Continue
                }
                _ => Control::Continue,
            }
        });
        block_ids.reverse();
        self.set_block_ids(block_ids);
    }

    fn check(&mut self, start_block_id: BlockNodeId) {
        self.push_func_resolved_info(FunctionResolvedInfo::new());
        self.traverse_blocks(start_block_id);
        let block_ids: Vec<_> = self.block_ids().map_or(vec![], Clone::clone);
        for &block_id in &block_ids {
            self.check_block(block_id);
        }
        self.set_dominators(self.compute_block_node_dominator());
        for &block_id in block_ids.iter().skip(1) {
            let (prev_resolved, prev_resolved_kind) = self.prev_resolved_info(block_id);
            if matches!(prev_resolved_kind, ResolvedKind::None) {
                continue;
            }
            let Some(resolved_info) = self.block_resolved_info_mut(block_id) else {
                self.pop_func_resolved_info();
                return;
            };
            if resolved_info.kind == ResolvedKind::Certain {
                match prev_resolved_kind {
                    ResolvedKind::Certain => {
                        let Some(prev_resolved) = prev_resolved else { continue };
                        let Some(resolved) = resolved_info.resolved else { continue };
                        let line = get_span_line(self.ctx.source_text(), prev_resolved.span);
                        self.ctx.diagnostic(already_resolved_diagnostic(line, resolved.span));
                    }
                    ResolvedKind::Potential => {
                        let Some(prev_resolved) = prev_resolved else { continue };
                        let Some(resolved) = resolved_info.resolved else { continue };
                        let line = get_span_line(self.ctx.source_text(), prev_resolved.span);
                        self.ctx.diagnostic(potentially_already_resolved_diagnostic(
                            line,
                            resolved.span,
                        ));
                    }
                    ResolvedKind::None => {}
                }
            } else {
                resolved_info.resolved = prev_resolved;
                resolved_info.kind = prev_resolved_kind;
            }
        }
        self.pop_func_resolved_info();
    }

    fn collect_block_resolved_calls(
        &mut self,
        block_id: BlockNodeId,
    ) -> Vec<&'a CallExpression<'a>> {
        let mut resolved = Vec::<&CallExpression>::new();
        if self.is_catch_start_block(block_id) {
            self.resolve_finder.try_block_end();
        }
        self.ctx.cfg().basic_block(block_id).instructions().iter().for_each(|instruction| {
            if !matches!(instruction.kind, InstructionKind::Statement) {
                return;
            }
            let Some(node_id) = instruction.node_id else { return };
            let node = self.ctx.nodes().get_node(node_id);
            match node.kind() {
                AstKind::ExpressionStatement(expr_stmt) => {
                    self.resolve_finder.visit_expression_statement(expr_stmt);
                    resolved.append(&mut self.resolve_finder.take_resolved());
                }
                AstKind::TryStatement(try_stmt) if try_stmt.handler.is_some() => {
                    self.resolve_finder.try_block_start();
                }
                _ => {}
            }
        });
        resolved
    }

    fn check_throwable_after_resolved(
        &mut self,
        block_id: BlockNodeId,
        resolved: &[&'a CallExpression<'a>],
    ) -> bool {
        let last_throwable_expr_span = self.resolve_finder.take_last_throwable_expr_span();
        if !self.resolve_finder.in_try_block() {
            return false;
        }
        if resolved.last().is_some_and(|resolved| {
            last_throwable_expr_span.is_some_and(|span| span.start > resolved.span.end)
        }) {
            true
        } else {
            let in_coming_block_ids = self.in_coming_block_ids(block_id);
            !in_coming_block_ids.is_empty()
                && in_coming_block_ids.iter().all(|block_id| {
                    self.block_resolved_info(*block_id)
                        .is_some_and(|resolved_info| resolved_info.throwable_after_resolved)
                })
        }
    }

    fn check_block(&mut self, block_id: BlockNodeId) {
        let resolved = self.collect_block_resolved_calls(block_id);
        let throwable_after_resolved = self.check_throwable_after_resolved(block_id, &resolved);
        if let Some(first_resolved) = resolved.first() {
            if resolved.len() > 1 {
                let line = get_span_line(self.ctx.source_text(), first_resolved.span);
                resolved.iter().skip(1).for_each(|&multi_resolved| {
                    self.ctx.diagnostic(already_resolved_diagnostic(line, multi_resolved.span));
                });
            }
            self.insert_block_resolved_info(
                block_id,
                BlockResolvedInfo::new(
                    resolved.first().copied(),
                    ResolvedKind::Certain,
                    throwable_after_resolved,
                ),
            );
        } else {
            self.insert_block_resolved_info(
                block_id,
                BlockResolvedInfo::new(None, ResolvedKind::None, throwable_after_resolved),
            );
        }
    }
}

fn get_resolve_symbol_id(expr: &Expression) -> (Option<SymbolId>, Option<SymbolId>) {
    let params = match expr {
        Expression::FunctionExpression(func_expr) => Some(func_expr.params.as_ref()),
        Expression::ArrowFunctionExpression(arrow_func_expr) => {
            Some(arrow_func_expr.params.as_ref())
        }
        _ => None,
    };
    let symbol_ids = params.map_or(vec![], |params| {
        params
            .items
            .iter()
            .map(|param| param.pattern.get_binding_identifier().map(BindingIdentifier::symbol_id))
            .collect::<Vec<_>>()
    });
    let resolve_symbol_id = symbol_ids.first().copied().unwrap_or(None);
    let reject_symbol_id = symbol_ids.get(1).copied().unwrap_or(None);
    (resolve_symbol_id, reject_symbol_id)
}

#[inline]
fn get_span_line(source_text: &str, span: Span) -> usize {
    source_text[..span.end as usize].lines().count()
}

#[derive(Debug)]
struct ResolveFinder<'a> {
    scoping: &'a Scoping,
    resolve_symbol_id: Option<SymbolId>,
    reject_symbol_id: Option<SymbolId>,
    resolved: Vec<&'a CallExpression<'a>>,
    // If it's a try block with a catch, then the try block's depth is 1
    try_block_depth: i32,
    last_throwable_expr_span: Option<Span>,
}

impl<'a> ResolveFinder<'a> {
    fn new(
        scoping: &'a Scoping,
        resolve_symbol_id: Option<SymbolId>,
        reject_symbol_id: Option<SymbolId>,
    ) -> Self {
        Self {
            scoping,
            resolve_symbol_id,
            reject_symbol_id,
            resolved: vec![],
            try_block_depth: 0,
            last_throwable_expr_span: None,
        }
    }

    #[inline]
    fn try_block_start(&mut self) {
        self.try_block_depth += 1;
    }

    #[inline]
    fn try_block_end(&mut self) {
        self.try_block_depth -= 1;
    }

    #[inline]
    fn in_try_block(&self) -> bool {
        self.try_block_depth > 0
    }

    #[inline]
    fn record_throwable_expr_span(&mut self, span: Span) {
        if self.in_try_block() {
            self.last_throwable_expr_span = Some(span);
        }
    }

    #[inline]
    fn take_resolved(&mut self) -> Vec<&'a CallExpression<'a>> {
        self.resolved.drain(..).collect()
    }

    #[inline]
    fn take_last_throwable_expr_span(&mut self) -> Option<Span> {
        let span = self.last_throwable_expr_span;
        self.last_throwable_expr_span = None;
        span
    }
}

impl<'a> Visit<'a> for ResolveFinder<'a> {
    fn leave_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::NewExpression(new_expr) => {
                self.record_throwable_expr_span(new_expr.span);
            }
            AstKind::ImportExpression(import_expr) => {
                self.record_throwable_expr_span(import_expr.span);
            }
            AstKind::YieldExpression(yield_expr) => {
                self.record_throwable_expr_span(yield_expr.span);
            }
            AstKind::StaticMemberExpression(static_member_expr) => {
                self.record_throwable_expr_span(static_member_expr.span);
            }
            AstKind::ComputedMemberExpression(computed_member_expr) => {
                self.record_throwable_expr_span(computed_member_expr.span);
            }
            _ => {}
        }
    }
    fn visit_call_expression(&mut self, call_expr: &CallExpression<'a>) {
        match &call_expr.callee {
            Expression::Identifier(ident) => {
                let symbol_id = self.scoping.get_reference(ident.reference_id()).symbol_id();
                if symbol_id == self.resolve_symbol_id || symbol_id == self.reject_symbol_id {
                    self.resolved.push(self.alloc(call_expr));
                } else {
                    self.record_throwable_expr_span(call_expr.span);
                }
            }
            _ => {
                self.record_throwable_expr_span(call_expr.span);
            }
        }
    }

    fn visit_function(
        &mut self,
        _it: &oxc_ast::ast::Function<'a>,
        _flags: oxc_semantic::ScopeFlags,
    ) {
    }

    fn visit_arrow_function_expression(&mut self, _it: &oxc_ast::ast::ArrowFunctionExpression<'a>) {
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "new Promise((resolve, reject) => {
    fn((error, value) => {
        if (error) {
            reject(error)
        } else {
            resolve(value)
        }
    })
})",
        "new Promise((resolve, reject) => {
    if (error) {
        reject(error)
    } else {
        resolve(value)
    }
})",
        "new Promise((resolve, reject) => {
    fn((error, value) => {
        if (error) {
            reject(error)
            return
        }
        resolve(value)
    })
})",
        "new Promise((resolve, reject) => {
    fn((error, value) => {
        if (error) {
            reject(error)
        }
        if (!error) {
            resolve(value)
        }
    })
})",
        "new Promise((resolve, reject) => {
    fn((error, value) => {
        if (error) {
            reject(error)
        }
        if (error) {
            return
        }
        resolve(value)
    })
})",
        "new Promise((resolve, reject) => {
    fn((error, value) => {
        if (error) {
            reject(error)
        }
        if (!error) {
            // process
        } else {
            // process
        }
        if(!error) {
            resolve(value)
        }
    })
})",
        "new Promise((resolve, reject) => {
    fn((error, value) => {
        if (error) {
            reject(error)
            return
        }
        if (!error) {
            // process
        } else {
            // process
        }

        resolve(value)
    })
})",
        "new Promise(async (resolve, reject) => {
    try {
        await foo();
        resolve();
    } catch (error) {
        reject(error);
    }
})",
        "new Promise(async (resolve, reject) => {
    try {
        const r = await foo();
        resolve(r);
    } catch (error) {
        reject(error);
    }
})",
        "new Promise(async (resolve, reject) => {
    try {
        const r = await foo();
        resolve(r());
    } catch (error) {
        reject(error);
    }
})",
        "new Promise(async (resolve, reject) => {
    try {
        const r = await foo();
        resolve(r.foo);
    } catch (error) {
        reject(error);
    }
})",
        "new Promise(async (resolve, reject) => {
    try {
        const r = await foo();
        resolve(new r());
    } catch (error) {
        reject(error);
    }
})",
        "new Promise(async (resolve, reject) => {
    try {
        const r = await foo();
        resolve(import(r));
    } catch (error) {
        reject(error);
    }
})",
        "new Promise((resolve, reject) => {
    fn(async function * () {
        try {
            const r = await foo();
            resolve(yield r);
        } catch (error) {
            reject(error);
        }
    })
})",
        "new Promise(async (resolve, reject) => {
    let a;
    try {
        const r = await foo();
        resolve();
        if(r) return;
    } catch (error) {
        reject(error);
    }
})",
        "new Promise((resolve, reject) => {
    try {
        try {
            resolve(value);
        } catch (error) {
            reject(error);
        }
    } catch (error) {
        reject(error);
    }
})",
    ];

    let fail = vec![
        "new Promise((resolve, reject) => {
    fn((error, value) => {
        if (error) {
            reject(error)
        }

        resolve(value)
    })
})",
        "new Promise((resolve, reject) => {
    if (error) {
        reject(error)
    }

    resolve(value)
})",
        "new Promise((resolve, reject) => {
    reject(error)
    resolve(value)
})",
        "new Promise((resolve, reject) => {
    fn((error, value) => {
        if (error) {
            reject(error)
        }
        if (!error) {
            // process
        } else {
            // process
        }

        resolve(value)
    })
})",
        "new Promise((resolve, reject) => {
    fn((error, value) => {
        if (error) {
            if (foo) {
                if (bar) reject(error)
            }
        }

        resolve(value)
    })
})",
        "new Promise((resolve, reject) => {
    fn((error, value) => {
        if (error) {
            reject(error)
        } else {
            return
        }

        resolve(value)
    })
})",
        "new Promise((resolve, reject) => {
    if(foo) {
        if (error) {
            reject(error)
        } else {
            return
        }
        resolve(value)
    }

    resolve(value)
})",
        "new Promise((resolve, reject) => {
    if (foo) {
        reject(error)
    } else {
        resolve(value)
    }
    if(bar) {
        resolve(value)
    }
})",
        "new Promise((resolve, reject) => {
    while (error) {
        reject(error)
    }
    resolve(value)
})",
        "new Promise((resolve, reject) => {
    for (let i = 0; i < 10; i++) {
        if (error) {
            resolve(error)
        }
    }
    resolve(value)
})",
        "new Promise((resolve, reject) => {
    do {
        if (error) {
            resolve()
        }
    } while (error)
    reject()
})",
        "new Promise((resolve, reject) => {
    try {
        reject(error)
    } finally {
        resolve(value)
    }
})",
        "new Promise((resolve, reject) => {
    try {
        if (error) {
            reject(error)
        }
    } finally {
        resolve(value)
    }
})",
        "new Promise(async (resolve, reject) => {
    try {
        const r = await foo();
        resolve();
        r();
    } catch (error) {
        reject(error);
    }
})",
        "new Promise(async (resolve, reject) => {
    let a;
    try {
        const r = await foo();
        resolve();
        a = r.foo;
    } catch (error) {
        reject(error);
    }
})",
        "new Promise(async (resolve, reject) => {
    let a;
    try {
        const r = await foo();
        resolve();
        a = new r();
    } catch (error) {
        reject(error);
    }
})",
        "new Promise(async (resolve, reject) => {
    let a;
    try {
        const r = await foo();
        resolve();
        import(r);
    } catch (error) {
        reject(error);
    }
})",
        "new Promise((resolve, reject) => {
    fn(async function * () {
        try {
            const r = await foo();
            resolve();
            yield r;
        } catch (error) {
            reject(error);
        }
    })
})",
    ];

    Tester::new(NoMultipleResolved::NAME, NoMultipleResolved::PLUGIN, pass, fail)
        .test_and_snapshot();
}
