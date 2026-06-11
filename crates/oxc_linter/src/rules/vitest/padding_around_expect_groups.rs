use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::padding_around_expect_groups::{DOCUMENTATION, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundExpectGroups;

declare_oxc_lint!(
    PaddingAroundExpectGroups,
    vitest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "1.68.0",
);

impl Rule for PaddingAroundExpectGroups {
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

    const VALID: &str = r#"
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
"#;

    const INVALID: &str = r#"
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
"#;

    let pass = vec![
        "expect(1).toBe(1);",
        "const thing = 123;\n\nexpect(thing).toBe(123);",
        "expect(a).toBe(1);\nexpect(b).toBe(2);",
        "expectTypeOf(1).toBeNumber();\nexpectTypeOf(2).toBeNumber();",
        "import { expect, test } from 'vitest';\n\ntest('foo', () => {\nexpect(1).toBe(1);\n});",
        "test('foo', async () => {\nconst a = 1;\n\nawait expect(Promise.resolve(a)).resolves.toBe(1);\nexpect(a).toBe(1);\n});",
        VALID,
    ];

    let fail = vec![
        "const thing = 123;\nexpect(thing).toBe(123);",
        "expect(thing).toBe(123);\nconst other = 456;",
        "const thing = 123;\nexpectTypeOf(thing).toBeNumber();",
        // expect and expectTypeOf are distinct groups
        "test('foo', () => {\nconst a = 1;\n\nexpect(a).toBe(1);\nexpectTypeOf(a).toBeNumber();\n});",
        INVALID,
    ];

    let fix = vec![
        (
            "const thing = 123;\nexpect(thing).toBe(123);",
            "const thing = 123;\n\nexpect(thing).toBe(123);",
        ),
        (
            "expect(thing).toBe(123);\nconst other = 456;",
            "expect(thing).toBe(123);\n\nconst other = 456;",
        ),
        (
            "const thing = 123;\nexpectTypeOf(thing).toBeNumber();",
            "const thing = 123;\n\nexpectTypeOf(thing).toBeNumber();",
        ),
        (
            "test('foo', () => {\nconst a = 1;\n\nexpect(a).toBe(1);\nexpectTypeOf(a).toBeNumber();\n});",
            "test('foo', () => {\nconst a = 1;\n\nexpect(a).toBe(1);\n\nexpectTypeOf(a).toBeNumber();\n});",
        ),
        (INVALID, VALID),
    ];

    Tester::new(PaddingAroundExpectGroups::NAME, PaddingAroundExpectGroups::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
