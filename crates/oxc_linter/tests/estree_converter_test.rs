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
fn test_if_statement() {
    let allocator = Allocator::default();
    let source_text = "if (x) return;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "IfStatement",
                "test": {
                    "type": "Identifier",
                    "name": "x",
                    "range": [4, 5]
                },
                "consequent": {
                    "type": "ReturnStatement",
                    "argument": null,
                    "range": [6, 13]
                },
                "alternate": null,
                "range": [0, 13]
            }
        ],
        "range": [0, 13]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::IfStatement(if_stmt) => {
            // Check test
            match &if_stmt.test {
                Expression::Identifier(ident) => {
                    assert_eq!(ident.name.as_str(), "x");
                }
                _ => panic!("Expected Identifier(x) as test"),
            }

            // Check consequent
            match &if_stmt.consequent {
                Statement::ReturnStatement(_) => {
                    // Good
                }
                _ => panic!("Expected ReturnStatement as consequent"),
            }

            // Check alternate is None
            assert!(if_stmt.alternate.is_none());
        }
        _ => panic!("Expected IfStatement, got {:?}", program.body[0]),
    }
}

#[test]
fn test_block_statement() {
    let allocator = Allocator::default();
    let source_text = "{ const x = 1; return x; }";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "BlockStatement",
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
                                    "range": [7, 8]
                                },
                                "init": {
                                    "type": "Literal",
                                    "value": 1,
                                    "raw": "1",
                                    "range": [11, 12]
                                },
                                "range": [7, 12]
                            }
                        ],
                        "range": [1, 13]
                    },
                    {
                        "type": "ReturnStatement",
                        "argument": {
                            "type": "Identifier",
                            "name": "x",
                            "range": [21, 22]
                        },
                        "range": [14, 23]
                    }
                ],
                "range": [0, 24]
            }
        ],
        "range": [0, 24]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::BlockStatement(block_stmt) => {
            assert_eq!(block_stmt.body.len(), 2);

            // First statement should be VariableDeclaration
            match &block_stmt.body[0] {
                Statement::VariableDeclaration(var_decl) => {
                    assert_eq!(var_decl.kind, oxc_ast::ast::VariableDeclarationKind::Const);
                }
                _ => panic!("Expected VariableDeclaration as first statement"),
            }

            // Second statement should be ReturnStatement
            match &block_stmt.body[1] {
                Statement::ReturnStatement(return_stmt) => {
                    assert!(return_stmt.argument.is_some());
                    match return_stmt.argument.as_ref().unwrap() {
                        Expression::Identifier(ident) => {
                            assert_eq!(ident.name.as_str(), "x");
                        }
                        _ => panic!("Expected Identifier(x) as return argument"),
                    }
                }
                _ => panic!("Expected ReturnStatement as second statement"),
            }
        }
        _ => panic!("Expected BlockStatement, got {:?}", program.body[0]),
    }
}

#[test]
fn test_unary_expression() {
    let allocator = Allocator::default();
    let source_text = "!x;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "UnaryExpression",
                    "operator": "!",
                    "prefix": true,
                    "argument": {
                        "type": "Identifier",
                        "name": "x",
                        "range": [1, 2]
                    },
                    "range": [0, 2]
                },
                "range": [0, 3]
            }
        ],
        "range": [0, 3]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::UnaryExpression(unary_expr) => {
                    assert_eq!(unary_expr.operator, oxc_syntax::operator::UnaryOperator::LogicalNot);

                    // Check argument
                    match &unary_expr.argument {
                        Expression::Identifier(ident) => {
                            assert_eq!(ident.name.as_str(), "x");
                        }
                        _ => panic!("Expected Identifier(x) as argument"),
                    }
                }
                _ => panic!("Expected UnaryExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}

#[test]
fn test_array_expression() {
    let allocator = Allocator::default();
    let source_text = "[1, 2, 3];";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "ArrayExpression",
                    "elements": [
                        {
                            "type": "Literal",
                            "value": 1,
                            "raw": "1",
                            "range": [1, 2]
                        },
                        {
                            "type": "Literal",
                            "value": 2,
                            "raw": "2",
                            "range": [4, 5]
                        },
                        {
                            "type": "Literal",
                            "value": 3,
                            "raw": "3",
                            "range": [7, 8]
                        }
                    ],
                    "range": [0, 9]
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
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::ArrayExpression(array_expr) => {
                    assert_eq!(array_expr.elements.len(), 3);

                    // Check first element
                    match &array_expr.elements[0] {
                        oxc_ast::ast::ArrayExpressionElement::NumericLiteral(n) => {
                            assert_eq!(n.value, 1.0);
                        }
                        _ => panic!("Expected NumericLiteral(1)"),
                    }

                    // Check second element
                    match &array_expr.elements[1] {
                        oxc_ast::ast::ArrayExpressionElement::NumericLiteral(n) => {
                            assert_eq!(n.value, 2.0);
                        }
                        _ => panic!("Expected NumericLiteral(2)"),
                    }

                    // Check third element
                    match &array_expr.elements[2] {
                        oxc_ast::ast::ArrayExpressionElement::NumericLiteral(n) => {
                            assert_eq!(n.value, 3.0);
                        }
                        _ => panic!("Expected NumericLiteral(3)"),
                    }
                }
                _ => panic!("Expected ArrayExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}

#[test]
fn test_object_expression() {
    let allocator = Allocator::default();
    let source_text = "{ a: 1, b: 2 };";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "ObjectExpression",
                    "properties": [
                        {
                            "type": "Property",
                            "key": {
                                "type": "Identifier",
                                "name": "a",
                                "range": [2, 3]
                            },
                            "value": {
                                "type": "Literal",
                                "value": 1,
                                "raw": "1",
                                "range": [5, 6]
                            },
                            "kind": "init",
                            "method": false,
                            "shorthand": false,
                            "computed": false,
                            "range": [2, 6]
                        },
                        {
                            "type": "Property",
                            "key": {
                                "type": "Identifier",
                                "name": "b",
                                "range": [8, 9]
                            },
                            "value": {
                                "type": "Literal",
                                "value": 2,
                                "raw": "2",
                                "range": [11, 12]
                            },
                            "kind": "init",
                            "method": false,
                            "shorthand": false,
                            "computed": false,
                            "range": [8, 12]
                        }
                    ],
                    "range": [0, 13]
                },
                "range": [0, 14]
            }
        ],
        "range": [0, 14]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::ObjectExpression(obj_expr) => {
                    assert_eq!(obj_expr.properties.len(), 2);

                    // Check first property
                    match &obj_expr.properties[0] {
                        oxc_ast::ast::ObjectPropertyKind::ObjectProperty(prop) => {
                            // Check key
                            match &prop.key {
                                oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => {
                                    assert_eq!(ident.name.as_str(), "a");
                                }
                                _ => panic!("Expected StaticIdentifier('a') as key"),
                            }

                            // Check value
                            match &prop.value {
                                Expression::NumericLiteral(n) => {
                                    assert_eq!(n.value, 1.0);
                                }
                                _ => panic!("Expected NumericLiteral(1) as value"),
                            }
                        }
                        _ => panic!("Expected ObjectProperty"),
                    }

                    // Check second property
                    match &obj_expr.properties[1] {
                        oxc_ast::ast::ObjectPropertyKind::ObjectProperty(prop) => {
                            // Check key
                            match &prop.key {
                                oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => {
                                    assert_eq!(ident.name.as_str(), "b");
                                }
                                _ => panic!("Expected StaticIdentifier('b') as key"),
                            }

                            // Check value
                            match &prop.value {
                                Expression::NumericLiteral(n) => {
                                    assert_eq!(n.value, 2.0);
                                }
                                _ => panic!("Expected NumericLiteral(2) as value"),
                            }
                        }
                        _ => panic!("Expected ObjectProperty"),
                    }
                }
                _ => panic!("Expected ObjectExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}

#[test]
fn test_logical_expression() {
    let allocator = Allocator::default();
    let source_text = "x && y;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "LogicalExpression",
                    "operator": "&&",
                    "left": {
                        "type": "Identifier",
                        "name": "x",
                        "range": [0, 1]
                    },
                    "right": {
                        "type": "Identifier",
                        "name": "y",
                        "range": [5, 6]
                    },
                    "range": [0, 6]
                },
                "range": [0, 7]
            }
        ],
        "range": [0, 7]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::LogicalExpression(logical_expr) => {
                    assert_eq!(logical_expr.operator, oxc_syntax::operator::LogicalOperator::And);

                    // Check left operand
                    match &logical_expr.left {
                        Expression::Identifier(ident) => {
                            assert_eq!(ident.name.as_str(), "x");
                        }
                        _ => panic!("Expected Identifier(x) as left operand"),
                    }

                    // Check right operand
                    match &logical_expr.right {
                        Expression::Identifier(ident) => {
                            assert_eq!(ident.name.as_str(), "y");
                        }
                        _ => panic!("Expected Identifier(y) as right operand"),
                    }
                }
                _ => panic!("Expected LogicalExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}

#[test]
fn test_conditional_expression() {
    let allocator = Allocator::default();
    let source_text = "x ? 1 : 2;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "ConditionalExpression",
                    "test": {
                        "type": "Identifier",
                        "name": "x",
                        "range": [0, 1]
                    },
                    "consequent": {
                        "type": "Literal",
                        "value": 1,
                        "raw": "1",
                        "range": [4, 5]
                    },
                    "alternate": {
                        "type": "Literal",
                        "value": 2,
                        "raw": "2",
                        "range": [8, 9]
                    },
                    "range": [0, 9]
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
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::ConditionalExpression(cond_expr) => {
                    // Check test
                    match &cond_expr.test {
                        Expression::Identifier(ident) => {
                            assert_eq!(ident.name.as_str(), "x");
                        }
                        _ => panic!("Expected Identifier(x) as test"),
                    }

                    // Check consequent
                    match &cond_expr.consequent {
                        Expression::NumericLiteral(n) => {
                            assert_eq!(n.value, 1.0);
                        }
                        _ => panic!("Expected NumericLiteral(1) as consequent"),
                    }

                    // Check alternate
                    match &cond_expr.alternate {
                        Expression::NumericLiteral(n) => {
                            assert_eq!(n.value, 2.0);
                        }
                        _ => panic!("Expected NumericLiteral(2) as alternate"),
                    }
                }
                _ => panic!("Expected ConditionalExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}

#[test]
fn test_assignment_expression() {
    let allocator = Allocator::default();
    let source_text = "x = 42;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "AssignmentExpression",
                    "operator": "=",
                    "left": {
                        "type": "Identifier",
                        "name": "x",
                        "range": [0, 1]
                    },
                    "right": {
                        "type": "Literal",
                        "value": 42,
                        "raw": "42",
                        "range": [4, 6]
                    },
                    "range": [0, 6]
                },
                "range": [0, 7]
            }
        ],
        "range": [0, 7]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{AssignmentTarget, Expression, Statement};
    match &program.body[0] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::AssignmentExpression(assign_expr) => {
                    assert_eq!(assign_expr.operator, oxc_syntax::operator::AssignmentOperator::Assign);

                    // Check left (assignment target)
                    match &assign_expr.left {
                        AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                            assert_eq!(ident.name.as_str(), "x");
                        }
                        _ => panic!("Expected AssignmentTargetIdentifier(x), got {:?}", assign_expr.left),
                    }

                    // Check right
                    match &assign_expr.right {
                        Expression::NumericLiteral(n) => {
                            assert_eq!(n.value, 42.0);
                        }
                        _ => panic!("Expected NumericLiteral(42) as right operand"),
                    }
                }
                _ => panic!("Expected AssignmentExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}

#[test]
fn test_update_expression() {
    let allocator = Allocator::default();
    let source_text = "x++;";
    
    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "UpdateExpression",
                    "operator": "++",
                    "argument": {
                        "type": "Identifier",
                        "name": "x",
                        "range": [0, 1]
                    },
                    "prefix": false,
                    "range": [0, 3]
                },
                "range": [0, 4]
            }
        ],
        "range": [0, 4]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);
    
    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());
    
    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::UpdateExpression(update_expr) => {
                    assert_eq!(update_expr.operator, oxc_syntax::operator::UpdateOperator::Increment);
                    assert_eq!(update_expr.prefix, false);
                    
                    // Check argument (UpdateExpression.argument is SimpleAssignmentTarget)
                    use oxc_ast::ast::SimpleAssignmentTarget;
                    match &update_expr.argument {
                        SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                            assert_eq!(ident.name.as_str(), "x");
                        }
                        _ => panic!("Expected AssignmentTargetIdentifier(x) as argument"),
                    }
                }
                _ => panic!("Expected UpdateExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
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
