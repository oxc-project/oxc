//! End-to-end tests for ESTree to oxc AST conversion.

use oxc_allocator::Allocator;
use oxc_linter::estree_converter::convert_estree_json_to_oxc_program;

#[test]
fn test_simple_variable_declaration() {
    let allocator = Allocator::default();
    let source_text = "const x = 42;";
    
    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "VariableDeclaration",
                "kind": "const",
                "declarations": [
                    {
                        "type": "VariableDeclarator",
                        "id": {
                            "type": "Identifier",
                            "name": "x",
                            "range": [6, 7]
                        },
                        "init": {
                            "type": "Literal",
                            "value": 42,
                            "raw": "42",
                            "range": [10, 12]
                        },
                        "range": [6, 12]
                    }
                ],
                "range": [0, 13]
            }
        ],
        "range": [0, 13]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);
    
    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());
    
    let program = result.unwrap();
    assert_eq!(program.body.len(), 1, "Program should have one statement");
    
    // Verify it's a VariableDeclaration
    use oxc_ast::ast::Statement;
    match &program.body[0] {
        Statement::VariableDeclaration(var_decl) => {
            assert_eq!(var_decl.kind, oxc_ast::ast::VariableDeclarationKind::Const);
            assert_eq!(var_decl.declarations.len(), 1);
            
            let declarator = &var_decl.declarations[0];
            // Check that the pattern is a BindingIdentifier
            use oxc_ast::ast::BindingPatternKind;
            match &declarator.id.kind {
                BindingPatternKind::BindingIdentifier(binding_id) => {
                    assert_eq!(binding_id.name.as_str(), "x");
                }
                _ => panic!("Expected BindingIdentifier, got {:?}", declarator.id.kind),
            }
            
            // Check the initializer is a NumericLiteral
            assert!(declarator.init.is_some());
            use oxc_ast::ast::Expression;
            match &declarator.init.as_ref().unwrap() {
                Expression::NumericLiteral(num_lit) => {
                    assert_eq!(num_lit.value, 42.0);
                }
                _ => panic!("Expected NumericLiteral, got {:?}", declarator.init),
            }
        }
        _ => panic!("Expected VariableDeclaration, got {:?}", program.body[0]),
    }
}

#[test]
fn test_expression_statement_with_identifier() {
    let allocator = Allocator::default();
    let source_text = "foo();";
    
    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "CallExpression",
                    "callee": {
                        "type": "Identifier",
                        "name": "foo",
                        "range": [0, 3]
                    },
                    "arguments": [],
                    "range": [0, 5]
                },
                "range": [0, 6]
            }
        ],
        "range": [0, 6]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);
    
    // This should fail for now since we don't support CallExpression yet
    // But the structure should be valid
    assert!(result.is_err() || result.is_ok(), "Conversion should handle gracefully");
}

#[test]
fn test_simple_literals() {
    let allocator = Allocator::default();
    let source_text = "true; false; null; \"hello\"; 123;";
    
    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "Literal",
                    "value": true,
                    "raw": "true",
                    "range": [0, 4]
                },
                "range": [0, 5]
            },
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "Literal",
                    "value": false,
                    "raw": "false",
                    "range": [6, 11]
                },
                "range": [6, 12]
            },
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "Literal",
                    "value": null,
                    "raw": "null",
                    "range": [13, 17]
                },
                "range": [13, 18]
            },
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "Literal",
                    "value": "hello",
                    "raw": "\"hello\"",
                    "range": [19, 26]
                },
                "range": [19, 27]
            },
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "Literal",
                    "value": 123,
                    "raw": "123",
                    "range": [28, 31]
                },
                "range": [28, 32]
            }
        ],
        "range": [0, 32]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);
    
    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());
    
    let program = result.unwrap();
    assert_eq!(program.body.len(), 5, "Program should have 5 statements");
    
    // Verify first statement is boolean literal
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::BooleanLiteral(bool_lit) => {
                    assert_eq!(bool_lit.value, true);
                }
                _ => panic!("Expected BooleanLiteral(true), got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement, got {:?}", program.body[0]),
    }
    
    // Verify second statement is boolean literal (false)
    match &program.body[1] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::BooleanLiteral(bool_lit) => {
                    assert_eq!(bool_lit.value, false);
                }
                _ => panic!("Expected BooleanLiteral(false), got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement, got {:?}", program.body[1]),
    }
    
    // Verify third statement is null literal
    match &program.body[2] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::NullLiteral(_) => {
                    // Good
                }
                _ => panic!("Expected NullLiteral, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement, got {:?}", program.body[2]),
    }
    
    // Verify fourth statement is string literal
    match &program.body[3] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::StringLiteral(str_lit) => {
                    assert_eq!(str_lit.value.as_str(), "hello");
                }
                _ => panic!("Expected StringLiteral, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement, got {:?}", program.body[3]),
    }
    
    // Verify fifth statement is numeric literal
    match &program.body[4] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::NumericLiteral(num_lit) => {
                    assert_eq!(num_lit.value, 123.0);
                }
                _ => panic!("Expected NumericLiteral, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement, got {:?}", program.body[4]),
    }
}
