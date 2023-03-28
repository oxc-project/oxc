use oxc_ast::{AstKind, Atom, Span};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::Symbol;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-function-assign): '{0}' is a function.")]
#[diagnostic(severity(warning))]
struct NoFunctionAssignDiagnostic(
    Atom,
    #[label("function {0} is declared here")] pub Span,
    #[label("{0} is re-assigned here")] pub Span,
);

#[derive(Debug, Default, Clone)]
pub struct NoFunctionAssign;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow reassigning `function` declarations
    ///
    /// ### Why is this bad?
    /// Overwriting/reassigning a function written as a FunctionDeclaration is often indicative of a mistake or issue.
    ///
    /// ### Example
    /// ```javascript
    ///
    /// function foo() {}
    /// foo = bar;
    /// ```
    NoFunctionAssign,
    nursery
);

impl Rule for NoFunctionAssign {
    fn run_on_symbol(&self, symbol: &Symbol, ctx: &LintContext<'_>) {
        if let AstKind::Function(_) = ctx.kind(symbol.declaration().into()) {
            for reference_id in symbol.references() {
                let reference =
                    ctx.semantic().symbols().get_resolved_reference(*reference_id).unwrap();
                if reference.is_write() {
                    ctx.diagnostic(NoFunctionAssignDiagnostic(
                        symbol.name().clone(),
                        symbol.span(),
                        reference.span(),
                    ));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function foo() { var foo = bar; }", None),
        ("function foo(foo) { foo = bar; }", None),
        ("function foo() { var foo; foo = bar; }", None),
        ("var foo = () => {}; foo = bar;", None),
        ("var foo = function() {}; foo = bar;", None),
        ("var foo = function() { foo = bar; };", None),
        ("import bar from 'bar'; function foo() { var foo = bar; }", None),
    ];

    let fail = vec![
        ("function foo() {}; foo = bar;", None),
        ("function foo() { foo = bar; }", None),
        ("foo = bar; function foo() { };", None),
        ("[foo] = bar; function foo() { };", None),
        ("({x: foo = 0} = bar); function foo() { };", None),
        ("function foo() { [foo] = bar; }", None),
        ("(function() { ({x: foo = 0} = bar); function foo() { }; })();", None),
        ("var a = function foo() { foo = 123; };", None),
    ];

    Tester::new(NoFunctionAssign::NAME, pass, fail).test_and_snapshot();
}
