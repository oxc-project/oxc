#![feature(let_chains)]
#![allow(clippy::too_many_lines)]

mod closure;
mod esbuild;
mod tdewolff;
mod terser;

use oxc_minifier::{Minifier, MinifierOptions};
use oxc_span::SourceType;

pub(crate) fn expect(source_text: &str, expected: &str) {
    let source_type = SourceType::default();
    let options = MinifierOptions { mangle: false, ..MinifierOptions::default() };
    let minified = Minifier::new(source_text, source_type, options).build();
    assert_eq!(expected, minified, "for source {source_text}");
}

pub(crate) fn expect_same(source_text: &str) {
    expect(source_text, source_text);
}

pub(crate) fn expect_reparse(source_text: &str) {
    let source_type = SourceType::default();
    let options = MinifierOptions { mangle: false, ..MinifierOptions::default() };
    let minified = Minifier::new(source_text, source_type, options).build();
    let minified2 = Minifier::new(&minified, source_type, options).build();
    assert_eq!(minified, minified2, "for source {source_text}");
}
