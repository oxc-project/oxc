use crate::{codegen, test};
use oxc_span::SourceType;
use oxc_transformer::{ESTarget, EnvOptions, TransformOptions};

#[test]
fn targets() {
    let cases = [
        ("() => {}"),
        ("a ** b"),
        // ("async function foo() {}"),
        ("({ ...x })"),
        ("try {} catch {}"),
        ("a ?? b"),
        ("a ||= b"),
        // ("class foo { static {} }"),
    ];

    // Test no transformation for default targets.
    for case in cases {
        let options = TransformOptions {
            env: EnvOptions::from_browerslist_query("defaults").unwrap(),
            ..TransformOptions::default()
        };
        assert_eq!(codegen(case, SourceType::mjs()), test(case, options));
    }

    // Test transformation for very low targets.
    for case in cases {
        let options = TransformOptions::from(ESTarget::ES5);
        let options_node = TransformOptions {
            env: EnvOptions::from_browerslist_query("node 0.10").unwrap(),
            ..TransformOptions::default()
        };
        assert_eq!(test(case, options), test(case, options_node));
    }
}
