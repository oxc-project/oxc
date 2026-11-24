use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    rules::shared::padding_around_expect_groups::{DOCUMENTATION, VITEST_CONFIGS, run},
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundExpectGroups;

declare_oxc_lint!(
    PaddingAroundExpectGroups,
    vitest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "next",
);

impl Rule for PaddingAroundExpectGroups {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Program(_)
            | AstKind::BlockStatement(_)
            | AstKind::FunctionBody(_)
            | AstKind::SwitchCase(_) => run(node, ctx, VITEST_CONFIGS),
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
        r#"
foo();
bar();

const someText = 'abc';
const someObject = {
  one: 1,
  two: 2,
};

test('thing one', () => {
  let abc = 123;

  expect(abc).toEqual(123);
  expect(123).toEqual(abc); // Line comment

  abc = 456;

  expect(abc).toEqual(456);
});

test('thing one', () => {
  const abc = 123;

  expect(abc).toEqual(123);

  const xyz = 987;

  expect(123).toEqual(abc); // Line comment
});

describe('someText', () => {
  describe('some condition', () => {
    test('foo', () => {
      const xyz = 987;

      // Comment
      expect(xyz).toEqual(987);
      expect(1)
        .toEqual(1);
      expect(true).toEqual(true);
    });
  });
});

test('awaited expect', async () => {
  const abc = 123;
  const hasAPromise = () => Promise.resolve('foo');

  await expect(hasAPromise()).resolves.toEqual('foo');
  expect(abc).toEqual(123);

  const efg = 456;

  expect(123).toEqual(abc);
  await expect(hasAPromise()).resolves.toEqual('foo');

  const hij = 789;

  await expect(hasAPromise()).resolves.toEqual('foo');
  await expect(hasAPromise()).resolves.toEqual('foo');

  const somethingElseAsync = () => Promise.resolve('bar');
  await somethingElseAsync();

  await expect(hasAPromise()).resolves.toEqual('foo');
});

test('expectTypeOf test', () => {
  const hoge = 123;

  expectTypeOf(hoge).toBeNumber();
  expectTypeOf(hoge).toBeNumber();

  const foo = "abc";

  // Comment
  expectTypeOf(foo).toBeString();
  expectTypeOf(foo).toBeString();
});
"#,
        "expect(1).toBe(1);",
        "test('foo', () => {\n  expect(1).toBe(1);\n  expect(2).toBe(2);\n});",
        "test('foo', () => {\n  const value = 1;\n\n  expect(value).toBe(1);\n});",
        "expect.assertions(1);\nexpect(1).toBe(1);",
        "const enabled = false;\nif (enabled) expect(1).toBe(1);",
        "expectTypeOf(value).toBeNumber();\nexpectTypeOf(value).toBeString();",
        "label:\nexpect(1).toBe(1);",
    ];

    let fail = vec![
        r#"
foo();
bar();

const someText = 'abc';
const someObject = {
  one: 1,
  two: 2,
};

test('thing one', () => {
  let abc = 123;
  expect(abc).toEqual(123);
  expect(123).toEqual(abc); // Line comment
  abc = 456;
  expect(abc).toEqual(456);
});

test('thing one', () => {
  const abc = 123;
  expect(abc).toEqual(123);

  const xyz = 987;
  expect(123).toEqual(abc); // Line comment
});

describe('someText', () => {
  describe('some condition', () => {
    test('foo', () => {
      const xyz = 987;
      // Comment
      expect(xyz).toEqual(987);
      expect(1)
        .toEqual(1);
      expect(true).toEqual(true);
    });
  });
});

test('awaited expect', async () => {
  const abc = 123;
  const hasAPromise = () => Promise.resolve('foo');
  await expect(hasAPromise()).resolves.toEqual('foo');
  expect(abc).toEqual(123);

  const efg = 456;
  expect(123).toEqual(abc);
  await expect(hasAPromise()).resolves.toEqual('foo');

  const hij = 789;
  await expect(hasAPromise()).resolves.toEqual('foo');
  await expect(hasAPromise()).resolves.toEqual('foo');

  const somethingElseAsync = () => Promise.resolve('bar');
  await somethingElseAsync();
  await expect(hasAPromise()).resolves.toEqual('foo');
});

test('expectTypeOf test', () => {
  const hoge = 123;
  expectTypeOf(hoge).toBeNumber();
  expectTypeOf(hoge).toBeNumber();
  const foo = "abc";
  // Comment
  expectTypeOf(foo).toBeString();
  expectTypeOf(foo).toBeString();
});
"#,
        "const value = 1;\nexpect(value).toBe(1);",
        "expect(value).toBe(1);\nfoo();",
        "const value = 1;\nexpect.soft(value).toBe(1);",
        "const value = 1;\nawait expect(promise).resolves.toBe(1);",
        "expect(value).toBe(1);\nexpectTypeOf(value).toBeNumber();",
        "{\n  const value = 1;\n  expect(value).toBe(1);\n}",
        "switch (value) {\n  case 1:\n    expect(value).toBe(1);\n    foo();\n}",
        "const value = 1\n;expect(value).toBe(1)",
        "const value = 1;\nlabel: expect(value).toBe(1);",
        "const value = 1;\nexpectTypeOf(value).toBeNumber();",
        "expectTypeOf(value).toBeNumber();\nexpect(value).toBe(1);",
    ];

    let fix = vec![
        (
            r#"
foo();
bar();

const someText = 'abc';
const someObject = {
  one: 1,
  two: 2,
};

test('thing one', () => {
  let abc = 123;
  expect(abc).toEqual(123);
  expect(123).toEqual(abc); // Line comment
  abc = 456;
  expect(abc).toEqual(456);
});

test('thing one', () => {
  const abc = 123;
  expect(abc).toEqual(123);

  const xyz = 987;
  expect(123).toEqual(abc); // Line comment
});

describe('someText', () => {
  describe('some condition', () => {
    test('foo', () => {
      const xyz = 987;
      // Comment
      expect(xyz).toEqual(987);
      expect(1)
        .toEqual(1);
      expect(true).toEqual(true);
    });
  });
});

test('awaited expect', async () => {
  const abc = 123;
  const hasAPromise = () => Promise.resolve('foo');
  await expect(hasAPromise()).resolves.toEqual('foo');
  expect(abc).toEqual(123);

  const efg = 456;
  expect(123).toEqual(abc);
  await expect(hasAPromise()).resolves.toEqual('foo');

  const hij = 789;
  await expect(hasAPromise()).resolves.toEqual('foo');
  await expect(hasAPromise()).resolves.toEqual('foo');

  const somethingElseAsync = () => Promise.resolve('bar');
  await somethingElseAsync();
  await expect(hasAPromise()).resolves.toEqual('foo');
});

test('expectTypeOf test', () => {
  const hoge = 123;
  expectTypeOf(hoge).toBeNumber();
  expectTypeOf(hoge).toBeNumber();
  const foo = "abc";
  // Comment
  expectTypeOf(foo).toBeString();
  expectTypeOf(foo).toBeString();
});
"#,
            r#"
foo();
bar();

const someText = 'abc';
const someObject = {
  one: 1,
  two: 2,
};

test('thing one', () => {
  let abc = 123;

  expect(abc).toEqual(123);
  expect(123).toEqual(abc); // Line comment

  abc = 456;

  expect(abc).toEqual(456);
});

test('thing one', () => {
  const abc = 123;

  expect(abc).toEqual(123);

  const xyz = 987;

  expect(123).toEqual(abc); // Line comment
});

describe('someText', () => {
  describe('some condition', () => {
    test('foo', () => {
      const xyz = 987;

      // Comment
      expect(xyz).toEqual(987);
      expect(1)
        .toEqual(1);
      expect(true).toEqual(true);
    });
  });
});

test('awaited expect', async () => {
  const abc = 123;
  const hasAPromise = () => Promise.resolve('foo');

  await expect(hasAPromise()).resolves.toEqual('foo');
  expect(abc).toEqual(123);

  const efg = 456;

  expect(123).toEqual(abc);
  await expect(hasAPromise()).resolves.toEqual('foo');

  const hij = 789;

  await expect(hasAPromise()).resolves.toEqual('foo');
  await expect(hasAPromise()).resolves.toEqual('foo');

  const somethingElseAsync = () => Promise.resolve('bar');
  await somethingElseAsync();

  await expect(hasAPromise()).resolves.toEqual('foo');
});

test('expectTypeOf test', () => {
  const hoge = 123;

  expectTypeOf(hoge).toBeNumber();
  expectTypeOf(hoge).toBeNumber();

  const foo = "abc";

  // Comment
  expectTypeOf(foo).toBeString();
  expectTypeOf(foo).toBeString();
});
"#,
        ),
        ("const value = 1;\nexpect(value).toBe(1);", "const value = 1;\n\nexpect(value).toBe(1);"),
        ("expect(value).toBe(1);\nfoo();", "expect(value).toBe(1);\n\nfoo();"),
        (
            "const value = 1;\nexpect.soft(value).toBe(1);",
            "const value = 1;\n\nexpect.soft(value).toBe(1);",
        ),
        (
            "const value = 1;\nawait expect(promise).resolves.toBe(1);",
            "const value = 1;\n\nawait expect(promise).resolves.toBe(1);",
        ),
        (
            "expect(value).toBe(1);\nexpectTypeOf(value).toBeNumber();",
            "expect(value).toBe(1);\n\nexpectTypeOf(value).toBeNumber();",
        ),
        (
            "{\n  const value = 1;\n  expect(value).toBe(1);\n}",
            "{\n  const value = 1;\n\n  expect(value).toBe(1);\n}",
        ),
        (
            "switch (value) {\n  case 1:\n    expect(value).toBe(1);\n    foo();\n}",
            "switch (value) {\n  case 1:\n    expect(value).toBe(1);\n\n    foo();\n}",
        ),
        ("const value = 1\n;expect(value).toBe(1)", "const value = 1\n\n;expect(value).toBe(1)"),
        (
            "const value = 1;\nlabel: expect(value).toBe(1);",
            "const value = 1;\n\nlabel: expect(value).toBe(1);",
        ),
        (
            "const value = 1;\nexpectTypeOf(value).toBeNumber();",
            "const value = 1;\n\nexpectTypeOf(value).toBeNumber();",
        ),
        (
            "expectTypeOf(value).toBeNumber();\nexpect(value).toBe(1);",
            "expectTypeOf(value).toBeNumber();\n\nexpect(value).toBe(1);",
        ),
    ];

    Tester::new(PaddingAroundExpectGroups::NAME, PaddingAroundExpectGroups::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
