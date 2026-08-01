use oxc_span::SourceType;
use oxc_transformer::TransformOptions;

use crate::test_with_source_type;

#[test]
fn preserve_file_coverage_comment_when_typescript_import_is_removed() {
    // https://github.com/oxc-project/oxc/issues/23667
    // Transform conformance clears comments before codegen, so this cross-cutting
    // parser + transformer + codegen behavior needs the integration harness.
    let cases = [
        (
            "/* v8 ignore file */\nimport unusedDefault, { unusedNamed } from './side-effects';\nexport default {};",
            "/* v8 ignore file */\nexport default {};\n",
        ),
        (
            "/* v8 ignore file */\nimport type { Foo } from './types';\nexport default { typed: 'stuff' } satisfies Foo;",
            "/* v8 ignore file */\nexport default { typed: 'stuff' };\n",
        ),
    ];

    for (source_text, expected) in cases {
        let output =
            test_with_source_type(source_text, SourceType::ts(), &TransformOptions::default())
                .expect("transform should succeed");
        assert_eq!(output, expected);
    }
}
