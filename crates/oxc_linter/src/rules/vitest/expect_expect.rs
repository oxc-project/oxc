use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::expect_expect::{DOCUMENTATION, ExpectExpectConfig},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct ExpectExpect(Box<ExpectExpectConfig>);

declare_oxc_lint!(
    ExpectExpect,
    vitest,
    correctness,
    config = ExpectExpectConfig,
    docs = DOCUMENTATION
);

impl Rule for ExpectExpect {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        ExpectExpectConfig::from_configuration(&value).map(|config| Self(Box::new(config)))
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        self.0.run_on_jest_node(jest_node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
                import { test } from 'vitest';
                test.skip(\"skipped test\", () => {})
            ",
            None,
        ),
        ("it.todo(\"will test something eventually\")", None),
        ("test.todo(\"will test something eventually\")", None),
        ("['x']();", None),
        ("it(\"should pass\", () => expect(true).toBeDefined())", None),
        ("test(\"should pass\", () => expect(true).toBeDefined())", None),
        ("it(\"should pass\", () => somePromise().then(() => expect(true).toBeDefined()))", None),
        ("it(\"should pass\", myTest); function myTest() { expect(true).toBeDefined() }", None),
        (
            "
                test('should pass', () => {
                    expect(true).toBeDefined();
                    foo(true).toBe(true);
                });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }]))
        ),
        (
            "
                import { bench } from 'vitest'

                bench('normal sorting', () => {
                    const x = [1, 5, 4, 2, 3]
                    x.sort((a, b) => {
                        return a - b
                    })
                }, { time: 1000 })
            ",
            None,
        ),
        (
            "it(\"should return undefined\", () => expectSaga(mySaga).returns());",
            Some(serde_json::json!([{ "assertFunctionNames": ["expectSaga"] }])),
        ),
        (
            "test('verifies expect method call', () => expect$(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect\\$"] }])),
        ),
        (
            "test('verifies expect method call', () => new Foo().expect(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["Foo.expect"] }])),
        ),
        (
            "
                test('verifies deep expect method call', () => {
                    tester.foo().expect(123);
                });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.foo.expect"] }])),
        ),
        (
            "
                    test('verifies chained expect method call', () => {
                        tester
                            .foo()
                            .bar()
                            .expect(456);
                    });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.foo.bar.expect"] }])),
        ),
        (
            "
                test(\"verifies the function call\", () => {
                    td.verify(someFunctionCall())
                })
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["td.verify"] }])),
        ),
        (
            "it(\"should pass\", () => expect(true).toBeDefined())",
            Some(serde_json::json!([{
                "assertFunctionNames": "undefined",
                "additionalTestBlockFunctions": "undefined",
            }])),
        ),
        (
            "
                theoretically('the number {input} is correctly translated to string', theories, theory => {
                    const output = NumberToLongString(theory.input);
                    expect(output).toBe(theory.expected);
                })
            ",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["theoretically"] }])),
        ),
        (
            "test('should pass *', () => expect404ToBeLoaded());",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect*"] }])),
        ),
        (
            "test('should pass *', () => expect.toHaveStatus404());",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect.**"] }])),
        ),
        (
            "test('should pass', () => tester.foo().expect(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.*.expect"] }])),
        ),
        (
            "test('should pass **', () => tester.foo().expect(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["**"] }])),
        ),
        (
            "test('should pass *', () => tester.foo().expect(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["*"] }])),
        ),
        (
            "test('should pass', () => tester.foo().expect(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.**"] }])),
        ),
        (
            "test('should pass', () => tester.foo().expect(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.*"] }])),
        ),
        (
            "test('should pass', () => tester.foo().bar().expectIt(456));",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.**.expect*"] }])),
        ),
        (
            "test('should pass', () => request.get().foo().expect(456));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.**.expect"] }])),
        ),
        (
            "test('should pass', () => request.get().foo().expect(456));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.**.e*e*t"] }])),
        ),
        (
            "
                import { test } from 'vitest';

                test('should pass', () => {
                    expect(true).toBeDefined();
                    foo(true).toBe(true);
                });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        (
            "
                import { test as checkThat } from 'vitest';

                checkThat('this passes', () => {
                    expect(true).toBeDefined();
                    foo(true).toBe(true);
                });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        (
            "
                const { test } = require('vitest');

                test('verifies chained expect method call', () => {
                    tester
                    .foo()
                    .bar()
                    .expect(456);
                });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.foo.bar.expect"] }])),
        ),
        (
            "
                it(\"should pass with 'typecheck' enabled\", () => {
                    expectTypeOf({ a: 1 }).toEqualTypeOf<{ a: number }>()
                });
            ",
            None
        ),
        (
            "
                import { assert, it } from 'vitest';

                it('test', () => {
                    assert.throws(() => {
                        throw Error('Invalid value');
                    });
                });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["assert"] }])),
        ),
        (
            "
                import { expectTypeOf } from 'vitest'

                expectTypeOf({ a: 1 }).toEqualTypeOf<{ a: number }>()
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expectTypeOf"] }])),
        ),
        (
            "
                import { assertType } from 'vitest'

                function concat(a: string, b: string): string
                function concat(a: number, b: number): number
                function concat(a: string | number, b: string | number): string | number

                assertType<string>(concat('a', 'b'))
                assertType<number>(concat(1, 2))
                // @ts-expect-error wrong types
                assertType(concat('a', 2))
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["assertType"] }])),
        ),
    ];

    let fail = vec![
        ("it(\"should fail\", () => {});", None),
        ("it(\"should fail\", myTest); function myTest() {}", None),
        ("test(\"should fail\", () => {});", None),
        (
            "afterEach(() => {});",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["afterEach"] }])),
        ),
        // Todo: currently it's not support
        // (
        //     "
        //         theoretically('the number {input} is correctly translated to string', theories, theory => {
        //             const output = NumberToLongString(theory.input);
        //         })
        //     ",
        //     Some(serde_json::json!([{ "additionalTestBlockFunctions": ["theoretically"] }])),
        // ),
        ("it(\"should fail\", () => { somePromise.then(() => {}); });", None),
        (
            "test(\"should fail\", () => { foo(true).toBe(true); })",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect"] }])),
        ),
        (
            "it(\"should also fail\",() => expectSaga(mySaga).returns());",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect"] }])),
        ),
        (
            "test('should fail', () => request.get().foo().expect(456));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.*.expect"] }])),
        ),
        (
            "test('should fail', () => request.get().foo().bar().expect(456));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.foo**.expect"] }])),
        ),
        (
            "test('should fail', () => tester.request(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.*"] }])),
        ),
        (
            "test('should fail', () => request(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.*"] }])),
        ),
        (
            "test('should fail', () => request(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.**"] }])),
        ),
        (
            "
                import { test as checkThat } from 'vitest';

                checkThat('this passes', () => {
                    // ...
                });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        // Todo: currently we couldn't support ignore the typecheck option.
        // (
        //     "
        //         it(\"should fail without 'typecheck' enabled\", () => {
        //             expectTypeOf({ a: 1 }).toEqualTypeOf<{ a: number }>()
        //         });
        //     ",
        //     None,
        // ),
    ];

    Tester::new(ExpectExpect::NAME, ExpectExpect::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
