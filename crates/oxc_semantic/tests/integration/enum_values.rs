use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_syntax::constant_value::ConstantValue;

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
fn const_variable_initializer() {
    assert_eq!(
        get_enum_member_value(r#"const o = "o"; enum E { A = o, B = 1 }"#, "A"),
        Some(ConstantValue::String("o".into()))
    );
}

#[test]
fn const_variable_aliases_and_scopes() {
    for (source, expected) in [
        (r#"const a = "a", b = a + "b"; enum E { X = b }"#, "ab"),
        (r#"const a = ("a" as const); enum E { X = a }"#, "a"),
        (r#"const a = "outer"; { const a = "inner"; enum E { X = a } }"#, "inner"),
        (r#"const a = "outer"; { const a = "inner"; } enum E { X = a }"#, "outer"),
        (r#"const a = "outer"; enum E { a = "member", X = a }"#, "member"),
        (r#"const a = "outer"; enum E { a = "member" } enum E { X = a }"#, "member"),
    ] {
        assert_eq!(
            get_enum_member_value(source, "X"),
            Some(ConstantValue::String(expected.into())),
            "{source}"
        );
    }
    assert_eq!(
        get_enum_member_value("const a = 2; enum E { X = a + 1, Y }", "Y"),
        Some(ConstantValue::Number(4.0))
    );
}

#[test]
fn nonconstant_variable_initializers() {
    for source in [
        r#"let a = "a"; enum E { X = a }"#,
        r#"var a = "a"; enum E { X = a }"#,
        "const a = foo(); enum E { X = a }",
        "const a = b; const b = a; enum E { X = a }",
        r#"enum E { X = a } const a = "a";"#,
        r#"const { a } = { a: "a" }; enum E { X = a }"#,
        r#"const a = "outer"; function f(a: string) { enum E { X = a } }"#,
    ] {
        assert_eq!(get_enum_member_value(source, "X"), None, "{source}");
    }
}

#[test]
fn const_initializers_wait_for_reference_resolution() {
    let source = r#"const a = "outer"; function f() { function g() { const b = a; enum E { X = b } return E; } const a = "inner"; return g(); }"#;
    assert_eq!(get_enum_member_value(source, "X"), None);
}

#[test]
fn parenthesized_const_and_enum_references() {
    assert_eq!(
        get_enum_member_value(r#"const a = "a", b = (a); enum E { X = (b) }"#, "X"),
        Some(ConstantValue::String("a".into()))
    );
    assert_eq!(
        get_enum_member_value("enum E { A = 1, B = (A), X }", "X"),
        Some(ConstantValue::Number(2.0))
    );
}

#[test]
fn expanding_const_strings_are_bounded() {
    let mut source = String::from("const s0 = 'x';");
    for i in 1..=20 {
        use std::fmt::Write;
        write!(source, "const s{i} = s{} + s{};", i - 1, i - 1).unwrap();
    }
    assert!(get_enum_member_value(&source, "X").is_none());
    source.push_str("enum E { X = s20 }");
    assert!(get_enum_member_value(&source, "X").is_none());
}

#[test]
fn deep_const_aliases() {
    use std::fmt::Write;
    for (literal, expected) in
        [("1", ConstantValue::Number(1.0)), ("'s'", ConstantValue::String("s".into()))]
    {
        let mut source = format!("const n0 = {literal};");
        for i in 1..=1000 {
            write!(source, "const n{i} = n{};", i - 1).unwrap();
        }
        source.push_str("enum E { X = n1000 }");
        assert_eq!(get_enum_member_value(&source, "X"), Some(expected));
    }
}

#[test]
fn nested_destructured_const_is_not_evaluated() {
    let source =
        format!("{}const {{ x }} = value; enum E {{ X = x }}{}", "{".repeat(64), "}".repeat(64));
    assert_eq!(get_enum_member_value(&source, "X"), None);
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
fn shadowed_infinity_nan_use_const_values() {
    let source =
        "function f() { const Infinity = 1; const NaN = 2; enum A { X = Infinity, Y = NaN } }";
    assert_eq!(get_enum_member_value(source, "X"), Some(ConstantValue::Number(1.0)));
    assert_eq!(get_enum_member_value(source, "Y"), Some(ConstantValue::Number(2.0)));
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
    // TypeScript leaves unary-on-string initializers unevaluated, so the enum
    // member keeps its verbatim initializer instead of folding to a wrong value.
    let source = r#"enum A { X = +"hello", Y = -"hello", Z = ~"hello" }"#;
    assert_eq!(get_enum_member_value(source, "X"), None);
    assert_eq!(get_enum_member_value(source, "Y"), None);
    assert_eq!(get_enum_member_value(source, "Z"), None);
}

#[test]
fn unary_on_string_member_reference() {
    // A unary operation on a member whose value is a string must decline for the
    // same reason — `+"4"` would otherwise fold to the string "4" (tsc: number 4)
    // and `-"3"`/`~"4"` to NaN/-1 instead of -3/-5, also breaking the reverse
    // mapping and any auto-increment member after it.
    let source = r#"enum A { X = "4", Y = -X, Z = ~X, W = +X }"#;
    assert_eq!(get_enum_member_value(source, "X"), Some(ConstantValue::String("4".into())));
    assert_eq!(get_enum_member_value(source, "Y"), None);
    assert_eq!(get_enum_member_value(source, "Z"), None);
    assert_eq!(get_enum_member_value(source, "W"), None);
}

#[test]
fn surrogate_member_string_kind() {
    let allocator = Allocator::default();
    let source = r#"enum E { "\uD800" = "x", B = E["\uD800"], C = "\uD800", D = C }"#;
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let semantic = SemanticBuilder::new().with_enum_eval(true).build(&parsed.program);
    assert!(semantic.diagnostics.is_empty());
    let scoping = semantic.semantic.scoping();
    let enum_symbol = scoping.symbol_ids().find(|&id| scoping.symbol_name(id) == "E").unwrap();
    let scope = scoping.get_enum_body_scopes(enum_symbol).unwrap()[0];
    let oxc_ast::ast::Statement::TSEnumDeclaration(decl) = &parsed.program.body[0] else {
        panic!()
    };
    for member in &decl.body.members {
        assert!(scoping.is_string_enum_member(scope, member.id.static_name()));
    }
}
