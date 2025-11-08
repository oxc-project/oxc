use itertools::Itertools;
use oxc_ast::{
    AstKind,
    ast::{BindingPatternKind, Expression, VariableDeclarationKind},
};
use oxc_cfg::{
    BlockNodeId, EdgeType, ErrorEdgeKind,
    graph::{
        Direction,
        visit::{Control, DfsEvent, EdgeRef},
    },
    visit::set_depth_first_search,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, Reference, ScopeId, SymbolId};
use oxc_span::Span;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{context::LintContext, rule::Rule};

use oxc_span::GetSpan;

fn no_useless_assignment_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("This assigned value is not used in subsequent statements.g")
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
    Declare,
    Read,
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpAtNode {
    pub op: Operation,
    pub node: NodeId, // can be in this block or any descendant along the path
    pub symbol: SymbolId,
}

/// One feasible path’s operations for a single symbol, starting at this block’s
/// outgoing frontier (may be empty at this block and begin in childs).
#[derive(Debug, Default, Clone)]
pub struct PathOps(pub Vec<OpAtNode>);

impl PathOps {
    pub fn extend<I: IntoIterator<Item = OpAtNode>>(&mut self, iter: I) {
        self.0.extend(iter);
    }

    pub fn push(&mut self, op: OpAtNode) {
        self.0.push(op);
    }

    pub fn first(&self) -> Option<&OpAtNode> {
        self.0.first()
    }

    pub fn last(&self) -> Option<&OpAtNode> {
        self.0.last()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, OpAtNode> {
        self.0.iter()
    }
}

/// All mutually exclusive path variants for a symbol at this block.
#[derive(Debug, Default, Clone)]
pub struct SymbolPaths(pub Vec<PathOps>);

impl SymbolPaths {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn paths_mut(&mut self) -> &mut Vec<PathOps> {
        &mut self.0
    }

    pub fn first_or_create(&mut self) -> &mut PathOps {
        if self.is_empty() {
            self.0.push(PathOps::default());
        }
        self.0.first_mut().unwrap()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, PathOps> {
        self.0.iter()
    }

    pub fn extend<I: IntoIterator<Item = PathOps>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

/// symbol -> path variants at a block
pub type BlockSymbolOps = FxHashMap<SymbolId, SymbolPaths>;

/// block -> (symbol -> path variants)
pub type CfgSymbolOps = FxHashMap<BlockNodeId, BlockSymbolOps>;

impl Rule for NoUselessAssignment {
    fn run_once<'a>(&self, ctx: &LintContext) {
        let mut cfg_symbol_ops: CfgSymbolOps = CfgSymbolOps::default();

        let mut used_symbols: FxHashSet<SymbolId> = FxHashSet::default();

        //walk through all symbols, collect their operations from declarations and references
        for symbol_id in ctx.scoping().symbol_ids() {
            let decl_node = ctx.symbol_declaration(symbol_id);
            let decl_node_id = decl_node.id();
            let decl_block_node_id = ctx.nodes().cfg_id(decl_node_id);

            if let AstKind::VariableDeclarator(var_decl) = decl_node.kind() {
                //skip const declarations
                if let AstKind::VariableDeclaration(var_declaration) =
                    ctx.nodes().parent_node(decl_node.id()).kind()
                {
                    if var_declaration.kind == VariableDeclarationKind::Const {
                        continue;
                    }
                }

                // Skip function and arrow function assignments
                if !matches!(
                    &var_decl.init,
                    Some(Expression::FunctionExpression(_))
                        | Some(Expression::ArrowFunctionExpression(_))
                ) {
                    let path_ops = cfg_symbol_ops
                        .entry(decl_block_node_id)
                        .or_default()
                        .entry(symbol_id)
                        .or_default()
                        .first_or_create();

                    path_ops.push(OpAtNode {
                        op: Operation::Declare,
                        node: decl_node.id(),
                        symbol: symbol_id,
                    });

                    // if there is an initializer, record a write operation at declaration
                    if !var_decl.init.is_none() {
                        path_ops.push(OpAtNode {
                            op: Operation::Write,
                            node: decl_node.id(),
                            symbol: symbol_id,
                        });
                    }

                    // cloned references to avoid double borrowing
                    let cloned_references =
                        ctx.symbol_references(symbol_id).cloned().collect::<Vec<_>>();

                    // reorder reference to handle assignment expression like a = a + 1
                    for reference in Self::reordered_references(ctx, cloned_references) {
                        if reference.is_read() {
                            used_symbols.insert(symbol_id);
                        }

                        // if delcaration is a destructuring pattern, and the reference is a write within the declaration span, skip it
                        if reference.is_write()
                            && matches!(
                                &var_decl.id.kind,
                                BindingPatternKind::ObjectPattern(_)
                                    | BindingPatternKind::ArrayPattern(_)
                            )
                            && decl_node.span().contains_inclusive(
                                ctx.nodes().get_node(reference.node_id()).span(),
                            )
                        {
                            continue;
                        }

                        let refrence_block_node_id = ctx.nodes().cfg_id(reference.node_id());

                        let path_ops = cfg_symbol_ops
                            .entry(refrence_block_node_id)
                            .or_default()
                            .entry(symbol_id)
                            .or_default()
                            .first_or_create();

                        match (reference.is_read(), reference.is_write()) {
                            // if both read and write, record both operations
                            (true, true) => path_ops.extend(vec![
                                OpAtNode {
                                    op: Operation::Read,
                                    node: reference.node_id(),
                                    symbol: symbol_id,
                                },
                                OpAtNode {
                                    op: Operation::Write,
                                    node: reference.node_id(),
                                    symbol: symbol_id,
                                },
                            ]),
                            (true, false) => path_ops.push(OpAtNode {
                                op: Operation::Read,
                                node: reference.node_id(),
                                symbol: symbol_id,
                            }),
                            (false, true) => path_ops.push(OpAtNode {
                                op: Operation::Write,
                                node: reference.node_id(),
                                symbol: symbol_id,
                            }),
                            (false, false) => continue,
                        };
                    }
                }
            }
        }

        set_depth_first_search(
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
                        Control::Continue
                    } else {
                        Control::Prune
                    }
                }
                // detecting useless assignments within a basic block
                DfsEvent::Discover(block_node_id, _) => {
                    let block_symbol_ops = cfg_symbol_ops.entry(block_node_id).or_default();

                    for (symbol_id, symbol_paths) in block_symbol_ops {
                        // Will only have one path when processing at this level
                        let path_ops = symbol_paths.first_or_create();

                        if let Some(mut prev_op_at_node) = path_ops.first() {
                            for op_at_node in path_ops.iter().skip(1) {
                                match (prev_op_at_node.op, op_at_node.op) {
                                    (Operation::Write, Operation::Write) => {
                                        if !Self::is_exported(ctx, *symbol_id)
                                            && !Self::is_in_try_block(ctx, block_node_id)
                                        {
                                            ctx.diagnostic(no_useless_assignment_diagnostic(
                                                ctx.nodes().get_node(prev_op_at_node.node).span(),
                                            ));
                                        }
                                    }
                                    _ => {}
                                }

                                prev_op_at_node = op_at_node;
                            }
                        }
                    }

                    Control::<()>::Continue
                }
                // backtrack and merge child block symbol operations
                DfsEvent::Finish(block_node_id, _) => {
                    let mut childs_block_symbol_ops: Vec<BlockSymbolOps> = vec![];

                    if let Some(loop_start) = Self::find_loop_start(ctx, block_node_id) {
                        childs_block_symbol_ops.push(Self::find_symbol_operations_in_loop(
                            ctx,
                            &cfg_symbol_ops,
                            loop_start,
                            block_node_id,
                        ));
                    }
                    // Remove the source symbol operations temporarily to avoid double borrowing
                    let mut parent_block_symbol_ops =
                        cfg_symbol_ops.remove(&block_node_id).unwrap_or_default();

                    childs_block_symbol_ops.extend(
                        ctx.cfg()
                            .graph()
                            .edges_directed(block_node_id, Direction::Outgoing)
                            .filter(|e: &oxc_cfg::graph::graph::EdgeReference<'_, EdgeType>| {
                                matches!(
                                    e.weight(),
                                    EdgeType::Normal
                                        | EdgeType::Jump
                                        | EdgeType::NewFunction
                                        | EdgeType::Finalize
                                        | EdgeType::Join
                                )
                            })
                            .map(|e| cfg_symbol_ops.get(&e.target()).cloned().unwrap_or_default()),
                    );

                    let useless_ops = Self::merge_child_block_symbol_ops(
                        &mut parent_block_symbol_ops,
                        &mut childs_block_symbol_ops,
                    );

                    for op in useless_ops {
                        if used_symbols.contains(&op.symbol)
                            && !Self::is_exported(ctx, op.symbol)
                            && !Self::is_in_try_block(ctx, block_node_id)
                            && Self::has_same_parent_variable_scope(
                                ctx,
                                ctx.scoping().symbol_scope_id(op.symbol),
                                ctx.nodes().get_node(op.node).scope_id(),
                            )
                        {
                            ctx.diagnostic(no_useless_assignment_diagnostic(
                                ctx.nodes().get_node(op.node).span(),
                            ));
                        }
                    }

                    if Self::is_in_try_block(ctx, block_node_id) {
                        Self::extend_catch_block_operations(
                            ctx,
                            &mut cfg_symbol_ops,
                            block_node_id,
                            &mut parent_block_symbol_ops,
                        );
                    }

                    // Re-insert the source symbol operations back
                    cfg_symbol_ops.insert(block_node_id, parent_block_symbol_ops);

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
    fn reordered_references(ctx: &LintContext, references: Vec<Reference>) -> Vec<Reference> {
        let mut reordered: Vec<Reference> = vec![];
        let mut pending_reference: Vec<Reference> = vec![];

        for reference in references {
            if reordered.is_empty() {
                reordered.push(reference);
                continue;
            }

            if let Some(last_reference) = reordered.last() {
                if let Some(assignment_node) = Self::get_assignment_node(ctx, &last_reference) {
                    if ctx
                        .nodes()
                        .get_node(assignment_node)
                        .span()
                        .contains_inclusive(ctx.nodes().get_node(reference.node_id()).span())
                    {
                        pending_reference.push(reference);
                        continue;
                    } else {
                        if let Some(assignment_reference) = reordered.pop() {
                            reordered.extend(Self::reordered_references(
                                ctx,
                                pending_reference.drain(..).collect(),
                            ));
                            reordered.push(assignment_reference);

                            reordered.push(reference);
                        }
                    }
                } else {
                    reordered.push(reference);
                }
            }
        }

        if !pending_reference.is_empty() {
            reordered.extend(Self::reordered_references(ctx, pending_reference));
        }

        reordered
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

    // Same as the main dfs backtrack to collect symbol operations, but limited to the loop body
    fn find_symbol_operations_in_loop(
        ctx: &LintContext,
        _cfg_symbol_ops: &CfgSymbolOps,
        loop_start: BlockNodeId,
        loop_end: BlockNodeId,
    ) -> BlockSymbolOps {
        let mut cfg_symbol_ops: CfgSymbolOps = _cfg_symbol_ops.clone();

        set_depth_first_search(ctx.cfg().graph(), Some(loop_start), |e| match e {
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
            DfsEvent::Discover(block_node_id, _) => {
                if block_node_id == loop_end {
                    return Control::Prune;
                }

                Control::Continue
            }
            DfsEvent::Finish(block_node_id, _) => {
                let mut parent_block_symbol_ops =
                    cfg_symbol_ops.remove(&block_node_id).unwrap_or_default();

                // Remove the source symbol operations temporarily to avoid double borrowing
                let mut childs_block_symbol_ops = ctx
                    .cfg()
                    .graph()
                    .edges_directed(block_node_id, Direction::Outgoing)
                    .filter(|e: &oxc_cfg::graph::graph::EdgeReference<'_, EdgeType>| {
                        matches!(
                            e.weight(),
                            EdgeType::Normal
                                | EdgeType::Jump
                                | EdgeType::NewFunction
                                | EdgeType::Finalize
                                | EdgeType::Join
                        )
                    })
                    .map(|e| cfg_symbol_ops.get(&e.target()).cloned().unwrap_or_default())
                    .collect();

                Self::merge_child_block_symbol_ops(
                    &mut parent_block_symbol_ops,
                    &mut childs_block_symbol_ops,
                );

                if Self::is_in_try_block(ctx, block_node_id) {
                    Self::extend_catch_block_operations(
                        ctx,
                        &mut cfg_symbol_ops,
                        block_node_id,
                        &mut parent_block_symbol_ops,
                    );
                }

                cfg_symbol_ops.insert(block_node_id, parent_block_symbol_ops);
                Control::Continue
            }
            _ => Control::Continue,
        });

        return cfg_symbol_ops.remove(&loop_start).unwrap_or_default();
    }

    fn merge_child_block_symbol_ops(
        parent_block_symbol_ops: &mut BlockSymbolOps,
        children_block_symbol_ops: &mut Vec<BlockSymbolOps>,
    ) -> Vec<OpAtNode> {
        let mut useless_op_at_nodes: Vec<OpAtNode> = vec![];

        for (symbol_id, parent_symbol_paths) in parent_block_symbol_ops.iter_mut() {
            // Will only have one path when processing at this level
            if let Some(last_parent_path_op) = parent_symbol_paths.first_or_create().last() {
                let mut is_useless: bool = true;

                'loop_children: for child_block_symbol_ops in children_block_symbol_ops.iter_mut() {
                    let child_symbol_paths = child_block_symbol_ops.entry(*symbol_id).or_default();

                    for child_symbol_path in child_symbol_paths.iter() {
                        if let Some(first_child_op_at_node) = child_symbol_path.first() {
                            match (last_parent_path_op.op, first_child_op_at_node.op) {
                                (_, Operation::Read) => {
                                    is_useless = false;
                                    break 'loop_children;
                                }
                                _ => {}
                            }
                        }
                    }
                }

                match last_parent_path_op.op {
                    Operation::Write => {
                        if is_useless {
                            useless_op_at_nodes.push(*last_parent_path_op);
                        }
                    }
                    _ => {}
                }
            }
        }

        // handle merge child symbol doesn't exist in parent symbol ops

        let existing_keys: Vec<SymbolId> = parent_block_symbol_ops.keys().copied().collect();

        for child_block_symbol_ops in children_block_symbol_ops.iter() {
            for (symbol_id, symbol_paths) in child_block_symbol_ops {
                // If the key existed in the source before merging targets, skip it.
                if existing_keys.contains(symbol_id) {
                    continue;
                }

                let parent_symbol_paths = parent_block_symbol_ops.entry(*symbol_id).or_default();

                for path_ops in symbol_paths.iter() {
                    parent_symbol_paths.paths_mut().push(path_ops.clone());
                }
            }
        }

        useless_op_at_nodes
    }

    fn extend_catch_block_operations(
        ctx: &LintContext,
        cfg_symbol_ops: &mut CfgSymbolOps,
        try_block_node_id: BlockNodeId,
        parent_block_symbol_ops: &mut BlockSymbolOps,
    ) {
        if let Some(catch_block) =
            ctx.cfg().graph().edges_directed(try_block_node_id, Direction::Outgoing).find_map(|e| {
                match e.weight() {
                    EdgeType::Error(ErrorEdgeKind::Explicit) => Some(e.target()),
                    _ => None,
                }
            })
        {
            if let Some(block_symbol_operations) = cfg_symbol_ops.get(&catch_block) {
                for (symbol_id, symbol_paths) in block_symbol_operations {
                    parent_block_symbol_ops
                        .entry(*symbol_id)
                        .or_default()
                        .extend(symbol_paths.iter().cloned());
                }
            }
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
