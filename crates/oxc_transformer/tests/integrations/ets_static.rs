use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_transformer::TransformOptions;

use crate::test_with_source_type;

#[test]
fn preserves_static_ets_syntax_and_transforms_children() {
    let source_type = SourceType::ets_static();
    let source = r#"package example.transform;
@interface Mark { value: string = "ok" }
@Mark({ value = "ok" })
final class Value {
  constructor named(value: int) {}
  overload constructor { named }
  method(value: int): int { return value ?? 0 }
}
native function consume(value: char): void;
let character: char = c'a'
let instance: Value = new Value()
let values: int[][] = new int[2][3]
function resolve(promise: Promise<int>): int { return await promise }
consume(character) { let nested: int = instance instanceof Value ? 1 : 0 }
"#;

    let output = test_with_source_type(source, source_type, &TransformOptions::default())
        .expect("static ETS transform should succeed");

    for syntax in [
        "package example.transform;",
        "@interface Mark",
        "final class Value",
        "constructor named(value: int)",
        "overload constructor {",
        "native function consume(value: char): void;",
        "c'a'",
        "new Value()",
        "new int[2][3]",
        "return await promise",
        "instance instanceof Value",
        "consume(character) {",
    ] {
        assert!(output.contains(syntax), "`{syntax}` was lost:\n{output}");
    }

    let allocator = Allocator::default();
    let reparsed = Parser::new(&allocator, &output, source_type).parse();
    assert!(
        reparsed.diagnostics.is_empty(),
        "Reparse errors: {:?}\n{output}",
        reparsed.diagnostics
    );
}
