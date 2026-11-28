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
    let output = test(source, &options).unwrap();
    assert!(output.contains("@decorate"));
    assert!(!output.contains("decorateElement"));

    // Listing the plugin lowers them.
    for version in ["", r#", { "version": "2023-11" }"#] {
        let options =
            from(&format!(r#"{{ "plugins": [["proposal-decorators"{version}]] }}"#)).unwrap();
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

/// A decorated class whose computed key may collide with a member name gives its public methods
/// temporary keys. Each keeps the source position of the key it replaces, so a source map lookup
/// at a member name finds that name, not the start of the member (`@dec` for a decorated one).
#[test]
fn standard_decorator_temporary_method_keys_keep_source_positions() {
    use std::path::Path;

    use oxc_allocator::Allocator;
    use oxc_codegen::{Codegen, CodegenOptions};
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_transformer::Transformer;

    let source = "const k = 'z';\nclass C {\n  @dec m() {}\n  plain() {}\n  [k]() {}\n}\n";
    let allocator = Allocator::default();
    let mut program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let options = TransformOptions {
        decorator: DecoratorOptions { lower: Some(true), ..DecoratorOptions::default() },
        ..TransformOptions::from(ESTarget::ES2022)
    };
    let ret = Transformer::new(&allocator, Path::new(""), &options)
        .build_with_scoping(scoping, &mut program);
    assert!(ret.diagnostics.is_empty());
    let ret = Codegen::new()
        .with_options(CodegenOptions {
            source_map_path: Some("test.js".into()),
            ..CodegenOptions::default()
        })
        .build(&program);
    let map = ret.map.unwrap();

    // 0-based `(line, column)` of `offset` in `text`.
    let position = |text: &str, offset: usize| {
        let line_start = text[..offset].rfind('\n').map_or(0, |i| i + 1);
        let line = text[..offset].matches('\n').count();
        (u32::try_from(line).unwrap(), u32::try_from(offset - line_start).unwrap())
    };
    // Like a source map consumer: the last mapping at or before `(line, column)` on that line.
    let original = |(line, column): (u32, u32)| {
        map.get_tokens()
            .filter(|t| t.get_dst_line() == line && t.get_dst_col() <= column)
            .map(|t| (t.get_dst_col(), (t.get_src_line(), t.get_src_col())))
            .max_by_key(|&(dst_col, _)| dst_col)
            .map(|(_, src)| src)
    };
    let mut temporary_keys = ret.code.match_indices("[_key").map(|(i, _)| i + 1);
    for member in ["m() {}", "plain() {}", "[k]() {}"] {
        let key = temporary_keys.next().expect("each public method has a temporary key");
        assert_eq!(
            original(position(&ret.code, key)),
            Some(position(source, source.find(member).unwrap())),
            "{member}\n{}",
            ret.code
        );
    }
}
