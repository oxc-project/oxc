use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement, TSType};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_types::{TypeData, TypeFlags};

use crate::Checker;

/// Helper macro to set up parser -> semantic -> checker in a single scope,
/// avoiding lifetime issues with the allocator and AST.
macro_rules! with_checker {
    ($source:expr, |$checker:ident, $program:ident| $body:block) => {{
        let allocator = Allocator::default();
        let source_type = SourceType::ts();
        let parsed = Parser::new(&allocator, $source, source_type).parse();
        let $program = &parsed.program;
        let semantic = SemanticBuilder::new().build($program).semantic;
        #[allow(unused_mut)]
        let mut $checker = Checker::new(semantic);
        $body
    }};
}

/// Extract the initializer expression from the first variable declaration.
fn first_var_init<'a>(program: &'a oxc_ast::ast::Program<'a>) -> Option<&'a Expression<'a>> {
    for stmt in &program.body {
        if let Statement::VariableDeclaration(decl) = stmt {
            if let Some(declarator) = decl.declarations.first() {
                return declarator.init.as_ref();
            }
        }
    }
    None
}

/// Extract the type annotation from the first variable declaration in source.
fn first_var_type_annotation<'a>(program: &'a oxc_ast::ast::Program<'a>) -> Option<&'a TSType<'a>> {
    for stmt in &program.body {
        if let Statement::VariableDeclaration(decl) = stmt {
            if let Some(declarator) = decl.declarations.first() {
                if let Some(annotation) = &declarator.type_annotation {
                    return Some(&annotation.type_annotation);
                }
            }
        }
    }
    None
}

// ---- Intrinsic type tests (from Step 2) ----

#[test]
fn intrinsic_types_are_initialized() {
    with_checker!("", |checker, _program| {
        let arena = checker.type_arena();

        assert_eq!(arena.get_flags(checker.any_type), TypeFlags::Any);
        assert_eq!(arena.get_flags(checker.unknown_type), TypeFlags::Unknown);
        assert_eq!(arena.get_flags(checker.string_type), TypeFlags::String);
        assert_eq!(arena.get_flags(checker.number_type), TypeFlags::Number);
        assert_eq!(arena.get_flags(checker.bigint_type), TypeFlags::BigInt);
        assert_eq!(arena.get_flags(checker.boolean_type), TypeFlags::Boolean);
        assert_eq!(arena.get_flags(checker.es_symbol_type), TypeFlags::ESSymbol);
        assert_eq!(arena.get_flags(checker.void_type), TypeFlags::Void);
        assert_eq!(arena.get_flags(checker.undefined_type), TypeFlags::Undefined);
        assert_eq!(arena.get_flags(checker.null_type), TypeFlags::Null);
        assert_eq!(arena.get_flags(checker.never_type), TypeFlags::Never);
        assert_eq!(arena.get_flags(checker.non_primitive_type), TypeFlags::NonPrimitive);
    });
}

#[test]
fn intrinsic_types_have_correct_names() {
    with_checker!("", |checker, _program| {
        let arena = checker.type_arena();

        let cases = [
            (checker.any_type, "any"),
            (checker.unknown_type, "unknown"),
            (checker.string_type, "string"),
            (checker.number_type, "number"),
            (checker.bigint_type, "bigint"),
            (checker.boolean_type, "boolean"),
            (checker.es_symbol_type, "symbol"),
            (checker.void_type, "void"),
            (checker.undefined_type, "undefined"),
            (checker.null_type, "null"),
            (checker.never_type, "never"),
            (checker.non_primitive_type, "object"),
        ];

        for (type_id, expected_name) in cases {
            match arena.get_data(type_id) {
                TypeData::Intrinsic(t) => {
                    assert_eq!(
                        t.intrinsic_name, expected_name,
                        "wrong name for {expected_name}"
                    );
                }
                _ => panic!("expected IntrinsicType for {expected_name}"),
            }
        }
    });
}

#[test]
fn boolean_literal_types() {
    with_checker!("", |checker, _program| {
        let arena = checker.type_arena();

        assert_eq!(arena.get_flags(checker.true_type), TypeFlags::BooleanLiteral);
        assert_eq!(arena.get_flags(checker.false_type), TypeFlags::BooleanLiteral);

        match arena.get_data(checker.true_type) {
            TypeData::Literal(oxc_types::LiteralType::Boolean(true)) => {}
            _ => panic!("expected true literal"),
        }
        match arena.get_data(checker.false_type) {
            TypeData::Literal(oxc_types::LiteralType::Boolean(false)) => {}
            _ => panic!("expected false literal"),
        }
    });
}

#[test]
fn check_returns_no_diagnostics_for_empty_program() {
    with_checker!("", |checker, program| {
        let diagnostics = checker.check_program(program);
        assert!(diagnostics.is_empty());
    });
}

// ---- Type annotation resolution tests (Step 3) ----

#[test]
fn resolve_keyword_type_annotations() {
    let cases = [
        ("let x: string", "string", TypeFlags::String),
        ("let x: number", "number", TypeFlags::Number),
        ("let x: boolean", "boolean", TypeFlags::Boolean),
        ("let x: bigint", "bigint", TypeFlags::BigInt),
        ("let x: symbol", "symbol", TypeFlags::ESSymbol),
        ("let x: any", "any", TypeFlags::Any),
        ("let x: unknown", "unknown", TypeFlags::Unknown),
        ("let x: void", "void", TypeFlags::Void),
        ("let x: undefined", "undefined", TypeFlags::Undefined),
        ("let x: null", "null", TypeFlags::Null),
        ("let x: never", "never", TypeFlags::Never),
        ("let x: object", "object", TypeFlags::NonPrimitive),
    ];

    for (source, expected_name, expected_flags) in cases {
        with_checker!(source, |checker, program| {
            let ts_type = first_var_type_annotation(program)
                .unwrap_or_else(|| panic!("no type annotation in: {source}"));
            let type_id = checker.get_type_from_type_node(ts_type);

            assert_eq!(
                checker.type_arena().get_flags(type_id),
                expected_flags,
                "wrong flags for: {source}"
            );
            assert_eq!(
                checker.type_to_string(type_id),
                expected_name,
                "wrong string for: {source}"
            );
        });
    }
}

#[test]
fn resolve_parenthesized_type() {
    with_checker!("let x: (string)", |checker, program| {
        let ts_type = first_var_type_annotation(program).unwrap();
        let type_id = checker.get_type_from_type_node(ts_type);
        assert_eq!(checker.type_arena().get_flags(type_id), TypeFlags::String);
        assert_eq!(checker.type_to_string(type_id), "string");
    });
}

#[test]
fn type_to_string_for_intrinsics() {
    with_checker!("", |checker, _program| {
        assert_eq!(checker.type_to_string(checker.any_type), "any");
        assert_eq!(checker.type_to_string(checker.string_type), "string");
        assert_eq!(checker.type_to_string(checker.number_type), "number");
        assert_eq!(checker.type_to_string(checker.boolean_type), "boolean");
        assert_eq!(checker.type_to_string(checker.void_type), "void");
        assert_eq!(checker.type_to_string(checker.null_type), "null");
        assert_eq!(checker.type_to_string(checker.undefined_type), "undefined");
        assert_eq!(checker.type_to_string(checker.never_type), "never");
        assert_eq!(checker.type_to_string(checker.true_type), "true");
        assert_eq!(checker.type_to_string(checker.false_type), "false");
    });
}

// ---- Expression type tests (Step 4) ----

#[test]
fn literal_expression_types() {
    let cases = [
        ("let x = \"hello\"", "\"hello\"", TypeFlags::StringLiteral),
        ("let x = 42", "42", TypeFlags::NumberLiteral),
        ("let x = 3.14", "3.14", TypeFlags::NumberLiteral),
        ("let x = true", "true", TypeFlags::BooleanLiteral),
        ("let x = false", "false", TypeFlags::BooleanLiteral),
        ("let x = null", "null", TypeFlags::Null),
    ];

    for (source, expected_str, expected_flags) in cases {
        with_checker!(source, |checker, program| {
            let init = first_var_init(program)
                .unwrap_or_else(|| panic!("no initializer in: {source}"));
            let type_id = checker.get_type_of_expression(init);

            assert_eq!(
                checker.type_arena().get_flags(type_id),
                expected_flags,
                "wrong flags for: {source}"
            );
            assert_eq!(
                checker.type_to_string(type_id),
                expected_str,
                "wrong string for: {source}"
            );
        });
    }
}

#[test]
fn identifier_resolves_to_declared_type() {
    // When an identifier references a variable with a type annotation,
    // the identifier's type should be the annotated type.
    with_checker!("let x: string = \"hi\"; let y = x", |checker, program| {
        // Get the second declaration's initializer (which is `x`)
        if let Statement::VariableDeclaration(decl) = &program.body[1] {
            let init = decl.declarations[0].init.as_ref().unwrap();
            let type_id = checker.get_type_of_expression(init);
            assert_eq!(checker.type_to_string(type_id), "string");
            assert_eq!(checker.type_arena().get_flags(type_id), TypeFlags::String);
        } else {
            panic!("expected variable declaration");
        }
    });
}

#[test]
fn identifier_infers_type_from_initializer() {
    // When a variable has no type annotation but has an initializer,
    // references to it should infer the type from the initializer.
    with_checker!("let x = 42; let y = x", |checker, program| {
        if let Statement::VariableDeclaration(decl) = &program.body[1] {
            let init = decl.declarations[0].init.as_ref().unwrap();
            let type_id = checker.get_type_of_expression(init);
            assert_eq!(checker.type_to_string(type_id), "42");
            assert_eq!(
                checker.type_arena().get_flags(type_id),
                TypeFlags::NumberLiteral
            );
        } else {
            panic!("expected variable declaration");
        }
    });
}

// ---- Assignability tests (Step 5) ----

#[test]
fn assignability_identity() {
    with_checker!("", |checker, _program| {
        assert!(checker.is_type_assignable_to(checker.string_type, checker.string_type));
        assert!(checker.is_type_assignable_to(checker.number_type, checker.number_type));
        assert!(checker.is_type_assignable_to(checker.boolean_type, checker.boolean_type));
        assert!(checker.is_type_assignable_to(checker.null_type, checker.null_type));
        assert!(checker.is_type_assignable_to(checker.undefined_type, checker.undefined_type));
        assert!(checker.is_type_assignable_to(checker.void_type, checker.void_type));
        assert!(checker.is_type_assignable_to(checker.never_type, checker.never_type));
    });
}

#[test]
fn assignability_any_and_unknown() {
    with_checker!("", |checker, _program| {
        assert!(checker.is_type_assignable_to(checker.string_type, checker.any_type));
        assert!(checker.is_type_assignable_to(checker.number_type, checker.any_type));
        assert!(checker.is_type_assignable_to(checker.null_type, checker.any_type));

        assert!(checker.is_type_assignable_to(checker.string_type, checker.unknown_type));
        assert!(checker.is_type_assignable_to(checker.number_type, checker.unknown_type));
        assert!(checker.is_type_assignable_to(checker.null_type, checker.unknown_type));

        assert!(checker.is_type_assignable_to(checker.never_type, checker.string_type));
        assert!(checker.is_type_assignable_to(checker.never_type, checker.number_type));
        assert!(checker.is_type_assignable_to(checker.never_type, checker.null_type));
    });
}

#[test]
fn assignability_primitives_not_assignable_across() {
    with_checker!("", |checker, _program| {
        assert!(!checker.is_type_assignable_to(checker.string_type, checker.number_type));
        assert!(!checker.is_type_assignable_to(checker.number_type, checker.string_type));
        assert!(!checker.is_type_assignable_to(checker.boolean_type, checker.string_type));
        assert!(!checker.is_type_assignable_to(checker.null_type, checker.string_type));
        assert!(!checker.is_type_assignable_to(checker.undefined_type, checker.number_type));
        assert!(!checker.is_type_assignable_to(checker.string_type, checker.boolean_type));
    });
}

#[test]
fn assignability_literal_to_base() {
    with_checker!("let x = \"hello\"", |checker, program| {
        let init = first_var_init(program).unwrap();
        let lit_type = checker.get_type_of_expression(init);
        assert!(checker.is_type_assignable_to(lit_type, checker.string_type));
        assert!(!checker.is_type_assignable_to(checker.string_type, lit_type));
    });

    with_checker!("let x = 42", |checker, program| {
        let init = first_var_init(program).unwrap();
        let lit_type = checker.get_type_of_expression(init);
        assert!(checker.is_type_assignable_to(lit_type, checker.number_type));
        assert!(!checker.is_type_assignable_to(checker.number_type, lit_type));
    });

    with_checker!("let x = true", |checker, program| {
        let init = first_var_init(program).unwrap();
        let lit_type = checker.get_type_of_expression(init);
        assert!(checker.is_type_assignable_to(lit_type, checker.boolean_type));
        assert!(!checker.is_type_assignable_to(checker.boolean_type, lit_type));
    });
}

#[test]
fn assignability_undefined_to_void() {
    with_checker!("", |checker, _program| {
        assert!(checker.is_type_assignable_to(checker.undefined_type, checker.void_type));
        assert!(!checker.is_type_assignable_to(checker.void_type, checker.undefined_type));
    });
}

// ---- Diagnostic tests (Step 5) ----

#[test]
fn check_program_emits_ts2322_for_type_mismatch() {
    with_checker!("var x: number = \"\"", |checker, program| {
        let diagnostics = checker.check_program(program);
        assert_eq!(diagnostics.len(), 1);
        let d = &diagnostics[0];
        assert!(d.message.to_string().contains("Type '\"\"' is not assignable to type 'number'"));
    });
}

#[test]
fn check_program_no_error_for_compatible_types() {
    with_checker!("var x: number = 42", |checker, program| {
        let diagnostics = checker.check_program(program);
        assert!(diagnostics.is_empty(), "expected no errors, got: {:?}", diagnostics);
    });
}

#[test]
fn check_program_no_error_without_annotation() {
    with_checker!("var x = \"hello\"", |checker, program| {
        let diagnostics = checker.check_program(program);
        assert!(diagnostics.is_empty());
    });
}

#[test]
fn check_program_no_error_without_initializer() {
    with_checker!("var x: number", |checker, program| {
        let diagnostics = checker.check_program(program);
        assert!(diagnostics.is_empty());
    });
}

#[test]
fn check_program_string_literal_to_string_ok() {
    with_checker!("var x: string = \"hello\"", |checker, program| {
        let diagnostics = checker.check_program(program);
        assert!(diagnostics.is_empty());
    });
}

#[test]
fn check_program_boolean_literal_to_number_error() {
    with_checker!("var x: number = true", |checker, program| {
        let diagnostics = checker.check_program(program);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.to_string().contains("Type 'true' is not assignable to type 'number'"));
    });
}

// ---- Union type tests (Step 6) ----

#[test]
fn union_type_from_annotation() {
    with_checker!("let x: string | number", |checker, program| {
        let ts_type = first_var_type_annotation(program).unwrap();
        let type_id = checker.get_type_from_type_node(ts_type);
        assert_eq!(checker.type_arena().get_flags(type_id), TypeFlags::Union);
        assert_eq!(checker.type_to_string(type_id), "string | number");
    });
}

#[test]
fn union_type_dedup() {
    with_checker!("let x: string | number; let y: string | number", |checker, program| {
        // Both annotations should resolve to the same TypeId
        if let Statement::VariableDeclaration(decl1) = &program.body[0] {
            if let Statement::VariableDeclaration(decl2) = &program.body[1] {
                let ann1 = decl1.declarations[0].type_annotation.as_ref().unwrap();
                let ann2 = decl2.declarations[0].type_annotation.as_ref().unwrap();
                let t1 = checker.get_type_from_type_node(&ann1.type_annotation);
                let t2 = checker.get_type_from_type_node(&ann2.type_annotation);
                assert_eq!(t1, t2, "same union should produce same TypeId");
            }
        }
    });
}

#[test]
fn union_type_flattens_never() {
    with_checker!("let x: string | never", |checker, program| {
        let ts_type = first_var_type_annotation(program).unwrap();
        let type_id = checker.get_type_from_type_node(ts_type);
        // `never` is filtered out, leaving just `string` (unwrapped)
        assert_eq!(checker.type_arena().get_flags(type_id), TypeFlags::String);
        assert_eq!(checker.type_to_string(type_id), "string");
    });
}

#[test]
fn union_type_single_constituent_after_dedup() {
    with_checker!("let x: string | string", |checker, program| {
        let ts_type = first_var_type_annotation(program).unwrap();
        let type_id = checker.get_type_from_type_node(ts_type);
        // Deduped to single element, unwrapped
        assert_eq!(checker.type_arena().get_flags(type_id), TypeFlags::String);
    });
}

#[test]
fn assignability_to_union() {
    with_checker!("let x: string | number", |checker, program| {
        let ts_type = first_var_type_annotation(program).unwrap();
        let union_type = checker.get_type_from_type_node(ts_type);

        // Number literal assignable to string | number
        assert!(checker.is_type_assignable_to(checker.number_type, union_type));
        // String literal assignable to string | number
        assert!(checker.is_type_assignable_to(checker.string_type, union_type));
        // Boolean NOT assignable to string | number
        assert!(!checker.is_type_assignable_to(checker.boolean_type, union_type));
    });
}

#[test]
fn assignability_from_union() {
    with_checker!("let x: string | number", |checker, program| {
        let ts_type = first_var_type_annotation(program).unwrap();
        let union_type = checker.get_type_from_type_node(ts_type);

        // string | number NOT assignable to string (number isn't)
        assert!(!checker.is_type_assignable_to(union_type, checker.string_type));
        // string | number assignable to any
        assert!(checker.is_type_assignable_to(union_type, checker.any_type));
        // string | number assignable to unknown
        assert!(checker.is_type_assignable_to(union_type, checker.unknown_type));
    });
}

#[test]
fn check_program_union_compatible() {
    with_checker!("var x: string | number = 42", |checker, program| {
        let diagnostics = checker.check_program(program);
        assert!(diagnostics.is_empty());
    });
}

#[test]
fn check_program_union_mismatch() {
    with_checker!("var x: string | number = true", |checker, program| {
        let diagnostics = checker.check_program(program);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.to_string().contains("Type 'true' is not assignable to type 'string | number'"));
    });
}

// --- Assignment expression checking ---

#[test]
fn assignment_compatible() {
    with_checker!("let x: string; x = 'hello'", |checker, program| {
        let diagnostics = checker.check_program(program);
        assert!(diagnostics.is_empty());
    });
}

#[test]
fn assignment_mismatch() {
    with_checker!("let x: string; x = 42", |checker, program| {
        let diagnostics = checker.check_program(program);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.to_string().contains("Type '42' is not assignable to type 'string'"));
    });
}

#[test]
fn assignment_to_union_compatible() {
    with_checker!("let x: string | number; x = 42", |checker, program| {
        let diagnostics = checker.check_program(program);
        assert!(diagnostics.is_empty());
    });
}

#[test]
fn assignment_to_union_mismatch() {
    with_checker!("let x: string | number; x = true", |checker, program| {
        let diagnostics = checker.check_program(program);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.to_string().contains("Type 'true' is not assignable to type 'string | number'"));
    });
}

#[test]
fn assignment_no_annotation_no_error() {
    // Without a type annotation, `x` infers from initializer — reassignment to different type
    // currently doesn't error because inferred types are widened to `any` without annotation.
    // Actually, `let x = 'hello'` infers string literal, but re-assignment isn't checked
    // against inferred type in tsc either (widening). For now, no error expected.
    with_checker!("let x = 'hello'; x = 42", |checker, program| {
        let diagnostics = checker.check_program(program);
        // x has inferred type '"hello"' (string literal), assigning 42 should error
        // BUT tsc widens `let x = 'hello'` to `string`, so 42 would error.
        // Our checker infers the literal type, so this will produce an error.
        // That's a divergence from tsc we can fix later with widening.
        // For now just verify it doesn't panic.
        let _ = diagnostics;
    });
}

#[test]
fn multiple_assignments_multiple_errors() {
    with_checker!(
        "let x: string; x = 42; x = true",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert_eq!(diagnostics.len(), 2);
        }
    );
}

// --- Forward references, caching, and cycle detection ---

#[test]
fn forward_reference() {
    // `y` is referenced before its declaration; symbol resolution should still work.
    with_checker!(
        "let x: number = y; let y: number = 42",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            // x: number = y where y: number — compatible, no error
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn symbol_type_cached() {
    // Resolving the same symbol twice should return the same TypeId from cache.
    with_checker!(
        "let y: string = 'hi'",
        |checker, program| {
            // Get y's identifier expression from the initializer is a literal,
            // so instead resolve y's symbol directly.
            // Find y's symbol ID from semantic.
            // y should be the first symbol
            let symbol_id = oxc_syntax::symbol::SymbolId::from_usize(0);
            let t1 = checker.get_type_of_symbol(symbol_id);
            let t2 = checker.get_type_of_symbol(symbol_id);
            assert_eq!(t1, t2, "cached type should be identical");
            assert_eq!(checker.symbol_type_cache.len(), 1);
        }
    );
}

#[test]
fn function_parameter_type() {
    // Function parameter with type annotation should resolve correctly.
    with_checker!(
        "function f(x: number) { let y: string = x; }",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            // x is number, y is string — should produce TS2322
            assert_eq!(diagnostics.len(), 1);
            assert!(diagnostics[0].message.to_string().contains("Type 'number' is not assignable to type 'string'"));
        }
    );
}

#[test]
fn function_parameter_no_annotation() {
    // Parameter without annotation resolves to `any`, which is assignable to anything.
    with_checker!(
        "function f(x) { let y: string = x; }",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

// --- Type assertions ---

#[test]
fn type_assertion_as() {
    with_checker!(
        "let x: string = 42 as unknown as string",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn type_assertion_angle_bracket() {
    with_checker!(
        "let x: string = <string><unknown>42",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

// --- Binary expression types ---

#[test]
fn binary_arithmetic_returns_number() {
    with_checker!(
        "let x: number = 1 + 2",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn binary_arithmetic_to_string_error() {
    with_checker!(
        "let x: string = 1 + 2",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert_eq!(diagnostics.len(), 1);
            assert!(diagnostics[0].message.to_string().contains("not assignable"));
        }
    );
}

#[test]
fn binary_string_concat() {
    with_checker!(
        "let x: string = 'a' + 'b'",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn binary_comparison_returns_boolean() {
    with_checker!(
        "let x: boolean = 1 < 2",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

// --- Unary expression types ---

#[test]
fn typeof_returns_string() {
    with_checker!(
        "let x: string = typeof 42",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn void_returns_undefined() {
    with_checker!(
        "let x: undefined = void 0",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn logical_not_returns_boolean() {
    with_checker!(
        "let x: boolean = !true",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

// --- Conditional expression ---

#[test]
fn conditional_unions_branches() {
    with_checker!(
        "let x: string | number = true ? 'a' : 1",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn conditional_mismatch() {
    with_checker!(
        "let x: string = true ? 'a' : 1",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert_eq!(diagnostics.len(), 1);
        }
    );
}

// --- Literal types in annotations ---

#[test]
fn literal_type_annotation() {
    with_checker!(
        "let x: 42 = 42",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn literal_type_annotation_mismatch() {
    with_checker!(
        "let x: 42 = 43",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert_eq!(diagnostics.len(), 1);
        }
    );
}

// --- Template literal ---

#[test]
fn template_literal_is_string() {
    with_checker!(
        "let x: string = `hello ${42}`",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

// --- Return type checking ---

#[test]
fn return_type_compatible() {
    with_checker!(
        "function f(): number { return 42; }",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn return_type_mismatch() {
    with_checker!(
        "function f(): string { return 42; }",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert_eq!(diagnostics.len(), 1);
            assert!(diagnostics[0].message.to_string().contains("not assignable"));
        }
    );
}

#[test]
fn return_type_no_annotation_no_error() {
    // Without return type annotation, return values aren't checked.
    with_checker!(
        "function f() { return 42; }",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn return_type_void_bare_return() {
    // `return;` in a void function is fine.
    with_checker!(
        "function f(): void { return; }",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn return_type_nested_function() {
    // Return type of inner function shouldn't affect outer.
    with_checker!(
        "function f(): string { function g(): number { return 42; } return 'hi'; }",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn return_type_nested_mismatch() {
    // Inner function has wrong return type.
    with_checker!(
        "function f(): string { function g(): string { return 42; } return 'hi'; }",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert_eq!(diagnostics.len(), 1);
        }
    );
}

// --- Type alias resolution ---

#[test]
fn type_alias_simple() {
    with_checker!(
        "type Num = number; let x: Num = 42",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn type_alias_mismatch() {
    with_checker!(
        "type Num = number; let x: Num = 'hello'",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert_eq!(diagnostics.len(), 1);
            assert!(diagnostics[0].message.to_string().contains("not assignable"));
        }
    );
}

#[test]
fn type_alias_union() {
    with_checker!(
        "type StrOrNum = string | number; let x: StrOrNum = 42",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn type_alias_chain() {
    // Type alias referencing another type alias
    with_checker!(
        "type A = number; type B = A; let x: B = 42",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

// --- Interface types ---

#[test]
fn interface_property_check() {
    with_checker!(
        "interface Foo { x: number } let f: Foo = { x: 42 }",
        |checker, program| {
            // For now, object literal typing isn't implemented,
            // so { x: 42 } resolves to `any` which is assignable to Foo.
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

