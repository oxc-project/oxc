use std::hash::BuildHasherDefault;

use itertools::Itertools;
use oxc_ast::{ast::VariableDeclarationKind, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{
    petgraph::visit::{depth_first_search, Control, DfsEvent},
    AstNode, BasicBlockId, EdgeType,
};
use oxc_span::{GetSpan, Span};
use rustc_hash::{FxHashMap, FxHasher};

use crate::{context::LintContext, rule::Rule};

fn no_unreachable_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("eslint(no-unreachable): Unreachable code.").with_labels([span.into()])
}

/// <https://github.com/eslint/eslint/blob/069aa680c78b8516b9a1b568519f1d01e74fb2a2/lib/rules/no-unreachable.js#L196>
#[derive(Debug, Default, Clone)]
pub struct NoUnreachable;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unreachable code after `return`, `throw`, `continue`, and `break` statements
    ///
    NoUnreachable,
    correctness
);

impl Rule for NoUnreachable {
    fn run_once(&self, ctx: &LintContext) {
        let nodes = ctx.nodes();
        let cfg = ctx.semantic().cfg();
        // let mut cache = FxHashMap::with_capacity_and_hasher(
        //     nodes.len(),
        //     BuildHasherDefault::<FxHasher>::default(),
        // );
        let mut v = vec![false; cfg.basic_blocks.len()];

        // for node in ctx.nodes().iter() {
        //     Self::is_reachabale(node, ctx, &mut cache);
        // }

        let Some(root) = nodes.root_node() else { unreachable!() };
        let graph = &ctx.semantic().cfg().graph;
        let _: Control<()> = depth_first_search(graph, Some(root.cfg_id()), |event| match event {
            DfsEvent::Discover(node, _) => {
                let _ = std::thread::yield_now();
                Control::Continue
            }
            //     if node == to {
            //         Control::Break(true)
            //     } else if let Some((loop_jump, loop_end)) = self.is_infinite_loop_start(node, nodes)
            //     {
            //         let (found, seen_break) =
            //             self.is_reachabale_with_infinite_loop(loop_jump, to, filter, loop_end);
            //         if found {
            //             Control::Break(true)
            //         } else if !seen_break {
            //             Control::Prune
            //         } else {
            //             Control::Continue
            //         }
            //     } else {
            //         Control::Continue
            //     }
            // }
            DfsEvent::TreeEdge(a, b) => {
                let _ = std::thread::yield_now();
                // let unreachable = !graph.edges_connecting(a, b).any(|edge| {
                //     !matches!(
                //         edge.weight(),
                //         EdgeType::NewFunction | EdgeType::Unreachable | EdgeType::Join
                //     )
                // });
                //
                // if unreachable {
                //     Control::Prune
                // } else {
                //     v[b.index()] = true;
                //     Control::Continue
                // }
                Control::Continue
            }
            _ => Control::Continue,
        });
        // for node in ctx.nodes().iter() {
        //     if !v[node.cfg_id().index()] {
        //         ctx.diagnostic(no_unreachable_diagnostic(node.kind().span()));
        //     }
        // }
    }
}

impl NoUnreachable {
    // pub(self) fn is_infinite_loop_start(
    //     &self,
    //     node: BasicBlockId,
    //     nodes: &AstNodes,
    // ) -> Option<(BasicBlockId, BasicBlockId)> {
    //     enum EvalConstConditionResult {
    //         NotFound,
    //         Fail,
    //         Eval(bool),
    //     }
    //     fn try_eval_const_condition(
    //         instruction: &Instruction,
    //         nodes: &AstNodes,
    //     ) -> EvalConstConditionResult {
    //         use EvalConstConditionResult::{Eval, Fail, NotFound};
    //         match instruction {
    //             Instruction { kind: InstructionKind::Condition, node_id: Some(id) } => {
    //                 match nodes.kind(*id) {
    //                     AstKind::BooleanLiteral(lit) => Eval(lit.value),
    //                     _ => Fail,
    //                 }
    //             }
    //             _ => NotFound,
    //         }
    //     }
    //
    //     fn get_jump_target(
    //         graph: &Graph<usize, EdgeType>,
    //         node: BasicBlockId,
    //     ) -> Option<BasicBlockId> {
    //         graph
    //             .edges_directed(node, Direction::Outgoing)
    //             .find_or_first(|e| matches!(e.weight(), EdgeType::Jump))
    //             .map(|it| it.target())
    //     }
    //
    //     let basic_block = self.basic_block(node);
    //     let mut backedges = self
    //         .graph
    //         .edges_directed(node, Direction::Incoming)
    //         .filter(|e| matches!(e.weight(), EdgeType::Backedge));
    //
    //     // if this node doesn't have an backedge it isn't a loop starting point.
    //     let backedge = backedges.next()?;
    //
    //     // TODO: it isn't true at the moment but I believe it should be.
    //     debug_assert!(
    //         backedges.next().is_none(),
    //         "there should only be one backedge to each basic block."
    //     );
    //
    //     // if instructions are empty we might be in a `for(;;)`.
    //     if basic_block.instructions().is_empty()
    //         && !self
    //             .graph
    //             .edges_directed(node, Direction::Outgoing)
    //             .any(|e| matches!(e.weight(), EdgeType::Backedge))
    //     {
    //         return get_jump_target(&self.graph, node).map(|it| (it, node));
    //     }
    //
    //     // if there are more than one instruction in this block it can't be a valid loop start.
    //     let Ok(only_instruction) = basic_block.instructions().iter().exactly_one() else {
    //         return None;
    //     };
    //
    //     // if there is exactly one and it is a condition instruction we are in a loop so we
    //     // check the condition to infer if it is always true.
    //     if let EvalConstConditionResult::Eval(true) =
    //         try_eval_const_condition(only_instruction, nodes)
    //     {
    //         get_jump_target(&self.graph, node).map(|it| (it, node))
    //     } else if let EvalConstConditionResult::Eval(true) =
    //         self.basic_block(backedge.source()).instructions().iter().exactly_one().map_or_else(
    //             |_| EvalConstConditionResult::NotFound,
    //             |it| try_eval_const_condition(it, nodes),
    //         )
    //     {
    //         get_jump_target(&self.graph, node).map(|it| (node, it))
    //     } else {
    //         None
    //     }
    // }
    // fn is_reachabale(node: &AstNode, ctx: &LintContext, cache: &mut FxHashMap<BasicBlockId, bool>) {
    //     // exit early if we are not visiting a statement.
    //     if !node.kind().is_statement() {
    //         return;
    //     }
    //
    //     // exit early if it is an empty statement.
    //     if matches!(node.kind(), AstKind::EmptyStatement(_)) {
    //         return;
    //     }
    //
    //     if matches! {
    //         node.kind(),
    //         AstKind::VariableDeclaration(decl)
    //             if matches!(decl.kind, VariableDeclarationKind::Var) && !decl.has_init()
    //     } {
    //         // Skip `var` declarations without any initialization,
    //         // These work because of the JavaScript hoisting rules.
    //         return;
    //     }
    //
    //     if matches!(cache.get(&node.cfg_id()), Some(false)) {
    //         return ctx.diagnostic(no_unreachable_diagnostic(node.kind().span()));
    //     }
    //
    //     let nodes = ctx.nodes();
    //     let Some(parent) = nodes
    //         .ancestors(node.id())
    //         .map(|id| nodes.get_node(id))
    //         .find_or_last(|it| it.kind().is_function_like())
    //     else {
    //         unreachable!()
    //     };
    //
    //     let is_reachable =
    //         if ctx.semantic().cfg().is_reachabale_deepscan(parent.cfg_id(), node.cfg_id(), nodes) {
    //             true
    //         } else {
    //             ctx.diagnostic(no_unreachable_diagnostic(node.kind().span()));
    //             false
    //         };
    //     cache.insert(node.cfg_id(), is_reachable);
    // }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function foo() { function bar() { return 1; } return bar(); }",
        // "function foo() { return bar(); function bar() { return 1; } }",
        // "function foo() { return x; var x; }",
        // "function foo() { var x = 1; var y = 2; }",
        // "function foo() { var x = 1; var y = 2; return; }",
        // "while (true) { switch (foo) { case 1: x = 1; x = 2;} }",
        // "while (true) { break; var x; }",
        // "while (true) { continue; var x, y; }",
        // "while (true) { throw 'message'; var x; }",
        // "while (true) { if (true) break; var x = 1; }",
        // "while (true) continue;",
        // "switch (foo) { case 1: break; var x; }",
        // "switch (foo) { case 1: break; var x; default: throw true; };",
        // "const arrow_direction = arrow => {  switch (arrow) { default: throw new Error();  };}",
        // "var x = 1; y = 2; throw 'uh oh'; var y;",
        // "function foo() { var x = 1; if (x) { return; } x = 2; }",
        // "function foo() { var x = 1; if (x) { } else { return; } x = 2; }",
        // "function foo() { var x = 1; switch (x) { case 0: break; default: return; } x = 2; }",
        // "function foo() { var x = 1; while (x) { return; } x = 2; }",
        // "function foo() { var x = 1; for (x in {}) { return; } x = 2; }",
        // "function foo() { var x = 1; try { return; } finally { x = 2; } }",
        // "function foo() { var x = 1; for (;;) { if (x) break; } x = 2; }",
        // "A: { break A; } foo()",
        // "function* foo() { try { yield 1; return; } catch (err) { return err; } }",
        // "function foo() { try { bar(); return; } catch (err) { return err; } }",
        // "function foo() { try { a.b.c = 1; return; } catch (err) { return err; } }",
        // "class C { foo = reachable; }",
        // "class C { foo = reachable; constructor() {} }",
        // "class C extends B { foo = reachable; }",
        // "class C extends B { foo = reachable; constructor() { super(); } }",
        // "class C extends B { static foo = reachable; constructor() {} }",
        // "function foo() { var x = 1; for (;x == 1;) { if (x) continue; } x = 2; }",
        // "
        // if (a) {
        //     a();
        // } else {
        //   for (let i = 1; i <= 10; i++) {
        //     b();
        //   }
        //
        //   for (let i = 1; i <= 10; i++) {
        //     c();
        //   }
        // }
        // ",
        // "
        // try {
        //     throw 'error';
        // } catch (err) {
        //     b();
        // }
        // c();
        // ",
        // "
        // export const getPagePreviewText = (page) => {
        //     if (!a) {
        //         return '';
        //     }
        //     while (a && b > c && d-- > 0) {
        //     }
        // };
        // ",
        // "
        // try {
        //     for (const a of b) {
        //         c();
        //     }
        //
        //     while (true) {
        //         d();
        //     }
        // } finally {
        // }
        // ",
        // "
        // switch (authType) {
        //   case 1:
        //     return a();
        //   case 2:
        //     return b();
        //   case 3:
        //     return c();
        // }
        // d();
        // ",
        // "
        // try {
        //   a();
        // } catch (e) {
        //   b();
        // } finally {
        //   c();
        // }
        // d();
        //     ",
    ];

    let fail = vec![
        // //[{ messageId: "unreachableCode", type: "VariableDeclaration" }]
        // "function foo() { return x; var x = 1; }",
        // //[{ messageId: "unreachableCode", type: "VariableDeclaration" }]
        // "function foo() { return x; var x, y = 1; }",
        // "while (true) { break; var x = 1; }",
        // //[{ messageId: "unreachableCode", type: "VariableDeclaration" }]
        // "while (true) { continue; var x = 1; }",
        // //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        // "function foo() { return; x = 1; }",
        // //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        // "function foo() { throw error; x = 1; }",
        // //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        // "while (true) { break; x = 1; }",
        // //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        // "while (true) { continue; x = 1; }",
        // //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        // "function foo() { switch (foo) { case 1: return; x = 1; } }",
        // //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        // "function foo() { switch (foo) { case 1: throw e; x = 1; } }",
        // //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        // "while (true) { switch (foo) { case 1: break; x = 1; } }",
        // //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        // "while (true) { switch (foo) { case 1: continue; x = 1; } }",
        // //[{ messageId: "unreachableCode", type: "VariableDeclaration" }]
        // "var x = 1; throw 'uh oh'; var y = 2;",
        // // [{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        // "function foo() { var x = 1; if (x) { return; } else { throw e; } x = 2; }",
        // //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        // "function foo() { var x = 1; if (x) return; else throw -1; x = 2; }",
        // //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        // "function foo() { var x = 1; try { return; } finally {} x = 2; }",
        // //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        // "function foo() { var x = 1; try { } finally { return; } x = 2; }",
        // //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        // "function foo() { var x = 1; do { return; } while (x); x = 2; }",
        // // [{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        // "function foo() { var x = 1; while (x) { if (x) break; else continue; x = 2; } }",
        // //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        // "function foo() { var x = 1; for (;;) { if (x) continue; } x = 2; }",
        // //[{ messageId: "unreachableCode", type: "ExpressionStatement" }]
        // "function foo() { var x = 1; while (true) { } x = 2; }",
        // "function foo() { var x = 1; do { } while (true); x = 2; }",
    ];

    Tester::new(NoUnreachable::NAME, pass, fail).test_and_snapshot();
}
