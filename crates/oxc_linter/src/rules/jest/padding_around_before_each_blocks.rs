use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    rules::shared::padding_around_before_each_blocks::{DOCUMENTATION, run},
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundBeforeEachBlocks;

declare_oxc_lint!(
    PaddingAroundBeforeEachBlocks,
    jest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "next",
);

impl Rule for PaddingAroundBeforeEachBlocks {
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

beforeEach(() => {
});

describe('someText', () => {
  const something = 'abc';

  // A comment
  beforeEach(() => {
    // stuff
  });

  beforeEach(() => {
    // other stuff
  });
});

describe('someText', () => {
  const something = 'abc';

  beforeEach(() => {
    // stuff
  });
});
",
        "beforeEach(() => {});",
        "const thing = 123;\n\nbeforeEach(() => {});",
        "describe('foo', () => {\n  beforeEach(() => {});\n});",
        "const thing = 123;\n\n// A comment\nbeforeEach(() => {});",
        "beforeEach(() => {});\n\nfoo();",
        "const enabled = false;\nif (enabled) beforeEach(() => {});",
        "const enabled = false; enabled && beforeEach(() => {});",
        "label:\nbeforeEach(() => {});",
    ];

    let fail = vec![
        r"
const someText = 'abc';
beforeEach(() => {
});
describe('someText', () => {
  const something = 'abc';
  // A comment
  beforeEach(() => {
    // stuff
  });
  beforeEach(() => {
    // other stuff
  });
});

describe('someText', () => {
  const something = 'abc';
  beforeEach(() => {
    // stuff
  });
});
",
        "const thing = 123;\nbeforeEach(() => {});",
        "const thing = 123;\n// A comment\nbeforeEach(() => {});",
        "beforeEach(() => {});\nfoo();",
        "const thing = 123;\nawait beforeEach(() => {});",
        "{\n  const thing = 123;\n  beforeEach(() => {});\n}",
        "switch (value) {\n  case 1:\n    beforeEach(() => {});\n    foo();\n}",
        "const thing = 123\n;beforeEach(() => {})",
        "const thing = 123;\nlabel: beforeEach(() => {});",
    ];

    let fix = vec![
        (
            r"
const someText = 'abc';
beforeEach(() => {
});
describe('someText', () => {
  const something = 'abc';
  // A comment
  beforeEach(() => {
    // stuff
  });
  beforeEach(() => {
    // other stuff
  });
});

describe('someText', () => {
  const something = 'abc';
  beforeEach(() => {
    // stuff
  });
});
",
            r"
const someText = 'abc';

beforeEach(() => {
});

describe('someText', () => {
  const something = 'abc';

  // A comment
  beforeEach(() => {
    // stuff
  });

  beforeEach(() => {
    // other stuff
  });
});

describe('someText', () => {
  const something = 'abc';

  beforeEach(() => {
    // stuff
  });
});
",
        ),
        (
            "const thing = 123;\nbeforeEach(() => {});",
            "const thing = 123;\n\nbeforeEach(() => {});",
        ),
        (
            "const thing = 123;\n// A comment\nbeforeEach(() => {});",
            "const thing = 123;\n\n// A comment\nbeforeEach(() => {});",
        ),
        ("beforeEach(() => {});\nfoo();", "beforeEach(() => {});\n\nfoo();"),
        (
            "const thing = 123;\nawait beforeEach(() => {});",
            "const thing = 123;\n\nawait beforeEach(() => {});",
        ),
        (
            "{\n  const thing = 123;\n  beforeEach(() => {});\n}",
            "{\n  const thing = 123;\n\n  beforeEach(() => {});\n}",
        ),
        (
            "switch (value) {\n  case 1:\n    beforeEach(() => {});\n    foo();\n}",
            "switch (value) {\n  case 1:\n    beforeEach(() => {});\n\n    foo();\n}",
        ),
        ("const thing = 123\n;beforeEach(() => {})", "const thing = 123\n\n;beforeEach(() => {})"),
        (
            "const thing = 123;\nlabel: beforeEach(() => {});",
            "const thing = 123;\n\nlabel: beforeEach(() => {});",
        ),
    ];

    Tester::new(
        PaddingAroundBeforeEachBlocks::NAME,
        PaddingAroundBeforeEachBlocks::PLUGIN,
        pass,
        fail,
    )
    .with_jest_plugin(true)
    .expect_fix(fix)
    .test_and_snapshot();
}
