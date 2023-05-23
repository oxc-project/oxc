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
