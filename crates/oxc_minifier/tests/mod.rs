#![allow(clippy::literal_string_with_formatting_args)]

mod ecmascript;
mod mangler;
mod peephole;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_compat::EngineTargets;
use oxc_parser::{ParseOptions, Parser};
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

pub(crate) use oxc_minifier::{
    CompressOptions, CompressOptionsKeepNames, CompressOptionsUnused, Compressor, TreeShakeOptions,
};

pub(crate) fn default_options() -> CompressOptions {
    CompressOptions {
        drop_debugger: false,
        unused: CompressOptionsUnused::Keep,
        ..CompressOptions::smallest()
    }
}

fn get_targets(target_list: &str) -> EngineTargets {
    EngineTargets::from_target(target_list).unwrap()
}

#[track_caller]
pub(crate) fn test(source_text: &str, expected: &str) {
    test_options(source_text, expected, &default_options());
}

#[track_caller]
pub(crate) fn test_same(source_text: &str) {
    test(source_text, source_text);
}

#[track_caller]
pub(crate) fn test_target(source_text: &str, expected: &str, target: &str) {
    let options = CompressOptions { target: get_targets(target), ..default_options() };
    test_options(source_text, expected, &options);
}

#[track_caller]
pub(crate) fn test_target_same(source_text: &str, target: &str) {
    test_target(source_text, source_text, target);
}

#[track_caller]
pub(crate) fn test_options(source_text: &str, expected: &str, options: &CompressOptions) {
    test_options_source_type(source_text, expected, SourceType::mjs(), options);
}

/// Assert output and iteration count, including the usual idempotency check.
#[track_caller]
pub(crate) fn test_options_with_iterations(
    source_text: &str,
    expected: &str,
    expected_iterations: u8,
    options: &CompressOptions,
) {
    let (first, iterations) =
        run_with_iterations(source_text, SourceType::mjs(), Some(options.clone()));
    let expected = run(expected, SourceType::mjs(), None);
    assert_eq!(first, expected, "\nfor source\n{source_text}\nexpect\n{expected}\ngot\n{first}");
    assert_eq!(
        iterations, expected_iterations,
        "\niteration count for source\n{source_text}\nexpect\n{expected_iterations}\ngot\n{iterations}"
    );

    let second = run(&first, SourceType::mjs(), Some(options.clone()));
    assert_eq!(
        first, second,
        "\nidempotency for source\n{source_text}\ngot\n{first}\nthen\n{second}"
    );
}

/// Assert one capped compression run and its iteration count without an
/// idempotency check. A deliberately low cap can conservatively retain code.
#[track_caller]
pub(crate) fn test_options_once_with_iterations(
    source_text: &str,
    expected: &str,
    expected_iterations: u8,
    options: &CompressOptions,
) {
    let (actual, iterations) =
        run_with_iterations(source_text, SourceType::mjs(), Some(options.clone()));
    let expected = run(expected, SourceType::mjs(), None);
    assert_eq!(actual, expected, "\nfor source\n{source_text}\nexpect\n{expected}\ngot\n{actual}");
    assert_eq!(
        iterations, expected_iterations,
        "\niteration count for source\n{source_text}\nexpect\n{expected_iterations}\ngot\n{iterations}"
    );
}

#[track_caller]
pub(crate) fn test_same_options(source_text: &str, options: &CompressOptions) {
    test_options(source_text, source_text, options);
}

#[track_caller]
pub(crate) fn test_smallest(source_text: &str, expected: &str) {
    test_options(source_text, expected, &CompressOptions::smallest());
}

#[track_caller]
pub(crate) fn test_same_smallest(source_text: &str) {
    test_smallest(source_text, source_text);
}

#[track_caller]
pub(crate) fn test_options_source_type(
    source_text: &str,
    expected: &str,
    source_type: SourceType,
    options: &CompressOptions,
) {
    let first = run(source_text, source_type, Some(options.clone()));
    let expected = run(expected, source_type, None);
    assert_eq!(first, expected, "\nfor source\n{source_text}\nexpect\n{expected}\ngot\n{first}");

    // Check idempotency
    let second = run(&first, source_type, Some(options.clone()));
    assert_eq!(
        first, second,
        "\nidempotency for source\n{source_text}\ngot\n{first}\nthen\n{second}"
    );
}

#[track_caller]
pub(crate) fn test_same_options_source_type(
    source_text: &str,
    source_type: SourceType,
    options: &CompressOptions,
) {
    test_options_source_type(source_text, source_text, source_type, options);
}

#[track_caller]
fn run(source_text: &str, source_type: SourceType, options: Option<CompressOptions>) -> String {
    run_with_iterations(source_text, source_type, options).0
}

#[track_caller]
fn run_with_iterations(
    source_text: &str,
    source_type: SourceType,
    options: Option<CompressOptions>,
) -> (String, u8) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type)
        .with_options(ParseOptions { allow_return_outside_function: true, ..Default::default() })
        .parse();
    assert!(!ret.panicked, "{source_text}");
    assert!(ret.diagnostics.is_empty(), "{source_text}");
    let mut program = ret.program;
    let iterations = options.map_or(0, |options| {
        let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
        Compressor::new(&allocator).build_with_scoping(&mut program, scoping, options)
    });
    let code = Codegen::new()
        .with_options(CodegenOptions { single_quote: true, ..Default::default() })
        .build(&program)
        .code;
    (code, iterations)
}
