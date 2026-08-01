mod deno;

use std::{fmt::Write, fs, path::Path, sync::Arc};

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn transform(path: &Path, source_text: &str) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let parser_ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(
        parser_ret.diagnostics.is_empty(),
        "Parser errors for {}: {:?}",
        path.display(),
        parser_ret.diagnostics
    );

    let id_ret =
        IsolatedDeclarations::new(&allocator, IsolatedDeclarationsOptions { strip_internal: true })
            .build(&parser_ret.program);
    let code = Codegen::new().build(&id_ret.program).code;

    let mut snapshot =
        format!("```\n==================== .D.TS ====================\n\n{code}\n\n");
    if !id_ret.diagnostics.is_empty() {
        let source = Arc::new(source_text.to_string());
        let error_messages = id_ret
            .diagnostics
            .iter()
            .map(|d| d.clone().with_source_code(Arc::clone(&source)))
            .fold(String::new(), |s, error| s + &format!("{error:?}"));

        write!(
            snapshot,
            "==================== Errors ====================\n{error_messages}\n\n```"
        )
        .unwrap();
    }

    snapshot
}

#[test]
fn snapshots() {
    insta::glob!("fixtures/*.{ts,tsx}", |path| {
        let source_text = fs::read_to_string(path).unwrap();
        let snapshot = transform(path, &source_text);
        let name = path.file_stem().unwrap().to_str().unwrap();
        insta::with_settings!({ prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
            insta::assert_snapshot!(name, snapshot);
        });
    });
}

#[test]
fn static_ets_declarations_round_trip() {
    let source = r#"package example.declarations;
export @interface Mark { value: string = "ok" }
export final class Value {
  constructor named(value: int) {}
  overload constructor { named }
  method(value: int): int { return value }
}
export interface Consumer {
  value: int;
  consume(value: int): int { return value }
}
export final struct Point {
  x: int = 0;
  static { initialize() }
  move(delta: int): int { return this.x + delta }
}
export native function consume(value: char): void;
"#;
    let allocator = Allocator::default();
    let source_type = SourceType::ets_static();
    let parser_ret = Parser::new(&allocator, source, source_type).parse();
    assert!(parser_ret.diagnostics.is_empty(), "Parse errors: {:?}", parser_ret.diagnostics);

    let ret = IsolatedDeclarations::new(&allocator, IsolatedDeclarationsOptions::default())
        .build(&parser_ret.program);
    assert!(ret.diagnostics.is_empty(), "Declaration errors: {:?}", ret.diagnostics);
    let output = Codegen::new().build(&ret.program).code;

    for syntax in [
        "export declare @interface Mark",
        "export declare final class Value",
        "constructor named(value: int)",
        "overload constructor {",
        "method(value: int): int;",
        "export interface Consumer",
        "value: int;",
        "consume(value: int): int;",
        "export declare final struct Point",
        "x: int;",
        "move(delta: int): int;",
        "export declare native function consume(value: char): void;",
    ] {
        assert!(output.contains(syntax), "`{syntax}` was lost:\n{output}");
    }
    assert!(!output.contains("static {"), "static block was retained:\n{output}");
    assert!(!output.contains("return "), "implementation body was retained:\n{output}");

    let reparsed_allocator = Allocator::default();
    let reparsed = Parser::new(&reparsed_allocator, &output, source_type).parse();
    assert!(
        reparsed.diagnostics.is_empty(),
        "Reparse errors: {:?}\n{output}",
        reparsed.diagnostics
    );
}
