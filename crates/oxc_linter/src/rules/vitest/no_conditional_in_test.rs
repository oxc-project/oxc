use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{is_type_of_jest_fn_call, JestFnKind, JestGeneralFnKind, PossibleJestNode},
};

fn no_conditional_in_test(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid having conditionals in test")
        .with_help("Remove the surrounding if statement.")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct NoConditionalInTest;

declare_oxc_lint!(
    /// ### What it does
    /// This rule aims to prevent conditional tests.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// test('my test', () => {
    ///   if (true) {
    ///     doTheThing()
    ///   }
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// test('my test', () => {
    ///   expect(true).toBe(true)
    /// })
    /// ```
    NoConditionalInTest,
    pedantic,
);

impl Rule for NoConditionalInTest {
    fn run<'a>(&self, node: &oxc_semantic::AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::IfStatement(if_statement) = node.kind() {
            let is_if_statement_in_test = ctx.nodes().iter_parents(node.id()).any(|node| {
                let AstKind::CallExpression(call_expr) = node.kind() else { return false };
                let vitest_node = PossibleJestNode { node, original: None };

                is_type_of_jest_fn_call(
                    call_expr,
                    &vitest_node,
                    ctx,
                    &[JestFnKind::General(JestGeneralFnKind::Test)],
                )
            });

            if is_if_statement_in_test {
                ctx.diagnostic(no_conditional_in_test(if_statement.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const x = y ? 1 : 0",
        "const foo = function (bar) {
			     return foo ? bar : null;
			      };
			      it('foo', () => {
			     foo();
			      });",
        "it.concurrent('foo', () => {
			     switch('bar') {}
			      })",
        "if (foo) {}",
        "it('foo', () => {})",
        r#"it("foo", function () {})"#,
        "it('foo', () => {}); function myTest() { if ('bar') {} }",
        "describe.each``('foo', () => {
			     afterEach(() => {
			       if ('bar') {}
			     });
			      })",
        "const values = something.map((thing) => {
			     if (thing.isFoo) {
			       return thing.foo
			     } else {
			       return thing.bar;
			     }
			      });
			   
			      describe('valid', () => {
			     it('still valid', () => {
			       expect(values).toStrictEqual(['foo']);
			     });
			      });",
    ];

    let fail = vec![
        "it('foo', function () {
			      if('bar') {}
			     });",
        " describe('foo', () => {
			      it('bar', () => {
			        if ('bar') {}
			      })
			      it('baz', () => {
			        if ('qux') {}
			        if ('quux') {}
			      })
			       })",
        r#"test("shows error", () => {
			      if (1 === 2) {
			        expect(true).toBe(false);
			      }
			       });
			     
			       test("does not show error", () => {
			      setTimeout(() => console.log("noop"));
			      if (1 === 2) {
			        expect(true).toBe(false);
			      }
			       });"#,
    ];

    Tester::new(NoConditionalInTest::NAME, pass, fail).test_and_snapshot();
}
