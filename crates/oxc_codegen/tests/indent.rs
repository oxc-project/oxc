mod common;

use common::test_opt;
use oxc_codegen::{CodegenOptions, Indent};

const TABS: CodegenOptions = CodegenOptions::new().with_single_quotes();
const SPACES2: CodegenOptions = TABS.with_indent(Indent::spaces(2));
const SPACES4: CodegenOptions = TABS.with_indent(Indent::spaces(4));

#[test]
fn test_if_statement() {
    let source = r#"if (true) { console.log("true"); }"#;

    // tabs
    let expected = "if (true) {\n\tconsole.log('true');\n}\n";

    test_opt(source, &expected, TABS);

    // 2 spaces
    let expected = "if (true) {\n  console.log('true');\n}\n";
    test_opt(source, &expected, SPACES2);

    // 4 spaces
    let expected = "if (true) {\n    console.log('true');\n}\n";
    test_opt(source, &expected, SPACES4);
}
