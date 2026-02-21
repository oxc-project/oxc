use oxc_span::SourceType;
use oxc_transformer::TransformOptions;

use crate::test_with_source_type;

#[test]
fn jsx_frag_quoted_string_pragma_in_comment() {
    let source = "// @jsxRuntime classic\n// @jsxFrag '['\nconsole.log(<></>);\n";
    let output = test_with_source_type(source, SourceType::jsx(), &TransformOptions::default())
        .expect("transform should succeed");

    assert!(output.contains("React.createElement"));
    assert!(output.contains(", null"));
    assert!(!output.contains("React.Fragment"));
    assert!(
        output.contains("React.createElement(\"[\", null)")
            || output.contains("React.createElement('[', null)")
    );
}

#[test]
fn jsx_frag_invalid_unquoted_pragma_falls_back_to_default() {
    let source = "// @jsxRuntime classic\n// @jsxFrag [\nconsole.log(<></>);\n";
    let output = test_with_source_type(source, SourceType::jsx(), &TransformOptions::default())
        .expect("transform should succeed");

    assert!(output.contains("React.createElement(React.Fragment, null)"));
}
