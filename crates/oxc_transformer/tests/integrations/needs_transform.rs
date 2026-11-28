use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{DecoratorOptions, ESTarget, EnvOptions, TransformOptions};

use crate::{codegen, test};

/// Whether the options would transform `source_text` as a plain JavaScript program.
fn needs_transform(source_text: &str, options: &TransformOptions) -> bool {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
    assert!(ret.diagnostics.is_empty(), "{source_text}");
    let scoping = SemanticBuilder::new().build(&ret.program).semantic.into_scoping();
    options.env.lowers_syntax() || options.decorator.transforms_decorators(&options.env, &scoping)
}

/// Programs using syntax each target group lowers or rejects, decorators, and syntax nothing
/// lowers.
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
    "@dec class C {}",
    "class C { @dec m() {} }",
    "class C { accessor x = 1; }",
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
    for options in &targets {
        for lower in [None, Some(false), Some(true)] {
            for legacy in [false, true] {
                let options = TransformOptions {
                    decorator: DecoratorOptions { lower, legacy, ..DecoratorOptions::default() },
                    ..options.clone()
                };
                for source_text in PROGRAMS {
                    if !needs_transform(source_text, &options) {
                        assert_eq!(
                            test(source_text, &options),
                            Ok(codegen(source_text, SourceType::mjs())),
                            "{source_text} was skipped but the transformer changed it",
                        );
                    }
                }
            }
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
    // Every concrete target lowers standard decorators, but that alone does not count.
    let es2026 = EnvOptions::from_target("es2026").unwrap();
    assert!(es2026.standard_decorators);

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

#[test]
fn transforms_decorators() {
    let decorated = "@dec class C {}";
    let member_decorated = "class C { @dec m() {} }";
    let accessor = "class C { accessor x = 1; }";
    let plain = "class C { m() {} }";

    let es2026 = TransformOptions::from(ESTarget::ES2026);
    let esnext = TransformOptions::from(ESTarget::ESNext);
    let with = |options: &TransformOptions, lower: Option<bool>, legacy: bool| TransformOptions {
        decorator: DecoratorOptions { lower, legacy, ..DecoratorOptions::default() },
        ..options.clone()
    };

    for (options, lowers) in [
        // The target decides when `lower` is omitted.
        (with(&es2026, None, false), true),
        (with(&esnext, None, false), false),
        (TransformOptions::default(), false),
        // `lower` overrides the target.
        (with(&es2026, Some(false), false), false),
        (with(&esnext, Some(true), false), true),
        // `legacy` overrides `lower` and the target.
        (with(&esnext, None, true), true),
        (with(&es2026, Some(false), true), true),
    ] {
        assert!(!options.env.lowers_syntax());
        for source_text in [decorated, member_decorated] {
            assert_eq!(needs_transform(source_text, &options), lowers, "{source_text}");
        }
        // Programs without decorators or `accessor` are never transformed for them.
        assert!(!needs_transform(plain, &options));
    }

    // Standard decorators lower auto-accessors.
    assert!(needs_transform(accessor, &es2026));
    assert!(!needs_transform(accessor, &with(&es2026, Some(false), false)));
    // Legacy decorators lower auto-accessors at any target.
    let legacy = with(&esnext, None, true);
    assert!(needs_transform(accessor, &legacy));
    assert_ne!(test(accessor, &legacy), Ok(codegen(accessor, SourceType::mjs())));
}
