use std::collections::HashSet;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, AstNodeId};
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{parse_jest_fn_call, JestFnKind, JestGeneralFnKind, PossibleJestNode},
};

fn use_prefer_each(span: Span, fn_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enforce using `each` rather than manual loops")
        .with_help(format!("Prefer using `{fn_name}.each` rather than a manual loop."))
        .with_label(span)
}

#[inline]
fn is_in_test(ctx: &LintContext<'_>, id: AstNodeId) -> bool {
    ctx.nodes().iter_parents(id).any(|node| {
        let AstKind::CallExpression(ancestor_call_expr) = node.kind() else { return false };
        let Some(ancestor_member_expr) = ancestor_call_expr.callee.as_member_expression() else {
            return false;
        };
        let Some(id) = ancestor_member_expr.object().get_identifier_reference() else {
            return false;
        };

        matches!(JestFnKind::from(id.name.as_str()), JestFnKind::General(JestGeneralFnKind::Test))
    })
}

#[derive(Debug, Default, Clone)]
pub struct PreferEach;

declare_oxc_lint!(
    /// ### What it does
    /// This rule enforces using `each` rather than manual loops.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// for (const item of items) {
    /// 	describe(item, () => {
    /// 		expect(item).toBe('foo')
    /// 	})
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// describe.each(items)('item', (item) => {
    /// 	expect(item).toBe('foo')
    /// })
    /// ```
    PreferEach,
    style,
);

impl Rule for PreferEach {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let mut skip = HashSet::<AstNodeId>::new();
        ctx.nodes().iter().for_each(|node| {
            Self::run(node, ctx, &mut skip);
        });
    }
}

impl PreferEach {
    fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>, skip: &mut HashSet<AstNodeId>) {
        let kind = node.kind();

        let AstKind::CallExpression(call_expr) = kind else { return };

        let Some(vitest_fn_call) =
            parse_jest_fn_call(call_expr, &PossibleJestNode { node, original: None }, ctx)
        else {
            return;
        };

        if !matches!(
            vitest_fn_call.kind(),
            JestFnKind::General(
                JestGeneralFnKind::Describe | JestGeneralFnKind::Hook | JestGeneralFnKind::Test
            )
        ) {
            return;
        }

        for parent_node in ctx.nodes().iter_parents(node.id()).skip(1) {
            match parent_node.kind() {
                AstKind::CallExpression(_) => {
                    return;
                }
                AstKind::ForStatement(_)
                | AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_) => {
                    if skip.contains(&parent_node.id()) || is_in_test(ctx, parent_node.id()) {
                        return;
                    }

                    let fn_name = if matches!(
                        vitest_fn_call.kind(),
                        JestFnKind::General(JestGeneralFnKind::Test)
                    ) {
                        "it"
                    } else {
                        "describe"
                    };

                    let span = match parent_node.kind() {
                        AstKind::ForStatement(statement) => {
                            Span::new(statement.span.start, statement.body.span().start)
                        }
                        AstKind::ForInStatement(statement) => {
                            Span::new(statement.span.start, statement.body.span().start)
                        }
                        AstKind::ForOfStatement(statement) => {
                            Span::new(statement.span.start, statement.body.span().start)
                        }
                        _ => unreachable!(),
                    };

                    skip.insert(parent_node.id());
                    ctx.diagnostic(use_prefer_each(span, fn_name));

                    break;
                }
                _ => {}
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"it("is true", () => { expect(true).toBe(false) });"#,
        r#"it.each(getNumbers())("only returns numbers that are greater than seven", number => {
			     expect(number).toBeGreaterThan(7);
			      });"#,
        r#"it("returns numbers that are greater than five", function () {
			     for (const number of getNumbers()) {
			       expect(number).toBeGreaterThan(5);
			     }
			      });"#,
        r#"it("returns things that are less than ten", function () {
			     for (const thing in things) {
			       expect(thing).toBeLessThan(10);
			     }
			      });"#,
        r#"it("only returns numbers that are greater than seven", function () {
			     const numbers = getNumbers();
			   
			     for (let i = 0; i < numbers.length; i++) {
			       expect(numbers[i]).toBeGreaterThan(7);
			     }
			      });"#,
    ];

    let fail = vec![
        "  for (const [input, expected] of data) {
			      it(`results in ${expected}`, () => {
			        expect(fn(input)).toBe(expected)
			      });
			       }",
        " for (const [input, expected] of data) {
			      describe(`when the input is ${input}`, () => {
			        it(`results in ${expected}`, () => {
			       expect(fn(input)).toBe(expected)
			        });
			      });
			       }",
        "for (const [input, expected] of data) {
			      describe(`when the input is ${input}`, () => {
			        it(`results in ${expected}`, () => {
			       expect(fn(input)).toBe(expected)
			        });
			      });
			       }
			     
			       for (const [input, expected] of data) {
			      it.skip(`results in ${expected}`, () => {
			        expect(fn(input)).toBe(expected)
			      });
			       }",
        "for (const [input, expected] of data) {
			      it.skip(`results in ${expected}`, () => {
			        expect(fn(input)).toBe(expected)
			      });
			       }",
        "it('is true', () => {
			      expect(true).toBe(false);
			       });
			     
			       for (const [input, expected] of data) {
			      it.skip(`results in ${expected}`, () => {
			        expect(fn(input)).toBe(expected)
			      });
			       }",
        " for (const [input, expected] of data) {
			      it.skip(`results in ${expected}`, () => {
			        expect(fn(input)).toBe(expected)
			      });
			       }
			     
			       it('is true', () => {
			      expect(true).toBe(false);
			       });",
        " it('is true', () => {
			      expect(true).toBe(false);
			       });
			     
			       for (const [input, expected] of data) {
			      it.skip(`results in ${expected}`, () => {
			        expect(fn(input)).toBe(expected)
			      });
			       }
			     
			       it('is true', () => {
			      expect(true).toBe(false);
			       });",
        "for (const [input, expected] of data) {
			      it(`results in ${expected}`, () => {
			        expect(fn(input)).toBe(expected)
			      });
			     
			      it(`results in ${expected}`, () => {
			        expect(fn(input)).toBe(expected)
			      });
			       }",
        "for (const [input, expected] of data) {
			      it(`results in ${expected}`, () => {
			        expect(fn(input)).toBe(expected)
			      });
			       }
			     
			       for (const [input, expected] of data) {
			      it(`results in ${expected}`, () => {
			        expect(fn(input)).toBe(expected)
			      });
			       }",
        "for (const [input, expected] of data) {
			      beforeEach(() => setupSomething(input));
			     
			      test(`results in ${expected}`, () => {
			        expect(doSomething()).toBe(expected)
			      });
			       }",
        r#"
			       for (const [input, expected] of data) {
			      it("only returns numbers that are greater than seven", function () {
			        const numbers = getNumbers(input);
			    
			        for (let i = 0; i < numbers.length; i++) {
			       expect(numbers[i]).toBeGreaterThan(7);
			        }
			      });
			       }
			     "#,
        r#"
			       for (const [input, expected] of data) {
			      beforeEach(() => setupSomething(input));
			     
			      it("only returns numbers that are greater than seven", function () {
			        const numbers = getNumbers();
			    
			        for (let i = 0; i < numbers.length; i++) {
			       expect(numbers[i]).toBeGreaterThan(7);
			        }
			      });
			       }
			     "#,
    ];

    Tester::new(PreferEach::NAME, pass, fail).test_and_snapshot();
}
