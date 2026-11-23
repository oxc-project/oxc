use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    rules::shared::padding_around_after_each_blocks::{DOCUMENTATION, run},
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundAfterEachBlocks;

declare_oxc_lint!(
    PaddingAroundAfterEachBlocks,
    vitest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "next",
);

impl Rule for PaddingAroundAfterEachBlocks {
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
const someText = 'abc';

afterEach(() => {
});

describe('someText', () => {
  const something = 'abc';

  // A comment
  afterEach(() => {
    // stuff
  });

  afterEach(() => {
    // other stuff
  });
});
describe('someText', () => {
  const something = 'abc';

  afterEach(() => {
    // stuff
  });
});
",
        "afterEach(() => {});",
        "const thing = 123;\n\nafterEach(() => {});",
        "describe('foo', () => {\n  afterEach(() => {});\n});",
        "const thing = 123;\n\n// A comment\nafterEach(() => {});",
        "afterEach(() => {});\n\nfoo();",
        "const enabled = false;\nif (enabled) afterEach(() => {});",
        "const enabled = false; enabled && afterEach(() => {});",
        "label:\nafterEach(() => {});",
    ];

    let fail = vec![
        r"
const someText = 'abc';
afterEach(() => {
});
describe('someText', () => {
  const something = 'abc';
  // A comment
  afterEach(() => {
    // stuff
  });
  afterEach(() => {
    // other stuff
  });
});
describe('someText', () => {
  const something = 'abc';
  afterEach(() => {
    // stuff
  });
});
",
        "const thing = 123;\nafterEach(() => {});",
        "const thing = 123;\n// A comment\nafterEach(() => {});",
        "afterEach(() => {});\nfoo();",
        "const thing = 123;\nawait afterEach(() => {});",
        "{\n  const thing = 123;\n  afterEach(() => {});\n}",
        "switch (value) {\n  case 1:\n    afterEach(() => {});\n    foo();\n}",
        "const thing = 123\n;afterEach(() => {})",
        "const thing = 123;\nlabel: afterEach(() => {});",
    ];

    let fix = vec![
        (
            r"
const someText = 'abc';
afterEach(() => {
});
describe('someText', () => {
  const something = 'abc';
  // A comment
  afterEach(() => {
    // stuff
  });
  afterEach(() => {
    // other stuff
  });
});
describe('someText', () => {
  const something = 'abc';
  afterEach(() => {
    // stuff
  });
});
",
            r"
const someText = 'abc';

afterEach(() => {
});

describe('someText', () => {
  const something = 'abc';

  // A comment
  afterEach(() => {
    // stuff
  });

  afterEach(() => {
    // other stuff
  });
});
describe('someText', () => {
  const something = 'abc';

  afterEach(() => {
    // stuff
  });
});
",
        ),
        ("const thing = 123;\nafterEach(() => {});", "const thing = 123;\n\nafterEach(() => {});"),
        (
            "const thing = 123;\n// A comment\nafterEach(() => {});",
            "const thing = 123;\n\n// A comment\nafterEach(() => {});",
        ),
        ("afterEach(() => {});\nfoo();", "afterEach(() => {});\n\nfoo();"),
        (
            "const thing = 123;\nawait afterEach(() => {});",
            "const thing = 123;\n\nawait afterEach(() => {});",
        ),
        (
            "{\n  const thing = 123;\n  afterEach(() => {});\n}",
            "{\n  const thing = 123;\n\n  afterEach(() => {});\n}",
        ),
        (
            "switch (value) {\n  case 1:\n    afterEach(() => {});\n    foo();\n}",
            "switch (value) {\n  case 1:\n    afterEach(() => {});\n\n    foo();\n}",
        ),
        ("const thing = 123\n;afterEach(() => {})", "const thing = 123\n\n;afterEach(() => {})"),
        (
            "const thing = 123;\nlabel: afterEach(() => {});",
            "const thing = 123;\n\nlabel: afterEach(() => {});",
        ),
    ];

    Tester::new(
        PaddingAroundAfterEachBlocks::NAME,
        PaddingAroundAfterEachBlocks::PLUGIN,
        pass,
        fail,
    )
    .with_vitest_plugin(true)
    .expect_fix(fix)
    .test_and_snapshot();
}
