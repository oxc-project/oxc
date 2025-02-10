use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{is_type_of_jest_fn_call, JestFnKind, PossibleJestNode},
};

fn no_conditional_in_test(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid having conditionals in tests.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoConditionalInTest;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow conditional statements in tests.
    ///
    /// ### Why is this bad?
    ///
    /// Conditional statements in tests can make the test harder to read and understand. It is better to have a single test case per test function.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// it('foo', () => {
    ///   if (true) {
    /// 	doTheThing();
    ///   }
    /// });
    ///
    /// it('bar', () => {
    ///   switch (mode) {
    /// 	case 'none':
    /// 	  generateNone();
    /// 	case 'single':
    /// 	  generateOne();
    /// 	case 'multiple':
    /// 	  generateMany();
    ///   }
    ///
    ///   expect(fixtures.length).toBeGreaterThan(-1);
    /// });
    ///
    /// it('baz', async () => {
    ///   const promiseValue = () => {
    /// 	return something instanceof Promise
    /// 	  ? something
    /// 	  : Promise.resolve(something);
    ///   };
    ///
    ///   await expect(promiseValue()).resolves.toBe(1);
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// describe('my tests', () => {
    ///   if (true) {
    /// 	it('foo', () => {
    /// 	  doTheThing();
    /// 	});
    ///   }
    /// });
    ///
    /// beforeEach(() => {
    ///   switch (mode) {
    /// 	case 'none':
    /// 	  generateNone();
    /// 	case 'single':
    /// 	  generateOne();
    /// 	case 'multiple':
    /// 	  generateMany();
    ///   }
    /// });
    ///
    /// it('bar', () => {
    ///   expect(fixtures.length).toBeGreaterThan(-1);
    /// });
    ///
    /// const promiseValue = something => {
    ///   return something instanceof Promise ? something : Promise.resolve(something);
    /// };
    ///
    /// it('baz', async () => {
    ///   await expect(promiseValue()).resolves.toBe(1);
    /// });
    /// ```
    NoConditionalInTest,
    jest,
    pedantic,
);

impl Rule for NoConditionalInTest {
    fn run<'a>(&self, node: &oxc_semantic::AstNode<'a>, ctx: &LintContext<'a>) {
        if matches!(
            node.kind(),
            AstKind::IfStatement(_)
                | AstKind::SwitchStatement(_)
                | AstKind::ConditionalExpression(_)
                | AstKind::LogicalExpression(_)
        ) {
            let is_if_statement_in_test = ctx.nodes().ancestors(node.id()).any(|node| {
                let AstKind::CallExpression(call_expr) = node.kind() else { return false };
                let vitest_node = PossibleJestNode { node, original: None };

                is_type_of_jest_fn_call(
                    call_expr,
                    &vitest_node,
                    ctx,
                    &[JestFnKind::General(crate::utils::JestGeneralFnKind::Test)],
                )
            });

            if is_if_statement_in_test {
                let span = match node.kind() {
                    AstKind::IfStatement(stmt) => stmt.span,
                    AstKind::SwitchStatement(stmt) => stmt.span,
                    AstKind::ConditionalExpression(expr) => expr.span,
                    AstKind::LogicalExpression(expr) => expr.span,
                    _ => unreachable!(),
                };

                ctx.diagnostic(no_conditional_in_test(span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const x = y ? 1 : 0",
        "
			      const foo = function (bar) {
			        return foo ? bar : null;
			      };
			
			      it('foo', () => {
			        foo();
			      });
			    ",
        "
			      const foo = function (bar) {
			        return foo ? bar : null;
			      };
			
			      it.each()('foo', function () {
			        foo();
			      });
			    ",
        "
			      fit.concurrent('foo', () => {
			        switch('bar') {}
			      })
			    ",
        "it('foo', () => {})",
        "
			      switch (true) {
			        case true: {}
			      }
			    ",
        "
			      it('foo', () => {});
			      function myTest() {
			        switch ('bar') {
			        }
			      }
			    ",
        "
			      foo('bar', () => {
			        switch(baz) {}
			      })
			    ",
        "
			      describe('foo', () => {
			        switch('bar') {}
			      })
			    ",
        "
			      describe.skip('foo', () => {
			        switch('bar') {}
			      })
			    ",
        "
			      describe.skip.each()('foo', () => {
			        switch('bar') {}
			      })
			    ",
        "
			      xdescribe('foo', () => {
			        switch('bar') {}
			      })
			    ",
        "
			      fdescribe('foo', () => {
			        switch('bar') {}
			      })
			    ",
        "
			      describe('foo', () => {
			        switch('bar') {}
			      })
			      switch('bar') {}
			    ",
        "
			      describe('foo', () => {
			        afterEach(() => {
			          switch('bar') {}
			        });
			      });
			    ",
        "
			      const values = something.map(thing => {
			        switch (thing.isFoo) {
			          case true:
			            return thing.foo;
			          default:
			            return thing.bar;
			        }
			      });
			
			      it('valid', () => {
			        expect(values).toStrictEqual(['foo']);
			      });
			    ",
        "
			      describe('valid', () => {
			        const values = something.map(thing => {
			          switch (thing.isFoo) {
			            case true:
			              return thing.foo;
			            default:
			              return thing.bar;
			          }
			        });
			        it('still valid', () => {
			          expect(values).toStrictEqual(['foo']);
			        });
			      });
			    ",
        "if (foo) {}",
        "it('foo', () => {})",
        r#"it("foo", function () {})"#,
        "it('foo', () => {}); function myTest() { if ('bar') {} }",
        "
			      foo('bar', () => {
			        if (baz) {}
			      })
			    ",
        "
			      describe('foo', () => {
			        if ('bar') {}
			      })
			    ",
        "
			      describe.skip('foo', () => {
			        if ('bar') {}
			      })
			    ",
        "
			      xdescribe('foo', () => {
			        if ('bar') {}
			      })
			    ",
        "
			      fdescribe('foo', () => {
			        if ('bar') {}
			      })
			    ",
        "
			      describe('foo', () => {
			        if ('bar') {}
			      })
			      if ('baz') {}
			    ",
        "
			      describe('foo', () => {
			        afterEach(() => {
			          if ('bar') {}
			        });
			      })
			    ",
        "
			      describe.each``('foo', () => {
			        afterEach(() => {
			          if ('bar') {}
			        });
			      })
			    ",
        "
			      describe('foo', () => {
			        beforeEach(() => {
			          if ('bar') {}
			        });
			      })
			    ",
        "const foo = bar ? foo : baz;",
        "
			      const values = something.map((thing) => {
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
			      });
			    ",
        "
			      describe('valid', () => {
			        const values = something.map((thing) => {
			          if (thing.isFoo) {
			            return thing.foo
			          } else {
			            return thing.bar;
			          }
			        });
			
			        describe('still valid', () => {
			          it('really still valid', () => {
			            expect(values).toStrictEqual(['foo']);
			          });
			        });
			      });
			    ",
        "
			      fit.concurrent('foo', () => {
			        if ('bar') {}
			      })
			    ",
    ];

    let fail = vec![
        "
			        it('foo', () => {
			          expect(bar ? foo : baz).toBe(boo);
			        })
			      ",
        "
			        it('foo', function () {
			          const foo = function (bar) {
			            return foo ? bar : null;
			          };
			        });
			      ",
        "
			        it('foo', () => {
			          const foo = bar ? foo : baz;
			        })
			      ",
        "
			        it('foo', () => {
			          const foo = bar ? foo : baz;
			        })
			        const foo = bar ? foo : baz;
			      ",
        "
			        it('foo', () => {
			          const foo = bar ? foo : baz;
			          const anotherFoo = anotherBar ? anotherFoo : anotherBaz;
			        })
			      ",
        "
			        it('is invalid', () => {
			          const values = something.map(thing => {
			            switch (thing.isFoo) {
			              case true:
			                return thing.foo;
			              default:
			                return thing.bar;
			            }
			          });
			
			          expect(values).toStrictEqual(['foo']);
			        });
			      ",
        "
			        it('foo', () => {
			          switch (true) {
			            case true: {}
			          }
			        })
			      ",
        "
			        it('foo', () => {
			          switch('bar') {}
			        })
			      ",
        "
			        it.skip('foo', () => {
			          switch('bar') {}
			        })
			      ",
        "
			        it.only('foo', () => {
			          switch('bar') {}
			        })
			      ",
        "
			        xit('foo', () => {
			          switch('bar') {}
			        })
			      ",
        "
			        fit('foo', () => {
			          switch('bar') {}
			        })
			      ",
        "
			        test('foo', () => {
			          switch('bar') {}
			        })
			      ",
        "
			        test.skip('foo', () => {
			          switch('bar') {}
			        })
			      ",
        "
			        test.only('foo', () => {
			          switch('bar') {}
			        })
			      ",
        "
			        xtest('foo', () => {
			          switch('bar') {}
			        })
			      ",
        "
			        xtest('foo', function () {
			          switch('bar') {}
			        })
			      ",
        "
			        describe('foo', () => {
			          it('bar', () => {
			
			            switch('bar') {}
			          })
			        })
			      ",
        "
			        describe('foo', () => {
			          it('bar', () => {
			            switch('bar') {}
			          })
			          it('baz', () => {
			            switch('qux') {}
			            switch('quux') {}
			          })
			        })
			      ",
        "
			        it('foo', () => {
			          callExpression()
			          switch ('bar') {}
			        })
			      ",
        "
			        describe('valid', () => {
			          describe('still valid', () => {
			            it('is not valid', () => {
			              const values = something.map((thing) => {
			                switch (thing.isFoo) {
			                  case true:
			                    return thing.foo;
			                  default:
			                    return thing.bar;
			                }
			              });
			
			              switch('invalid') {
			                case true:
			                  expect(values).toStrictEqual(['foo']);
			              }
			            });
			          });
			        });
			      ",
        "
			        it('foo', () => {
			          const foo = function(bar) {
			            if (bar) {
			              return 1;
			            } else {
			              return 2;
			            }
			          };
			        });
			      ",
        "
			        it('foo', () => {
			          function foo(bar) {
			            if (bar) {
			              return 1;
			            } else {
			              return 2;
			            }
			          };
			        });
			      ",
        "
			        it('foo', () => {
			          if ('bar') {}
			        })
			      ",
        "
			        it.skip('foo', () => {
			          if ('bar') {}
			        })
			      ",
        "
			        it.skip('foo', function () {
			          if ('bar') {}
			        })
			      ",
        "
			        it.only('foo', () => {
			          if ('bar') {}
			        })
			      ",
        "
			        xit('foo', () => {
			          if ('bar') {}
			        })
			      ",
        "
			        fit('foo', () => {
			          if ('bar') {}
			        })
			      ",
        "
			        test('foo', () => {
			          if ('bar') {}
			        })
			      ",
        "
			        test.skip('foo', () => {
			          if ('bar') {}
			        })
			      ",
        "
			        test.only('foo', () => {
			          if ('bar') {}
			        })
			      ",
        "
			        xtest('foo', () => {
			          if ('bar') {}
			        })
			      ",
        "
			        describe('foo', () => {
			          it('bar', () => {
			            if ('bar') {}
			          })
			        })
			      ",
        "
			        describe('foo', () => {
			          it('bar', () => {
			            if ('bar') {}
			          })
			          it('baz', () => {
			            if ('qux') {}
			            if ('quux') {}
			          })
			        })
			      ",
        "
			        it('foo', () => {
			          callExpression()
			          if ('bar') {}
			        })
			      ",
        "
			        it.each``('foo', () => {
			          callExpression()
			          if ('bar') {}
			        })
			      ",
        "
			        it.each()('foo', () => {
			          callExpression()
			          if ('bar') {}
			        })
			      ",
        "
			        it.only.each``('foo', () => {
			          callExpression()
			          if ('bar') {}
			        })
			      ",
        "
			        it.only.each()('foo', () => {
			          callExpression()
			          if ('bar') {}
			        })
			      ",
        "
			        describe('valid', () => {
			          describe('still valid', () => {
			            it('is invalid', () => {
			              const values = something.map((thing) => {
			                if (thing.isFoo) {
			                  return thing.foo
			                } else {
			                  return thing.bar;
			                }
			              });
			
			              if ('invalid') {
			                expect(values).toStrictEqual(['foo']);
			              }
			            });
			          });
			        });
			      ",
        r#"
			        test("shows error", () => {
			          if (1 === 2) {
			            expect(true).toBe(false);
			          }
			        });
			
			        test("does not show error", () => {
			          setTimeout(() => console.log("noop"));
			          if (1 === 2) {
			            expect(true).toBe(false);
			          }
			        });
			      "#,
    ];

    Tester::new(NoConditionalInTest::NAME, NoConditionalInTest::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
