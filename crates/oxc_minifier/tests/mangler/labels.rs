use std::fmt::Write;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_mangler::{MangleOptions, Mangler};
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

fn validate(source: &str) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let semantic = SemanticBuilder::new().with_check_syntax_error(true).build(&parsed.program);
    assert!(semantic.diagnostics.is_empty(), "{:?}", semantic.diagnostics);
}

#[test]
fn lexical_labels() {
    for (source, expected) in [
        (
            "longName: while (next()) { if (skip()) continue longName; break longName; }",
            "e: while (next()) { if (skip()) continue e; break e; }",
        ),
        (
            "first: { if (stop()) break first; work(); } second: { break second; }",
            "e: { if (stop()) break e; work(); } e: { break e; }",
        ),
        (
            "outer: inner: for (;;) { if (a()) continue outer; if (b()) continue inner; if (c()) break inner; break outer; }",
            "e: t: for (;;) { if (a()) continue e; if (b()) continue t; if (c()) break t; break e; }",
        ),
        (
            "outer: { inner: switch (value) { case 0: break; default: break outer; } while (next()) { if (skip()) continue; break; } }",
            "e: { t: switch (value) { case 0: break; default: break e; } while (next()) { if (skip()) continue; break; } }",
        ),
        (
            "outer: { (() => { outer: { break outer; } })(); break outer; }",
            "e: { (() => { e: { break e; } })(); break e; }",
        ),
        (
            "outer: { (function() { inner: { break inner; } })(); break outer; }",
            "e: { (function() { e: { break e; } })(); break e; }",
        ),
        (
            "outer: { (class { static { inner: { break inner; } } method() { inner: { break inner; } } }); break outer; }",
            "e: { (class { static { e: { break e; } } method() { e: { break e; } } }); break e; }",
        ),
        (
            "let e = 1; longLabel: { use(e); break longLabel; }",
            "let e = 1; e: { use(e); break e; }",
        ),
        (
            "function example(longName) { longName: { use(longName); break longName; } }",
            "function example(e) { e: { use(e); break e; } }",
        ),
        (
            r"\u006cabel: { break label; } label: { break \u006cabel; }",
            "e: { break e; } e: { break e; }",
        ),
        (
            "(async function* () { outer: for (;;) { yield await next(); continue outer; } })();",
            "(async function* () { e: for (;;) { yield await next(); continue e; } })();",
        ),
    ] {
        validate(source);
        super::test(source, expected, &MangleOptions::default());
        let output = super::mangle(source, &MangleOptions::default());
        validate(&output);
        assert_eq!(super::mangle(&output, &MangleOptions::default()), output);
    }
    super::test(
        "outer: { inner: { break outer; } } sibling: { break sibling; }",
        "slot_0: { slot_1: { break slot_0; } } slot_0: { break slot_0; }",
        &MangleOptions { debug: true, ..MangleOptions::default() },
    );
}

fn minify(source: &str, options: MinifierOptions) -> String {
    let allocator = Allocator::default();
    let mut parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty());
    let result = Minifier::new(options).minify(&allocator, &mut parsed.program);
    Codegen::new()
        .with_options(CodegenOptions::minify())
        .with_scoping(result.scoping)
        .build(&parsed.program)
        .code
}

#[test]
fn retained_labels_and_drop_labels() {
    let source = "export function search(rows) { outerSearch: for (const row of rows) { for (const item of row) { if (item === 0) continue outerSearch; if (item === 1) break outerSearch; visit(item); } } }";
    let unmangled = minify(source, MinifierOptions { mangle: None, ..MinifierOptions::default() });
    assert_eq!(unmangled.matches("outerSearch").count(), 3);
    let mangled = minify(source, MinifierOptions::default());
    assert!(!mangled.contains("outerSearch"));
    assert!(mangled.contains("continue e"));
    assert!(mangled.contains("break e"));
    validate(&mangled);
    assert_eq!(minify(&mangled, MinifierOptions::default()), mangled);

    let source =
        r"\u0044EBUG: { removed(); } keep: while (next()) { if (stop()) break keep; work(); }";
    let output = minify(
        source,
        MinifierOptions {
            compress: Some(CompressOptions {
                drop_labels: ["DEBUG".to_string(), "e".to_string()].into_iter().collect(),
                ..CompressOptions::default()
            }),
            ..MinifierOptions::default()
        },
    );
    assert!(!output.contains("removed"));
    assert!(output.contains("work()"));
    assert!(output.contains("break e"));
    validate(&output);
}

#[test]
fn labels_without_mangling() {
    let source = "original:while(next()){continue original}";
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    let semantic = SemanticBuilder::new().build(&parsed.program).semantic;
    let plain = Codegen::new().build(&parsed.program).code;
    assert_eq!(
        Codegen::new().with_scoping(Some(semantic.into_scoping())).build(&parsed.program).code,
        plain
    );
    assert!(plain.contains("original:"));
    assert_eq!(
        minify(
            source,
            MinifierOptions { compress: None, mangle: None, ..MinifierOptions::default() }
        ),
        "original:while(next()){continue original}"
    );
}

#[test]
fn deep_labels_skip_keywords() {
    // Cross the single-character alphabet and the candidates `in`, `if`, and `do`.
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            let mut source = String::from("export async function* run() { 'use strict'; ");
            for depth in 0..512 {
                write!(source, "label{depth}: {{ ").unwrap();
            }
            for depth in 0..512 {
                write!(source, "if (stop({depth})) break label{depth}; ").unwrap();
            }
            source.push_str(&"}".repeat(513));
            validate(&source);
            let output = super::mangle(&source, &MangleOptions::default());
            validate(&output);
            assert!(!output.contains("label"));
            assert_eq!(super::mangle(&output, &MangleOptions::default()), output);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn label_source_maps() {
    let source = "outerSearch: while (next()) {\ncontinue outerSearch;\nbreak outerSearch;\n}";
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    let result = Mangler::new().build(&parsed.program);
    let output = Codegen::new()
        .with_options(CodegenOptions {
            source_map_path: Some("labels.js".into()),
            ..CodegenOptions::minify()
        })
        .with_scoping(Some(result.scoping.clone_in_with_semantic_ids_with_another_arena()))
        .build(&parsed.program);
    let map = output.map.unwrap();
    let labels: Vec<_> = map
        .get_tokens()
        .filter(|token| token.get_name_id().and_then(|id| map.get_name(id)) == Some("outerSearch"))
        .map(|token| {
            assert_eq!(token.get_dst_line(), 0);
            assert_eq!(output.code.as_bytes()[token.get_dst_col() as usize], b'e');
            (token.get_src_line(), token.get_src_col())
        })
        .collect();
    assert_eq!(labels, [(0, 0), (1, 9), (2, 6)]);
    assert!(!output.code.contains("outerSearch"));

    let source = r"\u006futer: while (next()) { break outer; }";
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    let result = Mangler::new().build(&parsed.program);
    let output = Codegen::new()
        .with_options(CodegenOptions {
            source_map_path: Some("escaped.js".into()),
            ..CodegenOptions::minify()
        })
        .with_scoping(Some(result.scoping))
        .build(&parsed.program);
    let map = output.map.unwrap();
    assert_eq!(map.get_names().collect::<Vec<_>>(), [r"\u006futer", "outer"]);
}
