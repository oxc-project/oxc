use oxc_allocator::{Box, CloneIn};
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingRestElement, BlockStatement, Expression, FormalParameterKind,
    Function, FunctionType, Statement, TSAccessibility, TSThisParameter, TSTypeAnnotation,
    TSTypeParameterDeclaration, TSTypeParameterInstantiation, VariableDeclarationKind,
};
use oxc_ast::AstBuilder;
use oxc_span::SPAN;
use oxc_syntax::operator::UnaryOperator;

pub fn async_generator_step<'a>(builder: &'a AstBuilder) -> Statement<'a> {
    builder.statement_declaration(builder.declaration_function(
        FunctionType::FunctionDeclaration,
        SPAN,
        Some(builder.binding_identifier(SPAN, "asyncGeneratorStep")),
        false,
        false,
        false,
        None::<TSTypeParameterDeclaration>,
        None::<TSThisParameter>,
        builder.formal_parameters(
            SPAN,
            FormalParameterKind::FormalParameter,
            {
                let mut items = builder.vec();
                items.push(builder.formal_parameter(
                    SPAN,
                    builder.vec(),
                    builder.binding_pattern(
                        builder.binding_pattern_kind_binding_identifier(SPAN, "gen"),
                        None::<TSTypeAnnotation>,
                        false,
                    ),
                    None::<TSAccessibility>,
                    false,
                    false,
                ));
                items.push(builder.formal_parameter(
                    SPAN,
                    builder.vec(),
                    builder.binding_pattern(
                        builder.binding_pattern_kind_binding_identifier(SPAN, "resolve"),
                        None::<TSTypeAnnotation>,
                        false,
                    ),
                    None::<TSAccessibility>,
                    false,
                    false,
                ));
                items.push(builder.formal_parameter(
                    SPAN,
                    builder.vec(),
                    builder.binding_pattern(
                        builder.binding_pattern_kind_binding_identifier(SPAN, "reject"),
                        None::<TSTypeAnnotation>,
                        false,
                    ),
                    None::<TSAccessibility>,
                    false,
                    false,
                ));
                items.push(builder.formal_parameter(
                    SPAN,
                    builder.vec(),
                    builder.binding_pattern(
                        builder.binding_pattern_kind_binding_identifier(SPAN, "_next"),
                        None::<TSTypeAnnotation>,
                        false,
                    ),
                    None::<TSAccessibility>,
                    false,
                    false,
                ));
                items.push(builder.formal_parameter(
                    SPAN,
                    builder.vec(),
                    builder.binding_pattern(
                        builder.binding_pattern_kind_binding_identifier(SPAN, "_throw"),
                        None::<TSTypeAnnotation>,
                        false,
                    ),
                    None::<TSAccessibility>,
                    false,
                    false,
                ));
                items.push(builder.formal_parameter(
                    SPAN,
                    builder.vec(),
                    builder.binding_pattern(
                        builder.binding_pattern_kind_binding_identifier(SPAN, "key"),
                        None::<TSTypeAnnotation>,
                        false,
                    ),
                    None::<TSAccessibility>,
                    false,
                    false,
                ));
                items.push(builder.formal_parameter(
                    SPAN,
                    builder.vec(),
                    builder.binding_pattern(
                        builder.binding_pattern_kind_binding_identifier(SPAN, "arg"),
                        None::<TSTypeAnnotation>,
                        false,
                    ),
                    None::<TSAccessibility>,
                    false,
                    false,
                ));
                items
            },
            None::<BindingRestElement>,
        ),
        None::<TSTypeAnnotation>,
        Some(builder.function_body(SPAN, builder.vec(), {
            let mut items = builder.vec();
            items.push(builder.statement_try(
                SPAN,
                builder.block_statement(
                    SPAN,
                    builder.vec1(builder.statement_declaration(builder.declaration_variable(
                        SPAN,
                        VariableDeclarationKind::Var,
                        {
                            let mut items = builder.vec();
                            items.push(builder.variable_declarator(
                                SPAN,
                                VariableDeclarationKind::Var,
                                builder.binding_pattern(
                                    builder.binding_pattern_kind_binding_identifier(SPAN, "i"),
                                    None::<TSTypeAnnotation>,
                                    false,
                                ),
                                Some(builder.expression_call(
                                    SPAN,
                                    builder.expression_member(builder.member_expression_computed(
                                        SPAN,
                                        builder.expression_identifier_reference(SPAN, "gen"),
                                        builder.expression_identifier_reference(SPAN, "key"),
                                        false,
                                    )),
                                    None::<TSTypeParameterInstantiation>,
                                    builder.vec1(builder.argument_expression(
                                        builder.expression_identifier_reference(SPAN, "arg"),
                                    )),
                                    false,
                                )),
                                false,
                            ));
                            items.push(builder.variable_declarator(
                                SPAN,
                                VariableDeclarationKind::Var,
                                builder.binding_pattern(
                                    builder.binding_pattern_kind_binding_identifier(SPAN, "u"),
                                    None::<TSTypeAnnotation>,
                                    false,
                                ),
                                Some(builder.expression_member(builder.member_expression_static(
                                    SPAN,
                                    builder.expression_identifier_reference(SPAN, "i"),
                                    builder.identifier_name(SPAN, "value"),
                                    false,
                                ))),
                                false,
                            ));
                            items
                        },
                        false,
                    ))),
                ),
                Some(builder.catch_clause(
                    SPAN,
                    Some(builder.catch_parameter(
                        SPAN,
                        builder.binding_pattern(
                            builder.binding_pattern_kind_binding_identifier(SPAN, "gen"),
                            None::<TSTypeAnnotation>,
                            false,
                        ),
                    )),
                    builder.block_statement(
                        SPAN,
                        builder.vec1(builder.statement_return(
                            SPAN,
                            Some(builder.expression_unary(
                                SPAN,
                                UnaryOperator::Void,
                                builder.expression_call(
                                    SPAN,
                                    builder.expression_identifier_reference(SPAN, "reject"),
                                    None::<TSTypeParameterInstantiation>,
                                    builder.vec1(builder.argument_expression(
                                        builder.expression_identifier_reference(SPAN, "gen"),
                                    )),
                                    false,
                                ),
                            )),
                        )),
                    ),
                )),
                None::<BlockStatement>,
            ));
            items.push(builder.statement_expression(
                SPAN,
                builder.expression_conditional(
                    SPAN,
                    builder.expression_member(builder.member_expression_static(
                        SPAN,
                        builder.expression_identifier_reference(SPAN, "i"),
                        builder.identifier_name(SPAN, "done"),
                        false,
                    )),
                    builder.expression_call(
                        SPAN,
                        builder.expression_identifier_reference(SPAN, "resolve"),
                        None::<TSTypeParameterInstantiation>,
                        builder.vec1(builder.argument_expression(
                            builder.expression_identifier_reference(SPAN, "u"),
                        )),
                        false,
                    ),
                    builder.expression_call(
                        SPAN,
                        builder.expression_member(builder.member_expression_static(
                            SPAN,
                            builder.expression_call(
                                SPAN,
                                builder.expression_member(builder.member_expression_static(
                                    SPAN,
                                    builder.expression_identifier_reference(SPAN, "Promise"),
                                    builder.identifier_name(SPAN, "resolve"),
                                    false,
                                )),
                                None::<TSTypeParameterInstantiation>,
                                builder.vec1(builder.argument_expression(
                                    builder.expression_identifier_reference(SPAN, "u"),
                                )),
                                false,
                            ),
                            builder.identifier_name(SPAN, "then"),
                            false,
                        )),
                        None::<TSTypeParameterInstantiation>,
                        {
                            let mut items = builder.vec();
                            items.push(builder.argument_expression(
                                builder.expression_identifier_reference(SPAN, "_next"),
                            ));
                            items.push(builder.argument_expression(
                                builder.expression_identifier_reference(SPAN, "_throw"),
                            ));
                            items
                        },
                        false,
                    ),
                ),
            ));
            items
        })),
    ))
}

fn async_generator_step_caller<'a>(parameter: &'a str, builder: &'a AstBuilder) -> Statement<'a> {
    builder.statement_declaration(builder.declaration_function(
        FunctionType::FunctionDeclaration,
        SPAN,
        Some(builder.binding_identifier(SPAN, "_".to_owned() + parameter)),
        false,
        false,
        false,
        None::<TSTypeParameterDeclaration>,
        None::<TSThisParameter>,
        builder.formal_parameters(
            SPAN,
            FormalParameterKind::FormalParameter,
            builder.vec1(builder.formal_parameter(
                SPAN,
                builder.vec(),
                builder.binding_pattern(
                    builder.binding_pattern_kind_binding_identifier(SPAN, "gen"),
                    None::<TSTypeAnnotation>,
                    false,
                ),
                None::<TSAccessibility>,
                false,
                false,
            )),
            None::<BindingRestElement>,
        ),
        None::<TSTypeAnnotation>,
        Some(builder.function_body(
            SPAN,
            builder.vec(),
            builder.vec1(builder.statement_expression(
                SPAN,
                builder.expression_call(
                    SPAN,
                    builder.expression_identifier_reference(SPAN, "asyncGeneratorStep"),
                    None::<TSTypeParameterInstantiation>,
                    {
                        let mut items = builder.vec();
                        items.push(builder.argument_expression(
                            builder.expression_identifier_reference(SPAN, "key"),
                        ));
                        items.push(builder.argument_expression(
                            builder.expression_identifier_reference(SPAN, "_next"),
                        ));
                        items.push(builder.argument_expression(
                            builder.expression_identifier_reference(SPAN, "_throw"),
                        ));
                        items.push(builder.argument_expression(
                            builder.expression_identifier_reference(SPAN, "_next"),
                        ));
                        items.push(builder.argument_expression(
                            builder.expression_identifier_reference(SPAN, "_throw"),
                        ));
                        items.push(builder.argument_expression(
                            builder.expression_string_literal(SPAN, parameter),
                        ));
                        items.push(builder.argument_expression(
                            builder.expression_identifier_reference(SPAN, "gen"),
                        ));
                        items
                    },
                    false,
                ),
            )),
        )),
    ))
}

fn caller_promise<'a>(builder: &'a AstBuilder) -> Expression<'a> {
    builder.expression_new(
        SPAN,
        builder.expression_identifier_reference(SPAN, "Promise"),
        builder.vec1(builder.argument_expression(builder.expression_function(
            FunctionType::FunctionExpression,
            SPAN,
            None,
            false,
            false,
            false,
            None::<TSTypeParameterDeclaration>,
            None::<TSThisParameter>,
            builder.formal_parameters(
                SPAN,
                FormalParameterKind::FormalParameter,
                {
                    let mut items = builder.vec();
                    items.push(builder.formal_parameter(
                        SPAN,
                        builder.vec(),
                        builder.binding_pattern(
                            builder.binding_pattern_kind_binding_identifier(SPAN, "_next"),
                            None::<TSTypeAnnotation>,
                            false,
                        ),
                        None::<TSAccessibility>,
                        false,
                        false,
                    ));
                    items.push(builder.formal_parameter(
                        SPAN,
                        builder.vec(),
                        builder.binding_pattern(
                            builder.binding_pattern_kind_binding_identifier(SPAN, "_throw"),
                            None::<TSTypeAnnotation>,
                            false,
                        ),
                        None::<TSAccessibility>,
                        false,
                        false,
                    ));
                    items
                },
                None::<BindingRestElement>,
            ),
            None::<TSTypeAnnotation>,
            Some(builder.function_body(SPAN, builder.vec(), {
                let mut items = builder.vec();
                items.push(builder.statement_declaration(builder.declaration_variable(
                    SPAN,
                    VariableDeclarationKind::Var,
                    builder.vec1(builder.variable_declarator(
                        SPAN,
                        VariableDeclarationKind::Var,
                        builder.binding_pattern(
                            builder.binding_pattern_kind_binding_identifier(SPAN, "key"),
                            None::<TSTypeAnnotation>,
                            false,
                        ),
                        Some(builder.expression_call(
                            SPAN,
                            builder.expression_member(builder.member_expression_static(
                                SPAN,
                                builder.expression_identifier_reference(SPAN, "fn"),
                                builder.identifier_name(SPAN, "apply"),
                                false,
                            )),
                            None::<TSTypeParameterInstantiation>,
                            {
                                let mut items = builder.vec();
                                items.push(builder.argument_expression(
                                    builder.expression_identifier_reference(SPAN, "resolve"),
                                ));
                                items.push(builder.argument_expression(
                                    builder.expression_identifier_reference(SPAN, "reject"),
                                ));
                                items
                            },
                            false,
                        )),
                        false,
                    )),
                    false,
                )));
                items.push(async_generator_step_caller("next", builder));
                items.push(async_generator_step_caller("throw", builder));
                items.push(builder.statement_expression(
                    SPAN,
                    builder.expression_call(
                        SPAN,
                        builder.expression_identifier_reference(SPAN, "_next"),
                        None::<TSTypeParameterInstantiation>,
                        builder.vec1(builder.argument_expression(builder.void_0())),
                        false,
                    ),
                ));
                items
            })),
        ))),
        None::<TSTypeParameterInstantiation>,
    )
}

pub fn async_to_generator<'a>(builder: &'a AstBuilder) -> Statement<'a> {
    builder.statement_declaration(builder.declaration_function(
        FunctionType::FunctionDeclaration,
        SPAN,
        Some(builder.binding_identifier(SPAN, "_asyncToGenerator")),
        false,
        false,
        false,
        None::<TSTypeParameterDeclaration>,
        None::<TSThisParameter>,
        builder.formal_parameters(
            SPAN,
            FormalParameterKind::FormalParameter,
            builder.vec1(builder.formal_parameter(
                SPAN,
                builder.vec(),
                builder.binding_pattern(
                    builder.binding_pattern_kind_binding_identifier(SPAN, "fn"),
                    None::<TSTypeAnnotation>,
                    false,
                ),
                None::<TSAccessibility>,
                false,
                false,
            )),
            None::<BindingRestElement>,
        ),
        None::<TSTypeAnnotation>,
        Some(builder.function_body(
            SPAN,
            builder.vec(),
            builder.vec1(builder.statement_return(
                SPAN,
                Some(builder.expression_function(
                    FunctionType::FunctionExpression,
                    SPAN,
                    None,
                    false,
                    false,
                    false,
                    None::<TSTypeParameterDeclaration>,
                    None::<TSThisParameter>,
                    builder.formal_parameters(
                        SPAN,
                        FormalParameterKind::FormalParameter,
                        builder.vec(),
                        None::<BindingRestElement>,
                    ),
                    None::<TSTypeAnnotation>,
                    Some(builder.function_body(SPAN, builder.vec(), {
                        let mut items = builder.vec();
                        items.push(builder.statement_declaration(builder.declaration_variable(
                            SPAN,
                            VariableDeclarationKind::Var,
                            {
                                let mut items = builder.vec();
                                items.push(builder.variable_declarator(
                                    SPAN,
                                    VariableDeclarationKind::Var,
                                    builder.binding_pattern(
                                        builder.binding_pattern_kind_binding_identifier(
                                            SPAN, "resolve",
                                        ),
                                        None::<TSTypeAnnotation>,
                                        false,
                                    ),
                                    Some(builder.expression_this(SPAN)),
                                    false,
                                ));
                                items.push(builder.variable_declarator(
                                    SPAN,
                                    VariableDeclarationKind::Var,
                                    builder.binding_pattern(
                                        builder.binding_pattern_kind_binding_identifier(
                                            SPAN, "reject",
                                        ),
                                        None::<TSTypeAnnotation>,
                                        false,
                                    ),
                                    Some(
                                        builder.expression_identifier_reference(SPAN, "arguments"),
                                    ),
                                    false,
                                ));
                                items
                            },
                            false,
                        )));
                        items.push(builder.statement_return(SPAN, Some(caller_promise(builder))));
                        items
                    })),
                )),
            )),
        )),
    ))
}

pub fn function_apply<'a>(name: &'a str, builder: &'a AstBuilder) -> Statement<'a> {
    builder.statement_return(
        SPAN,
        Some(builder.expression_call(
            SPAN,
            builder.expression_member(builder.member_expression_static(
                SPAN,
                builder.expression_identifier_reference(SPAN, name),
                builder.identifier_name(SPAN, "apply"),
                false,
            )),
            None::<TSTypeParameterInstantiation>,
            {
                let mut items = builder.vec();
                items.push(builder.argument_expression(builder.expression_this(SPAN)));
                items.push(builder.argument_expression(
                    builder.expression_identifier_reference(SPAN, "arguments"),
                ));
                items
            },
            false,
        )),
    )
}

pub fn generate_caller_from_arrow<'a>(
    func: Box<ArrowFunctionExpression>,
    builder: &'a AstBuilder,
) -> Expression<'a> {
    let result = builder.function(
        FunctionType::FunctionExpression,
        func.span,
        None,
        true,
        false,
        false,
        func.type_parameters.clone_in(builder.allocator),
        None::<TSThisParameter>,
        func.params.clone_in(builder.allocator),
        func.return_type.clone_in(builder.allocator),
        Some(func.body.clone_in(builder.allocator)),
    );
    builder.expression_call(
        SPAN,
        builder.expression_identifier_reference(SPAN, "_asyncToGenerator"),
        None::<TSTypeParameterInstantiation>,
        builder.vec1(builder.argument_expression(builder.expression_from_function(result))),
        false,
    )
}

pub fn generate_caller_from_function<'a>(
    func: &Box<Function<'a>>,
    builder: &'a AstBuilder,
) -> Expression<'a> {
    let result = builder.function(
        FunctionType::FunctionExpression,
        func.span,
        None,
        true,
        false,
        false,
        func.type_parameters.clone_in(builder.allocator),
        None::<TSThisParameter>,
        func.params.clone_in(builder.allocator),
        func.return_type.clone_in(builder.allocator),
        func.body.clone_in(builder.allocator),
    );
    builder.expression_call(
        SPAN,
        builder.expression_identifier_reference(SPAN, "_asyncToGenerator"),
        None::<TSTypeParameterInstantiation>,
        builder.vec1(builder.argument_expression(builder.expression_from_function(result))),
        false,
    )
}
