use oxc_ast::{AstKind, ast::VariableDeclarationKind};
use oxc_cfg::{
    EdgeType, ErrorEdgeKind, Instruction, InstructionKind,
    graph::{
        Direction,
        visit::{EdgeRef},
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;
use std::collections::VecDeque;

use crate::{context::LintContext, rule::Rule};

fn no_unreachable_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unreachable code.").with_label(span)
}

/// <https://github.com/eslint/eslint/blob/069aa680c78b8516b9a1b568519f1d01e74fb2a2/lib/rules/no-unreachable.js#L196>
#[derive(Debug, Default, Clone)]
pub struct NoUnreachable;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unreachable code after `return`, `throw`, `continue`, and `break` statements
    ///
    /// ### Why is this bad?
    ///
    /// Unreachable code after a `return`, `throw`, `continue`, or `break` statement can never be run.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// function foo() {
    ///     return 2;
    ///     console.log("this will never be executed");
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// function foo() {
    ///     console.log("this will be executed");
    ///     return 2;
    /// }
    /// ```
    NoUnreachable,
    eslint,
    nursery
);

impl Rule for NoUnreachable {
    fn run_once(&self, ctx: &LintContext) {
        let nodes = ctx.nodes();
        let root = nodes.get_node(NodeId::ROOT);
        let cfg = ctx.cfg();
        let graph = cfg.graph();

        // A pre-allocated vector containing the reachability status of all the basic blocks.
        // We initialize this vector with all nodes set to `unreachable` since if we don't visit a
        // node in our paths then it should be unreachable by definition.
        let mut unreachables = vec![true; cfg.basic_blocks.len()];

        // All of the end points of infinite loops we encountered.
        let mut infinite_loops = Vec::new();

        // Set the root as reachable.
        unreachables[root.cfg_id().index()] = false;

        // In our first path we first check if each block is definitely unreachable, If it is then
        // we set it as such, If we encounter an infinite loop we keep its end block since it can
        // prevent other reachable blocks from ever getting executed.
        // Use iterative traversal to avoid stack overflow on deeply nested graphs
        {
            let mut stack = Vec::new();
            let mut visited = FxHashSet::default();
            let mut finished = FxHashSet::default();
            
            stack.push((root.cfg_id(), false)); // (node, is_finishing)
            
            while let Some((node, is_finishing)) = stack.pop() {
                if is_finishing {
                    if finished.insert(node) {
                        // This is equivalent to the DfsEvent::Finish event
                        let unreachable = cfg.basic_block(node).is_unreachable();
                        unreachables[node.index()] = unreachable;

                        if !unreachable {
                            if let Some(it) = cfg.is_infinite_loop_start(node, |instruction| {
                                use oxc_cfg::EvalConstConditionResult::{Eval, Fail, NotFound};
                                match instruction {
                                    Instruction { kind: InstructionKind::Condition, node_id: Some(id) } => {
                                        match nodes.kind(*id) {
                                            AstKind::BooleanLiteral(lit) => Eval(lit.value),
                                            _ => Fail,
                                        }
                                    }
                                    _ => NotFound,
                                }
                            }) {
                                infinite_loops.push(it);
                            }
                        }
                    }
                } else if visited.insert(node) {
                    // Push the finish event for this node (will be processed after children)
                    stack.push((node, true));
                    
                    // Push all neighbors for exploration (in reverse order to match DFS order)
                    let mut neighbors: Vec<_> = graph.edges_directed(node, Direction::Outgoing)
                        .map(|edge| edge.target())
                        .collect();
                    neighbors.reverse(); // Reverse to maintain DFS order when popping from stack
                    
                    for target in neighbors {
                        if !visited.contains(&target) {
                            stack.push((target, false));
                        }
                    }
                }
            }
        }

        // In the second path we go for each infinite loop end block and follow it marking all
        // edges as unreachable unless they have a reachable jump (eg. break).
        for loop_ in infinite_loops {
            // A loop end block usually is also its condition and start point but what is common
            // in all cases is that it may have `Jump` or `Backedge` edges so we only want to
            // follow the `Normal` edges as these are the exiting edges.
            let starts: Vec<_> = graph
                .edges_directed(loop_.1, Direction::Outgoing)
                .filter(|it| matches!(it.weight(), EdgeType::Normal))
                .map(|it| it.target())
                .collect();

            // Search with all `Normal` edges as starting point(s).
            // Use iterative breadth-first traversal to avoid stack overflow
            {
                let mut queue = VecDeque::from(starts);
                let mut visited = FxHashSet::default();
                
                while let Some(node) = queue.pop_front() {
                    if visited.insert(node) {
                        // This is equivalent to the DfsEvent::Discover event
                        let mut incoming = graph.edges_directed(node, Direction::Incoming);
                        let should_prune = incoming.any(|e| match e.weight() {
                            // `NewFunction` is always reachable
                            | EdgeType::NewFunction
                            // `Finalize` can be reachable if we encounter an error in the loop.
                            | EdgeType::Finalize
                            // Explicit `Error` can also be reachable if we encounter an error in the loop.
                            | EdgeType::Error(ErrorEdgeKind::Explicit) => true,

                            // If we have an incoming `Jump` and it is from a `Break` instruction,
                            // We know with high confidence that we are visiting a reachable block.
                            // NOTE: May cause false negatives but I couldn't think of one.
                            EdgeType::Jump
                                if cfg
                                    .basic_block(e.source())
                                    .instructions()
                                    .iter()
                                    .any(|it| matches!(it.kind, InstructionKind::Break(_))) =>
                            {
                                true
                            }
                            _ => false,
                        });
                        
                        if should_prune {
                            // We prune this branch if it is reachable from this point forward.
                            continue;
                        } else {
                            // Otherwise we set it to unreachable and continue.
                            unreachables[node.index()] = true;
                            
                            // Add neighbors to the queue
                            for edge in graph.edges_directed(node, Direction::Outgoing) {
                                let target = edge.target();
                                if !visited.contains(&target) {
                                    queue.push_back(target);
                                }
                            }
                        }
                    }
                }
            }
        }
        for node in ctx.nodes() {
            // exit early if we are not visiting a statement.
            if !node.kind().is_statement() {
                continue;
            }

            // exit early if it is an empty statement.
            if matches!(node.kind(), AstKind::EmptyStatement(_)) {
                continue;
            }

            if matches!(
                node.kind(),
                AstKind::VariableDeclaration(decl)
                    if matches!(decl.kind, VariableDeclarationKind::Var) && !decl.has_init()
            ) {
                // Skip `var` declarations without any initialization,
                // These work because of the JavaScript hoisting rules.
                continue;
            }

            if unreachables[node.cfg_id().index()] {
                ctx.diagnostic(no_unreachable_diagnostic(node.kind().span()));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function foo() { function bar() { return 1; } return bar(); }",
        "function foo() { return bar(); function bar() { return 1; } }",
        "function foo() { return x; var x; }",
        "function foo() { var x = 1; var y = 2; }",
        "function foo() { var x = 1; var y = 2; return; }",
        "while (true) { switch (foo) { case 1: x = 1; x = 2;} }",
        "while (true) { break; var x; }",
        "while (true) { continue; var x, y; }",
        "while (true) { throw 'message'; var x; }",
        "while (true) { if (true) break; var x = 1; }",
        "while (true) continue;",
        "switch (foo) { case 1: break; var x; }",
        "switch (foo) { case 1: break; var x; default: throw true; };",
        "const arrow_direction = arrow => {  switch (arrow) { default: throw new Error();  };}",
        "var x = 1; y = 2; throw 'uh oh'; var y;",
        "function foo() { var x = 1; if (x) { return; } x = 2; }",
        "function foo() { var x = 1; if (x) { } else { return; } x = 2; }",
        "function foo() { var x = 1; switch (x) { case 0: break; default: return; } x = 2; }",
        "function foo() { var x = 1; while (x) { return; } x = 2; }",
        "function foo() { var x = 1; for (x in {}) { return; } x = 2; }",
        "function foo() { var x = 1; try { return; } finally { x = 2; } }",
        "function foo() { var x = 1; for (;;) { if (x) break; } x = 2; }",
        "A: { break A; } foo()",
        "function* foo() { try { yield 1; return; } catch (err) { return err; } }",
        "function foo() { try { bar(); return; } catch (err) { return err; } }",
        "function foo() { try { a.b.c = 1; return; } catch (err) { return err; } }",
        "class C { foo = reachable; }",
        "class C { foo = reachable; constructor() {} }",
        "class C extends B { foo = reachable; }",
        "class C extends B { foo = reachable; constructor() { super(); } }",
        "class C extends B { static foo = reachable; constructor() {} }",
        "function foo() { var x = 1; for (;x == 1;) { if (x) continue; } x = 2; }",
        "
        if (a) {
            a();
        } else {
          for (let i = 1; i <= 10; i++) {
            b();
          }

          for (let i = 1; i <= 10; i++) {
            c();
          }
        }
        ",
        "
        try {
            throw 'error';
        } catch (err) {
            b();
        }
        c();
        ",
        "
        export const getPagePreviewText = (page) => {
            if (!a) {
                return '';
            }
            while (a && b > c && d-- > 0) {
            }
        };
        ",
        "
        try {
            for (const a of b) {
                c();
            }

            while (true) {
                d();
            }
        } finally {
        }
        ",
        "
        switch (authType) {
          case 1:
            return a();
          case 2:
            return b();
          case 3:
            return c();
        }
        d();
        ",
        "
        try {
          a();
        } catch (e) {
          b();
        } finally {
          c();
        }
        d();
        ",
        "
        try {
            while (true) {
                a();
            }
        } finally {
            b();
        }
        ",
        "
        try {
            a();
        } finally {
            b();
        }
        c();
        ",
        "
        try {
            while (true) {
                a();
            }
        } catch {
            b();
        }
        ",
    ];

    let fail = vec![
        //[{ messageId: "unreachableCode", type: "VariableDeclaration" }]
        "function foo() { return x; var x = 1; }",
        //[{ messageId: "unreachableCode", type: "VariableDeclaration" }]
        "function foo() { return x; var x, y = 1; }",
        "while (true) { break; var x = 1; }",
        //[{ messageId: "unreachableCode", type: "VariableDeclaration" }]
        "while (true) { continue; var x = 1; }",
        //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "function foo() { return; x = 1; }",
        //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "function foo() { throw error; x = 1; }",
        //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "while (true) { break; x = 1; }",
        //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "while (true) { continue; x = 1; }",
        //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "function foo() { switch (foo) { case 1: return; x = 1; } }",
        //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "function foo() { switch (foo) { case 1: throw e; x = 1; } }",
        //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "while (true) { switch (foo) { case 1: break; x = 1; } }",
        //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "while (true) { switch (foo) { case 1: continue; x = 1; } }",
        //[{ messageId: "unreachableCode", type: "VariableDeclaration" }]
        "var x = 1; throw 'uh oh'; var y = 2;",
        // [{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "function foo() { var x = 1; if (x) { return; } else { throw e; } x = 2; }",
        //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "function foo() { var x = 1; if (x) return; else throw -1; x = 2; }",
        //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "function foo() { var x = 1; try { return; } finally {} x = 2; }",
        //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "function foo() { var x = 1; try { } finally { return; } x = 2; }",
        //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "function foo() { var x = 1; do { return; } while (x); x = 2; }",
        // [{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "function foo() { var x = 1; while (x) { if (x) break; else continue; x = 2; } }",
        //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "function foo() { var x = 1; for (;;) { if (x) continue; } x = 2; }",
        //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "function foo() { var x = 1; while (true) { } x = 2; }",
        //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        "function foo() { var x = 1; do { } while (true); x = 2; }",
    ];

    Tester::new(NoUnreachable::NAME, NoUnreachable::PLUGIN, pass, fail).test_and_snapshot();
}
