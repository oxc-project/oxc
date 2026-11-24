use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    rules::shared::padding_around_describe_blocks::{DOCUMENTATION, run},
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundDescribeBlocks;

declare_oxc_lint!(
    PaddingAroundDescribeBlocks,
    vitest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "next",
);

impl Rule for PaddingAroundDescribeBlocks {
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
        r"
foo();
bar();

const someText = 'abc';
const someObject = {
  one: 1,
  two: 2,
};

// A comment before describe
describe('someText', () => {
  describe('some condition', () => {
  });

  describe('some other condition', () => {
  });
});

xdescribe('someObject', () => {
  // Another comment
  describe('some condition', () => {
    const anotherThing = 500;

    describe('yet another condition', () => { // A comment over here!
    });
  });
});

fdescribe('weird', () => {});

describe.skip('skip me', () => {});

const BOOP = 'boop';

describe
  .skip('skip me too', () => {
    // stuff
  });
",
        "describe('foo', () => {});",
        "const thing = 123;\n\ndescribe('foo', () => {});",
        "describe('foo', () => {});\n\ndescribe('bar', () => {});",
        "describe('foo', () => {\n  describe('bar', () => {});\n});",
        "test('foo', () => {});\ntest('bar', () => {});",
        "const enabled = false;\nif (enabled) describe('foo', () => {});",
        "label:\ndescribe('foo', () => {});",
    ];

    let fail = vec![
        r"
foo();
bar();

const someText = 'abc';
const someObject = {
  one: 1,
  two: 2,
};
// A comment before describe
describe('someText', () => {
  describe('some condition', () => {
  });
  describe('some other condition', () => {
  });
});
xdescribe('someObject', () => {
  // Another comment
  describe('some condition', () => {
    const anotherThing = 500;
    describe('yet another condition', () => { // A comment over here!
    });
  });
});fdescribe('weird', () => {});
describe.skip('skip me', () => {});
const BOOP = 'boop';
describe
  .skip('skip me too', () => {
    // stuff
  });
",
        "const thing = 123;\ndescribe('foo', () => {});",
        "describe('foo', () => {});\ndescribe('bar', () => {});",
        "xdescribe('foo', () => {});\nfdescribe('bar', () => {});",
        "const thing = 123;\ndescribe.each([1, 2])('%s', () => {});",
        "describe('foo', () => {});foo();",
        "{\n  const thing = 123;\n  describe('foo', () => {});\n}",
        "switch (value) {\n  case 1:\n    describe('foo', () => {});\n    foo();\n}",
        "const thing = 123\n;describe('foo', () => {})",
        "const thing = 123;\nlabel: describe('foo', () => {});",
    ];

    let fix = vec![
        (
            r"
foo();
bar();

const someText = 'abc';
const someObject = {
  one: 1,
  two: 2,
};
// A comment before describe
describe('someText', () => {
  describe('some condition', () => {
  });
  describe('some other condition', () => {
  });
});
xdescribe('someObject', () => {
  // Another comment
  describe('some condition', () => {
    const anotherThing = 500;
    describe('yet another condition', () => { // A comment over here!
    });
  });
});fdescribe('weird', () => {});
describe.skip('skip me', () => {});
const BOOP = 'boop';
describe
  .skip('skip me too', () => {
    // stuff
  });
",
            r"
foo();
bar();

const someText = 'abc';
const someObject = {
  one: 1,
  two: 2,
};

// A comment before describe
describe('someText', () => {
  describe('some condition', () => {
  });

  describe('some other condition', () => {
  });
});

xdescribe('someObject', () => {
  // Another comment
  describe('some condition', () => {
    const anotherThing = 500;

    describe('yet another condition', () => { // A comment over here!
    });
  });
});

fdescribe('weird', () => {});

describe.skip('skip me', () => {});

const BOOP = 'boop';

describe
  .skip('skip me too', () => {
    // stuff
  });
",
        ),
        (
            "const thing = 123;\ndescribe('foo', () => {});",
            "const thing = 123;\n\ndescribe('foo', () => {});",
        ),
        (
            "describe('foo', () => {});\ndescribe('bar', () => {});",
            "describe('foo', () => {});\n\ndescribe('bar', () => {});",
        ),
        (
            "xdescribe('foo', () => {});\nfdescribe('bar', () => {});",
            "xdescribe('foo', () => {});\n\nfdescribe('bar', () => {});",
        ),
        (
            "const thing = 123;\ndescribe.each([1, 2])('%s', () => {});",
            "const thing = 123;\n\ndescribe.each([1, 2])('%s', () => {});",
        ),
        ("describe('foo', () => {});foo();", "describe('foo', () => {});\n\nfoo();"),
        (
            "{\n  const thing = 123;\n  describe('foo', () => {});\n}",
            "{\n  const thing = 123;\n\n  describe('foo', () => {});\n}",
        ),
        (
            "switch (value) {\n  case 1:\n    describe('foo', () => {});\n    foo();\n}",
            "switch (value) {\n  case 1:\n    describe('foo', () => {});\n\n    foo();\n}",
        ),
        (
            "const thing = 123\n;describe('foo', () => {})",
            "const thing = 123\n\n;describe('foo', () => {})",
        ),
        (
            "const thing = 123;\nlabel: describe('foo', () => {});",
            "const thing = 123;\n\nlabel: describe('foo', () => {});",
        ),
    ];

    Tester::new(PaddingAroundDescribeBlocks::NAME, PaddingAroundDescribeBlocks::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
