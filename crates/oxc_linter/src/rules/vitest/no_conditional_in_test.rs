use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::no_conditional_in_test::{DOCUMENTATION, run},
};

#[derive(Debug, Default, Clone)]
pub struct NoConditionalInTest;

declare_oxc_lint!(NoConditionalInTest, vitest, pedantic, docs = DOCUMENTATION, version = "0.8.0",);

impl Rule for NoConditionalInTest {
    fn run<'a>(&self, node: &oxc_semantic::AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::IfStatement(_)
            | AstKind::SwitchStatement(_)
            | AstKind::ConditionalExpression(_)
            | AstKind::LogicalExpression(_) => {}
            _ => return,
        }

        run(node, ctx);
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
    ]);

    let mut fail = vec![
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

    fail.extend([
        r#"
                    import { it } from "vitest";
                    it('foo', () => {
                      if ('bar') {}
                    })
                  "#,
        r#"
                    import { test } from "vitest";
                    test('foo', () => {
                      switch('bar') {}
                    })
                  "#,
    ]);

    Tester::new(NoConditionalInTest::NAME, NoConditionalInTest::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
