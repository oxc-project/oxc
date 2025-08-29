#![allow(clippy::literal_string_with_formatting_args)]

#[path = "../src/tester.rs"]
mod tester;

mod ecmascript;
mod mangler;
mod peephole;

use oxc_minifier::{CompressOptions, CompressOptionsUnused, Compressor};
use oxc_span::SourceType;

pub(crate) use tester::default_options;

#[track_caller]
pub(crate) fn test_same(source_text: &str) {
    test(source_text, source_text);
}

#[track_caller]
pub(crate) fn test(source_text: &str, expected: &str) {
    test_options(source_text, expected, &tester::default_options());
}

#[track_caller]
pub(crate) fn test_options(source_text: &str, expected: &str, options: &CompressOptions) {
    test_options_source_type(source_text, expected, SourceType::mjs(), options);
}

#[track_caller]
pub(crate) fn test_options_source_type(
    source_text: &str,
    expected: &str,
    source_type: SourceType,
    options: &CompressOptions,
) {
    tester::test_options_source_type_with_idempotency(
        source_text,
        expected,
        source_type,
        options,
        true,
    );
}
