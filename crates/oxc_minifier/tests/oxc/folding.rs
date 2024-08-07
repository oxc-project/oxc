use oxc_minifier::CompressOptions;

fn test(source_text: &str, expected: &str) {
    let options =
        CompressOptions { remove_syntax: true, fold_constants: true, ..CompressOptions::default() };
    crate::test(source_text, expected, options);
}

#[test]
fn addition_folding() {
    test("1 + 1", "2");
    test("1 + 1 + 1", "3");
    test("0 + true", "1");
    test("x+''", "x+''");
}

#[test]
fn typeof_folding() {
    test("typeof x === 'undefined'", "typeof x>'u'");
    test("'undefined' === typeof x", "typeof x>'u'");
}

#[test]
fn test_join_vars() {
    let options = CompressOptions { join_vars: false, ..CompressOptions::default() };
    crate::test("var foo = 1; var bar = 2", "var foo=1;var bar=2", options);
    // join_vars: true
    let options = CompressOptions::default();
    crate::test("var foo = 1; var bar = 2", "var foo=1,bar=2", options);
}
