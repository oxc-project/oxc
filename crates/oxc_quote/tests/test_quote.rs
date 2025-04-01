use oxc_quote::{jsquote, jsquote_expr, private::*};

#[test]
fn basic_binary_expr() {
    let a = Allocator::default();
    let span = Span::empty(0);

    let expr: Expression = jsquote_expr!(&a, span, { 1234.5 + 2 });

    assert!(expr.content_eq(&Expression::BinaryExpression(Box::new_in(
        BinaryExpression {
            span,
            left: Expression::NumericLiteral(Box::new_in(
                NumericLiteral { span, value: 1234.5, raw: Some(Atom::new_const("1234.5"),), base: NumberBase::Float },
                &a
            )),
            operator: BinaryOperator::Addition,
            right: Expression::NumericLiteral(Box::new_in(
                NumericLiteral { span, value: 2.0, raw: Some(Atom::new_const("2"),), base: NumberBase::Decimal },
                &a
            )),
        },
        &a
    ))));
}

#[test]
fn basic_placeholder() {
    let a = Allocator::default();
    let span = Span::empty(0);

    let expr: Expression = jsquote_expr!(&a, span, { 10.0 });
    let stmt: Vec<Statement> = jsquote!(&a, span, { var foo = #expr; });
    let stmts: Vec<Statement> = jsquote!(&a, span, { #stmt; var bar = foo * 2; });

    assert!(stmts.content_eq(&Vec::from_array_in(
        [
            Statement::VariableDeclaration(Box::new_in(
                VariableDeclaration {
                    span,
                    kind: VariableDeclarationKind::Var,
                    declarations: Vec::from_array_in(
                        [VariableDeclarator {
                            span,
                            kind: VariableDeclarationKind::Var,
                            id: BindingPattern {
                                kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                                    BindingIdentifier { span, name: Atom::from("foo"), symbol_id: ::std::cell::Cell::new(None) },
                                    &a,
                                )),
                                type_annotation: None,
                                optional: false,
                            },
                            init: Some(Expression::NumericLiteral(Box::new_in(
                                NumericLiteral { span, value: 10f64, raw: Some(Atom::from("10.0")), base: NumberBase::Float },
                                &a,
                            ))),
                            definite: false,
                        }],
                        &a,
                    ),
                    declare: false,
                },
                &a,
            )),
            Statement::VariableDeclaration(Box::new_in(
                VariableDeclaration {
                    span,
                    kind: VariableDeclarationKind::Var,
                    declarations: Vec::from_array_in(
                        [VariableDeclarator {
                            span,
                            kind: VariableDeclarationKind::Var,
                            id: BindingPattern {
                                kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                                    BindingIdentifier { span, name: Atom::from("bar"), symbol_id: ::std::cell::Cell::new(None) },
                                    &a,
                                )),
                                type_annotation: None,
                                optional: false,
                            },
                            init: Some(Expression::BinaryExpression(Box::new_in(
                                BinaryExpression {
                                    span,
                                    left: Expression::Identifier(Box::new_in(
                                        IdentifierReference { span, name: Atom::from("foo"), reference_id: ::std::cell::Cell::new(None) },
                                        &a,
                                    )),
                                    operator: BinaryOperator::Multiplication,
                                    right: Expression::NumericLiteral(Box::new_in(
                                        NumericLiteral { span, value: 2f64, raw: Some(Atom::from("2")), base: NumberBase::Decimal },
                                        &a,
                                    )),
                                },
                                &a,
                            ))),
                            definite: false,
                        }],
                        &a,
                    ),
                    declare: false,
                },
                &a,
            )),
        ],
        &a,
    )));
}

#[test]
fn complex_usage() {
    let a = Allocator::default();
    let span = Span::empty(0);

    let expr = jsquote_expr!(&a, span, { 1330.0 + 7 });

    let fn1 = jsquote!(&a, span, {
        class Foo {
            get hello() {
                return #expr;
            }
        }
    });

    let ident = IdentifierReference { name: Atom::new_const("foobar"), span, reference_id: ::std::cell::Cell::new(None) };
    let ident_expr = Expression::Identifier(Box::new_in(ident.clone(), &a));

    let fn2 = jsquote!(&a, span, {
        var foobar = #expr;

        function get_foobar() {
            return #ident_expr;
        }
    });

    let console_message = jsquote_expr!(&a, span, { "Hello, OXC!" });

    let fn3 = jsquote!(&a, span, {
        const say_hello = () => {
            console.log(#console_message, "This is from quote!");
        };
    });

    let fn1_2 = Vec::from_array_in([fn1, fn2], &a);

    let stmts: Vec<Statement> = jsquote!(&a, span, {
        function first() {}
        function second() {}
        #fn1_2
        function third() {}
        #fn3
        function fourth() {}
    });

    assert!(stmts.content_eq(&Vec::from_array_in(
        [
            Statement::FunctionDeclaration(Box::new_in(
                Function {
                    span,
                    r#type: FunctionType::FunctionDeclaration,
                    r#id: Some(BindingIdentifier { span, r#name: Atom::from("first"), r#symbol_id: ::std::cell::Cell::new(None) }),
                    generator: false,
                    r#async: false,
                    declare: false,
                    type_parameters: None,
                    this_param: None,
                    params: Box::new_in(
                        FormalParameters { span, r#kind: FormalParameterKind::FormalParameter, r#items: Vec::new_in(&a), r#rest: None },
                        &a,
                    ),
                    return_type: None,
                    body: Some(Box::new_in(FunctionBody { span, r#directives: Vec::new_in(&a), r#statements: Vec::new_in(&a) }, &a,)),
                    scope_id: ::std::cell::Cell::new(None),
                    pure: false,
                },
                &a,
            )),
            Statement::FunctionDeclaration(Box::new_in(
                Function {
                    span,
                    r#type: FunctionType::FunctionDeclaration,
                    id: Some(BindingIdentifier { span, r#name: Atom::from("second"), r#symbol_id: ::std::cell::Cell::new(None) }),
                    generator: false,
                    r#async: false,
                    declare: false,
                    type_parameters: None,
                    this_param: None,
                    params: Box::new_in(
                        FormalParameters { span, r#kind: FormalParameterKind::FormalParameter, r#items: Vec::new_in(&a), r#rest: None },
                        &a,
                    ),
                    return_type: None,
                    body: Some(Box::new_in(FunctionBody { span, r#directives: Vec::new_in(&a), r#statements: Vec::new_in(&a) }, &a,)),
                    scope_id: ::std::cell::Cell::new(None),
                    pure: false,
                },
                &a,
            )),
            Statement::ClassDeclaration(Box::new_in(
                Class {
                    span,
                    r#type: ClassType::ClassDeclaration,
                    r#decorators: Vec::new_in(&a),
                    r#id: Some(BindingIdentifier { span, r#name: Atom::from("Foo"), r#symbol_id: ::std::cell::Cell::new(None) }),
                    r#type_parameters: None,
                    r#super_class: None,
                    r#super_type_arguments: None,
                    r#implements: None,
                    r#body: Box::new_in(
                        ClassBody {
                            span,
                            r#body: Vec::from_iter_in(
                                [ClassElement::MethodDefinition(Box::new_in(
                                    MethodDefinition {
                                        span,
                                        r#type: MethodDefinitionType::MethodDefinition,
                                        r#decorators: Vec::new_in(&a),
                                        r#key: PropertyKey::StaticIdentifier(Box::new_in(IdentifierName { span, r#name: Atom::from("hello") }, &a,)),
                                        r#value: Box::new_in(
                                            Function {
                                                span,
                                                r#type: FunctionType::FunctionExpression,
                                                r#id: None,
                                                r#generator: false,
                                                r#async: false,
                                                r#declare: false,
                                                r#type_parameters: None,
                                                r#this_param: None,
                                                r#params: Box::new_in(
                                                    FormalParameters {
                                                        span,
                                                        r#kind: FormalParameterKind::FormalParameter,
                                                        r#items: Vec::new_in(&a),
                                                        r#rest: None,
                                                    },
                                                    &a,
                                                ),
                                                r#return_type: None,
                                                r#body: Some(Box::new_in(
                                                    FunctionBody {
                                                        span,
                                                        r#directives: Vec::new_in(&a),
                                                        r#statements: Vec::from_array_in(
                                                            [Statement::ReturnStatement(Box::new_in(
                                                                ReturnStatement { span, r#argument: Some(expr.clone_in(&a)) },
                                                                &a,
                                                            ))],
                                                            &a,
                                                        ),
                                                    },
                                                    &a,
                                                )),
                                                r#scope_id: ::std::cell::Cell::new(None),
                                                r#pure: false,
                                            },
                                            &a,
                                        ),
                                        r#kind: MethodDefinitionKind::Get,
                                        r#computed: false,
                                        r#static: false,
                                        r#override: false,
                                        r#optional: false,
                                        r#accessibility: None,
                                    },
                                    &a,
                                ))],
                                &a
                            ),
                        },
                        &a
                    ),
                    r#abstract: false,
                    r#declare: false,
                    r#scope_id: ::std::cell::Cell::new(None),
                },
                &a
            )),
            Statement::VariableDeclaration(Box::new_in(
                VariableDeclaration {
                    span,
                    r#kind: VariableDeclarationKind::Var,
                    r#declarations: Vec::from_array_in(
                        [VariableDeclarator {
                            span,
                            r#kind: VariableDeclarationKind::Var,
                            r#id: BindingPattern {
                                r#kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                                    BindingIdentifier { span, r#name: Atom::from("foobar"), r#symbol_id: ::std::cell::Cell::new(None) },
                                    &a,
                                )),
                                r#type_annotation: None,
                                r#optional: false,
                            },
                            r#init: Some(expr.clone_in(&a)),
                            r#definite: false,
                        }],
                        &a
                    ),
                    r#declare: false,
                },
                &a
            )),
            Statement::FunctionDeclaration(Box::new_in(
                Function {
                    span,
                    r#type: FunctionType::FunctionDeclaration,
                    r#id: Some(BindingIdentifier { span, r#name: Atom::from("get_foobar"), r#symbol_id: ::std::cell::Cell::new(None) }),
                    r#generator: false,
                    r#async: false,
                    r#declare: false,
                    r#type_parameters: None,
                    r#this_param: None,
                    r#params: Box::new_in(
                        FormalParameters { span, r#kind: FormalParameterKind::FormalParameter, r#items: Vec::new_in(&a), r#rest: None },
                        &a
                    ),
                    r#return_type: None,
                    r#body: Some(Box::new_in(
                        FunctionBody {
                            span,
                            r#directives: Vec::new_in(&a),
                            r#statements: Vec::from_array_in(
                                [Statement::ReturnStatement(Box::new_in(ReturnStatement { span, r#argument: Some(ident_expr.clone_in(&a)) }, &a,))],
                                &a
                            ),
                        },
                        &a
                    )),
                    r#scope_id: ::std::cell::Cell::new(None),
                    r#pure: false,
                },
                &a
            )),
            Statement::FunctionDeclaration(Box::new_in(
                Function {
                    span,
                    r#type: FunctionType::FunctionDeclaration,
                    id: Some(BindingIdentifier { span, r#name: Atom::from("third"), r#symbol_id: ::std::cell::Cell::new(None) }),
                    generator: false,
                    r#async: false,
                    declare: false,
                    type_parameters: None,
                    this_param: None,
                    params: Box::new_in(
                        FormalParameters { span, r#kind: FormalParameterKind::FormalParameter, r#items: Vec::new_in(&a), r#rest: None },
                        &a,
                    ),
                    return_type: None,
                    body: Some(Box::new_in(FunctionBody { span, r#directives: Vec::new_in(&a), r#statements: Vec::new_in(&a) }, &a,)),
                    scope_id: ::std::cell::Cell::new(None),
                    pure: false,
                },
                &a,
            )),
            Statement::VariableDeclaration(Box::new_in(
                VariableDeclaration {
                    span,
                    r#kind: VariableDeclarationKind::Const,
                    r#declarations: Vec::from_array_in(
                        [VariableDeclarator {
                            span,
                            r#kind: VariableDeclarationKind::Const,
                            r#id: BindingPattern {
                                r#kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                                    BindingIdentifier { span, r#name: Atom::from("say_hello"), r#symbol_id: ::std::cell::Cell::new(None) },
                                    &a,
                                )),
                                r#type_annotation: None,
                                r#optional: false,
                            },
                            r#init: Some(Expression::ArrowFunctionExpression(Box::new_in(
                                ArrowFunctionExpression {
                                    span,
                                    r#expression: false,
                                    r#async: false,
                                    r#type_parameters: None,
                                    r#params: Box::new_in(
                                        FormalParameters {
                                            span,
                                            r#kind: FormalParameterKind::ArrowFormalParameters,
                                            r#items: Vec::new_in(&a),
                                            r#rest: None,
                                        },
                                        &a,
                                    ),
                                    r#return_type: None,
                                    r#body: Box::new_in(
                                        FunctionBody {
                                            span,
                                            r#directives: Vec::new_in(&a),
                                            r#statements: Vec::from_array_in(
                                                [Statement::ExpressionStatement(Box::new_in(
                                                    ExpressionStatement {
                                                        span,
                                                        r#expression: Expression::CallExpression(Box::new_in(
                                                            CallExpression {
                                                                span,
                                                                r#callee: Expression::StaticMemberExpression(Box::new_in(
                                                                    StaticMemberExpression {
                                                                        span,
                                                                        r#object: Expression::Identifier(Box::new_in(
                                                                            IdentifierReference {
                                                                                span,
                                                                                r#name: Atom::from("console"),
                                                                                r#reference_id: ::std::cell::Cell::new(None),
                                                                            },
                                                                            &a,
                                                                        )),
                                                                        r#property: IdentifierName { span, r#name: Atom::from("log") },
                                                                        r#optional: false,
                                                                    },
                                                                    &a,
                                                                )),
                                                                r#type_arguments: None,
                                                                r#arguments: Vec::from_array_in(
                                                                    [
                                                                        Argument::StringLiteral(Box::new_in(
                                                                            StringLiteral {
                                                                                span,
                                                                                r#value: Atom::from("Hello, OXC!"),
                                                                                r#raw: Some(Atom::from("\"Hello, OXC!\"")),
                                                                                r#lossy: false,
                                                                            },
                                                                            &a
                                                                        )),
                                                                        Argument::StringLiteral(Box::new_in(
                                                                            StringLiteral {
                                                                                span,
                                                                                r#value: Atom::from("This is from quote!"),
                                                                                r#raw: Some(Atom::from("\"This is from quote!\"")),
                                                                                r#lossy: false,
                                                                            },
                                                                            &a,
                                                                        ))
                                                                    ],
                                                                    &a,
                                                                ),
                                                                r#optional: false,
                                                                r#pure: false,
                                                            },
                                                            &a,
                                                        )),
                                                    },
                                                    &a,
                                                ))],
                                                &a,
                                            ),
                                        },
                                        &a,
                                    ),
                                    r#scope_id: ::std::cell::Cell::new(None),
                                    r#pure: false,
                                },
                                &a,
                            ))),
                            r#definite: false,
                        }],
                        &a
                    ),
                    r#declare: false,
                },
                &a
            )),
            Statement::FunctionDeclaration(Box::new_in(
                Function {
                    span,
                    r#type: FunctionType::FunctionDeclaration,
                    id: Some(BindingIdentifier { span, r#name: Atom::from("fourth"), r#symbol_id: ::std::cell::Cell::new(None) }),
                    generator: false,
                    r#async: false,
                    declare: false,
                    type_parameters: None,
                    this_param: None,
                    params: Box::new_in(
                        FormalParameters { span, r#kind: FormalParameterKind::FormalParameter, r#items: Vec::new_in(&a), r#rest: None },
                        &a,
                    ),
                    return_type: None,
                    body: Some(Box::new_in(FunctionBody { span, r#directives: Vec::new_in(&a), r#statements: Vec::new_in(&a) }, &a,)),
                    scope_id: ::std::cell::Cell::new(None),
                    pure: false,
                },
                &a,
            ))
        ],
        &a,
    )));
}
