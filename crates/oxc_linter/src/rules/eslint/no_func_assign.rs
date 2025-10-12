use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_func_assign_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is a function."))
        .with_label(span.label(format!("{name} is re-assigned here")))
}

#[derive(Debug, Default, Clone)]
pub struct NoFuncAssign;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow reassigning `function` declarations
    ///
    /// ### Why is this bad?
    ///
    /// Overwriting/reassigning a function written as a FunctionDeclaration is often indicative of
    /// a mistake or issue.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function foo() {}
    /// foo = bar;
    /// ```
    ///
    /// ```javascript
    /// function foo() {
    ///   foo = bar;
    /// }
    /// ```
    ///
    /// ```javascript
    /// let a = function hello() {
    ///   hello = 123;
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// let foo = function () {}
    /// foo = bar;
    /// ```
    ///
    /// ```javascript
    /// function baz(baz) { // `baz` is shadowed.
    ///   baz = bar;
    /// }
    /// ```
    ///
    /// ```
    /// function qux() {
    ///   const qux = bar;  // `qux` is shadowed.
    /// }
    /// ```
    NoFuncAssign,
    eslint,
    correctness
);

impl Rule for NoFuncAssign {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Function(func) = node.kind() else { return };

        let (func_name, symbol_id) = match &func.id {
            Some(id) => (id.name.as_str(), id.symbol_id()),
            None => return,
        };
        let symbol_table = ctx.scoping();
        for reference in symbol_table.get_resolved_references(symbol_id) {
            if reference.is_write() {
                ctx.diagnostic(no_func_assign_diagnostic(
                    func_name,
                    ctx.semantic().reference_span(reference),
                ));
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
        ("let a = function hello() { hello = 123;};", None),
    ];

    Tester::new(NoFuncAssign::NAME, NoFuncAssign::PLUGIN, pass, fail).test_and_snapshot();
}
