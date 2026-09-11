use std::fmt::Write;

use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_str::JSStrBuilder;
use oxc_syntax::constant_value::ConstantValue;

#[test]
fn lone_surrogate_constants_and_concatenation() {
    let source = r#"enum E { A = "\uD800", B = A + "\uDC00", C = `x${A}y`, D = "\uFFFDd800" }"#;
    assert_eq!(
        get_enum_member_value(source, "A"),
        Some(ConstantValue::Utf16String(Box::new([0xD800])))
    );
    assert_eq!(get_enum_member_value(source, "B"), Some(ConstantValue::String("𐀀".into())));
    assert_eq!(
        get_enum_member_value(source, "C"),
        Some(ConstantValue::Utf16String(Box::new([0x78, 0xD800, 0x79])))
    );
    assert_eq!(get_enum_member_value(source, "D"), Some(ConstantValue::String("�d800".into())));
}

#[test]
fn lone_surrogate_member_names_in_merged_enums() {
    let source = r#"enum E { "\uD800" = "\uD800", "\uFFFDd800" = "marker" }
        enum E { A = E["\uD800"] + "\uDC00", B = E["\uFFFDd800"] }"#;
    assert_eq!(get_enum_member_value(source, "A"), Some(ConstantValue::String("𐀀".into())));
    assert_eq!(get_enum_member_value(source, "B"), Some(ConstantValue::String("marker".into())));
}

#[test]
fn scoped_surrogate_member_values_survive_growth_and_cloning() {
    let (scoping, scopes) = {
        let mut source = String::new();
        for (name, offset) in [("A", 0), ("B", 100)] {
            write!(source, "enum {name} {{").unwrap();
            for index in 0..64 {
                write!(source, r#""\uD800𐀀{index}" = {},"#, offset + index).unwrap();
            }
            source.push('}');
        }
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(parsed.diagnostics.is_empty());
        let semantic = SemanticBuilder::new().with_enum_eval(true).build(&parsed.program);
        assert!(semantic.diagnostics.is_empty());
        let scopes: Vec<_> = parsed
            .program
            .body
            .iter()
            .map(|statement| {
                let Statement::TSEnumDeclaration(decl) = statement else { unreachable!() };
                decl.body.scope_id.get().unwrap()
            })
            .collect();
        (semantic.semantic.into_scoping().clone_in_with_semantic_ids_with_another_arena(), scopes)
    };

    // All original source and arena storage has been dropped. Look up the cloned
    // values using names from a different arena, after the table has grown.
    let allocator = Allocator::default();
    for index in 0..=64 {
        let mut name = JSStrBuilder::new_in(&allocator);
        name.push_code_unit(0xD800);
        name.push_str(&format!("𐀀{index}"));
        let name = name.into_js_str();
        for (scope, offset) in scopes.iter().zip([0, 100]) {
            let expected = (index < 64).then(|| ConstantValue::Number(f64::from(index + offset)));
            assert_eq!(scoping.get_enum_member_value_by_name(*scope, name), expected.as_ref());
        }
    }
}

fn get_enum_member_value(source: &str, member_name: &str) -> Option<ConstantValue> {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let parser_ret = Parser::new(&allocator, source, source_type).parse();
    assert!(parser_ret.diagnostics.is_empty(), "Parse errors: {:?}", parser_ret.diagnostics);
    let semantic_ret = SemanticBuilder::new().with_enum_eval(true).build(&parser_ret.program);
    assert!(semantic_ret.diagnostics.is_empty(), "Semantic errors: {:?}", semantic_ret.diagnostics);
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
fn enum_member_shadows_outer_binding() {
    let source = "var X = 4; enum A { X = 1, Y = X + 1 }";
    assert_eq!(get_enum_member_value(source, "X"), Some(ConstantValue::Number(1.0)));
    assert_eq!(get_enum_member_value(source, "Y"), Some(ConstantValue::Number(2.0)));
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
fn shadowed_infinity_nan_are_not_evaluated_as_globals() {
    let source =
        "function f() { const Infinity = 1; const NaN = 2; enum A { X = Infinity, Y = NaN } }";
    assert_eq!(get_enum_member_value(source, "X"), None);
    assert_eq!(get_enum_member_value(source, "Y"), None);
}

#[test]
fn merged_enum_members_named_infinity_nan_take_precedence_over_globals() {
    let source = "enum A { Infinity = 1, NaN = 2 } enum A { X = Infinity, Y = NaN }";
    assert_eq!(get_enum_member_value(source, "X"), Some(ConstantValue::Number(1.0)));
    assert_eq!(get_enum_member_value(source, "Y"), Some(ConstantValue::Number(2.0)));
}

#[test]
fn unevaluated_merged_enum_member_shadows_global() {
    let source = "enum A { Infinity = foo() } enum A { X = Infinity }";
    assert_eq!(get_enum_member_value(source, "X"), None);
}

#[test]
fn merged_enum_member_shadows_outer_binding() {
    let source =
        "function f() { const Infinity = 3; enum A { Infinity = 1 } enum A { X = Infinity } }";
    assert_eq!(get_enum_member_value(source, "X"), Some(ConstantValue::Number(1.0)));
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
fn auto_increment_after_unresolvable_fails() {
    // Auto-increment must propagate the "unknown" state of the previous member,
    // not silently restart at 0.
    let source = "enum A { X = foo(), Y }";
    debug_assert_eq!(get_enum_member_value(source, "X"), None);
    debug_assert_eq!(get_enum_member_value(source, "Y"), None);

    let source = "enum A { X = foo(), Y, Z }";
    debug_assert_eq!(get_enum_member_value(source, "Y"), None);
    debug_assert_eq!(get_enum_member_value(source, "Z"), None);
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
