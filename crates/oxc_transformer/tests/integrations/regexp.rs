use oxc_transformer::{HelperLoaderMode, TransformOptions};

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
