#[test]
fn test() {
    use super::PreferLowercaseTitle;
    use crate::rule::RuleMeta;
    use crate::tester::Tester;

    let pass: Vec<(&str, Option<serde_json::Value>)> = vec![
        ("it.each()", None),
        ("it.each()(1)", None),
        ("it.todo();", None),
        (r#"describe("oo", function () {})"#, None),
        (r#"test("foo", function () {})"#, None),
        ("test(`123`, function () {})", None),
    ];

    let fail: Vec<(&str, Option<serde_json::Value>)> = vec![
        (r#"it("Foo MM mm", function () {})"#, None),
        ("test(`Foo MM mm`, function () {})", None),
        (
            "test(`SFC Compile`, function () {})",
            Some(serde_json::json!([{ "lowercaseFirstCharacterOnly": false }])),
        ),
        ("bench(`Foo MM mm`, function () {})", None),
    ];

    let fix: Vec<(&str, &str, Option<serde_json::Value>)> = vec![
        (r#"it("Foo MM mm", function () {})"#, r#"it("foo MM mm", function () {})"#, None),
        ("test(`Foo MM mm`, function () {})", "test(`foo MM mm`, function () {})", None),
        (
            "test(`SFC Compile`, function () {})",
            "test(`sfc compile`, function () {})",
            Some(serde_json::json!([{ "lowercaseFirstCharacterOnly": false }])),
        ),
        ("bench(`Foo MM mm`, function () {})", "bench(`foo MM mm`, function () {})", None),
    ];

    Tester::new(PreferLowercaseTitle::NAME, PreferLowercaseTitle::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .with_snapshot_suffix("vitest")
        .test_and_snapshot();
}
