use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::no_large_snapshots::{DOCUMENTATION, NoLargeSnapshotsConfig},
};

#[derive(Debug, Default, Clone)]
pub struct NoLargeSnapshots(Box<NoLargeSnapshotsConfig>);

declare_oxc_lint!(
    NoLargeSnapshots,
    vitest,
    style,
    config = NoLargeSnapshotsConfig,
    docs = DOCUMENTATION,
    version = "0.4.3",
);

impl Rule for NoLargeSnapshots {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        NoLargeSnapshotsConfig::from_configuration(&value).map(|config| Self(Box::new(config)))
    }

    fn run_once(&self, ctx: &LintContext) {
        self.0.run_once(ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    // use std::path::PathBuf;

    fn generate_snapshot_lines(lines: usize) -> String {
        let repeated_lines = "line\n".repeat(lines);
        format!("`\n{repeated_lines}`")
    }

    // fn generate_exports_snapshot_string(lines: usize, title: Option<&str>) -> String {
    //     let title = title.unwrap_or("a big component 1");
    //     format!("exports[`{}`] = {};", title, generate_snapshot_lines(lines))
    // }

    fn generate_expect_inline_snaps_code(line: usize, matcher: &str) -> String {
        format!("expect(something).{}({});", matcher, generate_snapshot_lines(line))
    }

    fn generate_match_inline_snapshot(line: usize) -> String {
        generate_expect_inline_snaps_code(line, "toMatchInlineSnapshot")
    }

    fn generate_throw_error_matching_inline_snapshot(line: usize) -> String {
        generate_expect_inline_snaps_code(line, "toThrowErrorMatchingInlineSnapshot")
    }

    // Note: Currently oxlint didn't check `.snap` file
    //
    // #[cfg(target_os = "windows")]
    // let snap_path = "c:\\mock-component.jsx.snap";
    // #[cfg(target_os = "windows")]
    // let another_snap_path = "c:\\another-mock-component.jsx.snap";

    // #[cfg(not(target_os = "windows"))]
    // let snap_path = "/mock-component.jsx.snap";
    // #[cfg(not(target_os = "windows"))]
    // let another_snap_path = "/another-mock-component.jsx.snap";

    let two_match_inline_cases = generate_match_inline_snapshot(2);
    let two_throw_error_match_cases = generate_throw_error_matching_inline_snapshot(2);
    let twenty_match_inline_cases = generate_match_inline_snapshot(20);
    let sixty_match_inline_cases = generate_match_inline_snapshot(60);
    let sixty_cases = format!(
        "
            expect(
                functionUnderTest(
                    arg1,
                    arg2,
                    arg3
                )
            ).toMatchInlineSnapshot({})
        ",
        generate_snapshot_lines(60)
    );
    // let twenty_exports_snapshot = generate_exports_snapshot_string(20, None);
    // let fifty_eight_exports_snapshot = generate_exports_snapshot_string(58, None);

    let mut pass = vec![
        ("expect(something)", None, None, None),
        ("expect(something).toBe(1)", None, None, None),
        ("expect(something).toMatchInlineSnapshot", None, None, None),
        ("expect(something).toMatchInlineSnapshot()", None, None, None),
        (two_match_inline_cases.as_str(), None, None, None),
        (two_throw_error_match_cases.as_str(), None, None, None),
        (
            twenty_match_inline_cases.as_str(),
            Some(serde_json::json!([{ "maxSize": 19, "inlineMaxSize": 21 }])),
            None,
            None,
        ),
        (
            sixty_match_inline_cases.as_str(),
            Some(serde_json::json!([{ "maxSize": 61 }])),
            None,
            None,
        ),
        (sixty_cases.as_str(), Some(serde_json::json!([{ "maxSize": 61 }])), None, None),
        // '/mock-component.jsx.snap'
        // (twenty_exports_snapshot.as_str(), None, None, Some(PathBuf::from(snap_path))),
        // '/mock-component.jsx.snap'
        // (
        //     fifty_eight_exports_snapshot.as_str(),
        //     Some(serde_json::json!([{
        //         "allowedSnapshots": {
        //             snap_path.to_string(): ["a big component 1"]
        //         }
        //     }])),
        //     None,
        //     Some(PathBuf::from(snap_path)),
        // ),
        // '/mock-component.jsx.snap'
        // (
        //     twenty_exports_snapshot.as_str(),
        //     Some(serde_json::json!([{ "maxSize": 21, "inlineMaxSize": 19 }])),
        //     None,
        //     Some(PathBuf::from(snap_path)),
        // ),
    ];

    let fifty_match_inline_cases = generate_match_inline_snapshot(50);
    let fifty_throw_error_match_cases = generate_throw_error_matching_inline_snapshot(50);

    // let fifty_two_exports_snapshot = generate_exports_snapshot_string(58, None);
    // let one_hundred_exports_snapshot = generate_exports_snapshot_string(100, None);
    // let one_exports_snapshot = generate_exports_snapshot_string(1, None);
    // let fifty_eight_exports_snapshot = generate_exports_snapshot_string(58, None);
    // let vec_to_str = [
    //     generate_exports_snapshot_string(58, Some("a big component w/ text")),
    //     generate_exports_snapshot_string(58, Some("a big component 2")),
    // ]
    // .join("\n\n");

    let mut fail = vec![
        (fifty_match_inline_cases.as_str(), None, None, None),
        (fifty_throw_error_match_cases.as_str(), None, None, None),
        (
            fifty_throw_error_match_cases.as_str(),
            Some(serde_json::json!([{ "maxSize": 51, "inlineMaxSize": 50 }])),
            None,
            None,
        ),
        (
            fifty_throw_error_match_cases.as_str(),
            Some(serde_json::json!([{ "maxSize": 0 }])),
            None,
            None,
        ),
        // '/mock-component.jsx.snap'
        // (fifty_two_exports_snapshot.as_str(), None, None, Some(PathBuf::from(snap_path))),
        // '/mock-component.jsx.snap'
        // (
        //     one_hundred_exports_snapshot.as_str(),
        //     Some(serde_json::json!([{ "maxSize": 70 }])),
        //     None,
        //     Some(PathBuf::from(snap_path)),
        // ),
        // '/mock-component.jsx.snap'
        // (
        //     one_hundred_exports_snapshot.as_str(),
        //     Some(serde_json::json!([{ "maxSize": 70, "inlineMaxSize": 101 }])),
        //     None,
        //     Some(PathBuf::from(snap_path)),
        // ),
        // '/mock-component.jsx.snap'
        // (
        //     one_exports_snapshot.as_str(),
        //     Some(serde_json::json!([{ "maxSize": 0 }])),
        //     None,
        //     Some(PathBuf::from(snap_path)),
        // ),
        // '/mock-component.jsx.snap'
        // (
        //     fifty_eight_exports_snapshot.as_str(),
        //     Some(serde_json::json!([{
        //         "allowedSnapshots": {
        //             another_snap_path.to_string(): [r"a big component \d+"]
        //         }
        //     }])),
        //     None,
        //     Some(PathBuf::from(snap_path)),
        // ),
        // '/mock-component.jsx.snap'
        // (
        //     vec_to_str.as_str(),
        //     Some(serde_json::json!([{
        //         "allowedSnapshots": {
        //             snap_path.to_string(): [r"a big component \d+"],
        //         },
        //     }])),
        //     None,
        //     Some(PathBuf::from(snap_path)),
        // ),
        // '/mock-component.jsx.snap'
        // (
        //     vec_to_str.as_str(),
        //     Some(serde_json::json!([{
        //         "allowedSnapshots": {
        //             snap_path.to_string(): ["a big component 2"],
        //         },
        //     }])),
        //     None,
        //     Some(PathBuf::from(snap_path)),
        // ),
    ];

    let vitest_pass = vec![
        ("expect(something)", None, None, None),
        ("expect(something).toBe(1)", None, None, None),
        ("expect(something).toMatchInlineSnapshot", None, None, None),
        ("expect(something).toMatchInlineSnapshot()", None, None, None),
        (two_match_inline_cases.as_str(), None, None, None),
        (two_throw_error_match_cases.as_str(), None, None, None),
        (
            twenty_match_inline_cases.as_str(),
            Some(serde_json::json!([{ "maxSize": 19, "inlineMaxSize": 21 }])),
            None,
            None,
        ),
        (
            sixty_match_inline_cases.as_str(),
            Some(serde_json::json!([{ "maxSize": 61 }])),
            None,
            None,
        ),
        (sixty_cases.as_str(), Some(serde_json::json!([{ "maxSize": 61 }])), None, None),
    ];

    let vitest_fail = vec![
        (fifty_match_inline_cases.as_str(), None, None, None),
        (fifty_throw_error_match_cases.as_str(), None, None, None),
        (
            fifty_throw_error_match_cases.as_str(),
            Some(serde_json::json!([{ "maxSize": 51, "inlineMaxSize": 50 }])),
            None,
            None,
        ),
        (
            fifty_throw_error_match_cases.as_str(),
            Some(serde_json::json!([{ "maxSize": 0 }])),
            None,
            None,
        ),
    ];

    pass.extend(vitest_pass);
    fail.extend(vitest_fail);

    Tester::new(NoLargeSnapshots::NAME, NoLargeSnapshots::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
