use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_to_be::{DOCUMENTATION, run_on_jest_node},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PreferToBe;

declare_oxc_lint!(PreferToBe, jest, style, fix, docs = DOCUMENTATION, version = "0.2.14",);

impl Rule for PreferToBe {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run_on_jest_node(jest_node, ctx);
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    let pass = vec![
        ("expect(null).toBeNull();", None),
        ("expect(null).not.toBeNull();", None),
        ("expect(null).toBe(1);", None),
        ("expect(null).toBe(-1);", None),
        ("expect(null).toBe(...1);", None),
        ("expect(obj).toStrictEqual([ x, 1 ]);", None),
        ("expect(obj).toStrictEqual({ x: 1 });", None),
        ("expect(obj).not.toStrictEqual({ x: 1 });", None),
        ("expect(value).toMatchSnapshot();", None),
        ("expect(catchError()).toStrictEqual({ message: 'oh noes!' })", None),
        ("expect(\"something\");", None),
        ("expect(token).toStrictEqual(/[abc]+/g);", None),
        ("expect(token).toStrictEqual(new RegExp('[abc]+', 'g'));", None),
        ("expect(value).toEqual(dedent`my string`);", None),
        ("expect(0.1 + 0.2).toEqual(0.3);", None),
        // null
        ("expect(null).toBeNull();", None),
        ("expect(null).not.toBeNull();", None),
        ("expect(null).toBe(1);", None),
        ("expect(obj).toStrictEqual([ x, 1 ]);", None),
        ("expect(obj).toStrictEqual({ x: 1 });", None),
        ("expect(obj).not.toStrictEqual({ x: 1 });", None),
        ("expect(value).toMatchSnapshot();", None),
        ("expect(catchError()).toStrictEqual({ message: 'oh noes!' })", None),
        ("expect(\"something\");", None),
        ("expect(null).not.toEqual();", None),
        ("expect(null).toBe();", None),
        ("expect(null).toMatchSnapshot();", None),
        ("expect(\"a string\").toMatchSnapshot(null);", None),
        ("expect(\"a string\").not.toMatchSnapshot();", None),
        ("expect(null).toBe", None),
        // undefined
        ("expect(undefined).toBeUndefined();", None),
        ("expect(true).toBeDefined();", None),
        ("expect({}).toEqual({});", None),
        ("expect(something).toBe()", None),
        ("expect(something).toBe(somethingElse)", None),
        ("expect(something).toEqual(somethingElse)", None),
        ("expect(something).not.toBe(somethingElse)", None),
        ("expect(something).not.toEqual(somethingElse)", None),
        ("expect(undefined).toBe", None),
        ("expect(\"something\");", None),
        // NaN
        ("expect(NaN).toBeNaN();", None),
        ("expect(true).not.toBeNaN();", None),
        ("expect({}).toEqual({});", None),
        ("expect(something).toBe()", None),
        ("expect(something).toBe(somethingElse)", None),
        ("expect(something).toEqual(somethingElse)", None),
        ("expect(something).not.toBe(somethingElse)", None),
        ("expect(something).not.toEqual(somethingElse)", None),
        ("expect(undefined).toBe", None),
        ("expect(\"something\");", None),
        // undefined vs defined
        ("expect(NaN).toBeNaN();", None),
        ("expect(true).not.toBeNaN();", None),
        ("expect({}).toEqual({});", None),
        ("expect(something).toBe()", None),
        ("expect(something).toBe(somethingElse)", None),
        ("expect(something).toEqual(somethingElse)", None),
        ("expect(something).not.toBe(somethingElse)", None),
        ("expect(something).not.toEqual(somethingElse)", None),
        ("expect(undefined).toBe", None),
        ("expect(\"something\");", None),
        // typescript edition
        (
            "(expect('Model must be bound to an array if the multiple property is true') as any).toHaveBeenTipped()",
            None,
        ),
    ];

    let fail = vec![
        ("expect(value).toEqual(\"my string\");", None),
        ("expect(value).toStrictEqual(\"my string\");", None),
        ("expect(value).toStrictEqual(1);", None),
        ("expect(value).toStrictEqual(1,);", None),
        ("expect(value).toStrictEqual(-1);", None),
        ("expect(value).toEqual(`my string`);", None),
        ("expect(value)[\"toEqual\"](`my string`);", None),
        ("expect(value).toStrictEqual(`my ${string}`);", None),
        ("expect(loadMessage()).resolves.toStrictEqual(\"hello world\");", None),
        ("expect(loadMessage()).resolves[\"toStrictEqual\"](\"hello world\");", None),
        ("expect(loadMessage())[\"resolves\"].toStrictEqual(\"hello world\");", None),
        ("expect(loadMessage()).resolves.toStrictEqual(false);", None),
        // null
        ("expect(null).toBe(null);", None),
        ("expect(null).toEqual(null);", None),
        ("expect(null).toEqual(null,);", None),
        ("expect(null).toStrictEqual(null);", None),
        ("expect(\"a string\").not.toBe(null);", None),
        ("expect(\"a string\").not[\"toBe\"](null);", None),
        ("expect(\"a string\")[\"not\"][\"toBe\"](null);", None),
        ("expect(\"a string\").not.toEqual(null);", None),
        ("expect(\"a string\").not.toStrictEqual(null);", None),
        // undefined
        ("expect(undefined).toBe(undefined);", None),
        ("expect(undefined).toEqual(undefined);", None),
        ("expect(undefined).toStrictEqual(undefined);", None),
        ("expect(\"a string\").not.toBe(undefined);", None),
        ("expect(\"a string\").rejects.not.toBe(undefined);", None),
        ("expect(\"a string\").rejects.not[\"toBe\"](undefined);", None),
        ("expect(\"a string\").not.toEqual(undefined);", None),
        ("expect(\"a string\").not.toStrictEqual(undefined);", None),
        // NaN
        ("expect(NaN).toBe(NaN);", None),
        ("expect(NaN).toEqual(NaN);", None),
        ("expect(NaN).toStrictEqual(NaN);", None),
        ("expect(\"a string\").not.toBe(NaN);", None),
        ("expect(\"a string\").rejects.not.toBe(NaN);", None),
        ("expect(\"a string\")[\"rejects\"].not.toBe(NaN);", None),
        ("expect(\"a string\").not.toEqual(NaN);", None),
        ("expect(\"a string\").not.toStrictEqual(NaN);", None),
        // undefined vs defined
        ("expect(undefined).not.toBeDefined();", None),
        ("expect(undefined).resolves.not.toBeDefined();", None),
        ("expect(undefined).resolves.toBe(undefined);", None),
        ("expect(\"a string\").not.toBeUndefined();", None),
        ("expect(\"a string\").rejects.not.toBeUndefined();", None),
        // typescript edition
        ("expect(null).toEqual(1 as unknown as string as unknown as any);", None),
        ("expect(null).toEqual(-1 as unknown as string as unknown as any);", None),
        ("expect(\"a string\").not.toStrictEqual(\"string\" as number);", None),
        ("expect(null).toBe(null as unknown as string as unknown as any);", None),
        ("expect(\"a string\").not.toEqual(null as number);", None),
        ("expect(undefined).toBe(undefined as unknown as string as any);", None),
        ("expect(\"a string\").toEqual(undefined as number);", None),
    ];

    let fix = vec![
        ("expect(value).toEqual(\"my string\");", "expect(value).toBe(\"my string\");", None),
        ("expect(value).toStrictEqual(\"my string\");", "expect(value).toBe(\"my string\");", None),
        ("expect(value).toStrictEqual(1);", "expect(value).toBe(1);", None),
        ("expect(value).toStrictEqual(1,);", "expect(value).toBe(1,);", None),
        ("expect(value).toStrictEqual(-1);", "expect(value).toBe(-1);", None),
        ("expect(value).toEqual(`my string`);", "expect(value).toBe(`my string`);", None),
        ("expect(value)[\"toEqual\"](`my string`);", "expect(value)[\"toBe\"](`my string`);", None),
        (
            "expect(value).toStrictEqual(`my ${string}`);",
            "expect(value).toBe(`my ${string}`);",
            None,
        ),
        (
            "expect(loadMessage()).resolves.toStrictEqual(\"hello world\");",
            "expect(loadMessage()).resolves.toBe(\"hello world\");",
            None,
        ),
        (
            "expect(loadMessage()).resolves[\"toStrictEqual\"](\"hello world\");",
            "expect(loadMessage()).resolves[\"toBe\"](\"hello world\");",
            None,
        ),
        (
            "expect(loadMessage())[\"resolves\"].toStrictEqual(\"hello world\");",
            "expect(loadMessage())[\"resolves\"].toBe(\"hello world\");",
            None,
        ),
        (
            "expect(loadMessage()).resolves.toStrictEqual(false);",
            "expect(loadMessage()).resolves.toBe(false);",
            None,
        ),
        // null
        ("expect(null).toBe(null);", "expect(null).toBeNull();", None),
        ("expect(null).toEqual(null);", "expect(null).toBeNull();", None),
        ("expect(null).toEqual(null,);", "expect(null).toBeNull();", None),
        ("expect(null).toStrictEqual(null);", "expect(null).toBeNull();", None),
        ("expect(\"a string\").not.toBe(null);", "expect(\"a string\").not.toBeNull();", None),
        (
            "expect(\"a string\").not[\"toBe\"](null);",
            "expect(\"a string\").not[\"toBeNull\"]();",
            None,
        ),
        (
            "expect(\"a string\")[\"not\"][\"toBe\"](null);",
            "expect(\"a string\")[\"not\"][\"toBeNull\"]();",
            None,
        ),
        ("expect(\"a string\").not.toEqual(null);", "expect(\"a string\").not.toBeNull();", None),
        (
            "expect(\"a string\").not.toStrictEqual(null);",
            "expect(\"a string\").not.toBeNull();",
            None,
        ),
        // undefined
        ("expect(undefined).toBe(undefined);", "expect(undefined).toBeUndefined();", None),
        ("expect(undefined).toEqual(undefined);", "expect(undefined).toBeUndefined();", None),
        ("expect(undefined).toStrictEqual(undefined);", "expect(undefined).toBeUndefined();", None),
        ("expect(\"a string\").not.toBe(undefined);", "expect(\"a string\").toBeDefined();", None),
        (
            "expect(\"a string\").rejects.not.toBe(undefined);",
            "expect(\"a string\").rejects.toBeDefined();",
            None,
        ),
        (
            "expect(\"a string\").rejects.not[\"toBe\"](undefined);",
            "expect(\"a string\").rejects[\"toBeDefined\"]();",
            None,
        ),
        (
            "expect(\"a string\").not.toEqual(undefined);",
            "expect(\"a string\").toBeDefined();",
            None,
        ),
        (
            "expect(\"a string\").not.toStrictEqual(undefined);",
            "expect(\"a string\").toBeDefined();",
            None,
        ),
        // NaN
        ("expect(NaN).toBe(NaN);", "expect(NaN).toBeNaN();", None),
        ("expect(NaN).toEqual(NaN);", "expect(NaN).toBeNaN();", None),
        ("expect(NaN).toStrictEqual(NaN);", "expect(NaN).toBeNaN();", None),
        ("expect(\"a string\").not.toBe(NaN);", "expect(\"a string\").not.toBeNaN();", None),
        (
            "expect(\"a string\").rejects.not.toBe(NaN);",
            "expect(\"a string\").rejects.not.toBeNaN();",
            None,
        ),
        (
            "expect(\"a string\")[\"rejects\"].not.toBe(NaN);",
            "expect(\"a string\")[\"rejects\"].not.toBeNaN();",
            None,
        ),
        ("expect(\"a string\").not.toEqual(NaN);", "expect(\"a string\").not.toBeNaN();", None),
        (
            "expect(\"a string\").not.toStrictEqual(NaN);",
            "expect(\"a string\").not.toBeNaN();",
            None,
        ),
        // undefined vs defined
        ("expect(undefined).not.toBeDefined();", "expect(undefined).toBeUndefined();", None),
        (
            "expect(undefined).resolves.not.toBeDefined();",
            "expect(undefined).resolves.toBeUndefined();",
            None,
        ),
        (
            "expect(undefined).resolves.toBe(undefined);",
            "expect(undefined).resolves.toBeUndefined();",
            None,
        ),
        ("expect(\"a string\").not.toBeUndefined();", "expect(\"a string\").toBeDefined();", None),
        (
            "expect(\"a string\").rejects.not.toBeUndefined();",
            "expect(\"a string\").rejects.toBeDefined();",
            None,
        ),
        // typescript edition
        (
            "expect(null).toEqual(1 as unknown as string as unknown as any);",
            "expect(null).toBe(1 as unknown as string as unknown as any);",
            None,
        ),
        (
            "expect(null).toEqual(-1 as unknown as string as unknown as any);",
            "expect(null).toBe(-1 as unknown as string as unknown as any);",
            None,
        ),
        (
            "expect(\"a string\").not.toStrictEqual(\"string\" as number);",
            "expect(\"a string\").not.toBe(\"string\" as number);",
            None,
        ),
        (
            "expect(null).toBe(null as unknown as string as unknown as any);",
            "expect(null).toBeNull();",
            None,
        ),
        (
            "expect(\"a string\").not.toEqual(null as number);",
            "expect(\"a string\").not.toBeNull();",
            None,
        ),
        (
            "expect(undefined).toBe(undefined as unknown as string as any);",
            "expect(undefined).toBeUndefined();",
            None,
        ),
        (
            "expect(\"a string\").toEqual(undefined as number);",
            "expect(\"a string\").toBeUndefined();",
            None,
        ),
    ];

    Tester::new(PreferToBe::NAME, PreferToBe::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
