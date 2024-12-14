use oxc_span::SourceType;
use oxc_transformer::{ESTarget, EnvOptions, TransformOptions};

use crate::{codegen, test};

#[test]
fn targets() {
    let cases = [
        ("() => {}"),
        ("a ** b"),
        ("async function foo() {}"),
        ("({ ...x })"),
        ("try {} catch {}"),
        ("a ?? b"),
        ("a ||= b"),
        "1n ** 2n",
    ];

    // Test no transformation for default targets.
    let options = TransformOptions {
        env: EnvOptions::from_browserslist_query("defaults").unwrap(),
        ..TransformOptions::default()
    };
    for case in cases {
        assert_eq!(Ok(codegen(case, SourceType::mjs())), test(case, &options));
    }

    // Test transformation for very low targets.
    let options = TransformOptions::from(ESTarget::ES5);
    let options_node = TransformOptions {
        env: EnvOptions::from_browserslist_query("node 0.10").unwrap(),
        ..TransformOptions::default()
    };
    for case in cases {
        assert_eq!(test(case, &options), test(case, &options_node));
    }
}
