use oxc_minifier::CompressOptions;

fn test(source_text: &str, expected: &str) {
    let options = CompressOptions { remove_syntax: true, ..CompressOptions::all_false() };
    crate::test(source_text, expected, options);
}

#[test]
fn parens() {
    test("(((x)))", "x");
    test("(((a + b))) * c", "(a + b) * c");
}

#[test]
fn drop_console() {
    let options =
        CompressOptions { remove_syntax: true, drop_console: true, ..CompressOptions::all_false() };
    crate::test("console.log()", "", options);
}

#[test]
fn drop_debugger() {
    let options = CompressOptions {
        remove_syntax: true,
        drop_debugger: true,
        ..CompressOptions::all_false()
    };
    crate::test("debugger", "", options);
}
