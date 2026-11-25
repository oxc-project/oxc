use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    rules::shared::padding_around_after_all_blocks::{DOCUMENTATION, run},
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundAfterAllBlocks;

declare_oxc_lint!(
    PaddingAroundAfterAllBlocks,
    vitest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "1.66.0",
);

impl Rule for PaddingAroundAfterAllBlocks {
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
        "afterAll(() => {});",
        "const thing = 123;\n\nafterAll(() => {});",
        "describe('foo', () => {\nafterAll(() => {});\n});",
        "// This is a comment\nafterAll(() => {});",
        "import { afterAll } from 'vitest';\n\nafterAll(() => {});",
        "import { afterAll, describe } from 'vitest';\nimport { helper } from './helper';\n\nafterAll(() => {});",
        "import './setup';\n\nafterAll(() => {});",
        "/* leading block comment */\nafterAll(() => {});",
        "/**\n * JSDoc-style comment\n */\nafterAll(() => {});",
        "const thing = 123;\n\n/* attached to afterAll */\nafterAll(() => {});",
        "const thing = 123;\n\n/**\n * JSDoc attached to afterAll\n */\nafterAll(() => {});",
        "const thing = 123; /* trailing on prev */\n\nafterAll(() => {});",
        "describe('foo', () => {\nafterAll(() => {});\n\nafterAll(() => {});\n});",
        "const cleanUp = false; cleanUp && afterAll(() => {});",
        "const cleanUp = false;\nif (cleanUp)\n  afterAll(() => {});",
        "const cleanUp = false; if (cleanUp) afterAll(() => {});",
        "const cleanUp = false; if (cleanUp)\n  afterAll(() => {});",
        "const cleanUp = false; while (cleanUp) afterAll(() => {});",
        "const cleanUp = false; for (; cleanUp;) afterAll(() => {});",
        r"
const someText = 'abc';

afterAll(() => {
});

describe('someText', () => {
  const something = 'abc';

  // A comment
  afterAll(() => {
    // stuff
  });

  afterAll(() => {
    // other stuff
  });
});

describe('someText', () => {
  const something = 'abc';

  afterAll(() => {
    // stuff
  });
});
",
        "afterAll(() => {});\n\nfoo();",
        "label:\nafterAll(() => {});",
    ];

    let fail = vec![
        "const thing = 123;\nafterAll(() => {});",
        "const thing = 123;\n//My comment\nafterAll(() => {});",
        "import { afterAll } from 'vitest';\nafterAll(() => {});",
        "import { afterAll } from 'vitest';\nimport { helper } from './helper';\nafterAll(() => {});",
        "import './setup';\nafterAll(() => {});",
        "const thing = 123;\n/* block comment */\nafterAll(() => {});",
        "const thing = 123;\n/**\n * JSDoc comment\n */\nafterAll(() => {});",
        "describe('foo', () => {\nafterAll(() => {});\nafterAll(() => {});\n});",
        "import { afterAll } from 'vitest';\n/* setup notes */\nafterAll(() => {});",
        r"
const someText = 'abc';
afterAll(() => {
});
describe('someText', () => {
  const something = 'abc';
  // A comment
  afterAll(() => {
    // stuff
  });
  afterAll(() => {
    // other stuff
  });
});

describe('someText', () => {
  const something = 'abc';
  afterAll(() => {
    // stuff
  });
});
",
        "afterAll(() => {});\nfoo();",
        "{\n  const thing = 123;\n  afterAll(() => {});\n}",
        "switch (value) {\n  case 1:\n    afterAll(() => {});\n    foo();\n}",
        "const thing = 123\n;afterAll(() => {})",
        "const thing = 123;\nlabel: afterAll(() => {});",
    ];

    let fix = vec![
        ("const thing = 123;\nafterAll(() => {});", "const thing = 123;\n\nafterAll(() => {});"),
        (
            "const thing = 123;\n// This is a comment\nafterAll(() => {});",
            "const thing = 123;\n\n// This is a comment\nafterAll(() => {});",
        ),
        (
            "import { afterAll } from 'vitest';\nafterAll(() => {});",
            "import { afterAll } from 'vitest';\n\nafterAll(() => {});",
        ),
        ("import './setup';\nafterAll(() => {});", "import './setup';\n\nafterAll(() => {});"),
        (
            "const thing = 123;\n/* block comment */\nafterAll(() => {});",
            "const thing = 123;\n\n/* block comment */\nafterAll(() => {});",
        ),
        (
            "const thing = 123;\n/**\n * JSDoc comment\n */\nafterAll(() => {});",
            "const thing = 123;\n\n/**\n * JSDoc comment\n */\nafterAll(() => {});",
        ),
        (
            "describe('foo', () => {\nafterAll(() => {});\nafterAll(() => {});\n});",
            "describe('foo', () => {\nafterAll(() => {});\n\nafterAll(() => {});\n});",
        ),
        (
            "const thing = 123; afterAll(() => {}), foo();",
            "const thing = 123;\n\nafterAll(() => {}), foo();",
        ),
        (
            "const thing = 123; afterAll(() => {}) && foo();",
            "const thing = 123;\n\nafterAll(() => {}) && foo();",
        ),
        (
            "const thing = 123; await afterAll(() => {});",
            "const thing = 123;\n\nawait afterAll(() => {});",
        ),
        (
            "const thing = 123;\nawait afterAll(() => {});",
            "const thing = 123;\n\nawait afterAll(() => {});",
        ),
        (
            "const thing = 123; await /* hook */ afterAll(() => {});",
            "const thing = 123;\n\nawait /* hook */ afterAll(() => {});",
        ),
        (
            r"
const someText = 'abc';
afterAll(() => {
});
describe('someText', () => {
  const something = 'abc';
  // A comment
  afterAll(() => {
    // stuff
  });
  afterAll(() => {
    // other stuff
  });
});

describe('someText', () => {
  const something = 'abc';
  afterAll(() => {
    // stuff
  });
});
",
            r"
const someText = 'abc';

afterAll(() => {
});

describe('someText', () => {
  const something = 'abc';

  // A comment
  afterAll(() => {
    // stuff
  });

  afterAll(() => {
    // other stuff
  });
});

describe('someText', () => {
  const something = 'abc';

  afterAll(() => {
    // stuff
  });
});
",
        ),
        ("afterAll(() => {});\nfoo();", "afterAll(() => {});\n\nfoo();"),
        (
            "{\n  const thing = 123;\n  afterAll(() => {});\n}",
            "{\n  const thing = 123;\n\n  afterAll(() => {});\n}",
        ),
        (
            "switch (value) {\n  case 1:\n    afterAll(() => {});\n    foo();\n}",
            "switch (value) {\n  case 1:\n    afterAll(() => {});\n\n    foo();\n}",
        ),
        ("const thing = 123\n;afterAll(() => {})", "const thing = 123\n\n;afterAll(() => {})"),
        (
            "const thing = 123;\nlabel: afterAll(() => {});",
            "const thing = 123;\n\nlabel: afterAll(() => {});",
        ),
    ];

    Tester::new(PaddingAroundAfterAllBlocks::NAME, PaddingAroundAfterAllBlocks::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
