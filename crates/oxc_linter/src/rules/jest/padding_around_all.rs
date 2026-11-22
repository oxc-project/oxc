use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    rules::shared::padding_around_all::{DOCUMENTATION, JEST_CONFIGS, run},
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundAll;

declare_oxc_lint!(PaddingAroundAll, jest, style, fix, docs = DOCUMENTATION, version = "next",);

impl Rule for PaddingAroundAll {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Program(_)
            | AstKind::BlockStatement(_)
            | AstKind::FunctionBody(_)
            | AstKind::SwitchCase(_) => run(node, ctx, JEST_CONFIGS),
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
        "xyz:\nafterEach(() => {});",
        "expect(a).toBe(1);\nexpect(b).toBe(2);",
        "expectTypeOf(a).toBeNumber();\nexpectTypeOf(b).toBeString();",
        r"const thing = 123;

beforeEach(() => {});

test('foo', () => {
  const value = 1;

  expect(value).toBe(1);
});",
        "const value = 1;\nexpectTypeOf(value).toBeNumber();",
    ];

    let fail = vec![
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
        "const someText = 'abc'\n;afterEach(() => {})",
        "const someText = 'abc';\nxyz:\nafterEach(() => {});",
        r#"const expr = 'Papayas';
beforeEach(() => {});
it('does something?', () => {
  switch (expr) {
    case 'Oranges':
      expect(expr).toBe('Oranges');
      break;
    case 'Mangoes':
    case 'Papayas':
      const v = 1;
      expect(v).toBe(1);
      console.log('Mangoes and papayas are $2.79 a pound.');
      // Expected output: "Mangoes and papayas are $2.79 a pound."
      break;
    default:
      console.log(`Sorry, we are out of ${expr}.`);
  }
});"#,
        r"const thing = 123;
beforeEach(() => {});
test('foo', () => {
  const value = 1;
  expect(value).toBe(1);
});",
        "describe('a', () => {});\ntest('b', () => {});",
        "beforeEach(() => {});\nafterEach(() => {});",
        "test('a', () => {});\nexpect(1).toBe(1);",
        "expect(value).toBe(1);\nexpectTypeOf(value).toBeNumber();",
    ];

    let fix = vec![
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
        (
            "const someText = 'abc'\n;afterEach(() => {})",
            "const someText = 'abc'\n\n;afterEach(() => {})",
        ),
        (
            "const someText = 'abc';\nxyz:\nafterEach(() => {});",
            "const someText = 'abc';\n\nxyz:\nafterEach(() => {});",
        ),
        (
            r#"const expr = 'Papayas';
beforeEach(() => {});
it('does something?', () => {
  switch (expr) {
    case 'Oranges':
      expect(expr).toBe('Oranges');
      break;
    case 'Mangoes':
    case 'Papayas':
      const v = 1;
      expect(v).toBe(1);
      console.log('Mangoes and papayas are $2.79 a pound.');
      // Expected output: "Mangoes and papayas are $2.79 a pound."
      break;
    default:
      console.log(`Sorry, we are out of ${expr}.`);
  }
});"#,
            r#"const expr = 'Papayas';

beforeEach(() => {});

it('does something?', () => {
  switch (expr) {
    case 'Oranges':
      expect(expr).toBe('Oranges');

      break;
    case 'Mangoes':
    case 'Papayas':
      const v = 1;

      expect(v).toBe(1);

      console.log('Mangoes and papayas are $2.79 a pound.');
      // Expected output: "Mangoes and papayas are $2.79 a pound."
      break;
    default:
      console.log(`Sorry, we are out of ${expr}.`);
  }
});"#,
        ),
        (
            r"const thing = 123;
beforeEach(() => {});
test('foo', () => {
  const value = 1;
  expect(value).toBe(1);
});",
            r"const thing = 123;

beforeEach(() => {});

test('foo', () => {
  const value = 1;

  expect(value).toBe(1);
});",
        ),
        (
            "describe('a', () => {});\ntest('b', () => {});",
            "describe('a', () => {});\n\ntest('b', () => {});",
        ),
        (
            "beforeEach(() => {});\nafterEach(() => {});",
            "beforeEach(() => {});\n\nafterEach(() => {});",
        ),
        ("test('a', () => {});\nexpect(1).toBe(1);", "test('a', () => {});\n\nexpect(1).toBe(1);"),
        (
            "expect(value).toBe(1);\nexpectTypeOf(value).toBeNumber();",
            "expect(value).toBe(1);\n\nexpectTypeOf(value).toBeNumber();",
        ),
    ];

    Tester::new(PaddingAroundAll::NAME, PaddingAroundAll::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
