use oxc_span::SourceType;
use oxc_transformer::{ESTarget, EnvOptions, TransformOptions};

use crate::{codegen, test};

/// Programs using syntax each target group lowers or rejects, and syntax nothing lowers.
const PROGRAMS: &[&str] = &[
    "using x = acquire();",
    "class C { static { foo(); } }",
    "class C { #a = 1; x = 2; }",
    "await fetch('/');",
    "a ||= b;",
    "export * as ns from 'mod';",
    "a ?? b;",
    "a?.b;",
    "1n;",
    "export { foo as 'string-name' }; let foo;",
    "try {} catch {}",
    "({ ...x });",
    "async function* f() {}",
    "async function f() {}",
    "a ** b;",
    "() => {};",
    "/a/y;",
    "/a/u;",
    r"/\p{L}/u;",
    "/./s;",
    "/(?<a>x)/;",
    "/(?<=a)b/;",
    "/a/d;",
    "/[\\p{L}--a]/v;",
    "/(?<a>x)|(?<a>y)/;",
    "class C { m() {} }",
    "let x = 1;",
];

#[test]
fn skipping_leaves_program_unchanged() {
    let mut targets = vec![TransformOptions::default()];
    targets.extend(
        [
            ESTarget::ES2015,
            ESTarget::ES2016,
            ESTarget::ES2017,
            ESTarget::ES2018,
            ESTarget::ES2019,
            ESTarget::ES2020,
            ESTarget::ES2021,
            ESTarget::ES2022,
            ESTarget::ES2023,
            ESTarget::ES2024,
            ESTarget::ES2025,
            ESTarget::ES2026,
            ESTarget::ESNext,
        ]
        .map(TransformOptions::from),
    );
    for options in targets.iter().filter(|options| !options.env.lowers_syntax()) {
        for source_text in PROGRAMS {
            assert_eq!(
                test(source_text, options),
                Ok(codegen(source_text, SourceType::mjs())),
                "{source_text} was skipped but the transformer changed it",
            );
        }
    }
}

#[test]
fn lowers_syntax() {
    assert!(!EnvOptions::default().lowers_syntax());
    for target in ["esnext", "es2026"] {
        assert!(!EnvOptions::from_target(target).unwrap().lowers_syntax(), "{target}");
    }
    for target in [
        "es2015", "es2016", "es2017", "es2018", "es2019", "es2020", "es2021", "es2022", "es2023",
        "es2024", "es2025",
    ] {
        assert!(EnvOptions::from_target(target).unwrap().lowers_syntax(), "{target}");
    }

    let es2026 = EnvOptions::from_target("es2026").unwrap();
    // RegExp lowering alone counts. No target has this combination yet.
    for set in [
        |env: &mut EnvOptions| env.regexp.sticky_flag = true,
        |env: &mut EnvOptions| env.regexp.unicode_flag = true,
        |env: &mut EnvOptions| env.regexp.unicode_property_escapes = true,
        |env: &mut EnvOptions| env.regexp.dot_all_flag = true,
        |env: &mut EnvOptions| env.regexp.named_capture_groups = true,
        |env: &mut EnvOptions| env.regexp.look_behind_assertions = true,
        |env: &mut EnvOptions| env.regexp.match_indices = true,
        |env: &mut EnvOptions| env.regexp.set_notation = true,
    ] {
        let mut env = es2026;
        set(&mut env);
        assert!(env.lowers_syntax());
    }
}
