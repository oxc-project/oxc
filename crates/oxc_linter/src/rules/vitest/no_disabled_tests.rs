use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::no_disabled_tests::{DOCUMENTATION, run_on_jest_node},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct NoDisabledTests;

declare_oxc_lint!(NoDisabledTests, vitest, correctness, docs = DOCUMENTATION, version = "0.0.7",);

impl Rule for NoDisabledTests {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run_on_jest_node(jest_node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        ("describe('foo', function () {})", None),
        ("it('foo', function () {})", None),
        ("describe.only('foo', function () {})", None),
        ("it.only('foo', function () {})", None),
        ("it.each('foo', () => {})", None),
        ("it.concurrent('foo', function () {})", None),
        ("test('foo', function () {})", None),
        ("test.only('foo', function () {})", None),
        ("test.concurrent('foo', function () {})", None),
        ("describe[`${'skip'}`]('foo', function () {})", None),
        ("it.todo('fill this later')", None),
        ("var appliedSkip = describe.skip; appliedSkip.apply(describe)", None),
        ("var calledSkip = it.skip; calledSkip.call(it)", None),
        ("({ f: function () {} }).f()", None),
        ("(a || b).f()", None),
        ("itHappensToStartWithIt()", None),
        ("testSomething()", None),
        ("xdescribe.each([1, 2, 3])('%s', (a, b) => {});", None),
        ("xdescribe('foo', function () {})", None),
        ("xit('foo', function () {})", None),
        ("xtest('foo', function () {})", None),
        ("xit.each``('foo', function () {})", None),
        ("xtest.each``('foo', function () {})", None),
        ("xit.each([])('foo', function () {})", None),
        ("xtest.each([])('foo', function () {})", None),
        ("xitSomethingElse()", None),
        ("xitiViewMap()", None),
        (
            "import { pending } from 'actions'; test('foo', () => { expect(pending()).toEqual({}) })",
            None,
        ),
        (
            "const { pending } = require('actions'); test('foo', () => { expect(pending()).toEqual({}) })",
            None,
        ),
        (
            "test('foo', () => { const pending = getPending(); expect(pending()).toEqual({}) })",
            None,
        ),
        (
            "test('foo', () => { expect(pending()).toEqual({}) }); function pending() { return {} }",
            None,
        ),
        ("import { test } from './test-utils'; test('something');", None),
    ];

    pass.extend(
        [
            r#"describe("foo", function () {})"#,
            r#"it("foo", function () {})"#,
            r#"describe.only("foo", function () {})"#,
            r#"it.only("foo", function () {})"#,
            r#"it.each("foo", () => {})"#,
            r#"it.concurrent("foo", function () {})"#,
            r#"test("foo", function () {})"#,
            r#"test.only("foo", function () {})"#,
            r#"test.concurrent("foo", function () {})"#,
            r#"describe[`${"skip"}`]("foo", function () {})"#,
            r#"it.todo("fill this later")"#,
            "var appliedSkip = describe.skip; appliedSkip.apply(describe)",
            "var calledSkip = it.skip; calledSkip.call(it)",
            "({ f: function () {} }).f()",
            "(a || b).f()",
            "itHappensToStartWithIt()",
            "testSomething()",
            r#"xtest("foo", function () {})"#,
            r#"xit.each``("foo", function () {})"#,
            r#"xtest.each``("foo", function () {})"#,
            r#"xit.each([])("foo", function () {})"#,
            "xitSomethingElse()",
            "xitiViewMap()",
            r#"
            import { pending } from "actions"
            test("foo", () => {
              expect(pending()).toEqual({})
            })
        "#,
            "
            import { test } from './test-utils';
	    test('something');
        ",
            "import {describe, expect, test} from 'vitest';

                describe('example', () => {
                  const it = test.extend<{ result: number }>({
                    result: async ({}, use) => {
                      await use(42);
                    },
                  });

                  it('works', ({ result }) => {
                    expect(result).toBe(42);
                  });
                });

                ",
        ]
        .into_iter()
        .map(|x| (x, None)),
    );

    let mut fail = vec![
        ("describe.skip('foo', function () {})", None),
        ("describe.skip.each([1, 2, 3])('%s', (a, b) => {});", None),
        ("describe[`skip`]('foo', function () {})", None),
        ("describe['skip']('foo', function () {})", None),
        ("it.skip('foo', function () {})", None),
        ("it['skip']('foo', function () {})", None),
        ("test.skip('foo', function () {})", None),
        ("it.skip.each``('foo', function () {})", None),
        ("test.skip.each``('foo', function () {})", None),
        ("it.skip.each([])('foo', function () {})", None),
        ("test.skip.each([])('foo', function () {})", None),
        ("test['skip']('foo', function () {})", None),
        ("it('has title but no callback')", None),
        ("test('has title but no callback')", None),
        ("it('contains a call to pending', function () { pending() })", None),
        ("pending()", None),
        ("describe('contains a call to pending', function () { pending() })", None),
        ("import { test } from '@jest/globals';test('something');", None),
    ];

    fail.extend(
        [
            r#"describe.skip("foo", function () {})"#,
            r#"it("has title but no callback")"#,
            r#"test("has title but no callback")"#,
            r#"it("contains a call to pending", function () { pending() })"#,
            "pending();",
            r#"
            import { describe } from 'vitest';
            describe.skip("foo", function () {})
        "#,
        ]
        .into_iter()
        .map(|x| (x, None)),
    );

    Tester::new(NoDisabledTests::NAME, NoDisabledTests::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
