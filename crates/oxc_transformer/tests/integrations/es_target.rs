use std::str::FromStr;

use crate::{codegen, test};
use oxc_span::SourceType;
use oxc_transformer::{ESTarget, TransformOptions};

#[test]
fn es_target() {
    use std::fmt::Write;

    let cases = [
        ("es5", "() => {}"),
        ("es2015", "a ** b"),
        ("es2016", "async function foo() {}"),
        ("es2017", "({ ...x })"),
        ("es2018", "try {} catch {}"),
        ("es2019", "a ?? b"),
        ("es2020", "a ||= b"),
        ("es2021", "class foo { static {} }"),
    ];

    // Test no transformation for esnext.
    for (_, case) in cases {
        let options = TransformOptions::from(ESTarget::from_str("esnext").unwrap());
        assert_eq!(codegen(case, SourceType::mjs()), test(case, options));
    }

    let snapshot = cases.iter().enumerate().fold(String::new(), |mut w, (i, (target, case))| {
        let options = TransformOptions::from(ESTarget::from_str(target).unwrap());
        let result = test(case, options);
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
