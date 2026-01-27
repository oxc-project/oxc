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
    jest,
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
        ("it.todo('will test something eventually')", None),
        ("test.todo('will test something eventually')", None),
        ("['x']();", None),
        ("it('should pass', () => expect(true).toBeDefined())", None),
        ("test('should pass', () => expect(true).toBeDefined())", None),
        ("it('should pass', () => somePromise().then(() => expect(true).toBeDefined()))", None),
        ("it('should pass', myTest); function myTest() { expect(true).toBeDefined() }", None),
        (
            "
            test('should pass', () => {
                expect(true).toBeDefined();
                foo(true).toBe(true);
            });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        ("it('should return undefined',() => expectSaga(mySaga).returns());", Some(serde_json::json!([{ "assertFunctionNames": ["expectSaga"] }]))),
        ("test('verifies expect method call', () => expect$(123));", Some(serde_json::json!([{ "assertFunctionNames": ["expect\\$"] }]))),
        ("test('verifies expect method call', () => new Foo().expect(123));", Some(serde_json::json!([{ "assertFunctionNames": ["Foo.expect"] }]))),
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
        	test('verifies the function call', () => {
        	td.verify(someFunctionCall())
        	})
        ",
            Some(serde_json::json!([{ "assertFunctionNames": ["td.verify"] }])),
        ),
        (
            "it('should pass', () => expect(true).toBeDefined())",
            Some(serde_json::json!([
                {
                "assertFunctionNames": "undefined",
                "additionalTestBlockFunctions": "undefined",
                },
            ])),
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
        ("test('should pass *', () => expect404ToBeLoaded());", Some(serde_json::json!([{ "assertFunctionNames": ["expect*"] }]))),
        ("test('should pass *', () => expect.toHaveStatus404());", Some(serde_json::json!([{ "assertFunctionNames": ["expect.**"] }]))),
        ("test('should pass', () => tester.foo().expect(123));", Some(serde_json::json!([{ "assertFunctionNames": ["tester.*.expect"] }]))),
        ("test('should pass **', () => tester.foo().expect(123));", Some(serde_json::json!([{ "assertFunctionNames": ["**"] }]))),
        ("test('should pass *', () => tester.foo().expect(123));", Some(serde_json::json!([{ "assertFunctionNames": ["*"] }]))),
        ("test('should pass', () => tester.foo().expect(123));", Some(serde_json::json!([{ "assertFunctionNames": ["tester.**"] }]))),
        ("test('should pass', () => tester.foo().expect(123));", Some(serde_json::json!([{ "assertFunctionNames": ["tester.*"] }]))),
        ("test('should pass', () => tester.foo().bar().expectIt(456));", Some(serde_json::json!([{ "assertFunctionNames": ["tester.**.expect*"] }]))),
        ("test('should pass', () => request.get().foo().expect(456));", Some(serde_json::json!([{ "assertFunctionNames": ["request.**.expect"] }]))),
        ("test('should pass', () => request.get().foo().expect(456));", Some(serde_json::json!([{ "assertFunctionNames": ["request.**.e*e*t"] }]))),
        (
            "
        	import { test } from '@jest/globals';

        	test('should pass', () => {
        	expect(true).toBeDefined();
        	foo(true).toBe(true);
        	});
        ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        (
            "
        	import { test as checkThat } from '@jest/globals';

        	checkThat('this passes', () => {
        	expect(true).toBeDefined();
        	foo(true).toBe(true);
        	});
        ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        (
            "
        	const { test } = require('@jest/globals');

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
            r#"
            it("should not warn on await expect", async () => {
                const asyncFunction = async () => {
                    throw new Error('nope')
                };
                await expect(asyncFunction()).rejects.toThrow();
            });
            "#,
            None,
        ),
        (
            r"
            it('should not warn on expect in Promise.all', async () => {
                await Promise.all([
                    expect(new Promise((resolve) => { resolve(1); }) ).resolves.toBe(1),
                    expect(new Promise((_, reject) => { reject(new Error('Failed')); })).rejects.toThrowError('Failed'),
                ]);
            });
            ",
            None,
        ),
        (
            r#"
            it("should not warn on await expect", async () => {
                if(true) {
                    const asyncFunction = async () => {
                        throw new Error('nope')
                    };
                    await expect(asyncFunction()).rejects.toThrow();
                }
            });
            "#,
            None,
        ),
        (
            r#"
            it("should not warn on await expect", async () => {
                {
                    const asyncFunction = async () => {
                        throw new Error('nope')
                    };
                    await expect(asyncFunction()).rejects.toThrow();
                }
            });
            "#,
            None,
        ),
        ("it('test', async () => { const array = [1]; for (const element of array) { expect(element).toBe(1); } });", None),
        (r"it('msg', async () => { const r = foo(); return expect(r).rejects.toThrow(); });", None),
    ];

    let fail = vec![
        ("it(\"should fail\", () => {});", None),
        ("it(\"should fail\", myTest); function myTest() {}", None),
        ("test(\"should fail\", () => {});", None),
        ("test.skip(\"should fail\", () => {});", None),
        (
            "afterEach(() => {});",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["afterEach"] }])),
        ),
        // TODO: is this case usual? not support this now, which need visit all call expression and get it's node name
        // (
        //     "
        // 	theoretically('the number {input} is correctly translated to string', theories, theory => {
        // 	const output = NumberToLongString(theory.input);
        // 	})
        // ",
        //     Some(serde_json::json!([{ "additionalTestBlockFunctions": ["theoretically"] }])),
        // ),
        (r#"it("should fail", () => { somePromise.then(() => {}); });"#, None),
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
        	import { test as checkThat } from '@jest/globals';

        	checkThat('this passes', () => {
        	// ...
        	});
        ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        (
            "
        	import { test as checkThat } from '@jest/globals';

        	checkThat.skip('this passes', () => {
        	// ...
        	});
        ",
            None,
        ),
        (
            r#"
            it("should warn on non-assert await expression", async () => {
                const asyncFunction = async () => {
                    throw new Error('nope')
                };
                await foo(asyncFunction()).rejects.toThrow();
            });
            "#,
            None,
        ),
        (
            r#"
            test("event emitters bound to CLS context", function(t) {
                t.test("emitter with newListener that removes handler", function(t) {
                    ee.on("newListener", function handler(event: any) {
                        this.removeListener("newListener", handler);
                    });
                });
            });
            "#,
            None,
        ),
    ];

    Tester::new(ExpectExpect::NAME, ExpectExpect::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
