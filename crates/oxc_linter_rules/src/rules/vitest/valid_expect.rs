use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::valid_expect::{DOCUMENTATION, ValidExpectConfig},
};

#[derive(Debug, Clone)]
pub struct ValidExpect(Box<ValidExpectConfig>);

impl Default for ValidExpect {
    fn default() -> Self {
        Self(Box::new(ValidExpectConfig::default().allow_string_message_arg()))
    }
}

declare_oxc_lint!(
    ValidExpect,
    vitest,
    correctness,
    suggestion,
    config = ValidExpectConfig,
    docs = DOCUMENTATION,
    version = "0.0.14",
);

impl Rule for ValidExpect {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(Self(Box::new(ValidExpectConfig::from_configuration(&value).allow_string_message_arg())))
    }

    fn run_once(&self, ctx: &LintContext) {
        self.0.run_once(ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("expect.hasAssertions", None),
        ("expect.hasAssertions()", None),
        ("expect(\"something\").toEqual(\"else\");", None),
        ("expect(true).toBeDefined();", None),
        ("expect([1, 2, 3]).toEqual([1, 2, 3]);", None),
        ("expect(undefined).not.toBeDefined();", None),
        ("test(\"valid-expect\", () => { return expect(Promise.resolve(2)).resolves.toBeDefined(); });", None),
        ("test(\"valid-expect\", () => { return expect(Promise.reject(2)).rejects.toBeDefined(); });", None),
        ("test(\"valid-expect\", () => { return expect(Promise.resolve(2)).resolves.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", () => { return expect(Promise.resolve(2)).rejects.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", function () { return expect(Promise.resolve(2)).resolves.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", function () { return expect(Promise.resolve(2)).rejects.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", function () { return Promise.resolve(expect(Promise.resolve(2)).resolves.not.toBeDefined()); });", None),
        ("test(\"valid-expect\", function () { return Promise.resolve(expect(Promise.resolve(2)).rejects.not.toBeDefined()); });", None),
        ("test(\"valid-expect\", () => expect(Promise.resolve(2)).resolves.toBeDefined());", None),
        ("test(\"valid-expect\", () => expect(Promise.reject(2)).rejects.toBeDefined());", None),
        ("test(\"valid-expect\", () => expect(Promise.reject(2)).resolves.not.toBeDefined());", None),
        ("test(\"valid-expect\", () => expect(Promise.reject(2)).rejects.not.toBeDefined());", None),
        ("test(\"valid-expect\", async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", async () => { await expect(Promise.reject(2)).rejects.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", async function () { await expect(Promise.reject(2)).resolves.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", async function () { await expect(Promise.reject(2)).rejects.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", async () => { await Promise.resolve(expect(Promise.reject(2)).rejects.not.toBeDefined()); });", None),
        ("test(\"valid-expect\", async () => { await Promise.reject(expect(Promise.reject(2)).rejects.not.toBeDefined()); });", None),
        ("test(\"valid-expect\", async () => { await Promise.all([expect(Promise.reject(2)).rejects.not.toBeDefined(), expect(Promise.reject(2)).rejects.not.toBeDefined()]); });", None),
        ("test(\"valid-expect\", async () => { await Promise.race([expect(Promise.reject(2)).rejects.not.toBeDefined(), expect(Promise.reject(2)).rejects.not.toBeDefined()]); });", None),
        ("test(\"valid-expect\", async () => { await Promise.allSettled([expect(Promise.reject(2)).rejects.not.toBeDefined(), expect(Promise.reject(2)).rejects.not.toBeDefined()]); });", None),
        ("test(\"valid-expect\", async () => { await Promise.any([expect(Promise.reject(2)).rejects.not.toBeDefined(), expect(Promise.reject(2)).rejects.not.toBeDefined()]); });", None),
        ("test(\"valid-expect\", async () => { return expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log(\"valid-case\")); });", None),
        ("test(\"valid-expect\", async () => { return expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log(\"valid-case\")).then(() => console.log(\"another valid case\")); });", None),
        ("test(\"valid-expect\", async () => { return expect(Promise.reject(2)).resolves.not.toBeDefined().catch(() => console.log(\"valid-case\")); });", None),
        ("test(\"valid-expect\", async () => { return expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log(\"valid-case\")).catch(() => console.log(\"another valid case\")); });", None),
        ("test(\"valid-expect\", async () => { return expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => { expect(someMock).toHaveBeenCalledTimes(1); }); });", None),
        ("test(\"valid-expect\", async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log(\"valid-case\")); });", None),
        ("test(\"valid-expect\", async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log(\"valid-case\")).then(() => console.log(\"another valid case\")); });", None),
        ("test(\"valid-expect\", async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined().catch(() => console.log(\"valid-case\")); });", None),
        ("test(\"valid-expect\", async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log(\"valid-case\")).catch(() => console.log(\"another valid case\")); });", None),
        ("test(\"valid-expect\", async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => { expect(someMock).toHaveBeenCalledTimes(1); }); });", None),
        (
            "
                test(\"valid-expect\", () => {
                    return expect(functionReturningAPromise()).resolves.toEqual(1).then(() => {
                        return expect(Promise.resolve(2)).resolves.toBe(1);
                    });
                });
        ",
        None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    return expect(functionReturningAPromise()).resolves.toEqual(1).then(async () => {
                        await expect(Promise.resolve(2)).resolves.toBe(1);
                    });
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    return expect(functionReturningAPromise()).resolves.toEqual(1).then(() => expect(Promise.resolve(2)).resolves.toBe(1));
                });
            ",
            None,
        ),
        (
            "
                expect.extend({
                    toResolve(obj) {
                        return this.isNot
                            ? expect(obj).toBe(true)
                            : expect(obj).resolves.not.toThrow();
                    }
                });
            ",
            None,
        ),
        (
            "
                expect.extend({
                    toResolve(obj) {
                        return this.isNot
                            ? expect(obj).resolves.not.toThrow()
                            : expect(obj).toBe(true);
                    }
                });
            ",
            None,
        ),
        (
            "
                expect.extend({
                    toResolve(obj) {
                        return this.isNot
                        ? expect(obj).toBe(true)
                        : anotherCondition
                            ? expect(obj).resolves.not.toThrow()
                            : expect(obj).toBe(false)
                    }
                });
            ",
            None,
        ),
        ("expect(1).toBe(2);", Some(serde_json::json!([{ "maxArgs": 2 }]))),
        ("expect(1, \"1 !== 2\").toBe(2);", Some(serde_json::json!([{ "maxArgs": 2 }]))),
        ("expect(1, \"sum is incorrect\").toBe(1);", None),
        ("test(\"valid-expect\", () => { expect(1 + 2, \"sum is incorrect\").toBe(3); });", None),
        ("test(\"valid-expect\", () => { expect(1 + 2, `sum is ${label}`).toBe(3); });", None),
        (
            "test(\"valid-expect\", () => { expect(2).not.toBe(2); });",
            Some(serde_json::json!([{ "asyncMatchers": ["toRejectWith"] }])),
        ),
        (
            "test(\"valid-expect\", () => { expect(Promise.reject(2)).toRejectWith(2); });",
            Some(serde_json::json!([{ "asyncMatchers": ["toResolveWith"] }])),
        ),
        (
            "test(\"valid-expect\", async () => { await expect(Promise.resolve(2)).toResolve(); });",
            Some(serde_json::json!([{ "asyncMatchers": ["toResolveWith"] }])),
        ),
        (
            "test(\"valid-expect\", async () => { expect(Promise.resolve(2)).toResolve(); });",
            Some(serde_json::json!([{ "asyncMatchers": ["toResolveWith"] }])),
        ),
    ];

    let fail = vec![
        ("expect().toBe(2);", Some(serde_json::json!([{ "minArgs": "undefined", "maxArgs": "undefined" }]))),
        ("expect().toBe(true);", None),
        ("expect().toEqual(\"something\");", None),
        ("expect(\"something\", \"else\", \"entirely\").toEqual(\"something\");", Some(serde_json::json!([{ "maxArgs": 2 }]))),
        ("expect(\"something\", \"else\", \"entirely\").toEqual(\"something\");", Some(serde_json::json!([{ "maxArgs": 2, "minArgs": 2 }]))),
        ("expect(\"something\", \"else\", \"entirely\").toEqual(\"something\");", Some(serde_json::json!([{ "maxArgs": 2, "minArgs": 1 }]))),
        ("expect(\"something\").toEqual(\"something\");", Some(serde_json::json!([{ "minArgs": 2 }]))),
        ("expect(\"something\", \"else\").toEqual(\"something\");", Some(serde_json::json!([{ "maxArgs": 1, "minArgs": 3 }]))),
        ("expect(1, message).toBe(1);", None),
        ("expect(\"something\");", None),
        ("expect();", None),
        ("expect(true).toBeDefined;", None),
        ("expect(true).not.toBeDefined;", None),
        ("expect(true).nope.toBeDefined;", None),
        ("expect(true).nope.toBeDefined();", None),
        ("expect(true).not.resolves.toBeDefined();", None),
        ("expect(true).not.not.toBeDefined();", None),
        ("expect(true).resolves.not.exactly.toBeDefined();", None),
        ("expect(true).resolves;", None),
        ("expect(true).rejects;", None),
        ("expect(true).not;", None),
        ("expect(Promise.resolve(2)).resolves.toBeDefined();", None),
        ("expect(Promise.resolve(2)).rejects.toBeDefined();", None),
        ("expect(Promise.resolve(2)).resolves.toBeDefined();", Some(serde_json::json!([{ "alwaysAwait": true }]))),
        (
            "
                expect.extend({
                    toResolve(obj) {
                        this.isNot
                            ? expect(obj).toBe(true)
                            : expect(obj).resolves.not.toThrow();
                    }
                });
            ",
            None,
        ),
        (
            "
                expect.extend({
                    toResolve(obj) {
                        this.isNot
                            ? expect(obj).resolves.not.toThrow()
                            : expect(obj).toBe(true);
                    }
                });
            ",
            None,
        ),
        ("test(\"valid-expect\", () => { expect(Promise.resolve(2)).resolves.toBeDefined(); });", None),
        ("test(\"valid-expect\", () => { expect(Promise.resolve(2)).toResolve(); });", None),
        ("test(\"valid-expect\", () => { expect(Promise.resolve(2)).toResolve(); });", Some(serde_json::json!([{ "asyncMatchers": "undefined" }]))),
        ("test(\"valid-expect\", () => { expect(Promise.resolve(2)).toReject(); });", None),
        ("test(\"valid-expect\", () => { expect(Promise.resolve(2)).not.toReject(); });", None),
        ("test(\"valid-expect\", () => { expect(Promise.resolve(2)).resolves.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", () => { expect(Promise.resolve(2)).rejects.toBeDefined(); });", None),
        ("test(\"valid-expect\", () => { expect(Promise.resolve(2)).rejects.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", async () => { expect(Promise.resolve(2)).resolves.toBeDefined(); });", None),
        ("test(\"valid-expect\", async () => { expect(Promise.resolve(2)).resolves.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", () => { expect(Promise.reject(2)).toRejectWith(2); });", Some(serde_json::json!([{ "asyncMatchers": ["toRejectWith"] }]))),
        ("test(\"valid-expect\", () => { expect(Promise.reject(2)).rejects.toBe(2); });", Some(serde_json::json!([{ "asyncMatchers": ["toRejectWith"] }]))),
        (
            "
                test(\"valid-expect\", async () => {
                    expect(Promise.resolve(2)).resolves.not.toBeDefined();
                    expect(Promise.resolve(1)).rejects.toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", async () => {
                    await expect(Promise.resolve(2)).resolves.not.toBeDefined();
                    expect(Promise.resolve(1)).rejects.toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", async () => {
                    expect(Promise.resolve(2)).resolves.not.toBeDefined();
                    return expect(Promise.resolve(1)).rejects.toBeDefined();
                });
            ",
            Some(serde_json::json!([{ "alwaysAwait": true }])),
        ),
        ("
                test(\"valid-expect\", async () => {
                    expect(Promise.resolve(2)).resolves.not.toBeDefined();
                    return expect(Promise.resolve(1)).rejects.toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    Promise.x(expect(Promise.resolve(2)).resolves.not.toBeDefined());
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    Promise.resolve(expect(Promise.resolve(2)).resolves.not.toBeDefined());
                });
            ",
            Some(serde_json::json!([{ "alwaysAwait": true }])),
        ),
        (
            "
                test(\"valid-expect\", () => {
                    Promise.all([
                        expect(Promise.resolve(2)).resolves.not.toBeDefined(),
                        expect(Promise.resolve(3)).resolves.not.toBeDefined(),
                    ]);
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    Promise.x([
                        expect(Promise.resolve(2)).resolves.not.toBeDefined(),
                        expect(Promise.resolve(3)).resolves.not.toBeDefined(),
                    ]);
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    const assertions = [
                        expect(Promise.resolve(2)).resolves.not.toBeDefined(),
                        expect(Promise.resolve(3)).resolves.not.toBeDefined(),
                    ]
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    const assertions = [
                        expect(Promise.resolve(2)).toResolve(),
                        expect(Promise.resolve(3)).toReject(),
                    ]
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    const assertions = [
                        expect(Promise.resolve(2)).not.toResolve(),
                        expect(Promise.resolve(3)).resolves.toReject(),
                    ]
                });
            ",
            None,
        ),
        ("expect(Promise.resolve(2)).resolves.toBe;", None),
        (
            "
                test(\"valid-expect\", () => {
                    return expect(functionReturningAPromise()).resolves.toEqual(1).then(() => {
                        expect(Promise.resolve(2)).resolves.toBe(1);
                    });
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    return expect(functionReturningAPromise()).resolves.toEqual(1).then(async () => {
                        await expect(Promise.resolve(2)).resolves.toBe(1);
                        expect(Promise.resolve(4)).resolves.toBe(4);
                    });
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", async () => {
                    await expect(Promise.resolve(1));
                });
            ",
            None,
        ),
    ];

    let fix = vec![
        (
            "
                 expect.extend({
                   toResolve(obj) {
                  this.isNot
                    ? expect(obj).toBe(true)
                    : expect(obj).resolves.not.toThrow();
                   }
                 });
                  ",
            "
                 expect.extend({
                   async toResolve(obj) {
                  this.isNot
                    ? expect(obj).toBe(true)
                    : await expect(obj).resolves.not.toThrow();
                   }
                 });
                  ",
            None,
        ),
        (
            "
                 expect.extend({
                   toResolve(obj) {
                  this.isNot
                    ? expect(obj).resolves.not.toThrow()
                    : expect(obj).toBe(true);
                   }
                 });
                  ",
            "
                 expect.extend({
                   async toResolve(obj) {
                  this.isNot
                    ? await expect(obj).resolves.not.toThrow()
                    : expect(obj).toBe(true);
                   }
                 });
                  ",
            None,
        ),
        (
            r#"test("valid-expect", () => { expect(Promise.resolve(2)).resolves.toBeDefined(); });"#,
            r#"test("valid-expect", async () => { await expect(Promise.resolve(2)).resolves.toBeDefined(); });"#,
            None,
        ),
        (
            r#"test("valid-expect", () => { expect(Promise.resolve(2)).toResolve(); });"#,
            r#"test("valid-expect", async () => { await expect(Promise.resolve(2)).toResolve(); });"#,
            None,
        ),
        (
            r#"test("valid-expect", () => { expect(Promise.resolve(2)).toResolve(); });"#,
            r#"test("valid-expect", async () => { await expect(Promise.resolve(2)).toResolve(); });"#,
            Some(serde_json::json!([{ "asyncMatchers": "undefined" }])),
        ),
        (
            r#"test("valid-expect", () => { expect(Promise.resolve(2)).toReject(); });"#,
            r#"test("valid-expect", async () => { await expect(Promise.resolve(2)).toReject(); });"#,
            None,
        ),
        (
            r#"test("valid-expect", () => { expect(Promise.resolve(2)).not.toReject(); });"#,
            r#"test("valid-expect", async () => { await expect(Promise.resolve(2)).not.toReject(); });"#,
            None,
        ),
        (
            r#"test("valid-expect", () => { expect(Promise.resolve(2)).resolves.not.toBeDefined(); });"#,
            r#"test("valid-expect", async () => { await expect(Promise.resolve(2)).resolves.not.toBeDefined(); });"#,
            None,
        ),
        (
            r#"test("valid-expect", () => { expect(Promise.resolve(2)).rejects.toBeDefined(); });"#,
            r#"test("valid-expect", async () => { await expect(Promise.resolve(2)).rejects.toBeDefined(); });"#,
            None,
        ),
        (
            r#"test("valid-expect", () => { expect(Promise.resolve(2)).rejects.not.toBeDefined(); });"#,
            r#"test("valid-expect", async () => { await expect(Promise.resolve(2)).rejects.not.toBeDefined(); });"#,
            None,
        ),
        (
            r#"test("valid-expect", async () => { expect(Promise.resolve(2)).resolves.toBeDefined(); });"#,
            r#"test("valid-expect", async () => { await expect(Promise.resolve(2)).resolves.toBeDefined(); });"#,
            None,
        ),
        (
            r#"test("valid-expect", async () => { expect(Promise.resolve(2)).resolves.not.toBeDefined(); });"#,
            r#"test("valid-expect", async () => { await expect(Promise.resolve(2)).resolves.not.toBeDefined(); });"#,
            None,
        ),
        (
            r#"test("valid-expect", () => { expect(Promise.reject(2)).toRejectWith(2); });"#,
            r#"test("valid-expect", async () => { await expect(Promise.reject(2)).toRejectWith(2); });"#,
            Some(serde_json::json!([{ "asyncMatchers": ["toRejectWith"] }])),
        ),
        (
            r#"test("valid-expect", () => { expect(Promise.reject(2)).rejects.toBe(2); });"#,
            r#"test("valid-expect", async () => { await expect(Promise.reject(2)).rejects.toBe(2); });"#,
            Some(serde_json::json!([{ "asyncMatchers": ["toRejectWith"] }])),
        ),
        (
            r#"
                   test("valid-expect", async () => {
                  expect(Promise.resolve(2)).resolves.not.toBeDefined();
                  expect(Promise.resolve(1)).rejects.toBeDefined();
                   });
                 "#,
            r#"
                   test("valid-expect", async () => {
                  await expect(Promise.resolve(2)).resolves.not.toBeDefined();
                  await expect(Promise.resolve(1)).rejects.toBeDefined();
                   });
                 "#,
            None,
        ),
        (
            r#"
                   test("valid-expect", async () => {
                  await expect(Promise.resolve(2)).resolves.not.toBeDefined();
                  expect(Promise.resolve(1)).rejects.toBeDefined();
                   });
                 "#,
            r#"
                   test("valid-expect", async () => {
                  await expect(Promise.resolve(2)).resolves.not.toBeDefined();
                  await expect(Promise.resolve(1)).rejects.toBeDefined();
                   });
                 "#,
            None,
        ),
        (
            r#"
                   test("valid-expect", async () => {
                  expect(Promise.resolve(2)).resolves.not.toBeDefined();
                  return expect(Promise.resolve(1)).rejects.toBeDefined();
                   });
                 "#,
            r#"
                   test("valid-expect", async () => {
                  await expect(Promise.resolve(2)).resolves.not.toBeDefined();
                  await expect(Promise.resolve(1)).rejects.toBeDefined();
                   });
                 "#,
            Some(serde_json::json!([{ "alwaysAwait": true }])),
        ),
        (
            r#"
                   test("valid-expect", async () => {
                  expect(Promise.resolve(2)).resolves.not.toBeDefined();
                  return expect(Promise.resolve(1)).rejects.toBeDefined();
                   });
                 "#,
            r#"
                   test("valid-expect", async () => {
                  await expect(Promise.resolve(2)).resolves.not.toBeDefined();
                  return expect(Promise.resolve(1)).rejects.toBeDefined();
                   });
                 "#,
            None,
        ),
        (
            r#"
                   test("valid-expect", () => {
                  Promise.x(expect(Promise.resolve(2)).resolves.not.toBeDefined());
                   });
                 "#,
            r#"
                   test("valid-expect", async () => {
                  await Promise.x(expect(Promise.resolve(2)).resolves.not.toBeDefined());
                   });
                 "#,
            None,
        ),
        (
            r#"
                 test("valid-expect", () => {
                   Promise.resolve(expect(Promise.resolve(2)).resolves.not.toBeDefined());
                 });
                  "#,
            r#"
                 test("valid-expect", async () => {
                   await Promise.resolve(expect(Promise.resolve(2)).resolves.not.toBeDefined());
                 });
                  "#,
            Some(serde_json::json!([{ "alwaysAwait": true }])),
        ),
        (
            r#"
                 test("valid-expect", () => {
                  Promise.all([
                    expect(Promise.resolve(2)).resolves.not.toBeDefined(),
                    expect(Promise.resolve(3)).resolves.not.toBeDefined(),
                  ]);
                   });
                   "#,
            r#"
                 test("valid-expect", async () => {
                  await Promise.all([
                    expect(Promise.resolve(2)).resolves.not.toBeDefined(),
                    expect(Promise.resolve(3)).resolves.not.toBeDefined(),
                  ]);
                   });
                   "#,
            None,
        ),
        (
            r#"
                 test("valid-expect", () => {
                  Promise.x([
                    expect(Promise.resolve(2)).resolves.not.toBeDefined(),
                    expect(Promise.resolve(3)).resolves.not.toBeDefined(),
                  ]);
                   });"#,
            r#"
                 test("valid-expect", async () => {
                  await Promise.x([
                    expect(Promise.resolve(2)).resolves.not.toBeDefined(),
                    expect(Promise.resolve(3)).resolves.not.toBeDefined(),
                  ]);
                   });"#,
            None,
        ),
        (
            r#"
                   test("valid-expect", () => {
                  const assertions = [
                    expect(Promise.resolve(2)).resolves.not.toBeDefined(),
                    expect(Promise.resolve(3)).resolves.not.toBeDefined(),
                  ]
                   });
                 "#,
            r#"
                   test("valid-expect", async () => {
                  const assertions = [
                    await expect(Promise.resolve(2)).resolves.not.toBeDefined(),
                    await expect(Promise.resolve(3)).resolves.not.toBeDefined(),
                  ]
                   });
                 "#,
            None,
        ),
        (
            r#"
                 test("valid-expect", () => {
                   const assertions = [
                  expect(Promise.resolve(2)).toResolve(),
                  expect(Promise.resolve(3)).toReject(),
                   ]
                 });
                  "#,
            r#"
                 test("valid-expect", async () => {
                   const assertions = [
                  await expect(Promise.resolve(2)).toResolve(),
                  await expect(Promise.resolve(3)).toReject(),
                   ]
                 });
                  "#,
            None,
        ),
        (
            r#"
                   test("valid-expect", () => {
                  const assertions = [
                    expect(Promise.resolve(2)).not.toResolve(),
                    expect(Promise.resolve(3)).resolves.toReject(),
                  ]
                   });
                 "#,
            r#"
                   test("valid-expect", async () => {
                  const assertions = [
                    await expect(Promise.resolve(2)).not.toResolve(),
                    await expect(Promise.resolve(3)).resolves.toReject(),
                  ]
                   });
                 "#,
            None,
        ),
        (
            r#"
                   test("valid-expect", () => {
                  return expect(functionReturningAPromise()).resolves.toEqual(1).then(() => {
                    expect(Promise.resolve(2)).resolves.toBe(1);
                  });
                   });
                 "#,
            r#"
                   test("valid-expect", () => {
                  return expect(functionReturningAPromise()).resolves.toEqual(1).then(async () => {
                    await expect(Promise.resolve(2)).resolves.toBe(1);
                  });
                   });
                 "#,
            None,
        ),
        (
            r#"
                   test("valid-expect", () => {
                  return expect(functionReturningAPromise()).resolves.toEqual(1).then(async () => {
                    await expect(Promise.resolve(2)).resolves.toBe(1);
                    expect(Promise.resolve(4)).resolves.toBe(4);
                  });
                   });
                 "#,
            r#"
                   test("valid-expect", () => {
                  return expect(functionReturningAPromise()).resolves.toEqual(1).then(async () => {
                    await expect(Promise.resolve(2)).resolves.toBe(1);
                    await expect(Promise.resolve(4)).resolves.toBe(4);
                  });
                   });
                 "#,
            None,
        ),
    ];

    Tester::new(ValidExpect::NAME, ValidExpect::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
