#![feature(let_chains)]
#![allow(clippy::too_many_lines)]

mod closure;
mod esbuild;
mod oxc;
mod tdewolff;
mod terser;

use oxc_minifier::{CompressOptions, Minifier, MinifierOptions, PrinterOptions};
use oxc_span::SourceType;

pub(crate) fn test(source_text: &str, expected: &str) {
    let options = MinifierOptions { mangle: false, ..MinifierOptions::default() };
    test_with_options(source_text, expected, options);
}

pub(crate) fn test_with_options(source_text: &str, expected: &str, options: MinifierOptions) {
    let source_type = SourceType::default();
    let minified = Minifier::new(source_text, source_type, options).build();
    assert_eq!(expected, minified, "for source {source_text}");
}

pub(crate) fn test_same(source_text: &str) {
    test(source_text, source_text);
}

pub(crate) fn test_reparse(source_text: &str) {
    let source_type = SourceType::default();
    let options = MinifierOptions { mangle: false, ..MinifierOptions::default() };
    let minified = Minifier::new(source_text, source_type, options).build();
    let minified2 = Minifier::new(&minified, source_type, options).build();
    assert_eq!(minified, minified2, "for source {source_text}");
}

pub(crate) fn test_without_compress_booleans(source_text: &str, expected: &str) {
    let source_type = SourceType::default();
    let compress_options = CompressOptions { booleans: false, ..CompressOptions::default() };
    let options =
        MinifierOptions { mangle: false, compress: compress_options, print: PrinterOptions };
    let minified = Minifier::new(source_text, source_type, options).build();
    assert_eq!(expected, minified, "for source {source_text}");
}
