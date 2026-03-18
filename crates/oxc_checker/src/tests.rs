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
    with_checker!("", |checker, _program| {
        let diagnostics = checker.check();
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

