use oxc_span::SourceType;
use oxc_transformer::{ESTarget, EnvOptions, TransformOptions};

use crate::{codegen, test};

#[test]
fn baseline_targets() {
    // Use fixed dates and years for feature boundaries that change over time.
    for (query, target, source_text) in [
        ("baseline newly available", ESTarget::ES2026, "using x = acquire();"),
        ("baseline widely available on 2026-08-15", ESTarget::ES2025, "using x = acquire();"),
        ("baseline 2020", ESTarget::ES2021, "class C { static { foo(); } }"),
        ("baseline widely available on 2022-07-01", ESTarget::ES2020, "a ||= b;"),
        ("baseline widely available on 2022-07-01", ESTarget::ES2017, "({ ...x });"),
    ] {
        let options = TransformOptions {
            env: EnvOptions::from_browserslist_query(query).unwrap(),
            ..TransformOptions::default()
        };
        assert_eq!(
            test(source_text, &TransformOptions::from(target)),
            test(source_text, &options),
            "{query} should match {target} for {source_text}",
        );
    }
}

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

    // Test no transformation for modern targets.
    for query in [
        "defaults",
        "baseline widely available",
        "baseline newly available",
        "baseline 2020",
        "baseline widely available with downstream",
        "baseline widely available including kaios",
    ] {
        let options = TransformOptions {
            env: EnvOptions::from_browserslist_query(query).unwrap(),
            ..TransformOptions::default()
        };
        for case in cases {
            assert_eq!(
                Ok(codegen(case, SourceType::mjs())),
                test(case, &options),
                "{query}: {case}",
            );
        }
    }

    // Test transformation for very low targets.
    let options = TransformOptions::from(ESTarget::ES2015);
    let options_node = TransformOptions {
        env: EnvOptions::from_browserslist_query("node 6").unwrap(),
        ..TransformOptions::default()
    };
    for case in cases {
        assert_eq!(test(case, &options), test(case, &options_node));
    }
}
