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

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    assert_eq!(program.body.len(), 1, "Program should have one statement");

    // Verify it's an ExpressionStatement with CallExpression
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::CallExpression(call_expr) => {
                    // Verify callee is an Identifier
                    match &call_expr.callee {
                        Expression::Identifier(ident) => {
                            assert_eq!(ident.name.as_str(), "foo");
                        }
                        _ => panic!("Expected Identifier in callee, got {:?}", call_expr.callee),
                    }
                    // Verify arguments is empty
                    assert_eq!(call_expr.arguments.len(), 0);
                }
                _ => panic!("Expected CallExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement, got {:?}", program.body[0]),
    }
}

#[test]
fn test_call_expression_with_arguments() {
    let allocator = Allocator::default();
    let source_text = "foo(1, \"bar\", true);";

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
                    "arguments": [
                        {
                            "type": "Literal",
                            "value": 1,
                            "raw": "1",
                            "range": [4, 5]
                        },
                        {
                            "type": "Literal",
                            "value": "bar",
                            "raw": "\"bar\"",
                            "range": [7, 12]
                        },
                        {
                            "type": "Literal",
                            "value": true,
                            "raw": "true",
                            "range": [14, 18]
                        }
                    ],
                    "range": [0, 19]
                },
                "range": [0, 20]
            }
        ],
        "range": [0, 20]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::CallExpression(call_expr) => {
                    assert_eq!(call_expr.arguments.len(), 3);

                    // Check first argument is numeric literal
                    match &call_expr.arguments[0] {
                        oxc_ast::ast::Argument::NumericLiteral(n) => {
                            assert_eq!(n.value, 1.0);
                        }
                        _ => panic!("Expected NumericLiteral as first argument"),
                    }

                    // Check second argument is string literal
                    match &call_expr.arguments[1] {
                        oxc_ast::ast::Argument::StringLiteral(s) => {
                            assert_eq!(s.value.as_str(), "bar");
                        }
                        _ => panic!("Expected StringLiteral as second argument"),
                    }

                    // Check third argument is boolean literal
                    match &call_expr.arguments[2] {
                        oxc_ast::ast::Argument::BooleanLiteral(b) => {
                            assert_eq!(b.value, true);
                        }
                        _ => panic!("Expected BooleanLiteral as third argument"),
                    }
                }
                _ => panic!("Expected CallExpression"),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
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

#[test]
fn test_member_expression() {
    let allocator = Allocator::default();
    let source_text = "obj.prop;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "MemberExpression",
                    "object": {
                        "type": "Identifier",
                        "name": "obj",
                        "range": [0, 3]
                    },
                    "property": {
                        "type": "Identifier",
                        "name": "prop",
                        "range": [4, 8]
                    },
                    "computed": false,
                    "optional": false,
                    "range": [0, 8]
                },
                "range": [0, 9]
            }
        ],
        "range": [0, 9]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::StaticMemberExpression(member_expr) => {
                    // Check object
                    match &member_expr.object {
                        Expression::Identifier(ident) => {
                            assert_eq!(ident.name.as_str(), "obj");
                        }
                        _ => panic!("Expected Identifier(obj) as object"),
                    }

                    // Check property
                    assert_eq!(member_expr.property.name.as_str(), "prop");
                }
                _ => panic!("Expected StaticMemberExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}

#[test]
fn test_return_statement() {
    let allocator = Allocator::default();
    let source_text = "return 42;";
    
    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ReturnStatement",
                "argument": {
                    "type": "Literal",
                    "value": 42,
                    "raw": "42",
                    "range": [7, 9]
                },
                "range": [0, 10]
            }
        ],
        "range": [0, 10]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);
    
    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());
    
    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ReturnStatement(return_stmt) => {
            assert!(return_stmt.argument.is_some());
            match return_stmt.argument.as_ref().unwrap() {
                Expression::NumericLiteral(n) => {
                    assert_eq!(n.value, 42.0);
                }
                _ => panic!("Expected NumericLiteral(42) as return argument"),
            }
        }
        _ => panic!("Expected ReturnStatement, got {:?}", program.body[0]),
    }
}

#[test]
fn test_return_statement_no_argument() {
    let allocator = Allocator::default();
    let source_text = "return;";
    
    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ReturnStatement",
                "argument": null,
                "range": [0, 7]
            }
        ],
        "range": [0, 7]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);
    
    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());
    
    let program = result.unwrap();
    use oxc_ast::ast::Statement;
    match &program.body[0] {
        Statement::ReturnStatement(return_stmt) => {
            assert!(return_stmt.argument.is_none(), "Return statement should have no argument");
        }
        _ => panic!("Expected ReturnStatement, got {:?}", program.body[0]),
    }
}

#[test]
fn test_binary_expression() {
    let allocator = Allocator::default();
    let source_text = "1 + 2;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "BinaryExpression",
                    "operator": "+",
                    "left": {
                        "type": "Literal",
                        "value": 1,
                        "raw": "1",
                        "range": [0, 1]
                    },
                    "right": {
                        "type": "Literal",
                        "value": 2,
                        "raw": "2",
                        "range": [4, 5]
                    },
                    "range": [0, 5]
                },
                "range": [0, 6]
            }
        ],
        "range": [0, 6]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::BinaryExpression(bin_expr) => {
                    assert_eq!(bin_expr.operator, oxc_syntax::operator::BinaryOperator::Addition);

                    // Check left operand
                    match &bin_expr.left {
                        Expression::NumericLiteral(n) => {
                            assert_eq!(n.value, 1.0);
                        }
                        _ => panic!("Expected NumericLiteral(1) as left operand"),
                    }

                    // Check right operand
                    match &bin_expr.right {
                        Expression::NumericLiteral(n) => {
                            assert_eq!(n.value, 2.0);
                        }
                        _ => panic!("Expected NumericLiteral(2) as right operand"),
                    }
                }
                _ => panic!("Expected BinaryExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}
