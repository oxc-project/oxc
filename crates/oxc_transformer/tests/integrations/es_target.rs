use std::str::FromStr;

use oxc_span::SourceType;
use oxc_transformer::{DecoratorOptions, ESTarget, TransformOptions};

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
                    .map(|error| error.render_with_source_code(case.to_string()))
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
        DecoratorOptions { lower: false, ..DecoratorOptions::default() },
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
fn standard_decorators_default_and_legacy_selection() {
    let source = "@decorate class Example { @decorate method() {} }";
    let default_options = TransformOptions::from(ESTarget::ESNext);
    let standard_output = test(source, &default_options).unwrap();
    assert!(!standard_output.contains("@decorate"));
    assert!(standard_output.contains("decorateElement"));
    for decorator in [
        serde_json::from_str::<DecoratorOptions>("{}").unwrap(),
        serde_json::from_str::<DecoratorOptions>(r#"{"lower":true}"#).unwrap(),
    ] {
        let options = TransformOptions { decorator, ..default_options.clone() };
        assert_eq!(test(source, &options), Ok(standard_output.clone()));
    }

    let mut legacy_options = default_options;
    legacy_options.decorator.legacy = true;
    let legacy_output = test(source, &legacy_options).unwrap();
    assert!(!legacy_output.contains("@decorate"));
    assert!(legacy_output.contains("_decorate("));
    assert!(!legacy_output.contains("decorateElement"));
    legacy_options.decorator.lower = false;
    assert_eq!(test(source, &legacy_options), Ok(legacy_output));
}

#[test]
fn standard_decorators_compat_targets() {
    let source =
        "@decorate class Example { accessor value = 1; } class Plain { accessor value = 2; }";
    for options in [
        TransformOptions::default(),
        TransformOptions::from_target("esnext").unwrap(),
        TransformOptions::from_target("es2015").unwrap(),
        TransformOptions::from_target("safari16").unwrap(),
        TransformOptions::from_target("chrome999").unwrap(),
    ] {
        assert!(options.env.standard_decorators);
        let output = test(source, &options).unwrap();
        assert!(!output.contains("@decorate"));
        assert!(!output.contains("accessor value"));
        assert!(output.contains("decorateElement"));
        let plain = "class Plain { value = 1; #private = 2; read() { return this.#private; } static { effect(); } }";
        let mut preserve_options = options.clone();
        preserve_options.decorator.lower = false;
        assert_eq!(test(plain, &options), test(plain, &preserve_options));
    }
}

#[test]
fn standard_decorators_opt_out_keeps_other_transforms() {
    let source = "function run() { using resource = acquire(); @decorate class Example { @decorate method() {} } return Example; }";
    let mut options = TransformOptions::from(ESTarget::ES2022);
    options.decorator.lower = false;
    let output = test(source, &options).unwrap();
    assert!(output.contains("@decorate"));
    assert!(!output.contains("using resource"));
    assert!(!output.contains("decorateElement"));
}

#[test]
fn standard_decorators_opt_out_keeps_plain_class_field_lowering() {
    let mut options = TransformOptions::from(ESTarget::ES2015);
    options.decorator.lower = false;
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
