use oxc_ast::{AstKind, ast::BindingPatternKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_unassigned_vars_diagnostic(span: Span, msg: String) -> OxcDiagnostic {
    OxcDiagnostic::warn(msg)
        .with_help("Variable declared without assignment. Either assign a value or remove the declaration.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnassignedVars;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow let or var variables that are read but never assigned
    ///
    /// ### Why is this bad?
    ///
    /// This rule flags let or var declarations that are never assigned a value but are still read or used in the code.
    /// Since these variables will always be undefined, their usage is likely a programming mistake.
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
    suspicious,
);

impl Rule for NoUnassignedVars {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::VariableDeclarator(declarator) = node.kind() else {
            return;
        };
        if declarator.init.is_some() || declarator.kind.is_const() {
            return;
        }
        let Some(AstKind::VariableDeclaration(parent)) = ctx.nodes().parent_kind(node.id()) else {
            return;
        };
        if parent.declare {
            return;
        }
        let BindingPatternKind::BindingIdentifier(ident) = &declarator.id.kind else {
            return;
        };
        let Some(symbol_id) = ident.symbol_id.take() else {
            return;
        };
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
            let msg = format!(
                "'{}' is always 'undefined' because it's never assigned.",
                ident.name.as_str()
            );
            ctx.diagnostic(no_unassigned_vars_diagnostic(ident.span, msg));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "let x;",
        "var x;",
        "const x = 2",
        "const x = undefined; console.log(x);",
        "let y = undefined; console.log(y);",
        "var y = undefined; console.log(y);",
        "let a = x, b = y; console.log(a, b);",
        "var a = x, b = y; console.log(a, b);",
        "const foo = (two) => { let one; if (one !== two) one = two; }",
        // typescript
        "let z: number | undefined = undefined; log(z);",
        "declare let c: string | undefined; log(c);",
    ];

    let fail = vec![
        r"
            let status;
            if (status === 'ready') {
                console.log('Ready!');
            }

            let user;
            greet(user);

            function test() {
                let error;
                return error || 'Unknown error';
            }

            let options;
            const { debug } = options || {};

            let flag;
            while (!flag) {
                // Do something...
            }

            let config;
            function init() {
                return config?.enabled;
            }
        ",
        "let x; let a = x, b; console.log(x, a, b);",
        "const foo = (two) => { let one; if (one === two) {} }",
        "function test() { let error; return error || 'Unknown error'; }",
        "let options; const { debug } = options || {};",
        "let flag; while (!flag) { }",
        "let config; function init() { return config?.enabled; }",
        // typescript
        "let z: number | undefined; console.log(z);",
        "const foo = (two: string): void => { let one: string | undefined; if (one === two) {} }",
    ];

    Tester::new(NoUnassignedVars::NAME, NoUnassignedVars::PLUGIN, pass, fail).test_and_snapshot();
}
