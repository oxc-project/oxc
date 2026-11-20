use std::str::FromStr;

use oxc_span::SourceType;
use oxc_transformer::{BabelOptions, DecoratorOptions, ESTarget, TransformOptions};

use crate::{codegen, test, test_with_source_type};

#[test]
fn es_target() {
    use std::fmt::Write;

    let cases = [
        ("es6", "a ** b"),
        ("es2015", "a ** b"),
        ("es2016", "async function foo() {}"),
        ("es2017", "({ ...x })"),
        ("es2018", "try {} catch {}"),
        ("es2019", "a?.b"),
        ("es2019", "a ?? b"),
        ("es2020", "a ||= b"),
        ("es2019", "1n ** 2n"), // test target error
        ("es2021", "class foo { static {} }"),
        ("es2021", "class Foo { #a; }"),
        ("es2019", r#"export { foo as "string-name" };"#), // test arbitrary module namespace names warning
        ("es2021", "await fetch('/')"),                    // test top-level await target error
        ("es2022", "await fetch('/')"),                    // no error for es2022
        ("es2021", "async function f() { await fetch('/'); }"), // no error inside async function
        ("es2021", "{ await fetch('/'); }"), // test top-level await in block, should also error
    ];

    // Test no transformation for esnext.
    let options = TransformOptions::from(ESTarget::from_str("esnext").unwrap());
    for (_, case) in cases {
        assert_eq!(test(case, &options), Ok(codegen(case, SourceType::mjs())));
    }

    #[cfg_attr(miri, expect(unused_variables))]
    let snapshot =
        cases.into_iter().enumerate().fold(String::new(), |mut w, (i, (target, case))| {
            let options = TransformOptions::from_target(target).unwrap();
            let result = match test(case, &options) {
                Ok(code) => code,
                Err(errors) => errors
                    .into_iter()
                    .map(|error| error.render_with_source_code(case))
                    .collect::<Vec<_>>()
                    .join("\n"),
            };
            write!(w, "########## {i} {target}\n{case}\n----------\n{result}\n").unwrap();
            w
        });

    #[cfg(not(miri))]
    {
        insta::with_settings!({ prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
            insta::assert_snapshot!("es_target", snapshot);
        });
    }
}

#[test]
fn standard_decorators_opt_out() {
    let cases = [
        (
            "@decorate class Example<T> { @decorate value: number = 1; @decorate accessor count: number = 2; @decorate method(value: T): T { return value; } }",
            "@decorate class Example { @decorate value = 1; @decorate accessor count = 2; @decorate method(value) { return value; } }",
        ),
        ("class Plain { accessor value: number = 1; }", "class Plain { accessor value = 1; }"),
        (
            "class Example { @decorate static value: number = 1; @decorate item: number = 2; }",
            "class Example { @decorate static value = 1; @decorate item = 2; }",
        ),
        (
            "@decorate class Example { #value: number = 1; read(obj: Example) { obj.#value = 4; return obj.#value; } }",
            "@decorate class Example { #value = 1; read(obj) { obj.#value = 4; return obj.#value; } }",
        ),
    ];
    for decorator in [
        DecoratorOptions { lower: Some(false), ..DecoratorOptions::default() },
        serde_json::from_str::<DecoratorOptions>(r#"{"lower":false}"#).unwrap(),
    ] {
        for target in [ESTarget::ES2015, ESTarget::ES2020, ESTarget::ES2022, ESTarget::ESNext] {
            for set_public_class_fields in [false, true] {
                let mut options = TransformOptions { decorator, ..TransformOptions::from(target) };
                options.assumptions.set_public_class_fields = set_public_class_fields;
                for (source, expected) in cases {
                    assert_eq!(
                        test_with_source_type(source, SourceType::ts(), &options),
                        Ok(codegen(expected, SourceType::mjs())),
                        "{target}, set_public_class_fields={set_public_class_fields}",
                    );
                }
            }
        }
    }
}

#[test]
fn standard_decorators_follow_target() {
    let source = "@decorate class Example { @decorate method() {} }";

    // An unspecified target and `esnext` preserve proposal syntax.
    for options in [TransformOptions::default(), TransformOptions::from(ESTarget::ESNext)] {
        for decorator in [
            DecoratorOptions::default(),
            DecoratorOptions { lower: Some(false), ..DecoratorOptions::default() },
        ] {
            let options = TransformOptions { decorator, ..options.clone() };
            let preserved = test(source, &options).unwrap();
            assert!(preserved.contains("@decorate"));
            assert!(!preserved.contains("decorateElement"));
        }
    }

    let explicit = TransformOptions {
        decorator: serde_json::from_str::<DecoratorOptions>(r#"{"lower":true}"#).unwrap(),
        ..TransformOptions::from(ESTarget::ESNext)
    };
    let lowered = test(source, &explicit).unwrap();
    assert!(!lowered.contains("@decorate"));
    assert!(lowered.contains("decorateElement"));

    // Legacy decorators lower regardless of the standard option.
    let mut legacy_options = TransformOptions::from(ESTarget::ESNext);
    legacy_options.decorator.legacy = true;
    let legacy_output = test(source, &legacy_options).unwrap();
    assert!(!legacy_output.contains("@decorate"));
    assert!(legacy_output.contains("_decorate("));
    assert!(!legacy_output.contains("decorateElement"));
    legacy_options.decorator.lower = Some(false);
    assert_eq!(test(source, &legacy_options), Ok(legacy_output));
}

#[test]
fn standard_decorators_compat_targets() {
    let source =
        "@decorate class Example { accessor value = 1; } class Plain { accessor value = 2; }";
    let plain = "class Plain { value = 1; #private = 2; read() { return this.#private; } static { effect(); } }";
    for (options, lowers) in [
        (TransformOptions::default(), false),
        (TransformOptions::from_target("esnext").unwrap(), false),
        (TransformOptions::from_target("es2015").unwrap(), true),
        (TransformOptions::from_target("safari16").unwrap(), true),
        (TransformOptions::from_target("chrome999").unwrap(), true),
    ] {
        assert_eq!(options.env.standard_decorators, lowers);
        let output = test(source, &options).unwrap();
        assert_eq!(output.contains("@decorate"), !lowers);
        assert_eq!(output.contains("accessor value"), !lowers);
        assert_eq!(output.contains("decorateElement"), lowers);
        let mut preserve_options = options.clone();
        preserve_options.decorator.lower = Some(false);
        assert_eq!(test(plain, &options), test(plain, &preserve_options));
    }
}

#[test]
fn standard_decorators_skip_sources_without_decorator_syntax() {
    // Neither `@` nor `accessor`: decorator lowering is skipped entirely. The output must match
    // both an explicit opt-out and the full path, which a `@` in a comment forces.
    let plain = "for (let i = 0; i < 2; i++) list.push(class { static x = i; #y = 1; static { effect(this); } get y() { return this.#y; } });\nclass Plain { value = 1; static #count = 0; static next() { return ++Plain.#count; } }\n";
    let with_comment = format!("{plain}// @ in a comment\n");
    for target in ["es2015", "es2022", "chrome94", "safari15"] {
        let options = TransformOptions::from_target(target).unwrap();
        let mut preserve = options.clone();
        preserve.decorator.lower = Some(false);
        let fast = test(plain, &options).unwrap();
        assert_eq!(Ok(fast.clone()), test(plain, &preserve), "{target}");
        let full = test(&with_comment, &options).unwrap();
        let full = full.trim_end().trim_end_matches("// @ in a comment").trim_end();
        assert_eq!(full, fast.trim_end(), "{target}");
    }

    // `@` in package specifiers and comments keeps the fast path, but decorators anywhere else,
    // even directly after a keyword, are still lowered.
    let options = TransformOptions::from_target("es2022").unwrap();
    for source in [
        "import x from '@scope/pkg'; /** @internal */ export default@dec class {}",
        "function f() { return@dec class {}; } // @",
        "let s = `@`; [@dec class {}];",
    ] {
        let output = test(source, &options).unwrap();
        assert!(output.contains("decorateElement"), "{source}");
        assert!(!output.contains("@dec"), "{source}");
    }

    // `accessor` alone still lowers.
    let output =
        test("class A { accessor x = 1; }", &TransformOptions::from_target("es2022").unwrap())
            .unwrap();
    assert!(!output.contains("accessor x"));
}

#[test]
fn standard_decorators_opt_out_keeps_other_transforms() {
    let source = "function run() { using resource = acquire(); @decorate class Example { @decorate method() {} } return Example; }";
    let mut options = TransformOptions::from(ESTarget::ES2022);
    options.decorator.lower = Some(false);
    let output = test(source, &options).unwrap();
    assert!(output.contains("@decorate"));
    assert!(!output.contains("using resource"));
    assert!(!output.contains("decorateElement"));
}

#[test]
fn standard_decorators_opt_out_keeps_plain_class_field_lowering() {
    let mut options = TransformOptions::from(ESTarget::ES2015);
    options.decorator.lower = Some(false);
    let output =
        test("class Plain { value = 1; } class Example { @decorate value = 1; }", &options)
            .unwrap();
    assert!(output.contains("constructor()"));
    assert!(output.contains("@decorate value = 1;"));
    let output = test_with_source_type(
        "class Outer { #value = 1; make() { return class Inner { accessor x = 2; #value = 3; read(obj: Inner) { obj.#value = 4; return obj.#value; } }; } }",
        SourceType::ts(),
        &options,
    ).unwrap();
    assert!(output.contains("accessor x = 2;"));
    assert!(output.contains("#value = 3;"));
    assert!(output.contains("obj.#value = 4;"));
    assert!(output.contains("return obj.#value;"));
}

#[test]
fn babel_options_standard_decorators() {
    let source = "@decorate class Example { @decorate method() {} }";
    let from = |json: &str| {
        TransformOptions::try_from(&serde_json::from_str::<BabelOptions>(json).unwrap())
    };

    // `preset-env` does not include the decorators proposal, so its targets alone preserve it.
    let options = from(r#"{ "presets": [["env", { "targets": { "chrome": "80" } }]] }"#).unwrap();
    assert!(options.env.standard_decorators);
    let output = test(source, &options).unwrap();
    assert!(output.contains("@decorate"));
    assert!(!output.contains("decorateElement"));

    // Listing the plugin lowers them. The helpers only exist in `@oxc-project/runtime`, so Babel's
    // default external helpers are turned off.
    for version in ["", r#", { "version": "2023-11" }"#] {
        let options = from(&format!(
            r#"{{ "plugins": [["proposal-decorators"{version}]], "externalHelpers": false }}"#
        ))
        .unwrap();
        let output = test(source, &options).unwrap();
        assert!(!output.contains("@decorate"));
        assert!(output.contains("decorateElement"));
    }

    // Other proposal versions have different semantics and are rejected.
    let errors =
        from(r#"{ "plugins": [["proposal-decorators", { "version": "2022-03" }]] }"#).unwrap_err();
    assert_eq!(
        errors,
        vec![r#"proposal-decorators: version "2022-03" is not supported, only "2023-11" is"#]
    );
}

#[test]
fn target_list_pass() {
    // https://vite.dev/config/build-options.html#build-target
    let target = "es2020,edge88,firefox78,chrome87,safari14";
    let result = TransformOptions::from_target(target).unwrap();
    assert!(!result.env.es2019.optional_catch_binding);
    assert!(!result.env.es2020.nullish_coalescing_operator);
    assert!(result.env.es2021.logical_assignment_operators);
    assert!(result.env.es2022.class_static_block);
}

#[test]
fn target_list_fail() {
    let targets = [
        ("asdf", "Invalid target 'asdf'."),
        ("es2020,es2020", "'es2020' is already specified."),
        ("chrome1,chrome1", "'chrome1' is already specified."),
        (
            "chromeXXX",
            "All version numbers must be in the format \"X\", \"X.Y\", or \"X.Y.Z\" where X, Y, and Z are non-negative integers.",
        ),
    ];

    for (target, expected) in targets {
        let result = TransformOptions::from_target(target);
        assert_eq!(result.unwrap_err(), expected);
    }
}
