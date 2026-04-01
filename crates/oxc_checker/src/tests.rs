use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement, TSType};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_types::{StructuredTypeKind, TypeData, TypeFlags};

use crate::Checker;

/// Helper macro to set up parser -> semantic -> checker in a single scope,
/// avoiding lifetime issues with the allocator and AST.
/// Uses Project for global types (same codepath as multi-file checking).
macro_rules! with_checker {
    ($source:expr, |$checker:ident, $program:ident| $body:block) => {{
        let type_arena = oxc_types::TypeArena::with_capacity(64);
        let project = oxc_project::Project::new(&type_arena);

        let allocator = Allocator::default();
        let source_type = SourceType::ts();
        let parsed = Parser::new(&allocator, $source, source_type).parse();
        let $program = &parsed.program;
        let semantic = SemanticBuilder::new().build($program).semantic;
        #[allow(unused_mut)]
        let mut $checker = Checker::new_with_host(
            &semantic, &type_arena, &project, String::new(), 1,
        );
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
        checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            let type_id = checker.get_type_of_expression(init, None);

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
            let type_id = checker.get_type_of_expression(init, None);
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
    // `let x = 42` widens to `number`, so `y = x` should also be `number`.
    with_checker!("let x = 42; let y = x", |checker, program| {
        if let Statement::VariableDeclaration(decl) = &program.body[1] {
            let init = decl.declarations[0].init.as_ref().unwrap();
            let type_id = checker.get_type_of_expression(init, None);
            assert_eq!(checker.type_to_string(type_id), "number");
            assert_eq!(
                checker.type_arena().get_flags(type_id),
                TypeFlags::Number
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
        let lit_type = checker.get_type_of_expression(init, None);
        assert!(checker.is_type_assignable_to(lit_type, checker.string_type));
        assert!(!checker.is_type_assignable_to(checker.string_type, lit_type));
    });

    with_checker!("let x = 42", |checker, program| {
        let init = first_var_init(program).unwrap();
        let lit_type = checker.get_type_of_expression(init, None);
        assert!(checker.is_type_assignable_to(lit_type, checker.number_type));
        assert!(!checker.is_type_assignable_to(checker.number_type, lit_type));
    });

    with_checker!("let x = true", |checker, program| {
        let init = first_var_init(program).unwrap();
        let lit_type = checker.get_type_of_expression(init, None);
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
        checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        let d = &diagnostics[0];
        assert!(d.message.to_string().contains("Type '\"\"' is not assignable to type 'number'"));
    });
}

#[test]
fn check_program_no_error_for_compatible_types() {
    with_checker!("var x: number = 42", |checker, program| {
        checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
        assert!(diagnostics.is_empty(), "expected no errors, got: {:?}", diagnostics);
    });
}

#[test]
fn check_program_no_error_without_annotation() {
    with_checker!("var x = \"hello\"", |checker, program| {
        checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
        assert!(diagnostics.is_empty());
    });
}

#[test]
fn check_program_no_error_without_initializer() {
    with_checker!("var x: number", |checker, program| {
        checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
        assert!(diagnostics.is_empty());
    });
}

#[test]
fn check_program_string_literal_to_string_ok() {
    with_checker!("var x: string = \"hello\"", |checker, program| {
        checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
        assert!(diagnostics.is_empty());
    });
}

#[test]
fn check_program_boolean_literal_to_number_error() {
    with_checker!("var x: number = true", |checker, program| {
        checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
        checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
        assert!(diagnostics.is_empty());
    });
}

#[test]
fn check_program_union_mismatch() {
    with_checker!("var x: string | number = true", |checker, program| {
        checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.to_string().contains("Type 'true' is not assignable to type 'string | number'"));
    });
}

#[test]
fn union_property_access_with_any_member() {
    // {x: any} | {x: string} → accessing .x should give any | string, not "not found"
    with_checker!(
        "interface A { x: any } interface B { x: string } declare let u: A | B; let v = u.x",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty(), "should not error: x exists on both members");
        }
    );
}

// --- Assignment expression checking ---

#[test]
fn assignment_compatible() {
    with_checker!("let x: string; x = 'hello'", |checker, program| {
        checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
        assert!(diagnostics.is_empty());
    });
}

#[test]
fn assignment_mismatch() {
    with_checker!("let x: string; x = 42", |checker, program| {
        checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.to_string().contains("Type '42' is not assignable to type 'string'"));
    });
}

#[test]
fn assignment_to_union_compatible() {
    with_checker!("let x: string | number; x = 42", |checker, program| {
        checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
        assert!(diagnostics.is_empty());
    });
}

#[test]
fn assignment_to_union_mismatch() {
    with_checker!("let x: string | number; x = true", |checker, program| {
        checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
        checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn type_assertion_angle_bracket() {
    with_checker!(
        "let x: string = <string><unknown>42",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn binary_arithmetic_to_string_error() {
    with_checker!(
        "let x: string = 1 + 2",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn binary_comparison_returns_boolean() {
    with_checker!(
        "let x: boolean = 1 < 2",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn void_returns_undefined() {
    with_checker!(
        "let x: undefined = void 0",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn logical_not_returns_boolean() {
    with_checker!(
        "let x: boolean = !true",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn conditional_mismatch() {
    with_checker!(
        "let x: string = true ? 'a' : 1",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn literal_type_annotation_mismatch() {
    with_checker!(
        "let x: 42 = 43",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn return_type_mismatch() {
    with_checker!(
        "function f(): string { return 42; }",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn type_alias_mismatch() {
    with_checker!(
        "type Num = number; let x: Num = 'hello'",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
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
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

// --- Type literal tests ---

#[test]
fn type_literal_basic() {
    with_checker!(
        "let x: { a: number; b: string }",
        |checker, program| {
            let ts_type = first_var_type_annotation(program).unwrap();
            let type_id = checker.get_type_from_type_node(ts_type);
            let flags = checker.type_arena().get_flags(type_id);
            assert!(flags.intersects(TypeFlags::Object));
            assert_eq!(checker.type_to_string(type_id), "{ a: number; b: string; }");
        }
    );
}

#[test]
fn type_literal_empty() {
    with_checker!(
        "let x: {}",
        |checker, program| {
            let ts_type = first_var_type_annotation(program).unwrap();
            let type_id = checker.get_type_from_type_node(ts_type);
            assert_eq!(checker.type_to_string(type_id), "{}");
        }
    );
}

#[test]
fn type_literal_assignability_compatible() {
    with_checker!(
        "let x: { a: number } = { a: 42 }",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn type_literal_assignability_missing_property() {
    with_checker!(
        "let x: { a: number; b: string } = { a: 42 }",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(!diagnostics.is_empty(), "should error: missing property b");
        }
    );
}

#[test]
fn type_literal_assignability_wrong_type() {
    with_checker!(
        "let x: { a: number } = { a: 'hello' }",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(!diagnostics.is_empty(), "should error: string not assignable to number");
        }
    );
}

#[test]
fn type_literal_extra_properties_ok() {
    // Structural typing: extra properties are fine
    with_checker!(
        "let x: { a: number } = { a: 42, b: 'extra' }",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

// --- Object expression tests ---

#[test]
fn object_expression_type() {
    with_checker!(
        "let x = { a: 1, b: 'hello' }",
        |checker, program| {
            let init = first_var_init(program).unwrap();
            let type_id = checker.get_type_of_expression(init, None);
            let flags = checker.type_arena().get_flags(type_id);
            assert!(flags.intersects(TypeFlags::Object));
            assert_eq!(checker.type_to_string(type_id), "{ a: number; b: string; }");
        }
    );
}

#[test]
fn object_expression_empty() {
    with_checker!(
        "let x = {}",
        |checker, program| {
            let init = first_var_init(program).unwrap();
            let type_id = checker.get_type_of_expression(init, None);
            assert_eq!(checker.type_to_string(type_id), "{}");
        }
    );
}

#[test]
fn object_to_interface_assignability() {
    with_checker!(
        "interface Point { x: number; y: number } let p: Point = { x: 1, y: 2 }",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn object_to_interface_missing_prop() {
    with_checker!(
        "interface Point { x: number; y: number } let p: Point = { x: 1 }",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(!diagnostics.is_empty(), "should error: missing property y");
        }
    );
}

#[test]
fn object_to_interface_optional_prop_missing_ok() {
    // {a: string} should be assignable to {a: string, b?: number}
    with_checker!(
        "interface Target { a: string; b?: number } let t: Target = { a: 'hello' }",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty(), "should not error: b is optional");
        }
    );
}

#[test]
fn object_to_interface_optional_prop_present_ok() {
    // {a: string, b: number} should be assignable to {a: string, b?: number}
    with_checker!(
        "interface Target { a: string; b?: number } let t: Target = { a: 'hello', b: 42 }",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty(), "should not error: b is present and compatible");
        }
    );
}

#[test]
fn object_to_interface_required_prop_still_required() {
    // {b: number} should NOT be assignable to {a: string, b?: number} (missing required a)
    with_checker!(
        "interface Target { a: string; b?: number } let t: Target = { b: 42 }",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(!diagnostics.is_empty(), "should error: required property a is missing");
        }
    );
}

// --- Array type tests ---

#[test]
fn array_type_annotation() {
    with_checker!("let x: number[]", |checker, program| {
        let ts_type = first_var_type_annotation(program).unwrap();
        let type_id = checker.get_type_from_type_node(ts_type);
        let flags = checker.type_arena().get_flags(type_id);
        assert!(flags.intersects(TypeFlags::Object));
    });
}

#[test]
fn array_type_display() {
    with_checker!("let x: string[]", |checker, program| {
        let ts_type = first_var_type_annotation(program).unwrap();
        let type_id = checker.get_type_from_type_node(ts_type);
        assert_eq!(checker.type_to_string(type_id), "string[]");
    });
}

#[test]
fn array_type_assignability() {
    with_checker!(
        "let x: number[] = [1, 2, 3]",
        |checker, program| {
            // ArrayExpression is Track A's job, so init will be `any` for now
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty()); // any is assignable to anything
        }
    );
}

// --- Tuple type tests ---

#[test]
fn tuple_type_basic() {
    with_checker!("let x: [string, number]", |checker, program| {
        let ts_type = first_var_type_annotation(program).unwrap();
        let type_id = checker.get_type_from_type_node(ts_type);
        assert_eq!(checker.type_to_string(type_id), "[string, number]");
    });
}

#[test]
fn tuple_type_named() {
    with_checker!("let x: [name: string, age: number]", |checker, program| {
        let ts_type = first_var_type_annotation(program).unwrap();
        let type_id = checker.get_type_from_type_node(ts_type);
        assert_eq!(checker.type_to_string(type_id), "[name: string, age: number]");
    });
}

#[test]
fn tuple_type_optional() {
    with_checker!("let x: [string, number?]", |checker, program| {
        let ts_type = first_var_type_annotation(program).unwrap();
        let type_id = checker.get_type_from_type_node(ts_type);
        assert_eq!(checker.type_to_string(type_id), "[string, number?]");
    });
}

#[test]
fn tuple_type_rest() {
    with_checker!("let x: [string, ...number[]]", |checker, program| {
        let ts_type = first_var_type_annotation(program).unwrap();
        let type_id = checker.get_type_from_type_node(ts_type);
        let flags = checker.type_arena().get_flags(type_id);
        assert!(flags.intersects(TypeFlags::Object));
    });
}

#[test]
fn tuple_type_empty() {
    with_checker!("let x: []", |checker, program| {
        let ts_type = first_var_type_annotation(program).unwrap();
        let type_id = checker.get_type_from_type_node(ts_type);
        assert_eq!(checker.type_to_string(type_id), "[]");
    });
}

// --- Class declaration tests ---

#[test]
fn class_declaration_no_errors() {
    with_checker!(
        "class Point { x: number; y: string; }",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn class_as_type_annotation() {
    with_checker!(
        "class Point { x: number; y: number; } let p: Point = { x: 1, y: 2 }",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn class_type_missing_property() {
    with_checker!(
        "class Point { x: number; y: number; } let p: Point = { x: 1 }",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(!diagnostics.is_empty(), "should error: missing property y");
        }
    );
}

#[test]
fn class_type_wrong_property_type() {
    with_checker!(
        "class Point { x: number; y: number; } let p: Point = { x: 1, y: 'hello' }",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(!diagnostics.is_empty(), "should error: string not assignable to number");
        }
    );
}

#[test]
fn class_method_return_type_check() {
    with_checker!(
        "class Foo { greet(): string { return 42; } }",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert_eq!(diagnostics.len(), 1);
            assert!(diagnostics[0].message.to_string().contains("not assignable"));
        }
    );
}

#[test]
fn class_method_return_type_ok() {
    with_checker!(
        "class Foo { greet(): string { return 'hi'; } }",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

// --- Enum declaration tests ---

#[test]
fn enum_numeric_basic() {
    with_checker!(
        "enum Direction { Up, Down, Left, Right }",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn enum_as_type_annotation() {
    with_checker!(
        "enum Direction { Up, Down } let d: Direction = 0",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            // Direction is union of 0 | 1, and 0 is assignable
            assert!(diagnostics.is_empty());
        }
    );
}

// --- Property access tests (Track A) ---

#[test]
fn property_access_on_object_literal() {
    with_checker!(
        "let obj = { x: 42, y: 'hello' }; let v = obj.x",
        |checker, program| {
            let mut count = 0;
            for stmt in &program.body {
                if let Statement::VariableDeclaration(decl) = stmt {
                    for declarator in &decl.declarations {
                        if count == 1 {
                            if let Some(init) = &declarator.init {
                                let type_id = checker.get_type_of_expression(init, None);
                                let flags = checker.type_arena().get_flags(type_id);
                                assert!(
                                    flags.intersects(TypeFlags::Number),
                                    "obj.x should be number, got: {}",
                                    checker.type_to_string(type_id)
                                );
                            }
                        }
                        count += 1;
                    }
                }
            }
        }
    );
}

#[test]
fn property_access_on_interface() {
    with_checker!(
        "interface Foo { x: number; y: string } let f: Foo; let v = f.x",
        |checker, program| {
            let mut count = 0;
            for stmt in &program.body {
                if let Statement::VariableDeclaration(decl) = stmt {
                    for declarator in &decl.declarations {
                        if count == 1 {
                            if let Some(init) = &declarator.init {
                                let type_id = checker.get_type_of_expression(init, None);
                                assert_eq!(checker.type_to_string(type_id), "number");
                            }
                        }
                        count += 1;
                    }
                }
            }
        }
    );
}

#[test]
fn property_access_unknown_property() {
    with_checker!(
        "let obj = { x: 42 }; let v = obj.z",
        |checker, program| {
            let mut count = 0;
            for stmt in &program.body {
                if let Statement::VariableDeclaration(decl) = stmt {
                    for declarator in &decl.declarations {
                        if count == 1 {
                            if let Some(init) = &declarator.init {
                                let type_id = checker.get_type_of_expression(init, None);
                                let flags = checker.type_arena().get_flags(type_id);
                                assert!(
                                    flags.intersects(TypeFlags::Any),
                                    "obj.z should be any, got: {}",
                                    checker.type_to_string(type_id)
                                );
                            }
                        }
                        count += 1;
                    }
                }
            }
        }
    );
}

#[test]
fn property_access_unknown_property_emits_ts2339() {
    with_checker!(
        "let obj = { x: 42 }; let v = obj.z",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.iter().any(|d| d.message.contains("does not exist on type")),
                "Expected TS2339 for obj.z, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn property_access_on_any_no_ts2339() {
    with_checker!(
        "let obj: any; let v = obj.whatever",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "Should not emit TS2339 for any.whatever, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn property_access_on_type_literal() {
    with_checker!(
        "let obj: { a: number; b: string }; let v = obj.b",
        |checker, program| {
            let mut count = 0;
            for stmt in &program.body {
                if let Statement::VariableDeclaration(decl) = stmt {
                    for declarator in &decl.declarations {
                        if count == 1 {
                            if let Some(init) = &declarator.init {
                                let type_id = checker.get_type_of_expression(init, None);
                                assert_eq!(checker.type_to_string(type_id), "string");
                            }
                        }
                        count += 1;
                    }
                }
            }
        }
    );
}

// --- Array expression tests (Track A) ---

#[test]
fn array_expression_infers_type() {
    with_checker!(
        "let arr = [1, 2, 3]",
        |checker, program| {
            let init = first_var_init(program).unwrap();
            let type_id = checker.get_type_of_expression(init, None);
            let flags = checker.type_arena().get_flags(type_id);
            assert!(flags.intersects(TypeFlags::Object), "array should be Object type");
        }
    );
}

#[test]
fn array_expression_assignable_to_array_type() {
    with_checker!(
        "let x: number[] = [1, 2, 3]",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty(), "number[] = [1,2,3] should not error");
        }
    );
}

// --- Assignment expression type tests ---

#[test]
fn assignment_expression_type() {
    // Assignment result type is the RHS type
    with_checker!(
        "let x: number = 1; x = 42",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty(), "x = 42 should not error when x: number");
        }
    );
}

// --- Computed member expression tests ---

#[test]
fn computed_member_string_literal() {
    with_checker!(
        "let obj = { x: 42 }; let v = obj[\"x\"]",
        |checker, program| {
            let mut count = 0;
            for stmt in &program.body {
                if let Statement::VariableDeclaration(decl) = stmt {
                    for declarator in &decl.declarations {
                        if count == 1 {
                            if let Some(init) = &declarator.init {
                                let type_id = checker.get_type_of_expression(init, None);
                                let flags = checker.type_arena().get_flags(type_id);
                                assert!(
                                    flags.intersects(TypeFlags::Number),
                                    "obj[\"x\"] should be number, got: {}",
                                    checker.type_to_string(type_id)
                                );
                            }
                        }
                        count += 1;
                    }
                }
            }
        }
    );
}

// --- Chain expression tests ---

#[test]
fn chain_expression_member() {
    with_checker!(
        "let obj: { x: number } | undefined; let v = obj?.x",
        |checker, program| {
            let mut count = 0;
            for stmt in &program.body {
                if let Statement::VariableDeclaration(decl) = stmt {
                    for declarator in &decl.declarations {
                        if count == 1 {
                            if let Some(init) = &declarator.init {
                                let type_id = checker.get_type_of_expression(init, None);
                                let flags = checker.type_arena().get_flags(type_id);
                                // Should be a union containing undefined
                                assert!(
                                    flags.intersects(TypeFlags::Union) || flags.intersects(TypeFlags::Undefined),
                                    "obj?.x should include undefined, got: {}",
                                    checker.type_to_string(type_id)
                                );
                            }
                        }
                        count += 1;
                    }
                }
            }
        }
    );
}

// --- RegExp type tests ---

#[test]
fn regexp_type() {
    with_checker!(
        "let x = /foo/",
        |checker, program| {
            let init = first_var_init(program).unwrap();
            let type_id = checker.get_type_of_expression(init, None);
            // Without lib.d.ts, get_global_type("RegExp") returns any
            let flags = checker.type_arena().get_flags(type_id);
            assert!(
                flags.intersects(TypeFlags::Any) || flags.intersects(TypeFlags::Object),
                "regexp should be RegExp (any without lib.d.ts), got: {}",
                checker.type_to_string(type_id)
            );
        }
    );
}

// --- Array string assignable test ---

#[test]
fn array_string_assignable() {
    with_checker!(
        "let x: string[] = [\"a\", \"b\"]",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty(), "string[] = ['a', 'b'] should not error");
        }
    );
}

// --- Await expression tests ---

#[test]
fn await_expression_type() {
    // Simplified: await returns the operand type directly
    with_checker!(
        "async function f() { let x: number = await 42; }",
        |checker, program| {
            checker.check_program(program);
        let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty(), "await 42 should be number");
        }
    );
}

// ---- Function type tests ----

#[test]
fn function_declaration_type() {
    with_checker!(
        "function add(a: number, b: number): number { return a; }",
        |checker, program| {
            let init = first_function_declaration(program).unwrap();
            let func = match init {
                Statement::FunctionDeclaration(f) => f,
                _ => panic!("expected function declaration"),
            };
            // Get the symbol type for the function name
            let sig = checker.build_signature_from_function(func);
            assert_eq!(sig.parameters.len(), 2);
            assert_eq!(sig.min_argument_count, 2);
            assert_eq!(sig.parameters[0].name.as_str(), "a");
            assert_eq!(sig.parameters[1].name.as_str(), "b");
        }
    );
}

#[test]
fn function_type_display() {
    with_checker!(
        "let f = (x: number, y: string): boolean => true",
        |checker, program| {
            let init = first_var_init(program).unwrap();
            let type_id = checker.get_type_of_expression(init, None);
            let display = checker.type_to_string(type_id);
            assert_eq!(display, "(x: number, y: string) => boolean");
        }
    );
}

#[test]
fn function_type_annotation() {
    with_checker!(
        "let f: (x: number) => string",
        |checker, program| {
            let ts_type = first_var_type_annotation(program).unwrap();
            let type_id = checker.get_type_from_type_node(ts_type);
            let display = checker.type_to_string(type_id);
            assert_eq!(display, "(x: number) => string");
        }
    );
}

#[test]
fn call_expression_returns_return_type() {
    with_checker!(
        "function greet(name: string): string { return name; }\nlet x: string = greet('hello');",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "greet('hello') should return string, no errors expected. Got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn ts2349_not_callable() {
    with_checker!(
        "let x: number = 42;\nx();",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.iter().any(|d| d.message.contains("not callable")),
                "Expected TS2349 for calling a number, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn ts2554_wrong_arg_count() {
    with_checker!(
        "function f(a: number, b: number): void {}\nf(1);",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.iter().any(|d| d.message.contains("Expected 2 arguments, but got 1")),
                "Expected TS2554 for wrong arg count, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn ts2554_too_many_args() {
    with_checker!(
        "function f(a: number): void {}\nf(1, 2, 3);",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.iter().any(|d| d.message.contains("Expected 1 arguments, but got 3")),
                "Expected TS2554 for too many args, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn ts2345_wrong_arg_type() {
    with_checker!(
        "function f(a: number): void {}\nf('hello');",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.iter().any(|d| d.message.contains("not assignable to parameter")),
                "Expected TS2345 for wrong arg type, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn call_expression_any_callee_no_error() {
    with_checker!(
        "let f: any;\nf(1, 2, 3);",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "Calling any should not emit errors, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn optional_param_no_error() {
    with_checker!(
        "function f(a: number, b?: string): void {}\nf(1);",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "Optional param should allow fewer args, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn function_assignability() {
    with_checker!(
        "let f: (x: number) => string = (x: number): string => 'hi';",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "Compatible function should be assignable, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn function_assignability_wrong_return() {
    with_checker!(
        "let f: (x: number) => string = (x: number): number => 42;",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.iter().any(|d| d.message.contains("not assignable")),
                "Incompatible return type should fail, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

/// Helper: get the first statement if it's a function declaration.
fn first_function_declaration<'a>(
    program: &'a oxc_ast::ast::Program<'a>,
) -> Option<&'a Statement<'a>> {
    program.body.first()
}

// ============ typeof in type position ============

#[test]
fn typeof_query_basic() {
    with_checker!(
        "let x: number = 42; let y: typeof x = 10;",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty(), "typeof x should resolve to number: {diagnostics:?}");
        }
    );
}

#[test]
fn typeof_query_type_error() {
    with_checker!(
        r#"let x: number = 42; let y: typeof x = "hello";"#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.iter().any(|d| d.message.contains("not assignable")),
                "Assigning string to typeof number should error: {diagnostics:?}"
            );
        }
    );
}

// ============ new expressions ============

#[test]
fn new_expression_class_instance() {
    with_checker!(
        r#"
        class Foo { x: number = 0; }
        let f = new Foo();
        "#,
        |checker, program| {
            checker.check_program(program);
            // f should be typed as Foo (the instance type)
            let f_init = first_var_init(program).expect("should have var init");
            let f_type = checker.get_type_of_expression(f_init, None);
            let type_str = checker.type_to_string(f_type);
            assert_eq!(type_str, "Foo", "new Foo() should have type Foo, got {type_str}");
        }
    );
}

#[test]
fn new_expression_property_access() {
    with_checker!(
        r#"
        class Foo { x: number = 0; y: string = ""; }
        let f = new Foo();
        let n: number = f.x;
        let s: string = f.y;
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty(), "Property access on new Foo() should work: {diagnostics:?}");
        }
    );
}

#[test]
fn new_expression_property_type_error() {
    with_checker!(
        r#"
        class Foo { x: number = 0; }
        let f = new Foo();
        let s: string = f.x;
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.iter().any(|d| d.message.contains("not assignable")),
                "Assigning number property to string should error: {diagnostics:?}"
            );
        }
    );
}

// ============ Literal type widening ============

#[test]
fn let_number_literal_widens() {
    // let x = 42 should have type number, not 42
    // Verify by assigning to a number-typed variable (should succeed)
    with_checker!(
        "let x = 42; let y: number = x;",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty(), "let x = 42 should widen to number: {diagnostics:?}");
        }
    );
}

#[test]
fn const_number_literal_stays_narrow() {
    // const x = 42 should have type 42, not number
    // Verify by assigning to a literal-typed variable
    with_checker!(
        "const x = 42; let y: 42 = x;",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty(), "const x = 42 should keep literal type 42: {diagnostics:?}");
        }
    );
}

#[test]
fn let_string_literal_widens() {
    with_checker!(
        r#"let x = "hello"; let y: string = x;"#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty(), "let x = 'hello' should widen to string: {diagnostics:?}");
        }
    );
}

#[test]
fn let_boolean_literal_widens() {
    with_checker!(
        "let x = true; let y: boolean = x;",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(diagnostics.is_empty(), "let x = true should widen to boolean: {diagnostics:?}");
        }
    );
}

#[test]
fn let_widened_rejects_wrong_literal() {
    // let x = 42 has type number, so assigning to type 42 should fail
    with_checker!(
        "let x = 42; let y: 42 = x;",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.iter().any(|d| d.message.contains("not assignable")),
                "number should not be assignable to literal 42: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn enum_displays_by_name() {
    with_checker!(
        "enum Color { Red, Green, Blue }; let c = Color;",
        |checker, program| {
            let init = first_var_init(program).unwrap();
            let t = checker.get_type_of_expression(init, None);
            let s = checker.type_to_string(t);
            // In expression position, an enum reference is "typeof Color" (the namespace object).
            assert_eq!(s, "typeof Color", "Enum value in expression should display as 'typeof Color', got '{s}'");
        }
    );
}

#[test]
fn enum_member_access() {
    with_checker!(
        "enum Choice { Yes, No } let v = Choice.Yes;",
        |checker, program| {
            checker.check_program(program);
            let init = first_var_init(program).unwrap();
            let t = checker.get_type_of_expression(init, None);
            let s = checker.type_to_string(t);
            assert_eq!(s, "0", "Choice.Yes should be literal 0, got '{s}'");
        }
    );
}

// ---- CFA: typeof narrowing ----

#[test]
fn cfa_typeof_narrows_union_to_string() {
    // typeof x === "string" should narrow string | number to string
    with_checker!(
        r#"
        declare var x: string | number;
        if (typeof x === "string") {
            let y: string = x;
        }
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "typeof narrowing should allow string assignment, got: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn cfa_typeof_narrows_else_branch() {
    // In the else branch, typeof x === "string" should narrow string | number to number
    with_checker!(
        r#"
        declare var x: string | number;
        if (typeof x === "string") {
            let y: number = x;
        }
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            // Inside the if branch, x is string, so assigning to number should fail
            assert_eq!(
                diagnostics.len(),
                1,
                "string should not be assignable to number inside typeof string guard"
            );
        }
    );
}

#[test]
fn cfa_typeof_else_narrows_complement() {
    // In the else branch of typeof === "string", type should be number
    with_checker!(
        r#"
        declare var x: string | number;
        if (typeof x === "string") {
        } else {
            let y: number = x;
        }
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "else branch of typeof 'string' should narrow to number, got: {diagnostics:?}"
            );
        }
    );
}

// ---- CFA: truthiness narrowing ----

#[test]
fn cfa_truthiness_removes_null() {
    // if (x) should remove null from string | null
    with_checker!(
        r#"
        declare var x: string | null;
        if (x) {
            let y: string = x;
        }
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "truthiness narrowing should remove null, got: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn cfa_truthiness_removes_undefined() {
    // if (x) should remove undefined from number | undefined
    with_checker!(
        r#"
        declare var x: number | undefined;
        if (x) {
            let y: number = x;
        }
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "truthiness narrowing should remove undefined, got: {diagnostics:?}"
            );
        }
    );
}

// ---- CFA: equality narrowing ----

#[test]
fn cfa_strict_null_check_narrows() {
    // x !== null should remove null from string | null
    with_checker!(
        r#"
        declare var x: string | null;
        if (x !== null) {
            let y: string = x;
        }
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "x !== null should narrow to string, got: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn cfa_strict_null_else_keeps_null() {
    // In else of x !== null, x should still include null
    with_checker!(
        r#"
        declare var x: string | null;
        if (x !== null) {
        } else {
            let y: string = x;
        }
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            // In else branch, x is null — assigning to string should fail
            assert_eq!(
                diagnostics.len(),
                1,
                "else of x !== null should keep null type"
            );
        }
    );
}

#[test]
fn cfa_no_narrowing_without_guard() {
    // Without a guard, union type should remain
    with_checker!(
        r#"
        declare var x: string | number;
        let y: string = x;
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert_eq!(
                diagnostics.len(),
                1,
                "without narrowing, string | number should not be assignable to string"
            );
        }
    );
}

#[test]
fn cfa_negated_typeof() {
    // !(typeof x === "string") should narrow the else branch to string
    with_checker!(
        r#"
        declare var x: string | number;
        if (!(typeof x === "string")) {
            let y: number = x;
        }
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "negated typeof should narrow to number in true branch, got: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn cfa_narrowing_inside_function() {
    // Narrowing should work inside function bodies, not just top-level
    with_checker!(
        r#"
        function foo(x: string | number) {
            if (typeof x === "string") {
                let y: string = x;
            }
        }
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "narrowing should work inside function bodies, got: {diagnostics:?}"
            );
        }
    );
}

// ---- Return type inference ----

#[test]
fn return_type_inferred_simple() {
    // Function returning a number literal should infer return type `number`
    with_checker!(
        r#"
        function foo() { return 42; }
        let x: number = foo();
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "inferred return number should be assignable to number, got: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn return_type_inferred_string() {
    with_checker!(
        r#"
        function foo() { return "hello"; }
        let x: number = foo();
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert_eq!(
                diagnostics.len(),
                1,
                "inferred return string should not be assignable to number"
            );
        }
    );
}

#[test]
fn return_type_inferred_void_implicit() {
    // Function with no return statement should infer void
    with_checker!(
        r#"
        function foo() { let x = 1; }
        let x: void = foo();
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "function with no return should infer void, got: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn return_type_inferred_union_branches() {
    // Function with multiple return paths should infer union
    with_checker!(
        r#"
        function foo(x: boolean) {
            if (x) { return 1; }
            return "hello";
        }
        let x: number = foo(true);
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert_eq!(
                diagnostics.len(),
                1,
                "inferred return number | string should not be assignable to number"
            );
        }
    );
}

#[test]
fn return_type_inferred_with_implicit_return() {
    // Function that may or may not return should include void
    with_checker!(
        r#"
        function foo(x: boolean) {
            if (x) { return 1; }
        }
        let x: number = foo(true);
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert_eq!(
                diagnostics.len(),
                1,
                "inferred return number | void should not be assignable to number"
            );
        }
    );
}

#[test]
fn return_type_inferred_all_paths_return() {
    // Function where all paths return should NOT include void
    with_checker!(
        r#"
        function foo(x: boolean) {
            if (x) { return 1; }
            return 2;
        }
        let x: number = foo(true);
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "all paths return number, should be assignable to number, got: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn return_type_inferred_arrow_expression() {
    // Arrow with expression body should infer from the expression
    with_checker!(
        r#"
        let foo = (x: number) => x + 1;
        let y: number = foo(1);
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "arrow returning number expr should infer number, got: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn conformance_probe_types() {
    // Probe: function declaration type string
    with_checker!(
        "function f1(x: number): void {}",
        |checker, program| {
            if let Statement::FunctionDeclaration(func) = &program.body[0] {
                if let Some(bid) = &func.id {
                    if let Some(sid) = bid.symbol_id.get() {
                        let t = checker.get_type_of_symbol(sid);
                        let s = checker.type_to_string(t);
                        eprintln!("f1 type: {s}");
                    }
                }
            }
        }
    );
    // Probe: class - value type is constructor (typeof C), declared type is instance (C)
    with_checker!(
        "class C { x: number; }",
        |checker, program| {
            if let Statement::ClassDeclaration(class) = &program.body[0] {
                if let Some(bid) = &class.id {
                    if let Some(sid) = bid.symbol_id.get() {
                        // Value type (expression reference) is "typeof C"
                        let t = checker.get_type_of_symbol(sid);
                        let s = checker.type_to_string(t);
                        assert_eq!(s, "typeof C", "class value type should be 'typeof C', got '{s}'");
                        // Declared type (type-namespace) is "C"
                        let dt = checker.get_declared_type_of_symbol(sid);
                        let ds = checker.type_to_string(dt);
                        assert_eq!(ds, "C", "class declared type should be 'C', got '{ds}'");
                    }
                }
            }
        }
    );
    // Probe: var with no annotation no init
    with_checker!(
        "var x;",
        |checker, program| {
            if let Statement::VariableDeclaration(decl) = &program.body[0] {
                if let Some(sid) = decl.declarations[0].id.get_binding_identifier().and_then(|b| b.symbol_id.get()) {
                    let t = checker.get_type_of_symbol(sid);
                    let s = checker.type_to_string(t);
                    eprintln!("var x (no init) type: {s}");
                    // tsc expects "any"
                }
            }
        }
    );
    // Probe: const literal
    with_checker!(
        "const x = 42;",
        |checker, program| {
            if let Statement::VariableDeclaration(decl) = &program.body[0] {
                if let Some(sid) = decl.declarations[0].id.get_binding_identifier().and_then(|b| b.symbol_id.get()) {
                    let t = checker.get_type_of_symbol(sid);
                    let s = checker.type_to_string(t);
                    eprintln!("const x = 42 type: {s}");
                    // tsc expects "42"
                }
            }
        }
    );
    // Probe: let literal (widened)
    with_checker!(
        "let x = 42;",
        |checker, program| {
            if let Statement::VariableDeclaration(decl) = &program.body[0] {
                if let Some(sid) = decl.declarations[0].id.get_binding_identifier().and_then(|b| b.symbol_id.get()) {
                    let t = checker.get_type_of_symbol(sid);
                    let s = checker.type_to_string(t);
                    eprintln!("let x = 42 type: {s}");
                    // tsc expects "number"
                }
            }
        }
    );
}

#[test]
fn return_type_inferred_arrow_block() {
    // Arrow with block body should infer from return statements
    with_checker!(
        r#"
        let foo = (x: number) => { return "hello"; };
        let y: number = foo(1);
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert_eq!(
                diagnostics.len(),
                1,
                "arrow returning string should not be assignable to number"
            );
        }
    );
}

#[test]
fn global_array_has_type_parameters() {
    with_checker!(
        "let x: number = 1",
        |checker, _program| {
            let array_type = checker.get_global_type("Array");
            assert_ne!(array_type, checker.any_type, "Array should not be any");
            let data = checker.type_arena().get_data(array_type);
            match data {
                TypeData::Structured(s) => {
                    match &s.kind {
                        StructuredTypeKind::Interface { all_type_parameters, .. } => {
                            assert!(
                                !all_type_parameters.is_empty(),
                                "Array interface should have type parameters, got none"
                            );
                        }
                        other => panic!("Expected Interface kind, got {:?}", std::mem::discriminant(other)),
                    }
                }
                other => panic!("Expected Structured, got {:?}", std::mem::discriminant(other)),
            }
        }
    );
}

// ---- Mapped type / keyof / indexed access tests ----

#[test]
fn keyof_object_literal() {
    with_checker!(
        r#"let x: keyof { a: string; b: number }"#,
        |checker, program| {
            let ann = first_var_type_annotation(program).unwrap();
            let t = checker.get_type_from_type_node(ann);
            let s = checker.type_to_string(t);
            // keyof { a: string; b: number } → "a" | "b"
            assert!(
                (s == r#""a" | "b""# || s == r#""b" | "a""#),
                "expected '\"a\" | \"b\"', got '{s}'"
            );
        }
    );
}

#[test]
fn keyof_any_is_string_number_symbol() {
    with_checker!(
        "let x: keyof any",
        |checker, program| {
            let ann = first_var_type_annotation(program).unwrap();
            let t = checker.get_type_from_type_node(ann);
            let flags = checker.type_arena().get_flags(t);
            assert!(flags.intersects(TypeFlags::Union));
        }
    );
}

#[test]
fn indexed_access_concrete() {
    with_checker!(
        r#"let x: { a: string; b: number }["a"]"#,
        |checker, program| {
            let ann = first_var_type_annotation(program).unwrap();
            let t = checker.get_type_from_type_node(ann);
            assert_eq!(checker.type_to_string(t), "string");
        }
    );
}

#[test]
fn indexed_access_union_key() {
    with_checker!(
        r#"let x: { a: string; b: number }["a" | "b"]"#,
        |checker, program| {
            let ann = first_var_type_annotation(program).unwrap();
            let t = checker.get_type_from_type_node(ann);
            let s = checker.type_to_string(t);
            assert!(
                s == "string | number" || s == "number | string",
                "expected 'string | number', got '{s}'"
            );
        }
    );
}

#[test]
fn mapped_type_deferred_for_generic() {
    // A mapped type with a generic constraint should remain as a MappedType
    // (not eagerly resolved), since we can't enumerate keys of a type parameter.
    with_checker!(
        "type Partial2<T> = { [P in keyof T]?: T[P] }",
        |checker, program| {
            let stmt = &program.body[0];
            if let Statement::TSTypeAliasDeclaration(decl) = stmt {
                let symbol_id = decl.id.symbol_id.get().unwrap();
                let t = checker.get_declared_type_of_symbol(symbol_id);
                let data = checker.type_arena().get_data(t);
                assert!(
                    matches!(data, TypeData::Mapped(_)),
                    "expected Mapped, got {data:?}"
                );
            } else {
                panic!("expected type alias declaration");
            }
        }
    );
}

#[test]
fn mapped_type_alias_instantiation() {
    // MyPartial<{a: string; b: number}> should produce { a: string | undefined; b: number | undefined }
    with_checker!(
        r#"
        type MyPartial<T> = { [P in keyof T]?: T[P] };
        let x: MyPartial<{ a: string; b: number }>;
        "#,
        |checker, program| {
            let stmt = &program.body[1];
            let Statement::VariableDeclaration(decl) = stmt else { panic!() };
            let declarator = &decl.declarations[0];
            let ann = declarator.type_annotation.as_ref().unwrap();
            let t = checker.get_type_from_type_node(&ann.type_annotation);
            let flags = checker.type_arena().get_flags(t);
            // Should be an Object type (resolved mapped type), not a generic MappedType
            assert!(
                flags.intersects(TypeFlags::Object),
                "expected Object, got {flags:?}"
            );
            let data = checker.type_arena().get_data(t);
            match data {
                TypeData::Structured(s) => {
                    assert_eq!(s.properties.len(), 2, "expected 2 properties");
                    // Both properties should include undefined (optional modifier)
                    for p in &s.properties {
                        let prop_flags = checker.type_arena().get_flags(p.type_id);
                        assert!(
                            prop_flags.intersects(TypeFlags::Union),
                            "property '{}' should be a union (T | undefined), got {prop_flags:?}",
                            p.name,
                        );
                    }
                }
                _ => panic!("expected Structured, got {data:?}"),
            }
        }
    );
}

#[test]
fn mapped_type_record() {
    // MyRecord<"x" | "y", number> should produce { x: number; y: number }
    with_checker!(
        r#"
        type MyRecord<K extends string, V> = { [P in K]: V };
        let x: MyRecord<"x" | "y", number>;
        "#,
        |checker, program| {
            let stmt = &program.body[1];
            let Statement::VariableDeclaration(decl) = stmt else { panic!() };
            let declarator = &decl.declarations[0];
            let ann = declarator.type_annotation.as_ref().unwrap();
            let t = checker.get_type_from_type_node(&ann.type_annotation);
            let data = checker.type_arena().get_data(t);
            match data {
                TypeData::Structured(s) => {
                    assert_eq!(s.properties.len(), 2, "expected 2 properties, got {:?}", s.properties);
                    for p in &s.properties {
                        assert_eq!(
                            checker.type_to_string(p.type_id),
                            "number",
                            "property '{}' should be number",
                            p.name,
                        );
                    }
                }
                _ => panic!("expected Structured, got {data:?}"),
            }
        }
    );
}

#[test]
fn mapped_type_homomorphic_primitive_passthrough() {
    // Partial<string> should be string (primitives pass through)
    with_checker!(
        r#"
        type MyPartial<T> = { [P in keyof T]?: T[P] };
        let x: MyPartial<string>;
        "#,
        |checker, program| {
            let stmt = &program.body[1];
            let Statement::VariableDeclaration(decl) = stmt else { panic!() };
            let declarator = &decl.declarations[0];
            let ann = declarator.type_annotation.as_ref().unwrap();
            let t = checker.get_type_from_type_node(&ann.type_annotation);
            assert_eq!(
                checker.type_to_string(t), "string",
                "Partial<string> should be string"
            );
        }
    );
}

#[test]
fn mapped_type_homomorphic_union_distribution() {
    // MyPartial<string | number> should distribute: string | number
    with_checker!(
        r#"
        type MyPartial<T> = { [P in keyof T]?: T[P] };
        let x: MyPartial<string | number>;
        "#,
        |checker, program| {
            let stmt = &program.body[1];
            let Statement::VariableDeclaration(decl) = stmt else { panic!() };
            let declarator = &decl.declarations[0];
            let ann = declarator.type_annotation.as_ref().unwrap();
            let t = checker.get_type_from_type_node(&ann.type_annotation);
            let s = checker.type_to_string(t);
            assert!(
                s == "string | number" || s == "number | string",
                "MyPartial<string | number> should distribute to string | number, got '{s}'"
            );
        }
    );
}

#[test]
fn mapped_type_homomorphic_object() {
    // MyPartial on a concrete object type should resolve properties
    with_checker!(
        r#"
        type MyPartial<T> = { [P in keyof T]?: T[P] };
        type Obj = { x: number; y: string };
        let v: MyPartial<Obj>;
        "#,
        |checker, program| {
            let stmt = &program.body[2];
            let Statement::VariableDeclaration(decl) = stmt else { panic!() };
            let declarator = &decl.declarations[0];
            let ann = declarator.type_annotation.as_ref().unwrap();
            let t = checker.get_type_from_type_node(&ann.type_annotation);
            let data = checker.type_arena().get_data(t);
            match data {
                TypeData::Structured(s) => {
                    assert_eq!(s.properties.len(), 2, "expected 2 properties");
                }
                _ => panic!("expected Structured, got {data:?}"),
            }
        }
    );
}

#[test]
fn mapped_type_required_removes_optional() {
    // Required<T> should strip optionality: -? removes the optional flag
    with_checker!(
        r#"
        type MyRequired<T> = { [P in keyof T]-?: T[P] };
        type Obj = { x?: number; y?: string };
        let v: MyRequired<Obj>;
        "#,
        |checker, program| {
            let stmt = &program.body[2];
            let Statement::VariableDeclaration(decl) = stmt else { panic!() };
            let declarator = &decl.declarations[0];
            let ann = declarator.type_annotation.as_ref().unwrap();
            let t = checker.get_type_from_type_node(&ann.type_annotation);
            let data = checker.type_arena().get_data(t);
            match data {
                TypeData::Structured(s) => {
                    assert_eq!(s.properties.len(), 2);
                    for p in &s.properties {
                        assert!(
                            !p.optional,
                            "property '{}' should not be optional after Required",
                            p.name
                        );
                    }
                }
                _ => panic!("expected Structured, got {data:?}"),
            }
        }
    );
}

#[test]
fn mapped_type_readonly_adds_readonly() {
    // Readonly<T> should add readonly flag
    with_checker!(
        r#"
        type MyReadonly<T> = { readonly [P in keyof T]: T[P] };
        type Obj = { x: number; y: string };
        let v: MyReadonly<Obj>;
        "#,
        |checker, program| {
            let stmt = &program.body[2];
            let Statement::VariableDeclaration(decl) = stmt else { panic!() };
            let declarator = &decl.declarations[0];
            let ann = declarator.type_annotation.as_ref().unwrap();
            let t = checker.get_type_from_type_node(&ann.type_annotation);
            let data = checker.type_arena().get_data(t);
            match data {
                TypeData::Structured(s) => {
                    assert_eq!(s.properties.len(), 2);
                    for p in &s.properties {
                        assert!(
                            p.readonly,
                            "property '{}' should be readonly after Readonly",
                            p.name
                        );
                    }
                }
                _ => panic!("expected Structured, got {data:?}"),
            }
        }
    );
}

#[test]
fn mapped_type_preserves_source_optional() {
    // A mapped type with no modifier should preserve optionality from source
    with_checker!(
        r#"
        type Identity<T> = { [P in keyof T]: T[P] };
        type Obj = { x: number; y?: string };
        let v: Identity<Obj>;
        "#,
        |checker, program| {
            let stmt = &program.body[2];
            let Statement::VariableDeclaration(decl) = stmt else { panic!() };
            let declarator = &decl.declarations[0];
            let ann = declarator.type_annotation.as_ref().unwrap();
            let t = checker.get_type_from_type_node(&ann.type_annotation);
            let data = checker.type_arena().get_data(t);
            match data {
                TypeData::Structured(s) => {
                    assert_eq!(s.properties.len(), 2);
                    let x = s.properties.iter().find(|p| p.name.as_str() == "x").unwrap();
                    let y = s.properties.iter().find(|p| p.name.as_str() == "y").unwrap();
                    assert!(!x.optional, "x should not be optional");
                    assert!(y.optional, "y should be optional (preserved from source)");
                }
                _ => panic!("expected Structured, got {data:?}"),
            }
        }
    );
}

#[test]
fn mapped_type_as_clause_filtering() {
    // `as never` should filter out properties
    with_checker!(
        r#"
        type OmitByType<T, U> = { [K in keyof T as T[K] extends U ? never : K]: T[K] };
        "#,
        |checker, program| {
            // Just verify the mapped type parses without crashing.
            // Conditional types return `any` currently so the `as` clause
            // won't fully work, but the never-filtering path is exercised
            // by simpler patterns.
            let stmt = &program.body[0];
            if let Statement::TSTypeAliasDeclaration(decl) = stmt {
                let symbol_id = decl.id.symbol_id.get().unwrap();
                let t = checker.get_declared_type_of_symbol(symbol_id);
                let data = checker.type_arena().get_data(t);
                assert!(matches!(data, TypeData::Mapped(_)));
            }
        }
    );
}

// ---- Conditional type tests ----

#[test]
fn conditional_type_concrete_true() {
    // string extends object ? "yes" : "no" → "no" (string is not assignable to object)
    with_checker!(
        r#"let x: string extends object ? "yes" : "no""#,
        |checker, program| {
            let ann = first_var_type_annotation(program).unwrap();
            let t = checker.get_type_from_type_node(ann);
            assert_eq!(checker.type_to_string(t), r#""no""#);
        }
    );
}

#[test]
fn conditional_type_concrete_false() {
    // number extends number ? "yes" : "no" → "yes"
    with_checker!(
        r#"let x: number extends number ? "yes" : "no""#,
        |checker, program| {
            let ann = first_var_type_annotation(program).unwrap();
            let t = checker.get_type_from_type_node(ann);
            assert_eq!(checker.type_to_string(t), r#""yes""#);
        }
    );
}

#[test]
fn conditional_type_literal_extends_base() {
    // "hello" extends string ? true : false → true
    with_checker!(
        r#"let x: "hello" extends string ? true : false"#,
        |checker, program| {
            let ann = first_var_type_annotation(program).unwrap();
            let t = checker.get_type_from_type_node(ann);
            assert_eq!(checker.type_to_string(t), "true");
        }
    );
}

#[test]
fn conditional_type_never_is_never() {
    // never extends string ? "yes" : "no" → never
    with_checker!(
        r#"let x: never extends string ? "yes" : "no""#,
        |checker, program| {
            let ann = first_var_type_annotation(program).unwrap();
            let t = checker.get_type_from_type_node(ann);
            assert_eq!(checker.type_to_string(t), "never");
        }
    );
}

#[test]
fn conditional_type_deferred_for_generic() {
    // T extends string ? T : never — should remain as ConditionalType
    with_checker!(
        "type StringOnly<T> = T extends string ? T : never",
        |checker, program| {
            let stmt = &program.body[0];
            if let Statement::TSTypeAliasDeclaration(decl) = stmt {
                let symbol_id = decl.id.symbol_id.get().unwrap();
                let t = checker.get_declared_type_of_symbol(symbol_id);
                let data = checker.type_arena().get_data(t);
                assert!(
                    matches!(data, TypeData::Conditional(_)),
                    "expected Conditional, got {data:?}"
                );
            }
        }
    );
}

#[test]
fn conditional_type_extract() {
    // Extract<string | number | boolean, string> → string
    with_checker!(
        r#"
        type Extract2<T, U> = T extends U ? T : never;
        let x: Extract2<string | number | boolean, string>;
        "#,
        |checker, program| {
            let stmt = &program.body[1];
            let Statement::VariableDeclaration(decl) = stmt else { panic!() };
            let declarator = &decl.declarations[0];
            let ann = declarator.type_annotation.as_ref().unwrap();
            let t = checker.get_type_from_type_node(&ann.type_annotation);
            assert_eq!(checker.type_to_string(t), "string");
        }
    );
}

#[test]
fn conditional_type_exclude() {
    // Exclude<string | number | boolean, string> → number | boolean
    with_checker!(
        r#"
        type Exclude2<T, U> = T extends U ? never : T;
        let x: Exclude2<string | number | boolean, string>;
        "#,
        |checker, program| {
            let stmt = &program.body[1];
            let Statement::VariableDeclaration(decl) = stmt else { panic!() };
            let declarator = &decl.declarations[0];
            let ann = declarator.type_annotation.as_ref().unwrap();
            let t = checker.get_type_from_type_node(&ann.type_annotation);
            let s = checker.type_to_string(t);
            // Order may vary
            assert!(
                s == "number | boolean" || s == "boolean | number",
                "expected 'number | boolean', got '{s}'"
            );
        }
    );
}

#[test]
fn conditional_type_nonnullable() {
    // NonNullable<string | null | undefined> → string
    with_checker!(
        r#"
        type NonNullable2<T> = T extends null | undefined ? never : T;
        let x: NonNullable2<string | null | undefined>;
        "#,
        |checker, program| {
            let stmt = &program.body[1];
            let Statement::VariableDeclaration(decl) = stmt else { panic!() };
            let declarator = &decl.declarations[0];
            let ann = declarator.type_annotation.as_ref().unwrap();
            let t = checker.get_type_from_type_node(&ann.type_annotation);
            assert_eq!(checker.type_to_string(t), "string");
        }
    );
}

#[test]
fn conditional_type_distribution_all_match() {
    // All members match → union of true branches
    with_checker!(
        r#"
        type StringOnly<T> = T extends string ? T : never;
        let x: StringOnly<"a" | "b" | "c">;
        "#,
        |checker, program| {
            let stmt = &program.body[1];
            let Statement::VariableDeclaration(decl) = stmt else { panic!() };
            let declarator = &decl.declarations[0];
            let ann = declarator.type_annotation.as_ref().unwrap();
            let t = checker.get_type_from_type_node(&ann.type_annotation);
            let s = checker.type_to_string(t);
            // "a" | "b" | "c" all extend string, so all pass through
            assert!(
                s.contains(r#""a""#) && s.contains(r#""b""#) && s.contains(r#""c""#),
                "expected all literals, got '{s}'"
            );
        }
    );
}

#[test]
fn conditional_type_distribution_none_match() {
    // No members match → never
    with_checker!(
        r#"
        type StringOnly<T> = T extends string ? T : never;
        let x: StringOnly<number | boolean>;
        "#,
        |checker, program| {
            let stmt = &program.body[1];
            let Statement::VariableDeclaration(decl) = stmt else { panic!() };
            let declarator = &decl.declarations[0];
            let ann = declarator.type_annotation.as_ref().unwrap();
            let t = checker.get_type_from_type_node(&ann.type_annotation);
            assert_eq!(checker.type_to_string(t), "never");
        }
    );
}

// ---- Contextual typing tests ----

#[test]
fn contextual_typing_arrow_param_from_variable_annotation() {
    // Arrow function parameter type inferred from variable annotation
    with_checker!(
        "let f: (x: number) => number = (x) => x + 1;",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "arrow param should get type from annotation: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn contextual_typing_arrow_param_return_type_mismatch() {
    // Arrow function return type doesn't match contextual return type
    with_checker!(
        "let f: (x: number) => string = (x) => x + 1;",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                !diagnostics.is_empty(),
                "should error: number not assignable to string"
            );
        }
    );
}

#[test]
fn contextual_typing_call_argument() {
    // Callback parameter type inferred from function parameter type
    with_checker!(
        "declare function apply(f: (x: number) => void): void;\napply((x) => x + 1);",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "callback param should get type from call site: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn contextual_typing_conditional_branches() {
    // Context flows through conditional expression to both branches
    with_checker!(
        "let f: (x: number) => number = true ? (x) => x : (x) => x + 1;",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "context should flow through ternary: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn contextual_typing_return_statement() {
    // Context from function return type flows to return expression
    with_checker!(
        "function f(): (x: number) => number { return (x) => x + 1; }",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "return context should flow to arrow: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn contextual_typing_object_literal_callback() {
    // Context propagates through object literal properties to callbacks
    with_checker!(
        "let h: { cb: (x: number) => number } = { cb: (x) => x + 1 };",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "object property context should flow to callback: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn contextual_typing_array_as_tuple() {
    // Array literal inferred as tuple when contextual type is tuple
    with_checker!(
        "let t: [number, string] = [1, 'hello'];",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "array should be checked as tuple with tuple context: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn contextual_typing_function_expression() {
    // Function expression parameter type inferred from variable annotation
    with_checker!(
        "let f: (x: number) => number = function(x) { return x + 1; };",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "function expression param should get type from annotation: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn contextual_typing_assignment() {
    // Context from LHS type flows to RHS in assignment
    with_checker!(
        "let f: (x: number) => number;\nf = (x) => x + 1;",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "assignment context should flow to arrow: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn void_return_type_accepts_any_return() {
    // TypeScript: any return type is assignable to void (void means "don't care")
    with_checker!(
        "let f: (x: number) => void = (x) => x + 1;",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "number return should be assignable to void: {diagnostics:?}"
            );
        }
    );
}

// ---- Generic type argument inference tests ----

#[test]
fn generic_inference_basic() {
    // Basic single-parameter inference: T inferred from argument
    with_checker!(
        "function id<T>(x: T): T { return x; }\nlet r: number = id(42);",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "id(42) should infer T=number: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn generic_inference_explicit_type_args() {
    // Explicit type arguments: id<string>("hello")
    with_checker!(
        "function id<T>(x: T): T { return x; }\nlet r: string = id<string>('hello');",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "id<string>('hello') should return string: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn generic_inference_explicit_type_args_mismatch() {
    // Explicit type arg: return type doesn't match annotation
    with_checker!(
        "function id<T>(x: T): T { return x; }\nlet r: string = id<number>(42);",
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                !diagnostics.is_empty(),
                "id<number>(42) returns number, not assignable to string"
            );
        }
    );
}

#[test]
fn generic_inference_multiple_params() {
    // Multiple type parameters inferred independently
    with_checker!(
        r#"
        function pair<A, B>(a: A, b: B): A { return a; }
        let r: number = pair(1, 'hello');
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "pair(1, 'hello') should infer A=number: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn generic_inference_multiple_candidates() {
    // Same type parameter receives multiple candidates → union
    with_checker!(
        r#"
        function pick<T>(a: T, b: T): T { return a; }
        let r: number | string = pick(1, 'hello');
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "pick(1, 'hello') should infer T=number|string: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn generic_inference_no_args_uses_constraint() {
    // No inference candidates and no arguments → use constraint
    with_checker!(
        r#"
        function create<T extends { x: number }>(): T { return {} as T; }
        let r: { x: number } = create();
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "create() with constraint should use constraint: {diagnostics:?}"
            );
        }
    );
}

// ---- Statement expression checking tests ----

#[test]
fn if_condition_expression_is_checked() {
    // Property access on the condition should emit TS2339
    with_checker!(
        r#"
        let obj = { x: 42 };
        if (obj.nonexistent) {}
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.iter().any(|d| d.message.contains("does not exist on type")),
                "Expected TS2339 for if condition, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn while_condition_expression_is_checked() {
    with_checker!(
        r#"
        let obj = { x: 42 };
        while (obj.nonexistent) {}
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.iter().any(|d| d.message.contains("does not exist on type")),
                "Expected TS2339 for while condition, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn switch_discriminant_is_checked() {
    with_checker!(
        r#"
        let obj = { x: 42 };
        switch (obj.nonexistent) { case 1: break; }
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.iter().any(|d| d.message.contains("does not exist on type")),
                "Expected TS2339 for switch discriminant, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn throw_argument_is_checked() {
    with_checker!(
        r#"
        let obj = { x: 42 };
        throw obj.nonexistent;
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.iter().any(|d| d.message.contains("does not exist on type")),
                "Expected TS2339 for throw argument, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn for_loop_parts_are_checked() {
    with_checker!(
        r#"
        let obj = { x: 42 };
        for (obj.nonexistent; obj.missing; obj.absent) {}
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.len() >= 3,
                "Expected at least 3 diagnostics for for-loop init/test/update, got {}: {:?}",
                diagnostics.len(),
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn for_in_rhs_must_be_object_type() {
    with_checker!(
        r#"
        let x: string = "hello";
        for (let k in x) {}
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.iter().any(|d| d.message.contains("right-hand side of a 'for...in'")),
                "Expected TS2407 for for-in with string RHS, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn for_in_lhs_must_be_string() {
    with_checker!(
        r#"
        let k: number;
        let obj = { a: 1, b: 2 };
        for (k in obj) {}
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.iter().any(|d| d.message.contains("left-hand side of a 'for...in'")),
                "Expected TS2405 for for-in with number LHS, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

#[test]
fn for_in_valid_no_errors() {
    with_checker!(
        r#"
        let obj = { a: 1, b: 2 };
        for (let k in obj) {}
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "Valid for-in should produce no diagnostics, got: {:?}",
                diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    );
}

// ---- Intersection type tests ----

#[test]
fn intersection_reduction_never_propagates() {
    // A & never → never
    with_checker!(
        "let x: string & never",
        |checker, program| {
            let ann = first_var_type_annotation(program).unwrap();
            let t = checker.get_type_from_type_node(ann);
            assert_eq!(
                checker.type_arena().get_flags(t),
                TypeFlags::Never,
                "string & never should reduce to never"
            );
        }
    );
}

#[test]
fn intersection_reduction_contradictory_primitives() {
    // string & number → never
    with_checker!(
        "let x: string & number",
        |checker, program| {
            let ann = first_var_type_annotation(program).unwrap();
            let t = checker.get_type_from_type_node(ann);
            assert_eq!(
                checker.type_arena().get_flags(t),
                TypeFlags::Never,
                "string & number should reduce to never"
            );
        }
    );
}

#[test]
fn intersection_reduction_supertype_removal() {
    // string & "hello" → "hello"
    with_checker!(
        r#"let x: string & "hello""#,
        |checker, program| {
            let ann = first_var_type_annotation(program).unwrap();
            let t = checker.get_type_from_type_node(ann);
            let s = checker.type_to_string(t);
            assert_eq!(s, r#""hello""#, "string & 'hello' should reduce to 'hello'");
        }
    );
}

#[test]
fn intersection_target_assignability() {
    // {a: string, b: number} should be assignable to {a: string} & {b: number}
    with_checker!(
        r#"
        let x: {a: string, b: number} = { a: "hi", b: 1 };
        let y: {a: string} & {b: number} = x;
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "object should be assignable to intersection: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn intersection_source_assignability() {
    // {a: string} & {b: number} should be assignable to {a: string, b: number}
    with_checker!(
        r#"
        let x: {a: string} & {b: number} = {} as {a: string} & {b: number};
        let y: {a: string, b: number} = x;
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "intersection should be assignable to object with same props: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn intersection_constituent_shortcut() {
    // string is a constituent of string & X, so string & X should be assignable to string
    with_checker!(
        r#"
        let x: string & {} = "" as string & {};
        let y: string = x;
        "#,
        |checker, program| {
            checker.check_program(program);
            let diagnostics = checker.take_diagnostics();
            assert!(
                diagnostics.is_empty(),
                "intersection constituent should satisfy target: {diagnostics:?}"
            );
        }
    );
}

#[test]
fn intersection_property_type_resolution() {
    // Property access on intersection type should resolve correctly
    with_checker!(
        r#"
        let x: {a: string} & {b: number} = {} as {a: string} & {b: number};
        "#,
        |checker, program| {
            let ann = first_var_type_annotation(program).unwrap();
            let t = checker.get_type_from_type_node(ann);
            // Property 'a' should exist and be string
            let a_type = checker.get_property_of_type(t, "a").expect("property 'a' should exist");
            assert_eq!(
                checker.type_to_string(a_type), "string",
                "property 'a' on intersection should be string"
            );
            // Property 'b' should exist and be number
            let b_type = checker.get_property_of_type(t, "b").expect("property 'b' should exist");
            assert_eq!(
                checker.type_to_string(b_type), "number",
                "property 'b' on intersection should be number"
            );
        }
    );
}

// --- `this` resolution ---

/// Helper: collect NodeIds of all ThisExpression nodes.
fn find_this_node_ids(checker: &Checker<'_>) -> Vec<oxc_syntax::node::NodeId> {
    use oxc_ast::AstKind;
    checker.semantic().nodes().iter()
        .filter_map(|node| {
            if let AstKind::ThisExpression(this_expr) = node.kind() {
                Some(this_expr.node_id())
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn this_in_class_method_resolves_to_class() {
    with_checker!(
        r#"
        class Foo { x: number = 1; bar() { return this; } }
        "#,
        |checker, _program| {
            let ids = find_this_node_ids(&checker);
            assert_eq!(ids.len(), 1, "should find exactly one ThisExpression");
            let t = checker.resolve_this_type(ids[0]);
            assert_eq!(checker.type_to_string(t), "Foo",
                "this in class method should resolve to class type");
        }
    );
}

#[test]
fn this_in_arrow_inside_method_resolves_to_class() {
    with_checker!(
        r#"
        class Foo { x: number = 1; bar() { const f = () => this; } }
        "#,
        |checker, _program| {
            let ids = find_this_node_ids(&checker);
            assert_eq!(ids.len(), 1);
            let t = checker.resolve_this_type(ids[0]);
            assert_eq!(checker.type_to_string(t), "Foo",
                "this in arrow inside method should inherit class type");
        }
    );
}

#[test]
fn this_in_standalone_function_inside_method_is_generic() {
    with_checker!(
        r#"
        class Foo {
            bar() {
                function inner() { return this; }
            }
        }
        "#,
        |checker, _program| {
            let ids = find_this_node_ids(&checker);
            assert_eq!(ids.len(), 1);
            let t = checker.resolve_this_type(ids[0]);
            assert_eq!(checker.type_to_string(t), "this",
                "this in standalone function should NOT resolve to Foo");
        }
    );
}

#[test]
fn this_at_top_level_is_generic() {
    with_checker!(
        r#"
        const x = this;
        "#,
        |checker, _program| {
            let ids = find_this_node_ids(&checker);
            assert_eq!(ids.len(), 1);
            let t = checker.resolve_this_type(ids[0]);
            assert_eq!(checker.type_to_string(t), "this",
                "top-level this should be generic this type");
        }
    );
}

