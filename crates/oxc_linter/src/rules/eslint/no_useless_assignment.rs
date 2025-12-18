use itertools::Itertools;
use oxc_allocator::{Allocator, BitSet};
use oxc_ast::{
    AstKind,
    ast::{BindingPatternKind, Expression, VariableDeclarationKind},
};
use oxc_cfg::{
    BasicBlockId, BlockNodeId, EdgeType, ErrorEdgeKind,
    graph::{
        Direction,
        visit::{Control, DfsEvent, EdgeRef, depth_first_search},
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_index::IndexVec;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, Reference, ScopeId, SymbolId};
use oxc_span::Span;
use rustc_hash::FxHashSet;

use crate::{context::LintContext, rule::Rule};

use oxc_span::GetSpan;

fn no_useless_assignment_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("This assigned value is not used in subsequent statements.")
        .with_help("Consider removing or reusing the assigned value.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessAssignment;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Flags assignments where the newly assigned value is never read afterward (a “dead store”). This helps catch wasted work or accidental mistakes.
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
    correctness,
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Read = 0,
    Write = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpAtNode {
    pub op: Operation,
    pub node: NodeId,
}

pub type BlockOps = Vec<(SymbolId, Vec<OpAtNode>)>;

pub type CfgOps = IndexVec<BasicBlockId, Option<BlockOps>>;

pub struct TraverseState<'a> {
    // We reuse this single bitset for both Live-Out (temp) and Live-In (final)
    live: BitSet<'a>,
}

impl<'a> TraverseState<'a> {
    pub fn new(num_symbols: usize, allocator: &'a Allocator) -> Self {
        Self { live: BitSet::new_in(num_symbols, allocator) }
    }
}

pub type CfgTraverseState<'a> = IndexVec<BasicBlockId, TraverseState<'a>>;

impl Rule for NoUselessAssignment {
    fn run_once(&self, ctx: &LintContext) {
        let allocator = Allocator::default();
        let num_symbols = ctx.scoping().symbols_len();
        let num_blocks = ctx.cfg().basic_blocks.len();

        let mut cfg_ops: CfgOps = IndexVec::with_capacity(num_blocks);
        cfg_ops.resize_with(num_blocks, || None);

        // Track symbols that are read at least once globally, no-useless assignment ignore variables that are never read anywhere,
        // assume no-unused-vars will catch them
        let mut used_symbols = BitSet::new_in(num_symbols, &allocator);

        let mut cfg_traverse_state: CfgTraverseState<'_> =
            CfgTraverseState::with_capacity(num_blocks);
        // Pre-fill with empty BitSets
        cfg_traverse_state.resize_with(num_blocks, || TraverseState::new(num_symbols, &allocator));

        //walk through all symbols, collect their operations from declarations and references
        for symbol_id in ctx.scoping().symbol_ids() {
            let decl_node = ctx.symbol_declaration(symbol_id);

            if let AstKind::VariableDeclarator(var_decl) = decl_node.kind() {
                //skip const declarations
                if let AstKind::VariableDeclaration(var_declaration) =
                    ctx.nodes().parent_node(decl_node.id()).kind()
                    && var_declaration.kind == VariableDeclarationKind::Const
                {
                    continue;
                }

                // Skip function and arrow function assignments
                if !matches!(
                    &var_decl.init,
                    Some(
                        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
                    )
                ) {
                    let block_id =
                        Self::get_basic_block_id(ctx, ctx.nodes().cfg_id(decl_node.id()));

                    // Ensure outer slot exists
                    if cfg_ops[block_id].is_none() {
                        cfg_ops[block_id] = Some(Vec::new());
                    }
                    let block_ops_vec = cfg_ops[block_id].as_mut().unwrap();

                    // Find or Create entry in Vec (Linear Scan)
                    let ops_vec = if let Some(pos) =
                        block_ops_vec.iter().position(|(id, _)| *id == symbol_id)
                    {
                        &mut block_ops_vec[pos].1
                    } else {
                        block_ops_vec.push((symbol_id, Vec::new()));
                        &mut block_ops_vec.last_mut().unwrap().1
                    };

                    // if there is an initializer, record a write operation at declaration
                    if var_decl.init.is_some() {
                        ops_vec.push(OpAtNode { op: Operation::Write, node: decl_node.id() });
                    }

                    // reorder reference to handle assignment expression like a = a + 1
                    let references = ctx.symbol_references(symbol_id);
                    let ordered_refs = Self::reordered_references(ctx, references);

                    for reference in ordered_refs {
                        let op_node = reference.node_id();

                        if reference.is_read() {
                            let ref_block =
                                Self::get_basic_block_id(ctx, ctx.nodes().cfg_id(op_node));
                            let ref_ops_vec = Self::get_ops_mut(&mut cfg_ops, ref_block, symbol_id);

                            ref_ops_vec.push(OpAtNode { op: Operation::Read, node: op_node });
                            used_symbols.set_bit(symbol_id.index());
                        }

                        if reference.is_write() {
                            if matches!(
                                &var_decl.id.kind,
                                BindingPatternKind::ObjectPattern(_)
                                    | BindingPatternKind::ArrayPattern(_)
                            ) && decl_node.span().contains_inclusive(
                                ctx.nodes().get_node(reference.node_id()).span(),
                            ) {
                                continue;
                            }

                            let ref_block =
                                Self::get_basic_block_id(ctx, ctx.nodes().cfg_id(op_node));
                            let ref_ops_vec = Self::get_ops_mut(&mut cfg_ops, ref_block, symbol_id);

                            ref_ops_vec.push(OpAtNode { op: Operation::Write, node: op_node });
                        }
                    }
                }
            }
        }

        depth_first_search(
            ctx.cfg().graph(),
            Some(ctx.nodes().cfg_id(ctx.nodes().get_node(NodeId::ROOT).id())),
            |e| match e {
                DfsEvent::TreeEdge(a, b) => {
                    let edges = ctx.cfg().graph().edges_connecting(a, b).collect::<Vec<_>>();
                    if edges.iter().any(|e| {
                        matches!(
                            e.weight(),
                            EdgeType::Normal
                                | EdgeType::Jump
                                | EdgeType::NewFunction
                                | EdgeType::Error(ErrorEdgeKind::Explicit)
                                | EdgeType::Finalize
                                | EdgeType::Join
                        )
                    }) {
                        Control::<()>::Continue
                    } else {
                        Control::Prune
                    }
                }

                // backtrack and merge child block symbol operations
                DfsEvent::Finish(block_node_id, _) => {
                    let current_block_id = Self::get_basic_block_id(ctx, block_node_id);
                    let mut live: BitSet<'_> = BitSet::new_in(num_symbols, &allocator);
                    let mut live_from_catch = BitSet::new_in(num_symbols, &allocator);

                    let successors =
                        ctx.cfg().graph().edges_directed(block_node_id, Direction::Outgoing);

                    for edge in successors {
                        let succ_id = Self::get_basic_block_id(ctx, edge.target());

                        match edge.weight() {
                            // Normal Flow: We will process these through the block's Ops
                            EdgeType::Normal
                            | EdgeType::Jump
                            | EdgeType::NewFunction
                            | EdgeType::Finalize
                            | EdgeType::Join => {
                                live.union(&cfg_traverse_state[succ_id].live);
                            }
                            // Error Flow: This is the "Branch" that bypasses this block's Ops
                            EdgeType::Error(_) => {
                                live_from_catch.union(&cfg_traverse_state[succ_id].live);
                            }
                            EdgeType::Backedge => {
                                if let Some(loop_header) = Self::find_loop_start(ctx, block_node_id)
                                {
                                    let loop_header_block_id =
                                        Self::get_basic_block_id(ctx, loop_header);

                                    live.union(&cfg_traverse_state[loop_header_block_id].live);

                                    // FIX 2: Run robust loop analysis
                                    let mut loop_requirements =
                                        BitSet::new_in(num_symbols, &allocator);
                                    let mut visited = BitSet::new_in(num_blocks, &allocator);
                                    let mut killed_on_path =
                                        BitSet::new_in(num_symbols, &allocator);

                                    Self::analyze_loop_recursive(
                                        ctx,
                                        loop_header,
                                        loop_header,
                                        &cfg_ops,
                                        &mut loop_requirements,
                                        &mut killed_on_path,
                                        &mut visited,
                                        &allocator, // <--- Pass the allocator here
                                    );

                                    live.union(&loop_requirements);
                                }
                            }
                            EdgeType::Unreachable => {}
                        }
                    }

                    // Walk back from the end of the block to the start
                    if let Some(block_ops_vec) = &cfg_ops[current_block_id] {
                        for (symbol_id, ops) in block_ops_vec {
                            let sym_idx = symbol_id.index();

                            if !used_symbols.has_bit(sym_idx) && !Self::is_exported(ctx, *symbol_id)
                            {
                                // We don't need to track liveness for unused vars
                                continue;
                            }

                            for op in ops.iter().rev() {
                                match op.op {
                                    Operation::Write => {
                                        // CHECK: Is the value needed by anyone (locally or globally)?
                                        // If 'live' has the bit, it means a later instruction (or a child block) reads it.
                                        // If 'live' is empty, this write is dead.
                                        if !live.has_bit(sym_idx)
                                            && !live_from_catch.has_bit(sym_idx)
                                            && !Self::is_exported(ctx, *symbol_id)
                                            && !Self::is_in_try_block(ctx, block_node_id)
                                            && Self::has_same_parent_variable_scope(
                                                ctx,
                                                ctx.scoping().symbol_scope_id(*symbol_id),
                                                ctx.nodes().get_node(op.node).scope_id(),
                                            )
                                        {
                                            ctx.diagnostic(no_useless_assignment_diagnostic(
                                                ctx.nodes().get_node(op.node).span(),
                                            ));
                                        }
                                        // KILL: we are writing a new value here.
                                        // Therefore, we do not need the value from before this point.
                                        live.unset_bit(sym_idx);
                                    }
                                    Operation::Read => {
                                        // GEN: we are reading the variable.
                                        // Therefore, we need it to be live coming into this op.
                                        live.set_bit(sym_idx);
                                    }
                                }
                            }
                        }
                    }

                    live.union(&live_from_catch);

                    cfg_traverse_state[current_block_id].live = live;

                    Control::Continue
                }
                _ => Control::Continue,
            },
        );
    }
}

impl NoUselessAssignment {
    fn is_exported(ctx: &LintContext, symbol_id: SymbolId) -> bool {
        let symbol_name = ctx.scoping().symbol_name(symbol_id);
        ctx.module_record().exported_bindings.contains_key(symbol_name)
            || ctx.module_record().local_export_entries.iter().any(|e| {
                e.span == ctx.nodes().get_node(ctx.symbol_declaration(symbol_id).id()).span()
            })
    }

    // reorder reference to handle assignment expression like a = a + 1 by move the reference on the right side before the left side
    // This replaces your recursive function.
    // It groups "Assignment Write" with its "RHS Reads" and emits them in Execution Order (RHS first).
    fn reordered_references<'a>(
        ctx: &LintContext,
        references: impl Iterator<Item = &'a Reference>,
    ) -> Vec<&'a Reference> {
        let mut result = Vec::new();
        // Buffer to hold the LHS of an assignment until its RHS is processed
        let mut pending_assignment_lhs: Option<&Reference> = None;

        for reference in references {
            // If we have a pending LHS (e.g. 'a' in 'a = ...'),
            // check if the current 'reference' is inside the RHS of that assignment.
            if let Some(lhs) = pending_assignment_lhs {
                // Get the AssignmentExpression node from the LHS
                if let Some(assign_node_id) = Self::get_assignment_node(ctx, lhs) {
                    let assign_node = ctx.nodes().get_node(assign_node_id);

                    // If current reference is inside the assignment (which implies RHS,
                    // since we already passed the LHS), push it IMMEDIATELY (Read happens first)
                    if assign_node
                        .span()
                        .contains_inclusive(ctx.nodes().get_node(reference.node_id()).span())
                    {
                        result.push(reference);
                        continue;
                    }
                    // We moved past the assignment. Flush the LHS (Write happens last)
                    result.push(lhs);
                    pending_assignment_lhs = None;
                }
            }

            // Is this reference ITSELF a new LHS of an assignment?
            if reference.is_write() && Self::get_assignment_node(ctx, reference).is_some() {
                // Flush any previous pending LHS (chained assignments like a = b = 1)

                if let Some(prev) = pending_assignment_lhs.take() {
                    result.push(prev);
                }

                pending_assignment_lhs = Some(reference);
            } else {
                // Standard read/write, just push it
                result.push(reference);
            }
        }

        // Flush any remaining pending LHS
        if let Some(lhs) = pending_assignment_lhs {
            result.push(lhs);
        }

        result
    }

    fn find_loop_start(ctx: &LintContext, loop_end: BlockNodeId) -> Option<BlockNodeId> {
        let mut current = loop_end;
        let mut last: Option<BlockNodeId> = None;
        let mut visited: FxHashSet<BlockNodeId> = FxHashSet::default();

        // Follow a chain of backedge targets until there is no further backedge.
        // Guard against accidental cycles by tracking visited nodes.
        loop {
            if !visited.insert(current) {
                break;
            }

            let mut next_backedge: Option<BlockNodeId> = None;
            for edge in ctx.cfg().graph().edges_directed(current, Direction::Outgoing) {
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

    fn is_in_try_block(ctx: &LintContext, block_node_id: BlockNodeId) -> bool {
        ctx.cfg().graph().edges_directed(block_node_id, Direction::Outgoing).any(|e| {
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
        ctx: &LintContext,
        node: BlockNodeId,
        loop_header_id: BlockNodeId,
        cfg_ops: &CfgOps,
        result_gen: &mut BitSet,
        killed_on_path: &mut BitSet, // Changed to mutable reference
        visited: &mut BitSet,
        allocator: &Allocator,
    ) {
        let block_id = Self::get_basic_block_id(ctx, node);

        if visited.has_bit(block_id.index()) {
            return;
        }
        visited.set_bit(block_id.index());

        // Track bits we set in THIS block so we can undo them later
        let mut newly_killed = Vec::new();

        if let Some(block_ops_vec) = &cfg_ops[block_id] {
            for (symbol_id, ops) in block_ops_vec {
                let sym_idx = symbol_id.index();

                if result_gen.has_bit(sym_idx) || killed_on_path.has_bit(sym_idx) {
                    continue;
                }

                if let Some(first_op) = ops.first() {
                    match first_op.op {
                        Operation::Read => {
                            result_gen.set_bit(sym_idx);
                        }
                        Operation::Write => {
                            killed_on_path.set_bit(sym_idx);
                            newly_killed.push(sym_idx); // Remember to backtrack
                        }
                    }
                }
            }
        }

        for edge in ctx.cfg().graph().edges_directed(node, Direction::Outgoing) {
            match edge.weight() {
                EdgeType::Normal | EdgeType::Jump | EdgeType::NewFunction | EdgeType::Backedge => {
                    let target = edge.target();
                    if target == loop_header_id {
                        continue;
                    }

                    // Pass the same mutable bitset down
                    Self::analyze_loop_recursive(
                        ctx,
                        target,
                        loop_header_id,
                        cfg_ops,
                        result_gen,
                        killed_on_path,
                        visited,
                        allocator,
                    );
                }
                _ => {}
            }
        }

        // BACKTRACK: Remove only the bits that this specific block call added
        for sym_idx in newly_killed {
            killed_on_path.remove(sym_idx);
        }
    }

    fn get_basic_block_id(ctx: &LintContext, block_node_id: BlockNodeId) -> BasicBlockId {
        *ctx.cfg().graph().node_weight(block_node_id).expect("expected a valid node id in graph")
    }

    fn get_ops_mut(
        cfg_ops: &mut CfgOps,
        block_id: BasicBlockId,
        symbol_id: SymbolId,
    ) -> &mut Vec<OpAtNode> {
        if cfg_ops[block_id].is_none() {
            // Start small (4) to avoid allocating too much for sparse blocks
            cfg_ops[block_id] = Some(Vec::with_capacity(4));
        }

        let block_ops_vec = cfg_ops[block_id].as_mut().unwrap();

        if let Some(pos) = block_ops_vec.iter().position(|(id, _)| *id == symbol_id) {
            &mut block_ops_vec[pos].1
        } else {
            block_ops_vec.push((symbol_id, Vec::new()));
            &mut block_ops_vec.last_mut().unwrap().1
        }
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
        // Not respect exported directive.
        // "/* exported foo */
        //                 let foo = 'used';
        //                 console.log(foo);
        //                 foo = 'unused like but exported with directive';", // { "sourceType": "script" },
        // Not respect eslint directive.
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
        }", // {				"globals": { "console": false },			},
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
        // Not respect eslint directive.
        // r#"/*eslint test/jsx:1*/
        //                     function App() {
        //                         const A = "";
        //                         return <A/>;
        //                     }
        //                 "#, // {				"parserOptions": {					"ecmaFeatures": {						"jsx": true,					},				},			},
        // Not respect eslint directive.
        // r#"/*eslint test/jsx:1*/
        //                     function App() {
        //                         let A = "";
        //                         foo(A);
        //                         A = "A";
        //                         return <A/>;
        //                     }
        //                 "#, // {				"parserOptions": {					"ecmaFeatures": {						"jsx": true,					},				},			},
        // Not respect eslint directive.
        // r#"/*eslint test/jsx:1*/
        //                     function App() {
        //                         let A = "a";
        //                         A = "b";
        //                         A = "c";
        //                         foo(A);
        //                         return <A/>;
        //                     }
        //                 "#, // {				"parserOptions": {					"ecmaFeatures": {						"jsx": true,					},				},			}
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
    ];

    Tester::new(NoUselessAssignment::NAME, NoUselessAssignment::PLUGIN, pass, fail)
        .test_and_snapshot();
}
