use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

fn no_process_exit_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Don't use `process.exit()`")
        .with_help("Throw an error instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoProcessExit;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow `process.exit()`.
    ///
    /// ### Why is this bad?
    /// Only use `process.exit()` in CLI apps. Throw an error instead.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (problem) process.exit(1);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// if (problem) throw new Error("message");
    /// ```
    ///
    /// ```
    /// #!/usr/bin/env node
    /// if (problem) process.exit(1);
    /// ```
    NoProcessExit,
    unicorn,
    restriction,
    pending // TODO: suggestion
);

impl Rule for NoProcessExit {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(expr) = node.kind() {
            if is_method_call(expr, Some(&["process"]), Some(&["exit"]), None, None) {
                if has_hashbang(ctx)
                    || is_inside_process_event_handler(ctx, node)
                    || is_worker_threads_imported(ctx)
                {
                    return;
                }

                ctx.diagnostic(no_process_exit_diagnostic(expr.span));
            }
        }
    }
}

fn has_hashbang(ctx: &LintContext) -> bool {
    let Some(root) = ctx.nodes().root_node() else {
        return false;
    };
    let AstKind::Program(program) = root.kind() else { unreachable!() };
    program.hashbang.is_some()
}

fn is_inside_process_event_handler(ctx: &LintContext, node: &AstNode) -> bool {
    for parent in ctx.nodes().ancestors(node.id()) {
        if let AstKind::CallExpression(expr) = parent.kind() {
            if is_method_call(expr, Some(&["process"]), Some(&["on", "once"]), Some(1), None) {
                return true;
            }
        }
    }

    false
}

fn is_worker_threads_imported(ctx: &LintContext) -> bool {
    ctx.module_record().import_entries.iter().any(|entry| {
        matches!(entry.module_request.name(), "worker_threads" | "node:worker_threads")
    })
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("#!/usr/bin/env node\n\nprocess.exit();"),
        ("Process.exit()"),
        ("const x = process.exit;"),
        ("x(process.exit)"),
        (r#"process.on("SIGINT", function() { process.exit(1); })"#),
        (r#"process.on("SIGKILL", function() { process.exit(1); })"#),
        (r#"process.on("SIGINT", () => { process.exit(1); })"#),
        (r#"process.on("SIGINT", () => process.exit(1))"#),
        (r#"process.on("SIGINT", () => { if (true) { process.exit(1); } })"#),
        (r#"process.once("SIGINT", function() { process.exit(1); })"#),
        (r#"process.once("SIGKILL", function() { process.exit(1); })"#),
        (r#"process.once("SIGINT", () => { process.exit(1); })"#),
        (r#"process.once("SIGINT", () => process.exit(1))"#),
        (r#"process.once("SIGINT", () => { if (true) { process.exit(1); } })"#),
        // (r"
        // const {workerData, parentPort} = require('worker_threads');
        // process.exit(1);
        // "),
        // (r"
        // const {workerData, parentPort} = require('node:worker_threads');
        // process.exit(1);
        // "),
        // (r"
        // import {workerData, parentPort} from 'worker_threads';
        // process.exit(1);
        // "),
        // (r"
        // import foo from 'worker_threads';
        // process.exit(1);
        // "),
        // (r"
        // import foo from 'node:worker_threads';
        // process.exit(1);
        // "),
        // Not `CallExpression`
        ("new process.exit(1);"),
        // Not `MemberExpression`
        ("exit(1);"),
        // `callee.property` is not a `Identifier`
        // (r#"process["exit"](1);"#),
        // Computed
        ("process[exit](1);"),
        // Not exit
        ("process.foo(1);"),
        // Not `process`
        ("foo.exit(1);"),
        // `callee.object.type` is not a `Identifier`
        ("lib.process.exit(1);"),
    ];

    let fail = vec![
        ("process.exit();"),
        ("process.exit(1);"),
        ("x(process.exit(1));"),
        (r#"process.on("SIGINT", function() {});process.exit();"#),
        (r#"process.once("SIGINT", function() {}); process.exit(0)"#),
        (r"
            const mod = require('not_worker_threads');
            process.exit(1);
        "),
        (r"
            import mod from 'not_worker_threads';
            process.exit(1);
        "),
        // Not `CallExpression`
        (r"
            const mod = new require('worker_threads');
            process.exit(1);
        "),
        // Not `Literal` worker_threads
        (r"
            const mod = require(worker_threads);
            process.exit(1);
        "),
        // Not `CallExpression`
        (r#"new process.on("SIGINT", function() { process.exit(1); })"#),
        (r#"new process.once("SIGINT", function() { process.exit(1); })"#),
        // Not `MemberExpression`
        (r#"on("SIGINT", function() { process.exit(1); })"#),
        (r#"once("SIGINT", function() { process.exit(1); })"#),
        // `callee.property` is not a `Identifier`
        // (r#"process["on"]("SIGINT", function() { process.exit(1); })"#),
        // (r#"process["once"]("SIGINT", function() { process.exit(1); })"#),
        // Computed
        (r#"process[on]("SIGINT", function() { process.exit(1); })"#),
        (r#"process[once]("SIGINT", function() { process.exit(1); })"#),
        // Not `on` / `once`
        (r#"process.foo("SIGINT", function() { process.exit(1); })"#),
        // Not `process`
        (r#"foo.on("SIGINT", function() { process.exit(1); })"#),
        (r#"foo.once("SIGINT", function() { process.exit(1); })"#),
        // `callee.object.type` is not a `Identifier`
        (r#"lib.process.on("SIGINT", function() { process.exit(1); })"#),
        (r#"lib.process.once("SIGINT", function() { process.exit(1); })"#),
    ];

    Tester::new(NoProcessExit::NAME, NoProcessExit::PLUGIN, pass, fail).test_and_snapshot();
}
