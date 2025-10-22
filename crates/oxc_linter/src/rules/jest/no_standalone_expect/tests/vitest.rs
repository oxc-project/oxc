#[test]
fn test() {
    use super::super::NoStandaloneExpect;
    use crate::{rule::RuleMeta, tester::Tester};

    let pass = vec![
        ("beforeEach(() => { doSomething(); });", None),
        ("expect.any(String)", None),
        ("expect.extend({})", None),
        (r#"bench("a bench", () => {})"#, None),
        (r#"describe("a test", () => { it("an it", () => {expect(1).toBe(1); }); });"#, None),
        (
            r#"describe("a test", () => { it("an it", () => { const func = () => { expect(1).toBe(1); }; }); });"#,
            None,
        ),
        (r#"describe("a test", () => { const func = () => { expect(1).toBe(1); }; });"#, None),
        (r#"describe("a test", () => { function func() { expect(1).toBe(1); }; });"#, None),
        (r#"describe("a test", () => { const func = function(){ expect(1).toBe(1); }; });"#, None),
        (
            r#"describe.only.concurrent.todo("a test", () => { const func = function(){ expect(1).toBe(1); }; });"#,
            None,
        ),
        (r#"it("an it", () => expect(1).toBe(1))"#, None),
        (r#"it.only("an it", () => expect(1).toBe(1))"#, None),
        (r#"it.concurrent("an it", () => expect(1).toBe(1))"#, None),
        // TODO: it.extend.* and test.fails are not properly recognized by the parser yet
        // (r#"it.extend.skip("an it", ()  => expect(1).toBe(1))"#, None),
        (r#"test("a test", () => expect(1).toBe(1))"#, None),
        (r#"test.skip("a skipped test", () => expect(1).toBe(1))"#, None),
        // (r#"test.fails("a failing test", () => expect(1).toBe(1))"#, None),
        ("const func = function(){ expect(1).toBe(1); };", None),
        ("const func = () => expect(1).toBe(1);", None),
        ("{}", None),
        (r#"it.each([1, true])("trues", value => { expect(value).toBe(true); });"#, None),
        (
            r#"it.each([1, true])("trues", value => { expect(value).toBe(true); }); it("an it", () => { expect(1).toBe(1) });"#,
            None,
        ),
    ];

    let fail = vec![
        ("(() => {})('testing', () => expect(true).toBe(false))", None),
        ("expect.hasAssertions()", None),
        (
            "
			       describe('scenario', () => {
			      const t = Math.random() ? it.only : it;
			      t('testing', () => expect(true).toBe(false));
			       });
			     ",
            None,
        ),
        (
            "describe('scenario', () => {
			      const t = Math.random() ? it.only : it;
			      t('testing', () => expect(true).toBe(false));
			       });",
            None,
        ),
        (r#"describe("a test", () => { expect(1).toBe(1); });"#, None),
        (r#"describe("a test", () => expect(1).toBe(1));"#, None),
        (
            r#"describe("a test", () => { const func = () => { expect(1).toBe(1); }; expect(1).toBe(1); });"#,
            None,
        ),
        (
            r#"describe("a test", () => {  it(() => { expect(1).toBe(1); }); expect(1).toBe(1); });"#,
            None,
        ),
        ("expect(1).toBe(1);", None),
        ("{expect(1).toBe(1)}", None),
        (
            "
			     each([
			       [1, 1, 2],
			       [1, 2, 3],
			       [2, 1, 3],
			     ]).test('returns the result of adding %d to %d', (a, b, expected) => {
			       expect(a + b).toBe(expected);
			     });",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["test"] }])),
        ),
    ];

    Tester::new(NoStandaloneExpect::NAME, NoStandaloneExpect::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .with_snapshot_suffix("vitest")
        .test_and_snapshot();
}
