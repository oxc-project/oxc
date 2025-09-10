use std::str::FromStr;

use oxc_span::SourceType;
use oxc_transformer::{ESTarget, TransformOptions};

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
        ("es2020", "using disposable = new Resource();"), // test using syntax
        ("es2021", "await using asyncDisposable = new AsyncResource();"), // test await using syntax
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
fn test_using_with_esnext() {
    let source = "using disposable = new Resource();";
    
    // Test with esnext - should NOT transform
    let options = TransformOptions::from(ESTarget::from_str("esnext").unwrap());
    let result = test(source, &options).unwrap();
    
    // Should be unchanged
    assert_eq!(result.trim(), "using disposable = new Resource();");
    
    // Test with es2020 - should transform
    let options = TransformOptions::from(ESTarget::from_str("es2020").unwrap());
    let result = test(source, &options).unwrap();
    
    // Should be transformed (different from original)
    assert_ne!(result.trim(), "using disposable = new Resource();");
    println!("ES2020 transformed result: {}", result);
}

#[test]
fn test_await_using_with_esnext() {
    let source = "await using asyncDisposable = new AsyncResource();";
    
    // Test with esnext - should NOT transform
    let options = TransformOptions::from(ESTarget::from_str("esnext").unwrap());
    let result = test(source, &options).unwrap();
    
    // Should be unchanged
    assert_eq!(result.trim(), "await using asyncDisposable = new AsyncResource();");
    
    // Test with es2020 - should transform
    let options = TransformOptions::from(ESTarget::from_str("es2020").unwrap());
    let result = test(source, &options).unwrap();
    
    // Should be transformed (different from original)
    assert_ne!(result.trim(), "await using asyncDisposable = new AsyncResource();");
    println!("ES2020 await using transformed result: {}", result);
}

#[test]
fn test_playground_example() {
    // This is the exact example from the playground in the issue
    let source = r#"
using resource = {
  [Symbol.dispose]() {
    console.log("Disposing resource");
  }
};

console.log("Using resource");
"#;
    
    // Test with esnext - should NOT transform
    let options = TransformOptions::from(ESTarget::from_str("esnext").unwrap());
    let result = test(source, &options).unwrap();
    
    // Should be unchanged (except for some formatting)
    assert!(result.contains("using resource = {"));
    assert!(result.contains("[Symbol.dispose]() {"));
    assert!(!result.contains("_usingCtx"));
    assert!(!result.contains("try {"));
    
    // Test with es2020 - should transform
    let options = TransformOptions::from(ESTarget::from_str("es2020").unwrap());
    let result = test(source, &options).unwrap();
    
    // Should be transformed
    assert!(!result.contains("using resource = {"));
    assert!(result.contains("_usingCtx"));
    assert!(result.contains("try {"));
    
    println!("ESNext result (no transform):\n{}", test(source, &TransformOptions::from(ESTarget::from_str("esnext").unwrap())).unwrap());
    println!("ES2020 result (transformed):\n{}", test(source, &TransformOptions::from(ESTarget::from_str("es2020").unwrap())).unwrap());
}

#[test]
fn test_from_target_method() {
    let source = "using disposable = new Resource();";
    
    // Test with esnext using from_target method - should NOT transform
    let options = TransformOptions::from_target("esnext").unwrap();
    let result = test(source, &options).unwrap();
    assert_eq!(result.trim(), "using disposable = new Resource();");
    
    // Test with es2020 using from_target method - should transform
    let options = TransformOptions::from_target("es2020").unwrap();
    let result = test(source, &options).unwrap();
    assert_ne!(result.trim(), "using disposable = new Resource();");
    assert!(result.contains("_usingCtx"));
    
    // Test with esnext using from_target_list method - should NOT transform
    let options = TransformOptions::from_target_list(&["esnext"]).unwrap();
    let result = test(source, &options).unwrap();
    assert_eq!(result.trim(), "using disposable = new Resource();");
}

#[test]
fn test_esnext_mixed_with_other_targets() {
    let source = "using disposable = new Resource();";
    
    // Test with mixed targets including esnext - should NOT transform (esnext overrides)
    let options = TransformOptions::from_target_list(&["chrome80", "esnext", "firefox70"]).unwrap();
    let result = test(source, &options).unwrap();
    assert_eq!(result.trim(), "using disposable = new Resource();");
    
    // Test with mixed targets without esnext - should transform
    let options = TransformOptions::from_target_list(&["chrome80", "firefox70", "es2020"]).unwrap();
    let result = test(source, &options).unwrap();
    assert!(result.contains("_usingCtx"));
}

#[test]
fn test_complex_using_scenarios() {
    let source = r#"
function testComplexUsing() {
  using resource1 = createResource();
  await using resource2 = createAsyncResource();
  
  for (using item of items) {
    console.log(item);
  }
  
  for await (using asyncItem of asyncItems) {
    console.log(asyncItem);
  }
}
"#;
    
    // Test with esnext - should NOT transform any using syntax
    let options = TransformOptions::from(ESTarget::from_str("esnext").unwrap());
    let result = test(source, &options).unwrap();
    
    // All using syntax should remain untransformed
    assert!(result.contains("using resource1 = createResource();"));
    assert!(result.contains("await using resource2 = createAsyncResource();"));
    assert!(result.contains("for (using item of items)"));
    assert!(result.contains("for await (using asyncItem of asyncItems)"));
    assert!(!result.contains("_usingCtx"));
    assert!(!result.contains("try {"));
    
    // Test with es2020 - should transform all using syntax
    let options = TransformOptions::from(ESTarget::from_str("es2020").unwrap());
    let result = test(source, &options).unwrap();
    
    // All using syntax should be transformed
    assert!(!result.contains("using resource1 = createResource();"));
    assert!(!result.contains("await using resource2 = createAsyncResource();"));
    assert!(!result.contains("for (using item of items)"));
    assert!(!result.contains("for await (using asyncItem of asyncItems)"));
    assert!(result.contains("_usingCtx"));
    assert!(result.contains("try {"));
}
