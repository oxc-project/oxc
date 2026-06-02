use itertools::Itertools;
use oxc_allocator::{Allocator, BitSet, CloneIn};
use smallvec::SmallVec;

use oxc_ast::{
    AstKind,
    ast::{BindingPattern, Expression, VariableDeclarationKind},
};
use oxc_cfg::{
    BasicBlockId, BlockNodeId, ControlFlowGraph, EdgeType, ErrorEdgeKind, Graph,
    graph::{
        Direction,
        visit::{Control, DfsEvent, EdgeRef, depth_first_search},
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_index::IndexVec;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, Reference, ScopeId, SymbolId};
use oxc_span::GetSpan;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_useless_assignment_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("This assigned value is not used in subsequent statements.")
        .with_help("Consider removing or reusing the assigned value.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessAssignment;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Flags assignments where the newly assigned value is never read afterward (a "dead store"). This helps catch wasted work or accidental mistakes.
    ///
    /// ### Why is this bad?
    ///
    /// Dead stores add noise and can hide real bugs (e.g., you meant to use that value or wrote to the wrong variable). Removing them improves clarity and performance.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /* eslint no-useless-assignment: "error" */
    ///
    /// function fn1() {
    ///   let v = 'used';
    ///   doSomething(v);
    ///   v = 'unused';              // assigned but never read
    /// }
    ///
    /// function fn2() {
    ///   let v = 'used';
    ///   if (condition) {
    ///     v = 'unused';            // early return; this write is never observed
    ///     return;
    ///   }
    ///   doSomething(v);
    /// }
    ///
    /// function fn3() {
    ///   let v = 'used';
    ///   if (condition) {
    ///     doSomething(v);
    ///   } else {
    ///     v = 'unused';            // value not used later in this branch
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    ///
    /// function fn1() {
    ///   let v = 'used';
    ///   doSomething(v);
    ///   v = 'used-2';
    ///   doSomething(v);            // the reassigned value is read
    /// }
    ///
    /// function fn2() {
    ///   let v = 'used';
    ///   if (condition) {
    ///     v = 'used-2';
    ///     doSomething(v);          // reassignment is observed before returning
    ///     return;
    ///   }
    ///   doSomething(v);
    /// }
    ///
    /// function fn3() {
    ///   let v = 'used';
    ///   for (let i = 0; i < 10; i++) {
    ///     doSomething(v);
    ///     v = 'used in next iteration'; // used on the next loop pass
    ///   }
    /// }
    /// ```
    NoUselessAssignment,
    eslint,
    nursery,
    version = "1.59.0",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Read,
    Write,
}

#[derive(Debug, Clone, Copy)]
pub struct OpAtNode {
    pub op: Operation,
    pub node: NodeId,
    pub compact_idx: u32,
}

pub type BlockOps = Vec<OpAtNode>;

pub type CfgOps = IndexVec<BasicBlockId, BlockOps>;

pub type CfgTraverseState<'a> = IndexVec<BasicBlockId, BitSet<'a>>;

struct TrackedSymbol {
    symbol_id: SymbolId,
    scope_id: ScopeId,
    is_used: bool,
    is_exported: bool,
    has_captured_read: bool,
}

impl Rule for NoUselessAssignment {
    fn run_once(&self, ctx: &LintContext) {
        let allocator = Allocator::default();
        let graph = ctx.cfg().graph();
        let num_blocks = ctx.cfg().basic_blocks.len();

        // Single pass: collect ops and build tracking data.
        let mut cfg_ops: CfgOps = IndexVec::with_capacity(num_blocks);
        cfg_ops.resize_with(num_blocks, Vec::new);
        let mut tracked_symbols: Vec<TrackedSymbol> = Vec::new();

        for symbol_id in ctx.scoping().symbol_ids() {
            let decl_node = ctx.symbol_declaration(symbol_id);
            let AstKind::VariableDeclarator(var_decl) = decl_node.kind() else { continue };
            if let AstKind::VariableDeclaration(var_declaration) =
                ctx.nodes().parent_node(decl_node.id()).kind()
                && var_declaration.kind == VariableDeclarationKind::Const
            {
                continue;
            }
            if matches!(
                &var_decl.init,
                Some(Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_))
            ) {
                continue;
            }

            #[expect(clippy::cast_possible_truncation)]
            let compact_idx = tracked_symbols.len() as u32;
            tracked_symbols.push(TrackedSymbol {
                symbol_id,
                scope_id: ctx.scoping().symbol_scope_id(symbol_id),
                is_used: false,
                is_exported: Self::is_exported(ctx, symbol_id, decl_node),
                has_captured_read: false,
            });

            // Collect ops for this symbol (formerly Pass 2)
            if var_decl.init.is_some() {
                Self::push_op(
                    ctx,
                    graph,
                    &mut cfg_ops,
                    decl_node.id(),
                    Operation::Write,
                    compact_idx,
                );
            }

            // Process references inline with reordering for assignment expressions like a = a + 1
            let references = ctx.symbol_references(symbol_id);
            let mut pending_assignment_lhs: Option<(&Reference, bool)> = None;

            for reference in references {
                if let Some((lhs, previous_value_read)) = pending_assignment_lhs
                    && let Some(assign_node_id) = Self::get_assignment_node(ctx, lhs)
                {
                    let assign_node = ctx.nodes().get_node(assign_node_id);
                    if assign_node
                        .span()
                        .contains_inclusive(ctx.nodes().get_node(reference.node_id()).span())
                    {
                        if reference.is_read() && !reference.is_write() {
                            pending_assignment_lhs = Some((lhs, true));
                        }
                        Self::process_reference_deferred(
                            ctx,
                            graph,
                            &mut cfg_ops,
                            reference,
                            compact_idx,
                            var_decl,
                            decl_node,
                            &mut tracked_symbols[compact_idx as usize],
                            false,
                        );
                        continue;
                    }
                    Self::process_reference_deferred(
                        ctx,
                        graph,
                        &mut cfg_ops,
                        lhs,
                        compact_idx,
                        var_decl,
                        decl_node,
                        &mut tracked_symbols[compact_idx as usize],
                        previous_value_read,
                    );
                    pending_assignment_lhs = None;
                }

                if reference.is_write() && Self::get_assignment_node(ctx, reference).is_some() {
                    if let Some((prev, previous_value_read)) = pending_assignment_lhs.take() {
                        Self::process_reference_deferred(
                            ctx,
                            graph,
                            &mut cfg_ops,
                            prev,
                            compact_idx,
                            var_decl,
                            decl_node,
                            &mut tracked_symbols[compact_idx as usize],
                            previous_value_read,
                        );
                    }
                    pending_assignment_lhs = Some((reference, reference.is_read()));
                } else {
                    Self::process_reference_deferred(
                        ctx,
                        graph,
                        &mut cfg_ops,
                        reference,
                        compact_idx,
                        var_decl,
                        decl_node,
                        &mut tracked_symbols[compact_idx as usize],
                        false,
                    );
                }
            }

            if let Some((lhs, previous_value_read)) = pending_assignment_lhs {
                Self::process_reference_deferred(
                    ctx,
                    graph,
                    &mut cfg_ops,
                    lhs,
                    compact_idx,
                    var_decl,
                    decl_node,
                    &mut tracked_symbols[compact_idx as usize],
                    previous_value_read,
                );
            }
        }

        let num_tracked = tracked_symbols.len();

        // Early exit if no symbols to track
        if num_tracked == 0 {
            return;
        }

        let mut cfg_traverse_state: CfgTraverseState<'_> =
            CfgTraverseState::with_capacity(num_blocks);
        cfg_traverse_state.resize_with(num_blocks, || BitSet::new_in(num_tracked, &allocator));

        let mut scratch_live = BitSet::new_in(num_tracked, &allocator);
        let mut scratch_catch = BitSet::new_in(num_tracked, &allocator);

        // Pre-allocate scratch BitSets for loop analysis (reused via clear())
        let mut scratch_loop_req = BitSet::new_in(num_tracked, &allocator);
        let mut scratch_loop_visited = BitSet::new_in(num_blocks, &allocator);
        let mut scratch_loop_killed = BitSet::new_in(num_tracked, &allocator);
        let mut scratch_find_loop = BitSet::new_in(graph.node_count(), &allocator);
        let mut cached_loop_liveness: Vec<Option<BitSet<'_>>> =
            std::iter::repeat_with(|| None).take(graph.node_count()).collect();

        depth_first_search(
            graph,
            Some(ctx.nodes().cfg_id(ctx.nodes().get_node(NodeId::ROOT).id())),
            |e| match e {
                // backtrack and merge child block symbol operations
                DfsEvent::Finish(block_node_id, _) => {
                    let current_block_id = *graph
                        .node_weight(block_node_id)
                        .expect("expected a valid node id in graph");
                    scratch_live.clear();
                    scratch_catch.clear();

                    let successors = graph.edges_directed(block_node_id, Direction::Outgoing);

                    for edge in successors {
                        let succ_id = *graph
                            .node_weight(edge.target())
                            .expect("expected a valid node id in graph");

                        match edge.weight() {
                            // Normal Flow: We will process these through the block's Ops
                            EdgeType::Normal
                            | EdgeType::NewFunction
                            | EdgeType::Finalize
                            | EdgeType::Join => {
                                scratch_live.union(&cfg_traverse_state[succ_id]);
                            }
                            EdgeType::Jump => {
                                scratch_live.union(&cfg_traverse_state[succ_id]);

                                // `continue` edges are modeled as `Jump`s to the loop header, so
                                // account for values that are first observed on the next iteration.
                                if Self::is_continue_to_loop_header(
                                    ctx.cfg(),
                                    graph,
                                    block_node_id,
                                    edge.target(),
                                ) {
                                    Self::merge_loop_liveness(
                                        &allocator,
                                        graph,
                                        edge.target(),
                                        &cfg_ops,
                                        &mut cached_loop_liveness,
                                        &mut scratch_loop_req,
                                        &mut scratch_live,
                                        &mut scratch_loop_visited,
                                        &mut scratch_loop_killed,
                                    );
                                }
                            }
                            // Error Flow: This is the "Branch" that bypasses this block's Ops
                            EdgeType::Error(_) => {
                                scratch_catch.union(&cfg_traverse_state[succ_id]);
                            }
                            EdgeType::Backedge => {
                                scratch_find_loop.clear();
                                if let Some(loop_header) = Self::find_loop_start(
                                    graph,
                                    block_node_id,
                                    &mut scratch_find_loop,
                                ) {
                                    let loop_header_block_id = *graph
                                        .node_weight(loop_header)
                                        .expect("expected a valid node id in graph");

                                    scratch_live.union(&cfg_traverse_state[loop_header_block_id]);

                                    Self::merge_loop_liveness(
                                        &allocator,
                                        graph,
                                        loop_header,
                                        &cfg_ops,
                                        &mut cached_loop_liveness,
                                        &mut scratch_loop_req,
                                        &mut scratch_live,
                                        &mut scratch_loop_visited,
                                        &mut scratch_loop_killed,
                                    );
                                }
                            }
                            EdgeType::Unreachable => {}
                        }
                    }

                    let mut is_in_try_block = None;

                    // Walk back from the end of the block to the start
                    for op in cfg_ops[current_block_id].iter().rev() {
                        let compact_idx = op.compact_idx as usize;
                        let tracked_symbol = &tracked_symbols[compact_idx];

                        if !tracked_symbol.is_used && !tracked_symbol.is_exported {
                            continue;
                        }

                        match op.op {
                            Operation::Write => {
                                if !scratch_live.has_bit(compact_idx)
                                    && !scratch_catch.has_bit(compact_idx)
                                    && !tracked_symbol.is_exported
                                    && !tracked_symbol.has_captured_read
                                    && !*is_in_try_block.get_or_insert_with(|| {
                                        Self::is_in_try_block(graph, block_node_id)
                                    })
                                    && Self::has_same_parent_variable_scope(
                                        ctx,
                                        tracked_symbol.scope_id,
                                        ctx.nodes().get_node(op.node).scope_id(),
                                    )
                                {
                                    let symbol_id = tracked_symbol.symbol_id;
                                    let span =
                                        if ctx.scoping().symbol_declaration(symbol_id) == op.node {
                                            ctx.scoping().symbol_span(symbol_id)
                                        } else {
                                            ctx.nodes().get_node(op.node).span()
                                        };
                                    ctx.diagnostic(no_useless_assignment_diagnostic(span));
                                }
                                scratch_live.unset_bit(compact_idx);
                            }
                            Operation::Read => {
                                scratch_live.set_bit(compact_idx);
                            }
                        }
                    }

                    scratch_live.union(&scratch_catch);

                    std::mem::swap(&mut scratch_live, &mut cfg_traverse_state[current_block_id]);

                    Control::<()>::Continue
                }
                _ => Control::Continue,
            },
        );
    }
}

impl NoUselessAssignment {
    fn block_id_for_node(ctx: &LintContext, graph: &Graph, node_id: NodeId) -> BasicBlockId {
        *graph.node_weight(ctx.nodes().cfg_id(node_id)).expect("expected a valid node id in graph")
    }

    fn push_op(
        ctx: &LintContext,
        graph: &Graph,
        cfg_ops: &mut CfgOps,
        node: NodeId,
        op: Operation,
        compact_idx: u32,
    ) {
        let block_id = Self::block_id_for_node(ctx, graph, node);
        cfg_ops[block_id].push(OpAtNode { op, node, compact_idx });
    }

    fn is_exported(
        ctx: &LintContext,
        symbol_id: SymbolId,
        decl_node: &oxc_semantic::AstNode,
    ) -> bool {
        let symbol_name = ctx.scoping().symbol_name(symbol_id);
        ctx.module_record().exported_bindings.contains_key(symbol_name)
            || ctx.module_record().local_export_entries.iter().any(|e| e.span == decl_node.span())
    }

    #[expect(clippy::too_many_arguments)]
    fn process_reference_deferred(
        ctx: &LintContext,
        graph: &Graph,
        cfg_ops: &mut CfgOps,
        reference: &Reference,
        compact_idx: u32,
        var_decl: &oxc_ast::ast::VariableDeclarator,
        decl_node: &oxc_semantic::AstNode,
        tracked_symbol: &mut TrackedSymbol,
        previous_value_read: bool,
    ) {
        let op_node = reference.node_id();

        if reference.is_read() {
            Self::push_op(ctx, graph, cfg_ops, op_node, Operation::Read, compact_idx);
            tracked_symbol.is_used = true;
            if !tracked_symbol.has_captured_read
                && !Self::has_same_parent_variable_scope(
                    ctx,
                    tracked_symbol.scope_id,
                    ctx.nodes().get_node(op_node).scope_id(),
                )
            {
                tracked_symbol.has_captured_read = true;
            }
        }

        if reference.is_write() {
            if previous_value_read && !reference.is_read() {
                // Model RHS self-reads before the deferred assignment write.
                // This keeps the previous value live without making nested
                // read-write expressions like `x = x++` live after their write.
                Self::push_op(ctx, graph, cfg_ops, op_node, Operation::Read, compact_idx);
            }

            if matches!(
                &var_decl.id,
                BindingPattern::ObjectPattern(_) | BindingPattern::ArrayPattern(_)
            ) && decl_node
                .span()
                .contains_inclusive(ctx.nodes().get_node(reference.node_id()).span())
            {
                return;
            }

            Self::push_op(ctx, graph, cfg_ops, op_node, Operation::Write, compact_idx);
        }
    }

    fn find_loop_start(
        graph: &Graph,
        loop_end: BlockNodeId,
        visited: &mut BitSet,
    ) -> Option<BlockNodeId> {
        let mut current = loop_end;
        let mut last: Option<BlockNodeId> = None;

        loop {
            let idx = current.index();
            if visited.has_bit(idx) {
                break;
            }
            visited.set_bit(idx);

            let mut next_backedge: Option<BlockNodeId> = None;
            for edge in graph.edges_directed(current, Direction::Outgoing) {
                if matches!(edge.weight(), EdgeType::Backedge) {
                    next_backedge = Some(edge.target());
                    break;
                }
            }

            if let Some(target) = next_backedge {
                last = Some(target);
                current = target;
            } else {
                break;
            }
        }

        last
    }

    fn get_assignment_node(ctx: &LintContext, reference: &Reference) -> Option<NodeId> {
        let node = ctx.nodes().get_node(reference.node_id());
        let parent_node = ctx.nodes().parent_node(node.id());
        if matches!(node.kind(), AstKind::IdentifierReference(_))
            && matches!(parent_node.kind(), AstKind::AssignmentExpression(_))
        {
            Some(parent_node.id())
        } else {
            None
        }
    }

    fn is_in_try_block(graph: &Graph, block_node_id: BlockNodeId) -> bool {
        graph.edges_directed(block_node_id, Direction::Outgoing).any(|e| {
            matches!(e.weight(), EdgeType::Error(ErrorEdgeKind::Explicit) | EdgeType::Finalize)
        })
    }

    fn get_parent_variable_scope(ctx: &LintContext, scope_id: ScopeId) -> ScopeId {
        ctx.scoping()
            .scope_ancestors(scope_id)
            .find_or_last(|scope_id| ctx.scoping().scope_flags(*scope_id).is_var())
            .expect("scope iterator will always contain at least one element")
    }

    fn has_same_parent_variable_scope(
        ctx: &LintContext,
        scope_id_a: ScopeId,
        scope_id_b: ScopeId,
    ) -> bool {
        Self::get_parent_variable_scope(ctx, scope_id_a)
            == Self::get_parent_variable_scope(ctx, scope_id_b)
    }

    fn analyze_loop_recursive(
        graph: &Graph,
        node: BlockNodeId,
        loop_header_id: BlockNodeId,
        cfg_ops: &CfgOps,
        result_gen: &mut BitSet,
        killed_on_path: &mut BitSet,
        visited: &mut BitSet,
    ) {
        let block_id = *graph.node_weight(node).expect("expected a valid node id in graph");

        if visited.has_bit(block_id.index()) {
            return;
        }
        visited.set_bit(block_id.index());

        // Track bits we set in THIS block so we can undo them later
        let mut newly_killed: SmallVec<[usize; 8]> = SmallVec::new();

        for op in &cfg_ops[block_id] {
            let compact_idx = op.compact_idx as usize;

            if result_gen.has_bit(compact_idx) || killed_on_path.has_bit(compact_idx) {
                continue;
            }

            match op.op {
                Operation::Read => {
                    result_gen.set_bit(compact_idx);
                }
                Operation::Write => {
                    killed_on_path.set_bit(compact_idx);
                    newly_killed.push(compact_idx);
                }
            }
        }

        for edge in graph.edges_directed(node, Direction::Outgoing) {
            match edge.weight() {
                EdgeType::Normal | EdgeType::Jump | EdgeType::NewFunction | EdgeType::Backedge => {
                    let target = edge.target();
                    if target == loop_header_id {
                        continue;
                    }

                    Self::analyze_loop_recursive(
                        graph,
                        target,
                        loop_header_id,
                        cfg_ops,
                        result_gen,
                        killed_on_path,
                        visited,
                    );
                }
                _ => {}
            }
        }

        // BACKTRACK: Remove only the bits that this specific block call added
        for sym_idx in newly_killed {
            killed_on_path.unset_bit(sym_idx);
        }
    }

    fn merge_loop_liveness<'a>(
        allocator: &'a Allocator,
        graph: &Graph,
        loop_header: BlockNodeId,
        cfg_ops: &CfgOps,
        cached_loop_liveness: &mut [Option<BitSet<'a>>],
        scratch_loop_req: &mut BitSet<'a>,
        scratch_live: &mut BitSet<'a>,
        scratch_loop_visited: &mut BitSet<'a>,
        scratch_loop_killed: &mut BitSet<'a>,
    ) {
        if let Some(loop_liveness) = cached_loop_liveness[loop_header.index()].as_ref() {
            scratch_live.union(loop_liveness);
            return;
        }

        scratch_loop_req.clear();
        scratch_loop_visited.clear();
        scratch_loop_killed.clear();

        Self::analyze_loop_recursive(
            graph,
            loop_header,
            loop_header,
            cfg_ops,
            scratch_loop_req,
            scratch_loop_killed,
            scratch_loop_visited,
        );

        cached_loop_liveness[loop_header.index()] = Some(scratch_loop_req.clone_in(allocator));
        scratch_live.union(scratch_loop_req);
    }

    fn is_continue_to_loop_header(
        cfg: &ControlFlowGraph,
        graph: &Graph,
        source: BlockNodeId,
        target: BlockNodeId,
    ) -> bool {
        graph
            .edges_directed(target, Direction::Incoming)
            .any(|edge| matches!(edge.weight(), EdgeType::Backedge))
            && cfg.is_reachable(target, source)
    }
}
#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "let v = 'used';
                    console.log(v);
                    v = 'used-2'
                    console.log(v);",
        "function foo() {
                        let v = 'used';
                        console.log(v);
                        v = 'used-2';
                        console.log(v);
                    }",
        "function foo() {
                        let v = 'used';
                        if (condition) {
                            v = 'used-2';
                            console.log(v);
                            return
                        }
                        console.log(v);
                    }",
        "function foo() {
                        let v = 'used';
                        if (condition) {
                            console.log(v);
                        } else {
                            v = 'used-2';
                            console.log(v);
                        }
                    }",
        "function foo() {
                        let v = 'used';
                        if (condition) {
                            //
                        } else {
                            v = 'used-2';
                        }
                        console.log(v);
                    }",
        "var foo = function () {
                        let v = 'used';
                        console.log(v);
                        v = 'used-2'
                        console.log(v);
                    }",
        "var foo = () => {
                        let v = 'used';
                        console.log(v);
                        v = 'used-2'
                        console.log(v);
                    }",
        "class foo {
                        static {
                            let v = 'used';
                            console.log(v);
                            v = 'used-2'
                            console.log(v);
                        }
                    }",
        "function foo () {
                        let v = 'used';
                        for (let i = 0; i < 10; i++) {
                            console.log(v);
                            v = 'used in next iteration';
                        }
                    }",
        "function foo () {
                        let i = 0;
                        i++;
                        i++;
                        console.log(i);
                    }",
        "export let foo = 'used';
                    console.log(foo);
                    foo = 'unused like but exported';",
        "export function foo () {};
                    console.log(foo);
                    foo = 'unused like but exported';",
        "export class foo {};
                    console.log(foo);
                    foo = 'unused like but exported';",
        "export default function foo () {};
                    console.log(foo);
                    foo = 'unused like but exported';",
        "export default class foo {};
                    console.log(foo);
                    foo = 'unused like but exported';",
        "let foo = 'used';
                    export { foo };
                    console.log(foo);
                    foo = 'unused like but exported';",
        "function foo () {};
                    export { foo };
                    console.log(foo);
                    foo = 'unused like but exported';",
        "class foo {};
                    export { foo };
                    console.log(foo);
                    foo = 'unused like but exported';",
        // `exported` comments aren't supported
        // "/* exported foo */
        //                 let foo = 'used';
        //                 console.log(foo);
        //                 foo = 'unused like but exported with directive';", // { "sourceType": "script" },
        // test/use-a comments aren't supported
        // "/*eslint test/use-a:1*/
        //             let a = 'used';
        //             console.log(a);
        //             a = 'unused like but marked by markVariableAsUsed()';
        //             ",
        "v = 'used';
                    console.log(v);
                    v = 'unused'",
        "let v = 'used variable';",
        "function foo() {
                        return;

                        const x = 1;
                        if (y) {
                            bar(x);
                        }
                    }",
        "function foo() {
                        const x = 1;
                        console.log(x);
                        return;

                        x = 'Foo'
                    }",
        "function foo() {
                        let a = 42;
                        console.log(a);
                        a++;
                        console.log(a);
                    }",
        "function foo() {
                        let a = 42;
                        console.log(a);
                        a--;
                        console.log(a);
                    }",
        "function foo() {
                        let a = 42;
                        console.log(a);
                        a = 10;
                        a = a + 1;
                        console.log(a);
                    }",
        "function foo() {
                        let a = 42;
                        console.log(a);
                        a = 10;
                        if (cond) {
                            a = a + 1;
                        } else {
                            a = 2 + a;
                        }
                        console.log(a);
                    }",
        "function foo() {
                        let a = 'used', b = 'used', c = 'used', d = 'used';
                        console.log(a, b, c, d);
                        ({ a, arr: [b, c, ...d] } = fn());
                        console.log(a, b, c, d);
                    }",
        "function foo() {
                        let a = 'used', b = 'used', c = 'used';
                        console.log(a, b, c);
                        ({ a = 'unused', foo: b, ...c } = fn());
                        console.log(a, b, c);
                    }",
        "function foo() {
                        let a = {};
                        console.log(a);
                        a.b = 'unused like, but maybe used in setter';
                    }",
        "function foo() {
                        let a = { b: 42 };
                        console.log(a);
                        a.b++;
                    }",
        "function foo () {
                        let v = 'used';
                        console.log(v);
                        function bar() {
                            v = 'used in outer scope';
                        }
                        bar();
                        console.log(v);
                    }",
        "function foo () {
                        let v = 'used';
                        console.log(v);
                        setTimeout(() => console.log(v), 1);
                        v = 'used in other scope';
                    }",
        "function foo () {
                        let v = 'used';
                        console.log(v);
                        for (let i = 0; i < 10; i++) {
                            if (condition) {
                                v = 'maybe used';
                                continue;
                            }
                            console.log(v);
                        }
                    }",
        "/* globals foo */
                    const bk = foo;
                    foo = 42;
                    try {
                        // process
                    } finally {
                        foo = bk;
                    }",
        "
                        const bk = console;
                        console = { log () {} };
                        try {
                            // process
                        } finally {
                            console = bk;
                        }", // { "globals": { "console": false }, },
        "let message = 'init';
                    try {
                        const result = call();
                        message = result.message;
                    } catch (e) {
                        // ignore
                    }
                    console.log(message)",
        "let message = 'init';
                    try {
                        message = call().message;
                    } catch (e) {
                        // ignore
                    }
                    console.log(message)",
        "let v = 'init';
                    try {
                        v = callA();
                        try {
                            v = callB();
                        } catch (e) {
                            // ignore
                        }
                    } catch (e) {
                        // ignore
                    }
                    console.log(v)",
        "let v = 'init';
                    try {
                        try {
                            v = callA();
                        } catch (e) {
                            // ignore
                        }
                    } catch (e) {
                        // ignore
                    }
                    console.log(v)",
        "let a;
                    try {
                        foo();
                    } finally {
                        a = 5;
                    }
                    console.log(a);",
        "const obj = { a: 5 };
                    const { a, b = a } = obj;
                    console.log(b); // 5",
        "const arr = [6];
                    const [c, d = c] = arr;
                    console.log(d); // 6",
        "const obj = { a: 1 };
                    let {
                        a,
                        b = (a = 2)
                    } = obj;
                    console.log(a, b);",
        "let { a, b: {c = a} = {} } = obj;
                    console.log(c);",
        "function foo(){
                        let bar;
                        try {
                            bar = 2;
                            unsafeFn();
                            return { error: undefined };
                        } catch {
                            return { bar };
                        }
                    }
                    function unsafeFn() {
                        throw new Error();
                    }",
        "function foo(){
                        let bar, baz;
                        try {
                            bar = 2;
                            unsafeFn();
                            return { error: undefined };
                        } catch {
                           baz = bar;
                        }
                        return baz;
                    }
                    function unsafeFn() {
                        throw new Error();
                    }",
        "function foo(){
                        let bar;
                        try {
                            bar = 2;
                            unsafeFn();
                            bar = 4;
                        } catch {
                           // handle error
                        }
                        return bar;
                    }
                    function unsafeFn() {
                        throw new Error();
                    }",
        // test comments aren't supported
        // r#"/*eslint test/unknown-ref:1*/
        //             let a = "used";
        //             console.log(a);
        //             a = "unused";"#,
        // r#"/*eslint test/unknown-ref:1*/
        //             function foo() {
        //                 let a = "used";
        //                 console.log(a);
        //                 a = "unused";
        //             }"#,
        // r#"/*eslint test/unknown-ref:1*/
        //             function foo() {
        //                 let a = "used";
        //                 if (condition) {
        //                     a = "unused";
        //                     return
        //                 }
        //                 console.log(a);
        //             }"#,
        r#"
                            function App() {
                                const A = "";
                                return <A/>;
                            }
                        "#, // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        r#"
                            function App() {
                                let A = "";
                                foo(A);
                                A = "A";
                                return <A/>;
                            }
                        "#, // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        r#"
                            function App() {
                                let A = "a";
                                foo(A);
                                return <A/>;
                            }
                        "#, // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        "function App() {
                            let x = 0;
                            foo(x);
                            x = 1;
                            return <A prop={x} />;
                        }", // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        r#"function App() {
                            let x = "init";
                            foo(x);
                            x = "used";
                            return <A>{x}</A>;
                        }"#, // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        "function App() {
                            let props = { a: 1 };
                            foo(props);
                            props = { b: 2 };
                            return <A {...props} />;
                        }", // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        "function App() {
                            let NS = Lib;
                            return <NS.Cmp />;
                        }", // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        "function App() {
                            let a = 0;
                            a++;
                            return <A prop={a} />;
                        }", // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        "function App() {
                            const obj = { a: 1 };
                            const { a, b = a } = obj;
                            return <A prop={b} />;
                        }", // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        "function App() {
                            let { a, b: { c = a } = {} } = obj;
                            return <A prop={c} />;
                        }", // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        r#"function App() {
                            let x = "init";
                            if (cond) {
                                x = "used";
                                return <A prop={x} />;
                            }
                            return <A prop={x} />;
                        }"#, // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        "function App() {
                            let A;
                            if (cond) {
                              A = Foo;
                            } else {
                              A = Bar;
                            }
                            return <A />;
                        }", // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        "function App() {
                            let m;
                            try {
                              m = 2;
                              unsafeFn();
                              m = 4;
                            } catch (e) {
                              // ignore
                            }
                            return <A prop={m} />;
                        }", // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        "function App() {
                            const arr = [6];
                            const [c, d = c] = arr;
                            return <A prop={d} />;
                        }", // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        "function App() {
                            const obj = { a: 1 };
                            let {
                              a,
                              b = (a = 2)
                            } = obj;
                            return <A prop={a} />;
                        }", // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, }
        "
            let index = 0;
            while (index < length) {
                if (condition) {
                    index++;
                    continue;
                }
                while (index < length2) {
                    index++;
                }
            }
        ",
        "function createStore() {
                        const options = { onTrigger: undefined };
                        let isListening = false;
                        options.onTrigger = () => {
                            if (isListening) {
                                console.log('event');
                            }
                        };
                        isListening = true;
                        return options;
                    }",
        "let state = 0;
                    const api = { read: () => state };
                    state = 1;
                    export { api };",
        "function foo() {
                        let x = 0;
                        (() => console.log(x))();
                        x = 1;
                    }",
        "const rgb2lab = (rgb: RGB): LAB => {

    let [r, g, b] = rgb;

    r = (r > 0) ? ((r + 0) / 1) ** 2 : r / 1;
    g = (g > 0) ? ((g + 0) / 1) ** 2 : g / 1;
    b = (b > 0) ? ((b + 0) / 1) ** 2 : b / 1;

    let x = (r * 0 + g * 0 + b * 0) / 0;
    let y = (r * 0 + g * 0 + b * 0) / 1;
    let z = (r * 0 + g * 0 + b * 0) / 1;

    x = (x > 0) ? Math.cbrt(x) : (1 * x) + 16/116;
    y = (y > 0) ? Math.cbrt(y) : (1 * y) + 16/116;
    z = (z > 0) ? Math.cbrt(z) : (1 * z) + 16/116;

    return [(1 * y) - 16, 1 * (x - y), 1 * (y - z)];
};",
    ];

    let fail = vec![
        "let v = 'used';
                        console.log(v);
                        v = 'unused'",
        "function foo() {
                            let v = 'used';
                            console.log(v);
                            v = 'unused';
                        }",
        "function foo() {
                            let v = 'used';
                            if (condition) {
                                v = 'unused';
                                return
                            }
                            console.log(v);
                        }",
        "function foo() {
                            let v = 'used';
                            if (condition) {
                                console.log(v);
                            } else {
                                v = 'unused';
                            }
                        }",
        "var foo = function () {
                            let v = 'used';
                            console.log(v);
                            v = 'unused'
                        }",
        "var foo = () => {
                            let v = 'used';
                            console.log(v);
                            v = 'unused'
                        }",
        "class foo {
                            static {
                                let v = 'used';
                                console.log(v);
                                v = 'unused'
                            }
                        }",
        "function foo() {
                            let v = 'unused';
                            if (condition) {
                                v = 'used';
                                console.log(v);
                                return
                            }
                        }",
        "function foo() {
                            let v = 'used';
                            console.log(v);
                            v = 'unused';
                            v = 'unused';
                        }",
        "function foo() {
                            let v = 'used';
                            console.log(v);
                            v = 'unused';
                            v = 'used';
                            console.log(v);
                            v = 'used';
                            console.log(v);
                        }",
        "
                        let v;
                        v = 'unused';
                        if (foo) {
                            v = 'used';
                        } else {
                            v = 'used';
                        }
                        console.log(v);",
        "function foo() {
                            let v = 'used';
                            console.log(v);
                            v = 'unused';
                            v = 'unused';
                            v = 'used';
                            console.log(v);
                        }",
        "function foo() {
                            let v = 'unused';
                            if (condition) {
                                if (condition2) {
                                    v = 'used-2';
                                } else {
                                    v = 'used-3';
                                }
                            } else {
                                v = 'used-4';
                            }
                            console.log(v);
                        }",
        "function foo() {
                            let v;
                            if (condition) {
                                v = 'unused';
                            } else {
                                //
                            }
                            if (condition2) {
                                v = 'used-1';
                            } else {
                                v = 'used-2';
                            }
                            console.log(v);
                        }",
        "function foo() {
                            let v = 'used';
                            if (condition) {
                                v = 'unused';
                                v = 'unused';
                                v = 'used';
                            }
                            console.log(v);
                        }",
        "function foo() {
                            let a = 42;
                            console.log(a);
                            a++;
                        }",
        "function foo() {
                            let a = 42;
                            console.log(a);
                            a--;
                        }",
        "function foo() {
                            let a = 'used', b = 'used', c = 'used', d = 'used';
                            console.log(a, b, c, d);
                            ({ a, arr: [b, c,, ...d] } = fn());
                            console.log(c);
                        }",
        "function foo() {
                            let a = 'used', b = 'used', c = 'used';
                            console.log(a, b, c);
                            ({ a = 'unused', foo: b, ...c } = fn());
                        }",
        "function foo () {
                            let v = 'used';
                            console.log(v);
                            setTimeout(() => v = 42, 1);
                            v = 'unused and variable is only updated in other scopes';
                        }",
        "function foo() {
                            let v = 'used';
                            if (condition) {
                                let v = 'used';
                                console.log(v);
                                v = 'unused';
                            }
                            console.log(v);
                            v = 'unused';
                        }",
        "function foo() {
                            let v = 'used';
                            if (condition) {
                                console.log(v);
                                v = 'unused';
                            } else {
                                v = 'unused';
                            }
                        }",
        "function foo () {
                            let v = 'used';
                            console.log(v);
                            v = 'unused';
                            return;
                            console.log(v);
                        }",
        "function foo () {
                            let v = 'used';
                            console.log(v);
                            v = 'unused';
                            throw new Error();
                            console.log(v);
                        }",
        "function foo () {
                            let v = 'used';
                            console.log(v);
                            for (let i = 0; i < 10; i++) {
                                v = 'unused';
                                continue;
                                console.log(v);
                            }
                        }
                        function bar () {
                            let v = 'used';
                            console.log(v);
                            for (let i = 0; i < 10; i++) {
                                v = 'unused';
                                break;
                                console.log(v);
                            }
                        }",
        "function foo () {
                            let v = 'used';
                            console.log(v);
                            for (let i = 0; i < 10; i++) {
                                if (condition) {
                                    v = 'unused';
                                    break;
                                }
                                console.log(v);
                            }
                        }",
        "let message = 'unused';
                        try {
                            const result = call();
                            message = result.message;
                        } catch (e) {
                            message = 'used';
                        }
                        console.log(message)",
        "let message = 'unused';
                        try {
                            message = 'used';
                            console.log(message)
                        } catch (e) {
                        }",
        "let message = 'unused';
                        try {
                            message = call();
                        } catch (e) {
                            message = 'used';
                        }
                        console.log(message)",
        "let v = 'unused';
                        try {
                            v = callA();
                            try {
                                v = callB();
                            } catch (e) {
                                // ignore
                            }
                        } catch (e) {
                            v = 'used';
                        }
                        console.log(v)",
        "
                        var x = 1; // used
                        x = x + 1; // unused
                        x = 5; // used
                        f(x);",
        "
                        var x = 1; // used
                        x = // used
                            x++; // unused
                        f(x);",
        "const obj = { a: 1 };
                        let {
                            a,
                            b = (a = 2)
                        } = obj;
                        a = 3
                        console.log(a, b);",
        "const arr = [1, 2];
                        let [
                            a,
                            b
                        ] = arr;
                        a = 3
                        console.log(a, b);",
        r#"function App() {
                        let A = "unused";
                        A = "used";
                        return <A/>;
                        }"#, // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        r#"function App() {
                        let A = "unused";
                        A = "used";
                        return <A></A>;
                        }"#, // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        r#"function App() {
                        let A = "unused";
                        A = "used";
                        return <A.B />;
                        }"#, // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        r#"function App() {
                        let x = "used";
                        if (cond) {
                          return <A prop={x} />;
                        } else {
                          x = "unused";
                        }
                        }"#, // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        r#"function App() {
                        let A;
                        A = "unused";
                        if (cond) {
                          A = "used1";
                        } else {
                          A = "used2";
                        }
                        return <A/>;
                        }"#, // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        "function App() {
                        let message = 'unused';
                        try {
                          const result = call();
                          message = result.message;
                        } catch (e) {
                          message = 'used';
                        }
                        return <A prop={message} />;
                        }", // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        "function App() {
                        let x = 1;
                        x = x + 1;
                        x = 5;
                        return <A prop={x} />;
                        }", // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        "function App() {
                        let x = 1;
                        x = 2;
                        return <A>{x}</A>;
                        }", // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
        "function App() {
                        let x = 0;
                        x = 1;
                        x = 2;
                        return <A prop={x} />;
                        }", // { "parserOptions": { "ecmaFeatures": { "jsx": true }, }, },
    ];

    Tester::new(NoUselessAssignment::NAME, NoUselessAssignment::PLUGIN, pass, fail)
        .test_and_snapshot();
}
