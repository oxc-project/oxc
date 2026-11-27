use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    rules::shared::padding_around_test_blocks::{DOCUMENTATION, run},
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundTestBlocks;

declare_oxc_lint!(
    PaddingAroundTestBlocks,
    vitest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "1.75.0",
    short_description =
        "This rule enforces a line of padding before and after 1 or more `test`/`it` statements.",
);

impl Rule for PaddingAroundTestBlocks {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Program(_)
            | AstKind::BlockStatement(_)
            | AstKind::FunctionBody(_)
            | AstKind::SwitchCase(_) => run(node, ctx),
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.frameworks().is_test()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "test('foo', () => {});",
        "test('foo', () => {});\n\ntest('bar', () => {});",
        "const thing = 123;\n\ntest('foo', () => {});",
        "{ test('foo', () => {}); }",
        "describe('foo', () => {\ntest('bar', () => {});\n});",
        "const thing = 123;\n\n/* one */\n/* two */\ntest('foo', () => {});",
        "const slow = false; slow && test('slow', () => {});",
        "const slow = false;\nif (slow)\n  test('slow', () => {});",
        "const slow = false; if (slow) test('slow', () => {});",
        "const slow = false; if (slow)\n  test('slow', () => {});",
        "const slow = false; while (slow) test('slow', () => {});",
        "const slow = false; for (; slow;) test('slow', () => {});",
        r"
const foo = 'bar';
const bar = 'baz';

it('foo', () => {
  // stuff
});

fit('bar', () => {
  // stuff
});

test('foo foo', () => {});

test('bar bar', () => {});

// Nesting
describe('other bar', () => {
  const thing = 123;

  test('is another bar w/ test', () => {
  });

  // With a comment
  it('is another bar w/ it', () => {
  });

  test.skip('skipping', () => {}); // Another comment

  it.skip('skipping too', () => {});
});

xtest('weird', () => {});

test
  .skip('skippy skip', () => {});

xit('bar foo', () => {});
",
        "test('foo', () => {});\n\nfoo();",
        "label:\ntest('foo', () => {});",
    ];

    let fail = vec![
        "test('foo', () => {});test('bar', () => {});",
        "test('foo', () => {});\ntest('bar', () => {});",
        "const thing = 123;\n/* one */\n/* two */\ntest('foo', () => {});",
        r"
const foo = 'bar';
const bar = 'baz';
it('foo', () => {
  // stuff
});
test('foo foo', () => {});
test('bar bar', () => {});

// Nesting
 describe('other bar', () => {
     const thing = 123;
     test('is another bar w/ test', () => {
     });
     // With a comment
     it('is another bar w/ it', () => {
     });
     test.skip('skipping', () => {}); // Another comment
     it.skip('skipping too', () => {});
 });
 test
   .skip('skippy skip', () => {});
            ",
        r"
describe('other bar', function() {
    test('is another bar w/ test', () => {
    });
    it('is another bar w/ it', () => {
    });
});
            ",
        r"
const foo = 'bar';
const bar = 'baz';
it('foo', () => {
  // stuff
});
fit('bar', () => {
  // stuff
});
test('foo foo', () => {});
test('bar bar', () => {});

// Nesting
describe('other bar', () => {
  const thing = 123;
  test('is another bar w/ test', () => {
  });
  // With a comment
  it('is another bar w/ it', () => {
  });
  test.skip('skipping', () => {}); // Another comment
  it.skip('skipping too', () => {});
});xtest('weird', () => {});
test
  .skip('skippy skip', () => {});
xit('bar foo', () => {});
",
        "test('foo', () => {});\nfoo();",
        "{\n  const thing = 123;\n  test('foo', () => {});\n}",
        "switch (value) {\n  case 1:\n    test('foo', () => {});\n    foo();\n}",
        "const thing = 123\n;test('foo', () => {})",
        "const thing = 123;\nlabel: test('foo', () => {});",
    ];

    let fix = vec![
        (
            "test('foo', () => {});test('bar', () => {});",
            "test('foo', () => {});\n\ntest('bar', () => {});",
        ),
        (
            "test('foo', () => {});\ntest('bar', () => {});",
            "test('foo', () => {});\n\ntest('bar', () => {});",
        ),
        (
            "const thing = 123;\n/* one */\n/* two */\ntest('foo', () => {});",
            "const thing = 123;\n\n/* one */\n/* two */\ntest('foo', () => {});",
        ),
        (
            r"
const foo = 'bar';
const bar = 'baz';
it('foo', () => {
  // stuff
});
test('foo foo', () => {});
test('bar bar', () => {});

// Nesting
describe('other bar', () => {
    const thing = 123;
    test('is another bar w/ test', () => {
    });
    // With a comment
    it('is another bar w/ it', () => {
    });
    test.skip('skipping', () => {}); // Another comment
    it.skip('skipping too', () => {});
});
test
  .skip('skippy skip', () => {});
        ",
            r"
const foo = 'bar';
const bar = 'baz';

it('foo', () => {
  // stuff
});

test('foo foo', () => {});

test('bar bar', () => {});

// Nesting
describe('other bar', () => {
    const thing = 123;

    test('is another bar w/ test', () => {
    });

    // With a comment
    it('is another bar w/ it', () => {
    });

    test.skip('skipping', () => {}); // Another comment

    it.skip('skipping too', () => {});
});

test
  .skip('skippy skip', () => {});
        ",
        ),
        (
            r"
describe('other bar', function() {
    test('is another bar w/ test', () => {
    });
    it('is another bar w/ it', () => {
    });
});
            ",
            r"
describe('other bar', function() {
    test('is another bar w/ test', () => {
    });

    it('is another bar w/ it', () => {
    });
});
            ",
        ),
        (
            "const thing = 123; test('foo', () => {}), foo();",
            "const thing = 123;\n\ntest('foo', () => {}), foo();",
        ),
        (
            "const thing = 123; test('foo', () => {}) && foo();",
            "const thing = 123;\n\ntest('foo', () => {}) && foo();",
        ),
        (
            "const thing = 123; await test('foo', () => {});",
            "const thing = 123;\n\nawait test('foo', () => {});",
        ),
        (
            "const thing = 123;\nawait test('foo', () => {});",
            "const thing = 123;\n\nawait test('foo', () => {});",
        ),
        (
            "const thing = 123; await /* hook */ test('foo', () => {});",
            "const thing = 123;\n\nawait /* hook */ test('foo', () => {});",
        ),
        (
            r"
const foo = 'bar';
const bar = 'baz';
it('foo', () => {
  // stuff
});
fit('bar', () => {
  // stuff
});
test('foo foo', () => {});
test('bar bar', () => {});

// Nesting
describe('other bar', () => {
  const thing = 123;
  test('is another bar w/ test', () => {
  });
  // With a comment
  it('is another bar w/ it', () => {
  });
  test.skip('skipping', () => {}); // Another comment
  it.skip('skipping too', () => {});
});xtest('weird', () => {});
test
  .skip('skippy skip', () => {});
xit('bar foo', () => {});
",
            r"
const foo = 'bar';
const bar = 'baz';

it('foo', () => {
  // stuff
});

fit('bar', () => {
  // stuff
});

test('foo foo', () => {});

test('bar bar', () => {});

// Nesting
describe('other bar', () => {
  const thing = 123;

  test('is another bar w/ test', () => {
  });

  // With a comment
  it('is another bar w/ it', () => {
  });

  test.skip('skipping', () => {}); // Another comment

  it.skip('skipping too', () => {});
});

xtest('weird', () => {});

test
  .skip('skippy skip', () => {});

xit('bar foo', () => {});
",
        ),
        ("test('foo', () => {});\nfoo();", "test('foo', () => {});\n\nfoo();"),
        (
            "{\n  const thing = 123;\n  test('foo', () => {});\n}",
            "{\n  const thing = 123;\n\n  test('foo', () => {});\n}",
        ),
        (
            "switch (value) {\n  case 1:\n    test('foo', () => {});\n    foo();\n}",
            "switch (value) {\n  case 1:\n    test('foo', () => {});\n\n    foo();\n}",
        ),
        (
            "const thing = 123\n;test('foo', () => {})",
            "const thing = 123\n\n;test('foo', () => {})",
        ),
        (
            "const thing = 123;\nlabel: test('foo', () => {});",
            "const thing = 123;\n\nlabel: test('foo', () => {});",
        ),
    ];
    Tester::new(PaddingAroundTestBlocks::NAME, PaddingAroundTestBlocks::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
