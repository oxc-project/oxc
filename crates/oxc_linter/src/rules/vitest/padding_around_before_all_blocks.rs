use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    rules::shared::padding_around_before_all_blocks::{DOCUMENTATION, run},
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundBeforeAllBlocks;

declare_oxc_lint!(
    PaddingAroundBeforeAllBlocks,
    vitest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "next",
);

impl Rule for PaddingAroundBeforeAllBlocks {
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

beforeAll(() => {
});

describe('someText', () => {
  const something = 'abc';

  // A comment
  beforeAll(() => {
    // stuff
  });

  beforeAll(() => {
    // other stuff
  });
});

describe('someText', () => {
  const something = 'abc';

  beforeAll(() => {
    // stuff
  });
});
",
        "beforeAll(() => {});",
        "const thing = 123;\n\nbeforeAll(() => {});",
        "describe('foo', () => {\n  beforeAll(() => {});\n});",
        "const thing = 123;\n\n// A comment\nbeforeAll(() => {});",
        "beforeAll(() => {});\n\nfoo();",
        "const enabled = false;\nif (enabled) beforeAll(() => {});",
        "const enabled = false; enabled && beforeAll(() => {});",
        "label:\nbeforeAll(() => {});",
    ];

    let fail = vec![
        r"
const someText = 'abc';
beforeAll(() => {
});
describe('someText', () => {
  const something = 'abc';
  // A comment
  beforeAll(() => {
    // stuff
  });
  beforeAll(() => {
    // other stuff
  });
});

describe('someText', () => {
  const something = 'abc';
  beforeAll(() => {
    // stuff
  });
});
",
        "const thing = 123;\nbeforeAll(() => {});",
        "const thing = 123;\n// A comment\nbeforeAll(() => {});",
        "beforeAll(() => {});\nfoo();",
        "const thing = 123;\nawait beforeAll(() => {});",
        "{\n  const thing = 123;\n  beforeAll(() => {});\n}",
        "switch (value) {\n  case 1:\n    beforeAll(() => {});\n    foo();\n}",
        "const thing = 123\n;beforeAll(() => {})",
        "const thing = 123;\nlabel: beforeAll(() => {});",
    ];

    let fix = vec![
        (
            r"
const someText = 'abc';
beforeAll(() => {
});
describe('someText', () => {
  const something = 'abc';
  // A comment
  beforeAll(() => {
    // stuff
  });
  beforeAll(() => {
    // other stuff
  });
});

describe('someText', () => {
  const something = 'abc';
  beforeAll(() => {
    // stuff
  });
});
",
            r"
const someText = 'abc';

beforeAll(() => {
});

describe('someText', () => {
  const something = 'abc';

  // A comment
  beforeAll(() => {
    // stuff
  });

  beforeAll(() => {
    // other stuff
  });
});

describe('someText', () => {
  const something = 'abc';

  beforeAll(() => {
    // stuff
  });
});
",
        ),
        ("const thing = 123;\nbeforeAll(() => {});", "const thing = 123;\n\nbeforeAll(() => {});"),
        (
            "const thing = 123;\n// A comment\nbeforeAll(() => {});",
            "const thing = 123;\n\n// A comment\nbeforeAll(() => {});",
        ),
        ("beforeAll(() => {});\nfoo();", "beforeAll(() => {});\n\nfoo();"),
        (
            "const thing = 123;\nawait beforeAll(() => {});",
            "const thing = 123;\n\nawait beforeAll(() => {});",
        ),
        (
            "{\n  const thing = 123;\n  beforeAll(() => {});\n}",
            "{\n  const thing = 123;\n\n  beforeAll(() => {});\n}",
        ),
        (
            "switch (value) {\n  case 1:\n    beforeAll(() => {});\n    foo();\n}",
            "switch (value) {\n  case 1:\n    beforeAll(() => {});\n\n    foo();\n}",
        ),
        ("const thing = 123\n;beforeAll(() => {})", "const thing = 123\n\n;beforeAll(() => {})"),
        (
            "const thing = 123;\nlabel: beforeAll(() => {});",
            "const thing = 123;\n\nlabel: beforeAll(() => {});",
        ),
    ];

    Tester::new(
        PaddingAroundBeforeAllBlocks::NAME,
        PaddingAroundBeforeAllBlocks::PLUGIN,
        pass,
        fail,
    )
    .with_vitest_plugin(true)
    .expect_fix(fix)
    .test_and_snapshot();
}
