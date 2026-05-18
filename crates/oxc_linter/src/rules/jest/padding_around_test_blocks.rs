use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        JestGeneralFnKind, ParsedGeneralJestFnCall, PossibleJestNode, parse_general_jest_fn_call,
        report_missing_padding_before_jest_block,
    },
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundTestBlocks;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces a line of padding before and after 1 or more
    /// `test`/`it` statements.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent formatting of code can make the code more difficult to read
    /// and follow. This rule helps ensure that test blocks are visually
    /// separated from the rest of the code, making them easier to identify while
    /// looking through test files.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const thing = 123;
    /// test('foo', () => {});
    /// test('bar', () => {});
    /// ```
    ///
    /// ```js
    /// const thing = 123;
    /// it('foo', () => {});
    /// it('bar', () => {});
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const thing = 123;
    ///
    /// test('foo', () => {});
    ///
    /// test('bar', () => {});
    /// ```
    ///
    /// ```js
    /// const thing = 123;
    ///
    /// it('foo', () => {});
    ///
    /// it('bar', () => {});
    /// ```
    PaddingAroundTestBlocks,
    jest,
    style,
    fix,
    version = "1.13.0",
);

impl Rule for PaddingAroundTestBlocks {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, jest_node, ctx) else {
            return;
        };
        let ParsedGeneralJestFnCall { kind, name, .. } = &jest_fn_call;
        let Some(kind) = kind.to_general() else {
            return;
        };
        if kind != JestGeneralFnKind::Test {
            return;
        }
        report_missing_padding_before_jest_block(node, ctx, name);
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
        "it('foo', () => {});\nfit('bar', () => {});\ntest('baz', () => {});",
        "const thing = 123;\n/* one */\n/* two */\ntest('foo', () => {});",
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
            "it('foo', () => {});\nfit('bar', () => {});\ntest('baz', () => {});",
            "it('foo', () => {});\n\nfit('bar', () => {});\n\ntest('baz', () => {});",
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
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
