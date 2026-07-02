use oxc_ast::{AstKind, AstType, ast::VariableDeclarationKind};
use oxc_cfg::{Instruction, InstructionKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};

use crate::{
    context::ContextHost, context::LintContext, rule::Rule, utils::effective_unreachable_blocks,
};

fn no_unreachable_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unreachable code.")
        .with_help("Remove the unreachable code or fix the control flow to make it reachable.")
        .with_label(span)
}

/// <https://github.com/eslint/eslint/blob/069aa680c78b8516b9a1b568519f1d01e74fb2a2/lib/rules/no-unreachable.js#L196>
#[derive(Debug, Default, Clone)]
pub struct NoUnreachable;

const NEEDED_NODE_TYPES: &AstTypesBitset = &AstTypesBitset::from_types(&[
    AstType::ReturnStatement,
    AstType::ThrowStatement,
    AstType::BreakStatement,
    AstType::ContinueStatement,
    AstType::WhileStatement,
    AstType::DoWhileStatement,
    AstType::ForStatement,
]);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unreachable code after `return`, `throw`, `continue`, and `break` statements.
    ///
    /// This rule can be disabled for TypeScript code if `allowUnreachableCode: false` is configured
    /// in the `tsconfig.json`, as the TypeScript compiler enforces this check.
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
    correctness,
    version = "0.4.4",
    short_description = "Disallow unreachable code after `return`, `throw`, `continue`, and `break` statements.",
);

impl Rule for NoUnreachable {
    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.semantic().nodes().contains_any(NEEDED_NODE_TYPES)
    }

    fn run_once(&self, ctx: &LintContext) {
        let cfg = ctx.cfg();
        let graph = cfg.graph();
        let mut unreachable_statement_ids = Vec::new();
        let unreachables = effective_unreachable_blocks(ctx);

        for node in graph.node_indices() {
            if !unreachables[node.index()] {
                continue;
            }

            unreachable_statement_ids
                .extend(cfg.basic_block(node).instructions().iter().filter_map(statement_node_id));
        }

        report_unreachable_statements(ctx, unreachable_statement_ids);
    }
}

fn statement_node_id(instruction: &Instruction) -> Option<NodeId> {
    if matches!(
        instruction.kind,
        InstructionKind::Statement
            | InstructionKind::Return(_)
            | InstructionKind::Break(_)
            | InstructionKind::Continue(_)
            | InstructionKind::Throw
    ) {
        instruction.node_id
    } else {
        None
    }
}

fn report_unreachable_statements(ctx: &LintContext, mut unreachable_statement_ids: Vec<NodeId>) {
    let nodes = ctx.nodes();

    if unreachable_statement_ids.is_empty() {
        return;
    }

    unreachable_statement_ids.sort_unstable_by_key(|node_id| node_id.index());
    unreachable_statement_ids.dedup();

    let mut reported_statement_ids = Vec::new();

    for node_id in unreachable_statement_ids {
        let kind = nodes.kind(node_id);

        if !kind.is_statement() || should_skip_unreachable_statement(kind) {
            continue;
        }

        if has_reported_unreachable_ancestor(nodes, &reported_statement_ids, node_id) {
            continue;
        }

        reported_statement_ids.push(node_id);
        ctx.diagnostic(no_unreachable_diagnostic(kind.span()));
    }
}

fn has_reported_unreachable_ancestor(
    nodes: &oxc_semantic::AstNodes<'_>,
    reported_statement_ids: &[NodeId],
    node_id: NodeId,
) -> bool {
    for ancestor_id in nodes.ancestor_ids(node_id) {
        debug_assert!(
            ancestor_id < node_id,
            "ancestor nodes must be assigned lower NodeIds than descendants"
        );

        if matches!(nodes.kind(ancestor_id), AstKind::FunctionBody(_) | AstKind::StaticBlock(_)) {
            return false;
        }

        if reported_statement_ids
            .binary_search_by_key(&ancestor_id.index(), |reported_id| reported_id.index())
            .is_ok()
        {
            return true;
        }
    }

    false
}

fn should_skip_unreachable_statement(kind: AstKind<'_>) -> bool {
    matches!(kind, AstKind::EmptyStatement(_))
        || matches!(
            kind,
            AstKind::VariableDeclaration(decl)
                if matches!(decl.kind, VariableDeclarationKind::Var) && !decl.has_init()
        )
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
        "
        function foo() {
            if (Math.random() === 0.5) {
                while (true) { return 'greetings!'; }
            }
            return 'Hello, tsdown!';
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
        "
        function foo() {
            return;

            if (Math.random() > 0.5) {
                if (Math.random() > 0.5) {
                    console.log('test');
                }
            } else {
                console.log('test');

            }
        }
        ",
        "
        function foo() {
            return;

            if (Math.random() > 0.5) {
                function bar() {
                    return;
                    console.log('inner');
                }
            }
        }
        ",
        "function foo() { while (true) { return ''; } return ''; }",
    ];

    Tester::new(NoUnreachable::NAME, NoUnreachable::PLUGIN, pass, fail).test_and_snapshot();
}
