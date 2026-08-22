use oxc_transformer::{BabelOptions, HelperLoaderMode, TransformOptions};

use crate::test;

fn options(target: &str) -> TransformOptions {
    let mut options = TransformOptions::from_target(target).unwrap();
    options.helper_loader.mode = HelperLoaderMode::External;
    options
}

#[test]
fn duplicate_named_capture_groups_target_boundary() {
    let source = r"/(?<year>\d{4})|(?<year>\d{2})/;";

    assert_eq!(
        test(source, &options("es2024")).unwrap(),
        "/* @__PURE__ */ babelHelpers.wrapRegExp(/(\\d{4})|(\\d{2})/, { year: [1, 2] });\n"
    );
    assert_eq!(test(source, &options("es2025")).unwrap(), source.to_string() + "\n");
    assert_eq!(
        test(source, &options("chrome125")).unwrap(),
        test(source, &options("es2024")).unwrap()
    );
    assert_eq!(test(source, &options("chrome126")).unwrap(), source.to_string() + "\n");
}

#[test]
fn duplicate_named_capture_groups_rewrites_backreferences() {
    let source = r"/(?:(?<name>a)|(?<name>b))\k<name>/;";
    assert_eq!(
        test(source, &options("es2024")).unwrap(),
        "/* @__PURE__ */ babelHelpers.wrapRegExp(/(?:(a)|(b))\\1\\2/, { name: [1, 2] });\n"
    );
    assert_eq!(
        test(r"/(?<a>x)|(?<\u0061>y)\k<\u0061>/;", &options("es2024")).unwrap(),
        "/* @__PURE__ */ babelHelpers.wrapRegExp(/(x)|(y)\\1\\2/, { a: [1, 2] });\n"
    );
}

#[test]
fn duplicate_named_capture_groups_skips_runtime_wrapper_for_test() {
    let source = r#"
        /(?<year>\d{4})|(?<year>\d{2})/.test("24");
        const test = /(?<year>\d{4})|(?<year>\d{2})/.test;
    "#;
    assert_eq!(
        test(source, &options("es2024")).unwrap(),
        "/(\\d{4})|(\\d{2})/.test('24');\nconst test = /(\\d{4})|(\\d{2})/.test;\n"
    );
}

#[test]
fn duplicate_named_capture_groups_runtime_false() {
    let babel_options = serde_json::from_str::<BabelOptions>(
        r#"{
            "plugins": [
                ["transform-duplicate-named-capturing-groups-regex", { "runtime": false }]
            ]
        }"#,
    )
    .unwrap();
    let mut options = TransformOptions::try_from(&babel_options).unwrap();
    options.helper_loader.mode = HelperLoaderMode::External;

    assert_eq!(
        test(r"/(?:(?<name>a)|(?<name>b))\k<name>/;", &options).unwrap(),
        "/(?:(a)|(b))\\1\\2/;\n"
    );
}

#[test]
fn duplicate_named_capture_groups_proto_name() {
    assert_eq!(
        test(r"/(?<__proto__>a)|(?<__proto__>b)/;", &options("es2024")).unwrap(),
        "/* @__PURE__ */ babelHelpers.wrapRegExp(/(a)|(b)/, { ['__proto__']: [1, 2] });\n"
    );
}

#[test]
fn duplicate_named_capture_groups_combines_with_other_regexp_transforms() {
    assert_eq!(
        test(r"/(?<x>a)|(?<x>b)/s;", &options("es2017")).unwrap(),
        "/* @__PURE__ */ babelHelpers.wrapRegExp(new RegExp('(a)|(b)', 's'), { x: [1, 2] });\n"
    );
}
