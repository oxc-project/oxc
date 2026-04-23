use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_syntax::constant_value::ConstantValue;

fn get_enum_member_value(source: &str, member_name: &str) -> Option<ConstantValue> {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let parser_ret = Parser::new(&allocator, source, source_type).parse();
    assert!(parser_ret.errors.is_empty(), "Parse errors: {:?}", parser_ret.errors);
    let semantic_ret = SemanticBuilder::new().with_enum_eval(true).build(&parser_ret.program);
    assert!(semantic_ret.errors.is_empty(), "Semantic errors: {:?}", semantic_ret.errors);
    let scoping = semantic_ret.semantic.into_scoping();

    for symbol_id in scoping.symbol_ids() {
        if scoping.symbol_name(symbol_id) == member_name
            && scoping.symbol_flags(symbol_id).is_enum_member()
        {
            return scoping.get_enum_member_value(symbol_id).cloned();
        }
    }
    None
}

#[test]
fn auto_increment() {
    let source = "enum A { X, Y, Z }";
    debug_assert_eq!(get_enum_member_value(source, "X"), Some(ConstantValue::Number(0.0)));
    debug_assert_eq!(get_enum_member_value(source, "Y"), Some(ConstantValue::Number(1.0)));
    debug_assert_eq!(get_enum_member_value(source, "Z"), Some(ConstantValue::Number(2.0)));
}

#[test]
fn explicit_numeric() {
    let source = "enum A { X = 10, Y, Z }";
    debug_assert_eq!(get_enum_member_value(source, "X"), Some(ConstantValue::Number(10.0)));
    debug_assert_eq!(get_enum_member_value(source, "Y"), Some(ConstantValue::Number(11.0)));
    debug_assert_eq!(get_enum_member_value(source, "Z"), Some(ConstantValue::Number(12.0)));
}

#[test]
fn string_members() {
    let source = r#"enum A { X = "hello" }"#;
    debug_assert_eq!(
        get_enum_member_value(source, "X"),
        Some(ConstantValue::String("hello".into()))
    );
}

#[test]
fn binary_expressions() {
    let source = "enum A { X = 1 + 2, Y = 1 << 3 }";
    debug_assert_eq!(get_enum_member_value(source, "X"), Some(ConstantValue::Number(3.0)));
    debug_assert_eq!(get_enum_member_value(source, "Y"), Some(ConstantValue::Number(8.0)));
}

#[test]
fn unary_expressions() {
    let source = "enum A { X = -1, Y = ~0 }";
    debug_assert_eq!(get_enum_member_value(source, "X"), Some(ConstantValue::Number(-1.0)));
    debug_assert_eq!(get_enum_member_value(source, "Y"), Some(ConstantValue::Number(-1.0)));
}

#[test]
fn cross_member_reference() {
    let source = "enum A { X = 1, Y = X + 1 }";
    debug_assert_eq!(get_enum_member_value(source, "X"), Some(ConstantValue::Number(1.0)));
    debug_assert_eq!(get_enum_member_value(source, "Y"), Some(ConstantValue::Number(2.0)));
}

#[test]
fn infinity_nan() {
    let source = "enum A { X = Infinity, Y = NaN }";
    debug_assert_eq!(
        get_enum_member_value(source, "X"),
        Some(ConstantValue::Number(f64::INFINITY))
    );
    match get_enum_member_value(source, "Y") {
        Some(ConstantValue::Number(n)) => assert!(n.is_nan(), "Expected NaN, got {n}"),
        other => panic!("Expected Some(Number(NaN)), got {other:?}"),
    }
}

#[test]
fn string_concat() {
    let source = r#"enum A { X = "a" + "b" }"#;
    debug_assert_eq!(get_enum_member_value(source, "X"), Some(ConstantValue::String("ab".into())));
}

#[test]
fn unresolvable() {
    let source = "enum A { X = foo() }";
    debug_assert_eq!(get_enum_member_value(source, "X"), None);
}

#[test]
fn const_enum() {
    let source = "const enum A { X, Y }";
    debug_assert_eq!(get_enum_member_value(source, "X"), Some(ConstantValue::Number(0.0)));
    debug_assert_eq!(get_enum_member_value(source, "Y"), Some(ConstantValue::Number(1.0)));
}

#[test]
fn declare_enum() {
    let source = "declare enum A { X = 1 }";
    debug_assert_eq!(get_enum_member_value(source, "X"), Some(ConstantValue::Number(1.0)));
}

#[test]
fn auto_increment_after_string_fails() {
    let source = r#"enum A { X = "hello", Y }"#;
    debug_assert_eq!(
        get_enum_member_value(source, "X"),
        Some(ConstantValue::String("hello".into()))
    );
    debug_assert_eq!(get_enum_member_value(source, "Y"), None);
}

#[test]
fn parenthesized_expression() {
    let source = "enum A { X = (1 + 2) }";
    debug_assert_eq!(get_enum_member_value(source, "X"), Some(ConstantValue::Number(3.0)));
}

#[test]
fn template_literal() {
    let source = "enum A { X = `hello` }";
    debug_assert_eq!(
        get_enum_member_value(source, "X"),
        Some(ConstantValue::String("hello".into()))
    );
}

#[test]
fn cross_enum_member_expression() {
    let source = "enum A { X = 1 } enum B { Y = A.X + 1 }";
    debug_assert_eq!(get_enum_member_value(source, "Y"), Some(ConstantValue::Number(2.0)));
}

#[test]
fn cross_enum_computed_member_expression() {
    let source = r#"enum A { X = 1 } enum B { Y = A["X"] + 1 }"#;
    debug_assert_eq!(get_enum_member_value(source, "Y"), Some(ConstantValue::Number(2.0)));
}

#[test]
fn cross_enum_string_value() {
    let source = r#"enum A { X = "hello" } enum B { Y = A.X }"#;
    debug_assert_eq!(
        get_enum_member_value(source, "Y"),
        Some(ConstantValue::String("hello".into()))
    );
}

#[test]
fn bitwise_operations() {
    let source = "enum A { X = 0xFF & 0x0F, Y = 1 | 2, Z = 1 ^ 3 }";
    debug_assert_eq!(get_enum_member_value(source, "X"), Some(ConstantValue::Number(15.0)));
    debug_assert_eq!(get_enum_member_value(source, "Y"), Some(ConstantValue::Number(3.0)));
    debug_assert_eq!(get_enum_member_value(source, "Z"), Some(ConstantValue::Number(2.0)));
}

#[test]
fn merged_enum_cross_declaration_reference() {
    let source = "enum Foo { A = 1 } enum Foo { B = A + 1 }";
    debug_assert_eq!(get_enum_member_value(source, "A"), Some(ConstantValue::Number(1.0)));
    debug_assert_eq!(get_enum_member_value(source, "B"), Some(ConstantValue::Number(2.0)));
}

#[test]
fn unary_on_string() {
    // Babel uses JS coercion: +"s" → "s" (identity), -"s" → NaN, ~"s" → -1
    let source = r#"enum A { X = +"hello", Y = -"hello", Z = ~"hello" }"#;
    debug_assert_eq!(
        get_enum_member_value(source, "X"),
        Some(ConstantValue::String("hello".into()))
    );
    match get_enum_member_value(source, "Y") {
        Some(ConstantValue::Number(n)) => assert!(n.is_nan()),
        other => panic!("Expected Some(Number(NaN)), got {other:?}"),
    }
    debug_assert_eq!(get_enum_member_value(source, "Z"), Some(ConstantValue::Number(-1.0)));
}
