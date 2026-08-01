use oxc_allocator::Allocator;
use oxc_formatter::{JsFormatOptions, format};
use oxc_span::SourceType;

fn format_ets(source_text: &str) -> String {
    let allocator = Allocator::default();
    format(&allocator, source_text, SourceType::ets(), JsFormatOptions::default(), None)
        .unwrap()
        .print()
        .unwrap()
        .into_code()
}

fn format_ets_static(source_text: &str) -> String {
    let allocator = Allocator::default();
    format(&allocator, source_text, SourceType::ets_static(), JsFormatOptions::default(), None)
        .unwrap()
        .print()
        .unwrap()
        .into_code()
}

#[test]
fn ets_static_is_formatted_and_idempotent() {
    let source = r#"package example.formatter;
@interface Mark { value: string = "ok" }
@Mark({ value = "ok" })
final class Value {
  constructor named(value: int) {}
  overload constructor { named }
}
enum Color: int { Red, Green }
native function consume(value: char): void;
let character: char = c'a'
let values: int[] = new int[3]
let matrix: int[][] = new int[2][3]
function resolve(promise: Promise<int>): int { return await promise }
consume(character) { let nested: int = 1 }
"#;

    let expected = r#"package example.formatter;
@interface Mark {
  value: string = "ok";
}
@Mark({ value = "ok" })
final class Value {
  constructor named(value: int) {}
  overload constructor { named }
}
enum Color: int {
  Red,
  Green,
}
native function consume(value: char): void;
let character: char = c'a';
let values: int[] = new int[3];
let matrix: int[][] = new int[2][3];
function resolve(promise: Promise<int>): int {
  return await promise;
}
consume(character) {
  let nested: int = 1;
};
"#;

    let formatted = format_ets_static(source);
    assert_eq!(formatted, expected);
    assert_eq!(format_ets_static(&formatted), formatted);
}

#[test]
fn arkui_component_chain_comment_stays_in_place_after_reformat() {
    let source = r"struct S {
  build() {
    Row() {}
      .width(100)
      //.disabled(true)
      .height(200)
  }
}
";

    let first = format_ets(source);
    let second = format_ets(&first);

    let width = second.find(".width(100)").expect("width chain call");
    let comment = second.find("//.disabled(true)").expect("disabled chain comment");
    let height = second.find(".height(200)").expect("height chain call");

    assert!(
        width < comment && comment < height,
        "ArkUI component chain comment moved after reformat:\n{second}"
    );
}
