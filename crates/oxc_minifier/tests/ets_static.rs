use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

#[test]
fn minifies_static_ets_without_losing_syntax() {
    let source_type = SourceType::ets_static();
    let source = r#"package example.minify;
export @interface Mark { value: string = "ok" }
@Mark({ value = "ok" })
export final class Value {
  constructor named(value: int) {}
  overload constructor { named }
  method(value: int): int { let unused: int = 1; return value + 0 }
}
native function consume(value: char): void;
let character: char = c'a'
export function create(): Value { return new Value() }
export function createArray(): int[] { return new int[2] }
export function createMatrix(): int[][] { return new int[2][3] }
export function resolve(promise: Promise<int>): int { return await promise }
export function check(instance: Value): boolean { return instance instanceof Value }
consume(character) { let nested: int = 1 }
"#;

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, source_type).parse();
    assert!(ret.diagnostics.is_empty(), "Parse errors: {:?}", ret.diagnostics);
    let mut program = ret.program;
    let options = MinifierOptions { compress: Some(CompressOptions::default()), mangle: None };
    let ret = Minifier::new(options).minify(&allocator, &mut program);
    let output = Codegen::new()
        .with_options(CodegenOptions::minify())
        .with_scoping(ret.scoping)
        .build(&program)
        .code;

    for syntax in [
        "package example.minify;",
        "export @interface Mark",
        "final class Value",
        "constructor named(value:int)",
        "overload constructor{",
        "native function consume(value:char):void;",
        "c'a'",
        "new Value()",
        "new int[2]",
        "new int[2][3]",
        "return await promise",
        "instance instanceof Value",
        "consume(character){",
    ] {
        assert!(output.contains(syntax), "`{syntax}` was lost:\n{output}");
    }

    let reparsed_allocator = Allocator::default();
    let reparsed = Parser::new(&reparsed_allocator, &output, source_type).parse();
    assert!(
        reparsed.diagnostics.is_empty(),
        "Reparse errors: {:?}\n{output}",
        reparsed.diagnostics
    );
}
