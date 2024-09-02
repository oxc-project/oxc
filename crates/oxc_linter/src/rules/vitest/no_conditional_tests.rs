use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, is_type_of_jest_fn_call, JestFnKind, JestGeneralFnKind,
        PossibleJestNode,
    },
};

fn no_conditional_tests(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid having conditionals in tests")
        .with_help("Remove the surrounding if statement.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoConditionalTests;

declare_oxc_lint!(
    /// ### What it does
    /// The rule disallows the use of conditional statements within test cases to ensure that tests are deterministic and clearly readable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// describe('my tests', () => {
    ///     if (true) {
    ///         it('is awesome', () => {
    ///             doTheThing()
    ///         })
    ///     }
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// describe('my tests', () => {
    ///     it('is awesome', () => {
    ///         doTheThing()
    ///     })
    /// })
    /// ```
    NoConditionalTests,
    correctness,
);

impl Rule for NoConditionalTests {
    fn run_once(&self, ctx: &LintContext) {
        for node in &collect_possible_jest_call_node(ctx) {
            run(node, ctx);
        }
    }
}

fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;
    if let AstKind::CallExpression(call_expr) = node.kind() {
        if is_type_of_jest_fn_call(
            call_expr,
            possible_jest_node,
            ctx,
            &[
                JestFnKind::General(JestGeneralFnKind::Describe),
                JestFnKind::General(JestGeneralFnKind::Test),
            ],
        ) {
            let if_statement_node = ctx
                .nodes()
                .iter_parents(node.id())
                .find(|node| matches!(node.kind(), AstKind::IfStatement(_)));

            let Some(node) = if_statement_node else { return };

            if let AstKind::IfStatement(if_statement) = node.kind() {
                ctx.diagnostic(no_conditional_tests(if_statement.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"test("shows error", () => {});"#,
        r#"it("foo", function () {})"#,
        "it('foo', () => {}); function myTest() { if ('bar') {} }",
        r#"function myFunc(str: string) {
			    return str;
			  }
			  describe("myTest", () => {
			     it("convert shortened equal filter", () => {
			      expect(
			      myFunc("5")
			      ).toEqual("5");
			     });
			    });"#,
        r#"describe("shows error", () => {
			     if (1 === 2) {
			      myFunc();
			     }
			     expect(true).toBe(false);
			    });"#,
    ];

    let fail = vec![
        r#"describe("shows error", () => {
			    if(true) {
			     test("shows error", () => {
			      expect(true).toBe(true);
			     })
			    }
			   })"#,
        r#"
			   describe("shows error", () => {
			    if(true) {
			     it("shows error", () => {
			      expect(true).toBe(true);
			      })
			     }
			   })"#,
        r#"describe("errors", () => {
			    if (Math.random() > 0) {
			     test("test2", () => {
			     expect(true).toBeTruthy();
			    });
			    }
			   });"#,
    ];

    Tester::new(NoConditionalTests::NAME, pass, fail).test_and_snapshot();
}
