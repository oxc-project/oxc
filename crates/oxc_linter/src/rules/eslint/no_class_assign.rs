use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{CompactStr, Span};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-class-assign): Unexpected re-assignment of class {0}")]
#[diagnostic(severity(warning))]
struct NoClassAssignDiagnostic(
    CompactStr,
    #[label("{0} is declared as class here")] pub Span,
    #[label("{0} is re-assigned here")] pub Span,
);

#[derive(Debug, Default, Clone)]
pub struct NoClassAssign;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow reassigning class variables.
    ///
    /// ### Why is this bad?
    /// `ClassDeclaration` creates a variable that can be re-assigned,
    /// but the re-assignment is a mistake in most cases.
    ///
    /// ### Example
    /// ```javascript
    /// class A {}
    /// A = 123;
    /// let a = new A() // Error
    /// ```
    NoClassAssign,
    correctness
);

impl Rule for NoClassAssign {
    fn run_on_symbol(&self, symbol_id: SymbolId, ctx: &LintContext<'_>) {
        let symbol_table = ctx.semantic().symbols();
        if symbol_table.get_flag(symbol_id).is_class() {
            for reference in symbol_table.get_resolved_references(symbol_id) {
                if reference.is_write() {
                    ctx.diagnostic(NoClassAssignDiagnostic(
                        symbol_table.get_name(symbol_id).into(),
                        symbol_table.get_span(symbol_id),
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
        ("class A { } foo(A);", None),
        ("let A = class A { }; foo(A);", None),
        ("class A { b(A) { A = 0; } }", None),
        ("class A { b() { let A; A = 0; } }", None),
        ("let A = class { b() { A = 0; } }", None),
        ("let A = class B { foo() { A = 0; } }", None),
        ("let A = class A {}; A = 1", None),
        ("var x = 0; x = 1;", None),
        ("let x = 0; x = 1;", None),
        ("const x = 0; x = 1;", None),
        ("function x() {} x = 1;", None),
        ("function foo(x) { x = 1; }", None),
        ("try {} catch (x) { x = 1; }", None),
        ("if (foo) { class A {} } else { class A {} } A = 1;", None),
        // Sequence expression
        ("(class A {}, A = 1)", None),
    ];

    let fail = vec![
        ("class A { } A = 0;", None),
        ("class A { } ({A} = 0);", None),
        ("class A { } ({b: A = 0} = {});", None),
        ("A = 0; class A { }", None),
        ("class A { b() { A = 0; } }", None),
        ("let A = class A { b() { A = 0; } }", None),
        ("class A { } A = 0; A = 1;", None),
        ("if (foo) { class A {} A = 1; }", None),
    ];

    Tester::new(NoClassAssign::NAME, pass, fail).test_and_snapshot();
}
