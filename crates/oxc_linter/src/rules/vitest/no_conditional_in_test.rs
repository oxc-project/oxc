use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call},
};

fn no_conditional_in_test(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Remove conditional tests").with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

Disallow conditional tests.

### Why is this bad?

A test that contains conditionals can pass or fail depending on runtime
conditions, which makes the test non-deterministic and harder to reason
about. It is better to have a single unconditional test case per test.

### Examples

Examples of **incorrect** code for this rule:

```js
test('my test', () => {
  if (true) {
    doTheThing()
  }
})
```

Examples of **correct** code for this rule:

```js
test('my test', () => {
  expect(true).toBe(true)
})
```

### Notes

Like [`eslint-plugin-vitest`](https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/src/rules/no-conditional-in-test.ts),
this rule only reports an `if` statement that is a direct statement of the
`test`/`it` callback body. Conditionals in functions defined inside the test,
and `switch` statements, ternaries, and logical expressions, are not reported.
If you want the wider net that also reports those, use
`jest/no-conditional-in-test` instead.
";

#[derive(Debug, Default, Clone)]
pub struct NoConditionalInTest;

declare_oxc_lint!(NoConditionalInTest, vitest, pedantic, docs = DOCUMENTATION, version = "0.8.0",);

impl Rule for NoConditionalInTest {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // See <https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/src/rules/no-conditional-in-test.ts>
        // Only an `if` statement that is a direct statement of the `test`/`it`
        // callback body is reported:
        // `IfStatement` -> `FunctionBody` -> callback function -> test `CallExpression`.
        let AstKind::IfStatement(stmt) = node.kind() else { return };

        let mut ancestors = ctx.nodes().ancestors(node.id());

        // The `if` must be a direct statement of a function body, not nested in
        // an inner function, block, or another statement.
        if !matches!(ancestors.next().map(AstNode::kind), Some(AstKind::FunctionBody(_))) {
            return;
        }

        // The body must belong to the callback function itself.
        if !matches!(
            ancestors.next().map(AstNode::kind),
            Some(AstKind::Function(_) | AstKind::ArrowFunctionExpression(_))
        ) {
            return;
        }

        // The callback must be directly passed to a `test`/`it` call.
        let Some(call_node) = ancestors.next() else { return };
        let AstKind::CallExpression(call_expr) = call_node.kind() else { return };
        let Some(possible_jest_node) =
            PossibleJestNode::from_call_expression(call_node, call_expr, ctx)
        else {
            return;
        };

        if is_type_of_jest_fn_call(
            call_expr,
            &possible_jest_node,
            ctx,
            &[JestFnKind::General(JestGeneralFnKind::Test)],
        ) {
            ctx.diagnostic(no_conditional_in_test(stmt.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
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
                  it.concurrent('foo', () => {
                    switch('bar') {}
                  })
                ",
        "
                  fit.concurrent('foo', () => {
                    switch('bar') {}
                  })
                ",
        "
                  fit.concurrent('foo', () => {
                    if ('bar') {}
                  })
                ",
        "it('foo', () => {})",
        r#"it("foo", function () {})"#,
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
        // `switch` statements, ternaries, and logical expressions are not
        // reported, matching eslint-plugin-vitest.
        "
                    it('foo', () => {
                      expect(bar ? foo : baz).toBe(boo);
                    })
                  ",
        "
                    it('foo', () => {
                      const foo = bar ? foo : baz;
                    })
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
                      bar && baz();
                    })
                  ",
        // Conditionals in functions defined inside the test are not reported.
        "
                    it('foo', () => {
                      const foo = function (bar) {
                        return foo ? bar : null;
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
                      function Greeting(props) {
                        if (!props.name) return 'Hello, stranger'
                        return `Hello, ${props.name}`
                      }

                      expect(Greeting({})).toBe('Hello, stranger');
                    });
                  ",
        "
                    it('foo', () => {
                      const queryFn = (page) => (page > 0 ? 'next' : 'first')

                      expect(queryFn(0)).toBe('first');
                    });
                  ",
        "
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
                  ",
        // An `if` that is not a direct statement of the test callback body is
        // not reported.
        "
                    it('foo', () => {
                      {
                        if ('bar') {
                        } else if ('baz') {
                        }
                      }
                    })
                  ",
    ];

    pass.extend([
        r#"
                  import { describe } from "vitest";
                  describe('foo', () => {
                    if ('bar') {}
                  })
                "#,
        r#"
                  import { bench } from "vitest";
                  bench('foo', () => {
                    if ('bar') {}
                  })
                "#,
        r"import { test as baseTest } from 'vitest';
        const customTest = baseTest.extend({});
        function helper(customTest) {
            customTest('foo', () => { if (condition) {} });
        }",
        r"import { test as baseTest } from 'other';
        const customTest = baseTest.extend({});
        customTest('foo', () => { if (condition) {} });",
        r"const test = (name, callback) => callback();
        test('foo', () => { if (condition) {} });",
    ]);

    let mut fail = vec![
        "
                    it('foo', () => {
                      if ('bar') {}
                    })
                  ",
        "
                    it('foo', function () {
                      if ('bar') {}
                    })
                  ",
        "
                    it.skip('foo', () => {
                      if ('bar') {}
                    })
                  ",
        "
                    it.only('foo', () => {
                      if ('bar') {}
                    })
                  ",
        "
                    it.concurrent('foo', () => {
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
                    test('foo', function () {
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
                      if ('bar') {
                      } else if ('baz') {
                      }
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
                    it.each([1, 2])('foo', (value) => {
                      if (value) {}
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
                        it('is not valid', () => {
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

    fail.extend([
        r#"
                    import { it } from "vitest";
                    it('foo', () => {
                      if ('bar') {}
                    })
                  "#,
        r#"
                    import { test } from "vitest";
                    test('foo', function () {
                      if ('bar') {}
                    })
                  "#,
        r"import { test as baseTest } from 'vitest';
        const extendedTest = baseTest.extend({});
        const customTest = extendedTest.extend({});
        customTest.only('foo', () => {
            if (condition) {}
        });",
        r"import { it as customTest } from 'vitest';
        customTest.each([true])('foo', value => {
            if (value) {}
        });",
    ]);

    Tester::new(NoConditionalInTest::NAME, NoConditionalInTest::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
