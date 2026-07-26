use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::padding_around_test_blocks::{DOCUMENTATION, run},
    utils::PossibleJestNode,
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
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(jest_node, ctx);
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
    ];
    Tester::new(PaddingAroundTestBlocks::NAME, PaddingAroundTestBlocks::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
