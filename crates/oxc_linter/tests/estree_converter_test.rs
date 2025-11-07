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
fn test_sequence_expression() {
    let allocator = Allocator::default();
    let source_text = "x, y, z;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "SequenceExpression",
                    "expressions": [
                        {
                            "type": "Identifier",
                            "name": "x",
                            "range": [0, 1]
                        },
                        {
                            "type": "Identifier",
                            "name": "y",
                            "range": [3, 4]
                        },
                        {
                            "type": "Identifier",
                            "name": "z",
                            "range": [6, 7]
                        }
                    ],
                    "range": [0, 7]
                },
                "range": [0, 8]
            }
        ],
        "range": [0, 8]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::SequenceExpression(seq_expr) => {
                    assert_eq!(seq_expr.expressions.len(), 3);

                    // Check first expression
                    match &seq_expr.expressions[0] {
                        Expression::Identifier(ident) => {
                            assert_eq!(ident.name.as_str(), "x");
                        }
                        _ => panic!("Expected Identifier(x) as first expression"),
                    }

                    // Check second expression
                    match &seq_expr.expressions[1] {
                        Expression::Identifier(ident) => {
                            assert_eq!(ident.name.as_str(), "y");
                        }
                        _ => panic!("Expected Identifier(y) as second expression"),
                    }

                    // Check third expression
                    match &seq_expr.expressions[2] {
                        Expression::Identifier(ident) => {
                            assert_eq!(ident.name.as_str(), "z");
                        }
                        _ => panic!("Expected Identifier(z) as third expression"),
                    }
                }
                _ => panic!("Expected SequenceExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}

#[test]
fn test_this_expression() {
    let allocator = Allocator::default();
    let source_text = "this;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "ThisExpression",
                    "range": [0, 4]
                },
                "range": [0, 5]
            }
        ],
        "range": [0, 5]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::ThisExpression(_) => {
                    // ThisExpression has no fields to check
                }
                _ => panic!("Expected ThisExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}

#[test]
fn test_new_expression() {
    let allocator = Allocator::default();
    let source_text = "new Foo();";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "NewExpression",
                    "callee": {
                        "type": "Identifier",
                        "name": "Foo",
                        "range": [4, 7]
                    },
                    "arguments": [],
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
                Expression::NewExpression(new_expr) => {
                    // Check callee
                    match &new_expr.callee {
                        Expression::Identifier(ident) => {
                            assert_eq!(ident.name.as_str(), "Foo");
                        }
                        _ => panic!("Expected Identifier(Foo) as callee"),
                    }

                    // Check arguments (empty)
                    assert_eq!(new_expr.arguments.len(), 0);
                }
                _ => panic!("Expected NewExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}

#[test]
fn test_while_statement() {
    let allocator = Allocator::default();
    let source_text = "while (x) { }";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "WhileStatement",
                "test": {
                    "type": "Identifier",
                    "name": "x",
                    "range": [7, 8]
                },
                "body": {
                    "type": "BlockStatement",
                    "body": [],
                    "range": [10, 12]
                },
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
        Statement::WhileStatement(while_stmt) => {
            // Check test
            match &while_stmt.test {
                Expression::Identifier(ident) => {
                    assert_eq!(ident.name.as_str(), "x");
                }
                _ => panic!("Expected Identifier(x) as test"),
            }

            // Check body
            match &while_stmt.body {
                Statement::BlockStatement(block) => {
                    assert_eq!(block.body.len(), 0);
                }
                _ => panic!("Expected BlockStatement as body"),
            }
        }
        _ => panic!("Expected WhileStatement, got {:?}", program.body[0]),
    }
}

#[test]
fn test_for_statement() {
    let allocator = Allocator::default();
    let source_text = "for (let i = 0; i < 10; i++) { }";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ForStatement",
                "init": {
          "type": "VariableDeclaration",
                    "kind": "let",
          "declarations": [
            {
              "type": "VariableDeclarator",
              "id": {
                "type": "Identifier",
                                "name": "i",
                                "range": [9, 10]
              },
              "init": {
                "type": "Literal",
                                "value": 0,
                                "raw": "0",
                                "range": [13, 14]
                            },
                            "range": [9, 14]
                        }
                    ],
                    "range": [5, 15]
                },
                "test": {
                    "type": "BinaryExpression",
                    "operator": "<",
                    "left": {
                        "type": "Identifier",
                        "name": "i",
                        "range": [17, 18]
                    },
                    "right": {
                        "type": "Literal",
                        "value": 10,
                        "raw": "10",
                        "range": [21, 23]
                    },
                    "range": [17, 23]
                },
                "update": {
                    "type": "UpdateExpression",
                    "operator": "++",
                    "argument": {
                        "type": "Identifier",
                        "name": "i",
                        "range": [25, 26]
                    },
                    "prefix": false,
                    "range": [25, 28]
                },
                "body": {
                    "type": "BlockStatement",
                    "body": [],
                    "range": [30, 32]
                },
                "range": [0, 33]
            }
        ],
        "range": [0, 33]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ForStatement(for_stmt) => {
            // Check init (VariableDeclaration)
            use oxc_ast::ast::ForStatementInit;
            match &for_stmt.init {
                Some(ForStatementInit::VariableDeclaration(var_decl)) => {
                    assert_eq!(var_decl.kind, oxc_ast::ast::VariableDeclarationKind::Let);
                }
                _ => panic!("Expected VariableDeclaration as init"),
            }

            // Check test
            match &for_stmt.test {
                Some(Expression::BinaryExpression(bin_expr)) => {
                    assert_eq!(bin_expr.operator, oxc_syntax::operator::BinaryOperator::LessThan);
                }
                _ => panic!("Expected BinaryExpression as test"),
            }

            // Check update
            match &for_stmt.update {
                Some(Expression::UpdateExpression(update_expr)) => {
                    assert_eq!(update_expr.operator, oxc_syntax::operator::UpdateOperator::Increment);
                }
                _ => panic!("Expected UpdateExpression as update"),
            }

            // Check body
            match &for_stmt.body {
                Statement::BlockStatement(block) => {
                    assert_eq!(block.body.len(), 0);
                }
                _ => panic!("Expected BlockStatement as body"),
            }
        }
        _ => panic!("Expected ForStatement, got {:?}", program.body[0]),
    }
}

#[test]
fn test_break_statement() {
    let allocator = Allocator::default();
    let source_text = "break;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "BreakStatement",
                "label": null,
                "range": [0, 6]
            }
        ],
        "range": [0, 6]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::Statement;
    match &program.body[0] {
        Statement::BreakStatement(break_stmt) => {
            assert!(break_stmt.label.is_none());
        }
        _ => panic!("Expected BreakStatement, got {:?}", program.body[0]),
    }
}

#[test]
fn test_continue_statement() {
    let allocator = Allocator::default();
    let source_text = "continue;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ContinueStatement",
                "label": null,
                "range": [0, 9]
            }
        ],
        "range": [0, 9]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::Statement;
    match &program.body[0] {
        Statement::ContinueStatement(continue_stmt) => {
            assert!(continue_stmt.label.is_none());
        }
        _ => panic!("Expected ContinueStatement, got {:?}", program.body[0]),
    }
}

#[test]
fn test_throw_statement() {
    let allocator = Allocator::default();
    let source_text = "throw new Error();";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ThrowStatement",
                "argument": {
                    "type": "NewExpression",
                    "callee": {
                        "type": "Identifier",
                        "name": "Error",
                        "range": [6, 11]
                    },
                    "arguments": [],
              "range": [6, 13]
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
        Statement::ThrowStatement(throw_stmt) => {
            match &throw_stmt.argument {
                Expression::NewExpression(new_expr) => {
                    match &new_expr.callee {
                        Expression::Identifier(ident) => {
                            assert_eq!(ident.name.as_str(), "Error");
                        }
                        _ => panic!("Expected Identifier(Error) as callee"),
                    }
                }
                _ => panic!("Expected NewExpression as argument"),
            }
        }
        _ => panic!("Expected ThrowStatement, got {:?}", program.body[0]),
    }
}

#[test]
fn test_do_while_statement() {
    let allocator = Allocator::default();
    let source_text = "do { } while (x);";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "DoWhileStatement",
                "body": {
                    "type": "BlockStatement",
                    "body": [],
                    "range": [3, 5]
                },
                "test": {
                    "type": "Identifier",
                    "name": "x",
                    "range": [13, 14]
                },
                "range": [0, 16]
            }
        ],
        "range": [0, 16]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::DoWhileStatement(do_while_stmt) => {
            // Check body
            match &do_while_stmt.body {
                Statement::BlockStatement(block) => {
                    assert_eq!(block.body.len(), 0);
                }
                _ => panic!("Expected BlockStatement as body"),
            }

            // Check test
            match &do_while_stmt.test {
                Expression::Identifier(ident) => {
                    assert_eq!(ident.name.as_str(), "x");
                }
                _ => panic!("Expected Identifier(x) as test"),
            }
        }
        _ => panic!("Expected DoWhileStatement, got {:?}", program.body[0]),
    }
}

#[test]
fn test_for_in_statement() {
    let allocator = Allocator::default();
    let source_text = "for (let x in obj) { }";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ForInStatement",
                "left": {
                    "type": "VariableDeclaration",
                    "kind": "let",
                    "declarations": [
                        {
                            "type": "VariableDeclarator",
                            "id": {
                                "type": "Identifier",
                                "name": "x",
                                "range": [9, 10]
                            },
                            "init": null,
                            "range": [9, 10]
                        }
                    ],
                    "range": [5, 11]
                },
                "right": {
                    "type": "Identifier",
                    "name": "obj",
                    "range": [15, 18]
                },
                "body": {
                    "type": "BlockStatement",
                    "body": [],
                    "range": [20, 22]
                },
                "range": [0, 23]
            }
        ],
        "range": [0, 23]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ForInStatement(for_in_stmt) => {
            // Check left (should be a binding pattern)
            use oxc_ast::ast::ForStatementLeft;
            match &for_in_stmt.left {
                ForStatementLeft::VariableDeclaration(var_decl) => {
                    assert_eq!(var_decl.kind, oxc_ast::ast::VariableDeclarationKind::Let);
                }
                _ => panic!("Expected VariableDeclaration as left"),
            }

            // Check right
            match &for_in_stmt.right {
                Expression::Identifier(ident) => {
                    assert_eq!(ident.name.as_str(), "obj");
                }
                _ => panic!("Expected Identifier(obj) as right"),
            }

            // Check body
            match &for_in_stmt.body {
                Statement::BlockStatement(block) => {
                    assert_eq!(block.body.len(), 0);
                }
                _ => panic!("Expected BlockStatement as body"),
            }
        }
        _ => panic!("Expected ForInStatement, got {:?}", program.body[0]),
    }
}

#[test]
fn test_empty_statement() {
    let allocator = Allocator::default();
    let source_text = ";";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "EmptyStatement",
                "range": [0, 1]
            }
        ],
        "range": [0, 1]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::Statement;
    match &program.body[0] {
        Statement::EmptyStatement(_) => {
            // EmptyStatement has no fields to check
        }
        _ => panic!("Expected EmptyStatement, got {:?}", program.body[0]),
    }
}

#[test]
fn test_await_expression() {
    let allocator = Allocator::default();
    let source_text = "await promise;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "AwaitExpression",
                    "argument": {
                        "type": "Identifier",
                        "name": "promise",
                        "range": [6, 13]
                    },
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
                Expression::AwaitExpression(await_expr) => {
                    match &await_expr.argument {
                        Expression::Identifier(ident) => {
                            assert_eq!(ident.name.as_str(), "promise");
                        }
                        _ => panic!("Expected Identifier(promise) as argument"),
                    }
                }
                _ => panic!("Expected AwaitExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}

#[test]
fn test_yield_expression() {
    let allocator = Allocator::default();
    let source_text = "yield value;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "YieldExpression",
                    "argument": {
                        "type": "Identifier",
                        "name": "value",
                        "range": [6, 11]
                    },
                    "delegate": false,
                    "range": [0, 11]
                },
                "range": [0, 12]
            }
        ],
        "range": [0, 12]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::YieldExpression(yield_expr) => {
                    assert_eq!(yield_expr.delegate, false);
                    match &yield_expr.argument {
                        Some(Expression::Identifier(ident)) => {
                            assert_eq!(ident.name.as_str(), "value");
                        }
                        _ => panic!("Expected Some(Identifier(value)) as argument"),
                    }
                }
                _ => panic!("Expected YieldExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}

#[test]
fn test_labeled_statement() {
    let allocator = Allocator::default();
    let source_text = "label: x = 1;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "LabeledStatement",
                "label": {
                    "type": "Identifier",
                    "name": "label",
                    "range": [0, 5]
                },
                "body": {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "AssignmentExpression",
                        "operator": "=",
                        "left": {
                            "type": "Identifier",
                            "name": "x",
                            "range": [7, 8]
                        },
                        "right": {
                            "type": "Literal",
                            "value": 1,
                            "raw": "1",
                            "range": [11, 12]
                        },
                        "range": [7, 12]
                    },
                    "range": [7, 13]
                },
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
        Statement::LabeledStatement(labeled_stmt) => {
            // Check label
            assert_eq!(labeled_stmt.label.name.as_str(), "label");

            // Check body
            match &labeled_stmt.body {
                Statement::ExpressionStatement(expr_stmt) => {
                    match &expr_stmt.expression {
                        Expression::AssignmentExpression(assign_expr) => {
                            assert_eq!(assign_expr.operator, oxc_syntax::operator::AssignmentOperator::Assign);
                        }
                        _ => panic!("Expected AssignmentExpression in body"),
                    }
                }
                _ => panic!("Expected ExpressionStatement as body"),
            }
        }
        _ => panic!("Expected LabeledStatement, got {:?}", program.body[0]),
    }
}

#[test]
fn test_super_expression() {
    let allocator = Allocator::default();
    let source_text = "super.method();";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "CallExpression",
                    "callee": {
                        "type": "MemberExpression",
                        "object": {
                            "type": "Super",
                            "range": [0, 5]
                        },
                        "property": {
                            "type": "Identifier",
                            "name": "method",
                            "range": [6, 12]
                        },
                        "computed": false,
                        "range": [0, 12]
                    },
                    "arguments": [],
                    "range": [0, 14]
                },
                "range": [0, 15]
            }
        ],
        "range": [0, 15]
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
                    match &call_expr.callee {
                        Expression::StaticMemberExpression(member_expr) => {
                            match &member_expr.object {
                                Expression::Super(_) => {
                                    // Super expression found
                                }
                                _ => panic!("Expected Super as object"),
                            }
                            // StaticMemberExpression.property is IdentifierName
                            assert_eq!(member_expr.property.name.as_str(), "method");
                        }
                        _ => panic!("Expected StaticMemberExpression as callee"),
                    }
                }
                _ => panic!("Expected CallExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}

#[test]
fn test_switch_statement() {
    let allocator = Allocator::default();
    let source_text = "switch (x) { case 1: break; }";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "SwitchStatement",
                "discriminant": {
                    "type": "Identifier",
                    "name": "x",
                    "range": [8, 9]
                },
                "cases": [
                    {
                        "type": "SwitchCase",
                        "test": {
                            "type": "Literal",
                            "value": 1,
                            "raw": "1",
                            "range": [15, 16]
                        },
                        "consequent": [
                            {
                                "type": "BreakStatement",
                                "label": null,
                                "range": [18, 23]
                            }
                        ],
                        "range": [11, 23]
                    }
                ],
                "range": [0, 25]
            }
        ],
        "range": [0, 25]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::SwitchStatement(switch_stmt) => {
            // Check discriminant
            match &switch_stmt.discriminant {
                Expression::Identifier(ident) => {
                    assert_eq!(ident.name.as_str(), "x");
                }
                _ => panic!("Expected Identifier(x) as discriminant"),
            }

            // Check cases
            assert_eq!(switch_stmt.cases.len(), 1);
            let case = &switch_stmt.cases[0];
            match &case.test {
                Some(Expression::NumericLiteral(lit)) => {
                    assert_eq!(lit.value, 1.0);
                }
                _ => panic!("Expected Some(NumericLiteral(1)) as test"),
            }
            assert_eq!(case.consequent.len(), 1);
            match &case.consequent[0] {
                Statement::BreakStatement(_) => {
                    // Break statement found
                }
                _ => panic!("Expected BreakStatement in consequent"),
            }
        }
        _ => panic!("Expected SwitchStatement, got {:?}", program.body[0]),
    }
}

#[test]
fn test_spread_element() {
    let allocator = Allocator::default();
    let source_text = "[...arr];";

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
                            "type": "SpreadElement",
                            "argument": {
                                "type": "Identifier",
                                "name": "arr",
                                "range": [4, 7]
                            },
                            "range": [1, 7]
                        }
                    ],
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
                Expression::ArrayExpression(arr_expr) => {
                    assert_eq!(arr_expr.elements.len(), 1);
                    match &arr_expr.elements[0] {
                        oxc_ast::ast::ArrayExpressionElement::SpreadElement(spread) => {
                            match &spread.argument {
                                Expression::Identifier(ident) => {
                                    assert_eq!(ident.name.as_str(), "arr");
                                }
                                _ => panic!("Expected Identifier(arr) as argument"),
                            }
                        }
                        _ => panic!("Expected SpreadElement"),
                    }
                }
                _ => panic!("Expected ArrayExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}

#[test]
fn test_try_statement() {
    let allocator = Allocator::default();
    let source_text = "try { } catch (e) { }";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "TryStatement",
                "block": {
                    "type": "BlockStatement",
                    "body": [],
                    "range": [4, 6]
                },
                "handler": {
                    "type": "CatchClause",
                    "param": {
                        "type": "Identifier",
                        "name": "e",
                        "range": [14, 15]
                    },
                    "body": {
                        "type": "BlockStatement",
                        "body": [],
                        "range": [17, 19]
                    },
                    "range": [8, 19]
                },
                "finalizer": null,
                "range": [0, 20]
            }
        ],
        "range": [0, 20]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::Statement;
    match &program.body[0] {
        Statement::TryStatement(try_stmt) => {
            // Check block (it's a Box<BlockStatement>)
            assert_eq!(try_stmt.block.body.len(), 0);

            // Check handler (it's Option<Box<CatchClause>>)
            match &try_stmt.handler {
                Some(catch_clause) => {
                    match &catch_clause.param {
                        Some(catch_param) => {
                            // CatchParameter has a pattern field
                            if let Some(binding_id) = catch_param.pattern.get_binding_identifier() {
                                assert_eq!(binding_id.name.as_str(), "e");
                            } else {
                                panic!("Expected BindingIdentifier(e) as param");
                            }
                        }
                        _ => panic!("Expected Some(CatchParameter) as param"),
                    }
                    assert_eq!(catch_clause.body.body.len(), 0);
                }
                _ => panic!("Expected Some(CatchClause) as handler"),
            }

            // Check finalizer (should be None)
            assert!(try_stmt.finalizer.is_none());
        }
        _ => panic!("Expected TryStatement, got {:?}", program.body[0]),
    }
}

#[test]
fn test_template_literal() {
    let allocator = Allocator::default();
    let source_text = "`hello ${name}`;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "TemplateLiteral",
                    "quasis": [
                        {
                            "type": "TemplateElement",
                            "value": {
                                "raw": "hello ",
                                "cooked": "hello "
                            },
                            "tail": false,
                            "range": [1, 7]
                        },
                        {
                            "type": "TemplateElement",
                            "value": {
                                "raw": "",
                                "cooked": ""
                            },
                            "tail": true,
                            "range": [15, 16]
                        }
                    ],
                    "expressions": [
                        {
                            "type": "Identifier",
                            "name": "name",
                            "range": [9, 13]
                        }
                    ],
                    "range": [0, 16]
                },
                "range": [0, 17]
            }
        ],
        "range": [0, 17]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::TemplateLiteral(template_lit) => {
                    assert_eq!(template_lit.quasis.len(), 2);
                    assert_eq!(template_lit.expressions.len(), 1);

                    // Check first quasi
                    assert_eq!(template_lit.quasis[0].value.raw.as_str(), "hello ");
                    assert_eq!(template_lit.quasis[0].tail, false);

                    // Check expression
                    match &template_lit.expressions[0] {
                        Expression::Identifier(ident) => {
                            assert_eq!(ident.name.as_str(), "name");
                        }
                        _ => panic!("Expected Identifier(name) as expression"),
                    }

                    // Check second quasi (tail)
                    assert_eq!(template_lit.quasis[1].tail, true);
                }
                _ => panic!("Expected TemplateLiteral, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}

#[test]
fn test_tagged_template_expression() {
    let allocator = Allocator::default();
    let source_text = "tag`hello ${name}`;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "TaggedTemplateExpression",
                    "tag": {
                        "type": "Identifier",
                        "name": "tag",
                        "range": [0, 3]
                    },
                    "quasi": {
                        "type": "TemplateLiteral",
                        "quasis": [
                            {
                                "type": "TemplateElement",
                                "value": {
                                    "raw": "hello ",
                                    "cooked": "hello "
                                },
                                "tail": false,
                                "range": [4, 10]
                            },
                            {
                                "type": "TemplateElement",
                                "value": {
                                    "raw": "",
                                    "cooked": ""
                                },
                                "tail": true,
                                "range": [18, 19]
                            }
                        ],
                        "expressions": [
                            {
                                "type": "Identifier",
                                "name": "name",
                                "range": [12, 16]
                            }
                        ],
                        "range": [3, 19]
                    },
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
                Expression::TaggedTemplateExpression(tagged) => {
                    // Check tag
                    match &tagged.tag {
                        Expression::Identifier(ident) => {
                            assert_eq!(ident.name.as_str(), "tag");
                        }
                        _ => panic!("Expected Identifier(tag) as tag"),
                    }

                    // Check quasi (template literal - it's a direct TemplateLiteral, not Expression)
                    assert_eq!(tagged.quasi.quasis.len(), 2);
                    assert_eq!(tagged.quasi.expressions.len(), 1);
                }
                _ => panic!("Expected TaggedTemplateExpression, got {:?}", expr_stmt.expression),
            }
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}

#[test]
fn test_function_declaration() {
    let allocator = Allocator::default();
    let source_text = "function foo() { return 1; }";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "FunctionDeclaration",
                "id": {
                    "type": "Identifier",
                    "name": "foo",
                    "range": [9, 12]
                },
                "params": [],
                "body": {
                    "type": "BlockStatement",
                    "body": [
                        {
                            "type": "ReturnStatement",
                            "argument": {
                                "type": "Literal",
                                "value": 1,
                                "raw": "1",
                                "range": [25, 26]
                            },
                            "range": [18, 27]
                        }
                    ],
                    "range": [15, 28]
                },
                "generator": false,
                "async": false,
                "range": [0, 28]
            }
        ],
        "range": [0, 28]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::Statement;
    match &program.body[0] {
        Statement::FunctionDeclaration(function_decl) => {
            let function = &function_decl.function;
            // Check id
            match &function.id {
                Some(oxc_ast::ast::BindingIdentifier { name, .. }) => {
                    assert_eq!(name.as_str(), "foo");
                }
                _ => panic!("Expected Some(BindingIdentifier(foo)) as id"),
            }

            // Check params (empty) - params is Box<FormalParameters>
            assert_eq!(function.params.items.len(), 0);

            // Check body - FunctionBody is a struct with statements field
            assert_eq!(function.body.statements.len(), 1);

            // Check generator and async flags
            assert_eq!(function.generator, false);
            assert_eq!(function.r#async, false);
        }
        _ => panic!("Expected FunctionDeclaration, got {:?}", program.body[0]),
    }
}

#[test]
fn test_arrow_function_expression() {
    let allocator = Allocator::default();
    let source_text = "() => 1;";

    let estree_json = r#"
    {
        "type": "Program",
        "body": [
            {
                "type": "ExpressionStatement",
                "expression": {
                    "type": "ArrowFunctionExpression",
                    "params": [],
                    "body": {
                        "type": "Literal",
                        "value": 1,
                        "raw": "1",
                        "range": [5, 6]
                    },
                    "async": false,
                    "range": [0, 7]
                },
                "range": [0, 8]
            }
        ],
        "range": [0, 8]
    }
    "#;

    let result = convert_estree_json_to_oxc_program(estree_json, source_text, &allocator);

    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());

    let program = result.unwrap();
    use oxc_ast::ast::{Expression, Statement};
    match &program.body[0] {
        Statement::ExpressionStatement(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::ArrowFunctionExpression(arrow) => {
                    // Check params (empty) - params is Box<FormalParameters>
                    assert_eq!(arrow.params.items.len(), 0);

                    // Check body (expression) - ArrowFunctionExpression has expression flag and body
                    assert_eq!(arrow.expression, true);
                    // body is FunctionBody which has statements, but for expression arrow functions,
                    // we need to check if there's a way to get the expression
                    // For now, just check that expression is true

                    // Check async flag
                    assert_eq!(arrow.r#async, false);
                }
                _ => panic!("Expected ArrowFunctionExpression, got {:?}", expr_stmt.expression),
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
