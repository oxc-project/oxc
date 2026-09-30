use oxc_ast::{AstKind, ast::BindingPattern};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_unassigned_vars_diagnostic(span: Span, ident_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "'{ident_name}' is always 'undefined' because it's never assigned.",
    ))
    .with_help(
        "Variable declared without assignment. Either assign a value or remove the declaration.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnassignedVars;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow let or var variables that are read but never assigned.
    ///
    /// #### Ignored Files
    /// This rule ignores `.svelte` and `.vue` files entirely. Oxlint only parses the
    /// `<script>` blocks of these files, so a binding that the template assigns looks
    /// like it is never assigned. In Svelte the template writes through
    /// `bind:this={el}` and `bind:value={x}`; in Vue a `<script setup>` `let` is a
    /// `setup-let` binding that `v-model="x"` and inline handlers such as
    /// `@click="x = 1"` assign to directly.
    ///
    /// ### Why is this bad?
    ///
    /// This rule flags let or var declarations that are never assigned a value but are still read or used in the code.
    /// Since these variables will always be `undefined`, their usage is likely a programming mistake.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// let status;
    /// if (status === 'ready') {
    ///     console.log('Ready!');
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// let message = "hello";
    /// console.log(message);
    ///
    /// let user;
    /// user = getUser();
    /// console.log(user.name);
    /// ```
    NoUnassignedVars,
    eslint,
    correctness,
    version = "1.10.0",
    short_description = "Disallow let or var variables that are read but never assigned.",
);

impl Rule for NoUnassignedVars {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::VariableDeclarator(declarator) = node.kind() else {
            return;
        };
        if declarator.init.is_some() {
            return;
        }
        let parent_node = ctx.nodes().parent_node(node.id());
        let AstKind::VariableDeclaration(parent) = parent_node.kind() else {
            return;
        };
        if parent.kind.is_const() || parent.declare {
            return;
        }
        let grand_parent = ctx.nodes().parent_node(parent_node.id());
        if matches!(
            grand_parent.kind(),
            AstKind::ForStatement(_) | AstKind::ForInStatement(_) | AstKind::ForOfStatement(_)
        ) {
            return;
        }
        if ctx.nodes().ancestors(node.id()).skip(1).any(|ancestor| {
            matches!(
                ancestor.kind(),
                AstKind::TSExternalModuleDeclaration(_)
                    | AstKind::TSNamespaceDeclaration(_)
                    | AstKind::TSGlobalDeclaration(_)
            )
        }) {
            return;
        }
        let BindingPattern::BindingIdentifier(ident) = &declarator.id else {
            return;
        };
        let symbol_id = ident.symbol_id();
        let mut has_read = false;
        for reference in ctx.symbol_references(symbol_id) {
            if reference.is_write() {
                return;
            }
            if reference.is_read() {
                has_read = true;
            }
        }
        if has_read {
            ctx.diagnostic(no_unassigned_vars_diagnostic(ident.span, ident.name.as_str()));
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        // ignore svelte/vue: their templates can assign a binding, which we can't see.
        !ctx.file_extension().is_some_and(|ext| ext == "svelte" || ext == "vue")
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "let x;",
        "var x;",
        "const x = undefined; log(x);",
        "let y = undefined; log(y);",
        "var y = undefined; log(y);",
        "let a = x, b = y; log(a, b);",
        "var a = x, b = y; log(a, b);",
        "const foo = (two) => { let one; if (one !== two) one = two; }",
        "let z: number | undefined = undefined; log(z);",
        "declare let c: string | undefined; log(c);",
        "
                        const foo = (two: string): void => {
                            let one: string | undefined;
                            if (one !== two) {
                                one = two;
                            }
                        }
                    ",
        "
                        declare module 'module' {
                            import type { T } from 'module';
                            let x: T;
                            export = x;
                        }
                    ",
        "for (let p of pathToRemove) { p.remove() }",
    ];

    let fail = vec![
        "let x; let a = x, b; log(x, a, b);",
        "const foo = (two) => { let one; if (one === two) {} }",
        "let user; greet(user);",
        "function test() { let error; return error || 'Unknown error'; }",
        "let options; const { debug } = options || {};",
        "let flag; while (!flag) { }",
        "let config; function init() { return config?.enabled; }",
        "let x: number; log(x);",
        "let x: number | undefined; log(x);",
        "const foo = (two: string): void => { let one: string | undefined; if (one === two) {} }",
        "
                            declare module 'module' {
                                let x: string;
                            }
                            let y: string;
                            console.log(y);
                        ",
    ];

    Tester::new(NoUnassignedVars::NAME, NoUnassignedVars::PLUGIN, pass, fail).test_and_snapshot();
}

#[test]
fn test_svelte() {
    use crate::tester::Tester;

    // The markup is stripped before linting; it documents the template-side write
    // (`bind:this`, `bind:value`) that makes these bindings assigned in reality.
    let pass = vec![
        "<script lang=\"ts\">
            let dialog: HTMLDialogElement;
            function openDialog() { dialog.showModal(); }
         </script>
         <dialog bind:this={dialog}></dialog>",
        "<script>
            let value;
            log(value);
         </script>
         <input bind:value />",
        // Both blocks of a two-script component are linted; neither may report.
        "<script module>
            let shared;
            log(shared);
         </script>
         <script>
            let value;
            log(value);
         </script>
         <input bind:value />",
    ];

    Tester::new(NoUnassignedVars::NAME, NoUnassignedVars::PLUGIN, pass, vec![])
        .change_rule_path("test.svelte")
        .test();
}

#[test]
fn test_vue() {
    use crate::tester::Tester;

    // A `<script setup>` `let` is a `setup-let` binding: `v-model` and inline handlers
    // compile to direct assignments to it, so the script alone cannot see the write.
    let pass = vec![
        "<script setup>
            let msg;
            log(msg);
         </script>
         <template><input v-model=\"msg\" /></template>",
        "<script setup>
            let count;
            log(count);
         </script>
         <template><button @click=\"count = 1\">go</button></template>",
    ];

    Tester::new(NoUnassignedVars::NAME, NoUnassignedVars::PLUGIN, pass, vec![])
        .change_rule_path("test.vue")
        .test();
}
