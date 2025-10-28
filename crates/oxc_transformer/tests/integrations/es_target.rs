use std::str::FromStr;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{ESTarget, TransformOptions, Transformer};

use crate::{codegen, test};

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
        ("chrome89", r#"export { foo as "string-name" };"#), // test arbitrary module namespace names warning
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
                    .map(|err| format!("{:?}", err.with_source_code(case.to_string())))
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
        assert_eq!(result.unwrap_err().to_string(), expected);
    }
}

#[test]
fn arbitrary_module_namespace_names_warning() {
    // Test that warning is emitted for Chrome 89 (which doesn't support the feature)
    let source = r#"const foo = 1; export { foo as "string-name" };"#;
    let options = TransformOptions::from_target("chrome89").unwrap();

    // Debug: print the option value
    eprintln!(
        "arbitrary_module_namespace_names: {}",
        options.env.es2020.arbitrary_module_namespace_names
    );

    // For module code, we need to use mjs source type
    let source_type = SourceType::mjs();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, source_type).parse();
    let mut program = ret.program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let ret = Transformer::new(&allocator, std::path::Path::new("test.mjs"), &options)
        .build_with_scoping(scoping, &mut program);

    // Should have warnings
    assert!(!ret.errors.is_empty(), "Expected warnings for arbitrary module namespace names");

    // Check that the warning message is correct
    let error_msg = format!("{:?}", ret.errors[0]);
    assert!(error_msg.contains("Arbitrary module namespace identifier names"));
}
