use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-func-assign): '{0}' is a function.")]
#[diagnostic(severity(warning))]
struct NoFuncAssignDiagnostic(Atom, #[label("{0} is re-assigned here")] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoFuncAssign;

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
    NoFuncAssign,
    correctness
);

impl Rule for NoFuncAssign {
    fn run_on_symbol(&self, symbol_id: SymbolId, ctx: &LintContext<'_>) {
        let symbol_table = ctx.semantic().symbols();
        let decl = symbol_table.get_declaration(symbol_id);
        if let AstKind::Function(_) = ctx.nodes().kind(decl) {
            for reference_id in symbol_table.get_resolved_references(symbol_id) {
                let reference = symbol_table.get_reference(*reference_id);
                if reference.is_write() {
                    ctx.diagnostic(NoFuncAssignDiagnostic(
                        symbol_table.get_name(symbol_id).clone(),
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
        // TODO
        // ("var a = function foo() { foo = 123; };", None),
    ];

    Tester::new(NoFuncAssign::NAME, pass, fail).test_and_snapshot();
}
