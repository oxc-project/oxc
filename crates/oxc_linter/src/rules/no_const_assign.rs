use indextree::NodeId;
use oxc_ast::{Atom, Span};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::Symbol;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-const-assign): Unexpected re-assignment of const variable {0}")]
#[diagnostic(severity(warning))]
struct NoConstAssignDiagnostic(
    Atom,
    #[label("{0} is declared here as const")] pub Span,
    #[label("{0} is re-assigned here")] pub Span,
);

#[derive(Debug, Default, Clone)]
pub struct NoConstAssign;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow reassigning const variables
    ///
    /// ### Why is this bad?
    /// We cannot modify variables that are declared using const keyword.
    /// It will raise a runtime error.
    ///
    /// ### Example
    /// ```javascript
    /// const a = 0;
    /// a = 1;
    /// ```
    NoConstAssign,
    correctness
);

impl Rule for NoConstAssign {
    fn run_on_symbol(&self, symbol: &Symbol, ctx: &LintContext<'_>) {
        if symbol.is_const() {
            symbol.for_each_reference(|reference_id| {
                let node_id = NodeId::from(reference_id);
                let node = &ctx.nodes()[node_id];
                if let Some(reference) = ctx.get_reference(node) && reference.is_write() {
                    ctx.diagnostic(NoConstAssignDiagnostic(
                        symbol.name().clone(),
                        symbol.span(),
                        reference.span,
                    ));
                }
            });
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("const x = 0; { let x; x = 1; }", None),
        ("const x = 0; function a(x) { x = 1; }", None),
        ("const x = 0; foo(x);", None),
        ("for (const x in [1,2,3]) { foo(x); }", None),
        ("for (const x of [1,2,3]) { foo(x); }", None),
        ("const x = {key: 0}; x.key = 1;", None),
        ("var x = 0; x = 1;", None),
        ("let x = 0; x = 1;", None),
        ("function x() {} x = 1;", None),
        ("function foo(x) { x = 1; }", None),
        ("class X {} X = 1;", None),
        ("try {} catch (x) { x = 1; }", None),
        ("const a = 1; { let a = 2; { a += 1; } }", None),
    ];

    let fail = vec![
        ("const x = 0; x = 1;", None),
        ("const {a: x} = {a: 0}; x = 1;", None),
        ("const x = 0; ({x} = {x: 1});", None),
        ("const x = 0; ({a: x = 1} = {});", None),
        ("const x = 0; x += 1;", None),
        ("const x = 0; ++x;", None),
        ("for (const i = 0; i < 10; ++i) { foo(i); }", None),
        ("const x = 0; x = 1; x = 2;", None),
        ("const x = 0; function foo() { x = x + 1; }", None),
        ("const x = 0; function foo(a) { x = a; }", None),
        ("const x = 0; while (true) { x = x + 1; }", None),
        ("const x = 0; function foo(a) { function bar(b) { x = b; } bar(123); }", None),
        // Error even if the declaration comes after the assignment, which aligns with eslint
        ("x = 123; const x = 1;", None),
        // Binding patterns
        ("const [a, b, ...[c, ...d]] = [1, 2, 3, 4, 5]; d = 123", None),
        ("const d = 123; [a, b, ...[c, ...d]] = [1, 2, 3, 4, 5]", None),
        ("const b = 0; ({a, ...b} = {a: 1, c: 2, d: 3})", None),
    ];

    Tester::new(NoConstAssign::NAME, pass, fail).test_and_snapshot();
}
