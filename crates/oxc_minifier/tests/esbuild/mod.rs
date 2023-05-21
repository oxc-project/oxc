use oxc_minifier::{Minifier, MinifierOptions};
use oxc_span::SourceType;

fn expect_minify(source_text: &str, expected: &str) {
    let source_type = SourceType::default();
    let options = MinifierOptions { mangle: false, ..MinifierOptions::default() };
    let minified = Minifier::new(source_text, source_type, options).build();
    assert_eq!(expected, minified);
}

#[test]
#[ignore]
fn number() {
    expect_minify("x = 1e-100", "x=1e-100");
    expect_minify("x = 1e-5", "x=1e-5");
    expect_minify("x = 1e-4", "x=1e-4");
    expect_minify("x = 1e-3", "x=.001");
    expect_minify("x = 1e-2", "x=.01");
    expect_minify("x = 1e-1", "x=.1");
    expect_minify("x = 1e0", "x=1");
    expect_minify("x = 1e1", "x=10");
    expect_minify("x = 1e2", "x=100");
    expect_minify("x = 1e3", "x=1e3");
    expect_minify("x = 1e4", "x=1e4");
    expect_minify("x = 1e100", "x=1e100");
}
