//! Handwritten generation for nodes with cross-field or contextual invariants.

#![expect(clippy::redundant_pub_crate)]

use rand::Rng;

use oxc_allocator::{Box as ArenaBox, Vec as ArenaVec};
use oxc_ast::ast::*;
use oxc_span::SPAN;

use crate::AstGenerator;

pub(crate) fn generate_expression<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Expression<'a> {
    generator.with_expr_depth(generate_expression_inner)
}

fn generate_expression_inner<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Expression<'a> {
    let context = generator.context();

    if !generator.at_limit() {
        if generator.is_typescript()
            && generator.can_nest_exprs(2)
            && generator.random_index(8) == 0
        {
            let expression =
                generator.with_expr_depth(|generator| match generator.random_index(5) {
                    0 => Expression::TSAsExpression(generator.random_box()),
                    1 => Expression::TSSatisfiesExpression(generator.random_box()),
                    2 => generate_ts_type_assertion(generator),
                    3 => Expression::TSNonNullExpression(generator.random_box()),
                    4 => Expression::TSInstantiationExpression(generator.random_box()),
                    _ => unreachable!(),
                });
            return wrap_expression_as_call_argument(generator, expression);
        }
        if context.in_async && generator.random_index(12) == 0 {
            return Expression::AwaitExpression(generator.random_box());
        }
        if context.in_generator && generator.random_index(12) == 0 {
            return Expression::YieldExpression(generator.random_box());
        }
        if generator.source_type().is_module() && generator.random_index(16) == 0 {
            return Expression::ImportMeta(generator.random_box());
        }
        if context.allow_new_target && generator.random_index(16) == 0 {
            return Expression::NewTarget(generator.random_box());
        }
    }

    let choices = if generator.at_limit() { 8 } else { 29 };
    match generator.random_index(choices) {
        0 => Expression::new_boolean_literal(SPAN, generator.random_bool(), generator.ast()),
        1 => Expression::new_null_literal(SPAN, generator.ast()),
        2 => Expression::new_numeric_literal(
            SPAN,
            f64::from(i32::try_from(generator.random_index(2_001)).unwrap() - 1_000),
            None,
            NumberBase::Decimal,
            generator.ast(),
        ),
        3 => {
            let value = generator.random_str();
            Expression::new_string_literal(SPAN, value, None, generator.ast())
        }
        4 => {
            let name = generator.random_ident();
            Expression::new_identifier(SPAN, name, generator.ast())
        }
        5 => generate_bigint_literal(generator),
        6 => generate_regexp_literal(generator),
        7 => Expression::TemplateLiteral(generator.random_box()),
        8 => Expression::ArrayExpression(generator.random_box()),
        9 => Expression::ObjectExpression(generator.random_box()),
        10 => Expression::BinaryExpression(generator.random_box()),
        11 => Expression::LogicalExpression(generator.random_box()),
        12 => Expression::ConditionalExpression(generator.random_box()),
        13 => Expression::CallExpression(generator.random_box()),
        14 => Expression::NewExpression(generator.random_box()),
        15 => Expression::UnaryExpression(generator.random_box()),
        16 => Expression::ParenthesizedExpression(generator.random_box()),
        17 => Expression::FunctionExpression(generator.random_box()),
        18 => Expression::ArrowFunctionExpression(generator.random_box()),
        19 => generate_assignment_expression(generator),
        20 if generator.can_nest_exprs(2) => generate_class_expression(generator),
        20 => Expression::new_identifier(SPAN, generator.random_ident(), generator.ast()),
        21 => generate_import_expression(generator),
        22 => generate_sequence_expression(generator),
        23 => generate_tagged_template_expression(generator),
        24 => Expression::ThisExpression(generator.random_box()),
        25 => generate_update_expression(generator),
        26 => Expression::ComputedMemberExpression(generator.random_box()),
        27 => Expression::StaticMemberExpression(generator.random_box()),
        28 => generate_chain_expression(generator),
        _ => unreachable!(),
    }
}

fn generate_bigint_literal<'a, R: Rng + ?Sized>(
    generator: &AstGenerator<'a, '_, R>,
) -> Expression<'a> {
    Expression::new_big_int_literal(
        SPAN,
        oxc_str::Str::from_str_in("0", generator.ast()),
        None,
        BigintBase::Decimal,
        generator.ast(),
    )
}

fn generate_regexp_literal<'a, R: Rng + ?Sized>(
    generator: &AstGenerator<'a, '_, R>,
) -> Expression<'a> {
    let pattern =
        RegExpPattern { text: oxc_str::Str::from_str_in("a", generator.ast()), pattern: None };
    let regex = RegExp { pattern, flags: RegExpFlags::empty() };
    Expression::new_reg_exp_literal(SPAN, regex, None, generator.ast())
}

fn generate_ts_type_assertion<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Expression<'a> {
    let expression = Expression::new_identifier(SPAN, generator.random_ident(), generator.ast());
    Expression::TSTypeAssertion(TSTypeAssertion::boxed(
        SPAN,
        generator.generate(),
        expression,
        generator.ast(),
    ))
}

fn wrap_expression_as_call_argument<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
    expression: Expression<'a>,
) -> Expression<'a> {
    let callee = Expression::new_identifier(SPAN, generator.random_ident(), generator.ast());
    let mut arguments = ArenaVec::with_capacity_in(1, generator.ast());
    arguments.push(expression.into());
    Expression::CallExpression(CallExpression::boxed(
        SPAN,
        callee,
        None,
        arguments,
        false,
        generator.ast(),
    ))
}

pub(crate) fn generate_call_expression<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> CallExpression<'a> {
    let callee = Expression::new_identifier(SPAN, generator.random_ident(), generator.ast());
    let type_arguments =
        (generator.is_typescript() && generator.random_bool()).then(|| generator.random_box());
    let arguments = generator.random_vec();
    CallExpression::new(SPAN, callee, type_arguments, arguments, false, generator.ast())
}

pub(crate) fn generate_new_expression<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> NewExpression<'a> {
    let callee = Expression::new_identifier(SPAN, generator.random_ident(), generator.ast());
    let type_arguments =
        (generator.is_typescript() && generator.random_bool()).then(|| generator.random_box());
    let arguments = generator.random_vec();
    NewExpression::new(SPAN, callee, type_arguments, arguments, generator.ast())
}

fn generate_assignment_expression<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Expression<'a> {
    let target = AssignmentTarget::AssignmentTargetIdentifier(IdentifierReference::boxed(
        SPAN,
        generator.random_ident(),
        generator.ast(),
    ));
    Expression::AssignmentExpression(AssignmentExpression::boxed(
        SPAN,
        AssignmentOperator::Assign,
        target,
        generator.generate(),
        generator.ast(),
    ))
}

fn generate_update_expression<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Expression<'a> {
    let target = SimpleAssignmentTarget::AssignmentTargetIdentifier(IdentifierReference::boxed(
        SPAN,
        generator.random_ident(),
        generator.ast(),
    ));
    Expression::UpdateExpression(UpdateExpression::boxed(
        SPAN,
        UpdateOperator::Increment,
        false,
        target,
        generator.ast(),
    ))
}

fn generate_import_expression<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Expression<'a> {
    let source =
        Expression::new_string_literal(SPAN, generator.random_str(), None, generator.ast());
    Expression::ImportExpression(ImportExpression::boxed(SPAN, source, None, None, generator.ast()))
}

fn generate_sequence_expression<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Expression<'a> {
    let mut expressions = ArenaVec::with_capacity_in(2, generator.ast());
    expressions.push(generator.generate());
    expressions.push(generator.generate());
    Expression::SequenceExpression(SequenceExpression::boxed(SPAN, expressions, generator.ast()))
}

fn generate_tagged_template_expression<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Expression<'a> {
    let tag = Expression::new_identifier(SPAN, generator.random_ident(), generator.ast());
    Expression::TaggedTemplateExpression(TaggedTemplateExpression::boxed(
        SPAN,
        tag,
        None,
        generator.generate(),
        generator.ast(),
    ))
}

fn generate_chain_expression<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Expression<'a> {
    let object = Expression::new_identifier(SPAN, generator.random_ident(), generator.ast());
    let property = Expression::new_identifier(SPAN, generator.random_ident(), generator.ast());
    let member = ComputedMemberExpression::boxed(SPAN, object, property, true, generator.ast());
    Expression::ChainExpression(ChainExpression::boxed(
        SPAN,
        ChainElement::ComputedMemberExpression(member),
        generator.ast(),
    ))
}

fn generate_class_expression<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Expression<'a> {
    let expression = generator.with_expr_depth(|generator| {
        Expression::ClassExpression(ArenaBox::new_in(
            generate_class_with_type(generator, ClassType::ClassExpression),
            generator.ast(),
        ))
    });
    wrap_expression_as_call_argument(generator, expression)
}

fn generate_class_with_type<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
    class_type: ClassType,
) -> Class<'a> {
    let id = (class_type == ClassType::ClassDeclaration)
        .then(|| BindingIdentifier::new(SPAN, generator.random_ident(), generator.ast()));
    let is_derived =
        !generator.at_limit() && generator.can_nest_exprs(2) && generator.random_index(6) == 0;
    let body = generator.with_context(
        |context| {
            context.in_async = false;
            context.in_generator = false;
            context.allow_new_target = false;
        },
        |generator| generate_class_body(generator, is_derived),
    );
    let decorators = if generator.is_typescript() && generator.random_index(4) == 0 {
        let expression =
            Expression::new_identifier(SPAN, generator.random_ident(), generator.ast());
        let mut decorators = ArenaVec::with_capacity_in(1, generator.ast());
        decorators.push(Decorator::new(SPAN, expression, generator.ast()));
        decorators
    } else {
        ArenaVec::new_in(generator.ast())
    };
    let heritage = is_derived.then(|| {
        ClassHeritage::new(
            Expression::new_identifier(SPAN, generator.random_ident(), generator.ast()),
            None,
            generator.ast(),
        )
    });
    Class::new(
        SPAN,
        class_type,
        decorators,
        id,
        None,
        heritage,
        ArenaVec::new_in(generator.ast()),
        body,
        false,
        false,
        generator.ast(),
    )
}

fn generate_class_body<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
    is_derived: bool,
) -> ArenaBox<'a, ClassBody<'a>> {
    let mut elements = ArenaVec::new_in(generator.ast());
    if !generator.at_limit() {
        if is_derived {
            elements.push(ClassElement::MethodDefinition(ArenaBox::new_in(
                generate_super_method_definition(generator),
                generator.ast(),
            )));
            return ClassBody::boxed(SPAN, elements, generator.ast());
        }
        let choices = if generator.can_nest_exprs(2) { 5 } else { 4 };
        let choice = generator.random_index(choices);
        if choice == 4 {
            generate_private_class_elements(generator, &mut elements);
            return ClassBody::boxed(SPAN, elements, generator.ast());
        }
        let element = match choice {
            0 => ClassElement::StaticBlock(StaticBlock::boxed(
                SPAN,
                ArenaVec::new_in(generator.ast()),
                generator.ast(),
            )),
            1 => ClassElement::MethodDefinition(ArenaBox::new_in(
                generate_method_definition(generator),
                generator.ast(),
            )),
            2 => ClassElement::PropertyDefinition(ArenaBox::new_in(
                generate_property_definition(generator),
                generator.ast(),
            )),
            3 => ClassElement::AccessorProperty(ArenaBox::new_in(
                generate_accessor_property(generator),
                generator.ast(),
            )),
            _ => unreachable!(),
        };
        elements.push(element);
    }
    ClassBody::boxed(SPAN, elements, generator.ast())
}

fn generate_super_method_definition<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> MethodDefinition<'a> {
    let object = Expression::Super(Super::boxed(SPAN, generator.ast()));
    let property = IdentifierName::new(SPAN, generator.random_ident(), generator.ast());
    let expression = Expression::StaticMemberExpression(StaticMemberExpression::boxed(
        SPAN,
        object,
        property,
        false,
        generator.ast(),
    ));
    let mut statements = ArenaVec::with_capacity_in(1, generator.ast());
    if generator.reserve_stmt() {
        statements.push(Statement::ExpressionStatement(ExpressionStatement::boxed(
            SPAN,
            expression,
            generator.ast(),
        )));
    }
    let body =
        FunctionBody::boxed(SPAN, ArenaVec::new_in(generator.ast()), statements, generator.ast());
    let mut function =
        generate_function_with_type(generator, FunctionType::FunctionExpression, None);
    function.r#async = false;
    function.generator = false;
    function.body = Some(body);
    MethodDefinition::new(
        SPAN,
        MethodDefinitionType::MethodDefinition,
        ArenaVec::new_in(generator.ast()),
        static_property_key(generator),
        ArenaBox::new_in(function, generator.ast()),
        MethodDefinitionKind::Method,
        false,
        false,
        false,
        false,
        None,
        generator.ast(),
    )
}

fn private_name<'a, R: Rng + ?Sized>(generator: &AstGenerator<'a, '_, R>) -> PrivateIdentifier<'a> {
    PrivateIdentifier::new(
        SPAN,
        oxc_str::Ident::from_str_in("private", generator.ast()),
        generator.ast(),
    )
}

fn generate_private_class_elements<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
    elements: &mut ArenaVec<'a, ClassElement<'a>>,
) {
    let private_key =
        PropertyKey::PrivateIdentifier(ArenaBox::new_in(private_name(generator), generator.ast()));
    elements.push(ClassElement::PropertyDefinition(ArenaBox::new_in(
        PropertyDefinition::new(
            SPAN,
            PropertyDefinitionType::PropertyDefinition,
            ArenaVec::new_in(generator.ast()),
            private_key,
            None,
            None,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            None,
            generator.ast(),
        ),
        generator.ast(),
    )));

    let private_in = Expression::PrivateInExpression(PrivateInExpression::boxed(
        SPAN,
        private_name(generator),
        Expression::ThisExpression(ThisExpression::boxed(SPAN, generator.ast())),
        generator.ast(),
    ));
    elements.push(ClassElement::PropertyDefinition(ArenaBox::new_in(
        generate_property_definition_with_value(generator, private_in),
        generator.ast(),
    )));

    let private_field = Expression::PrivateFieldExpression(PrivateFieldExpression::boxed(
        SPAN,
        Expression::ThisExpression(ThisExpression::boxed(SPAN, generator.ast())),
        private_name(generator),
        false,
        generator.ast(),
    ));
    elements.push(ClassElement::PropertyDefinition(ArenaBox::new_in(
        generate_property_definition_with_value(generator, private_field),
        generator.ast(),
    )));
}

fn generate_property_definition_with_value<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
    value: Expression<'a>,
) -> PropertyDefinition<'a> {
    PropertyDefinition::new(
        SPAN,
        PropertyDefinitionType::PropertyDefinition,
        ArenaVec::new_in(generator.ast()),
        static_property_key(generator),
        None,
        Some(value),
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        None,
        generator.ast(),
    )
}

fn static_property_key<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> PropertyKey<'a> {
    PropertyKey::StaticIdentifier(IdentifierName::boxed(
        SPAN,
        generator.random_ident(),
        generator.ast(),
    ))
}

fn generate_method_definition<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> MethodDefinition<'a> {
    let function = generate_function_with_type(generator, FunctionType::FunctionExpression, None);
    MethodDefinition::new(
        SPAN,
        MethodDefinitionType::MethodDefinition,
        ArenaVec::new_in(generator.ast()),
        static_property_key(generator),
        ArenaBox::new_in(function, generator.ast()),
        MethodDefinitionKind::Method,
        false,
        false,
        false,
        false,
        None,
        generator.ast(),
    )
}

fn generate_property_definition<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> PropertyDefinition<'a> {
    PropertyDefinition::new(
        SPAN,
        PropertyDefinitionType::PropertyDefinition,
        ArenaVec::new_in(generator.ast()),
        static_property_key(generator),
        None,
        generator.random_option(),
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        None,
        generator.ast(),
    )
}

fn generate_accessor_property<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> AccessorProperty<'a> {
    AccessorProperty::new(
        SPAN,
        AccessorPropertyType::AccessorProperty,
        ArenaVec::new_in(generator.ast()),
        static_property_key(generator),
        None,
        generator.random_option(),
        false,
        false,
        false,
        false,
        None,
        generator.ast(),
    )
}

fn generate_for_statement<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Statement<'a> {
    let body = generate_loop_body(generator);
    Statement::ForStatement(ForStatement::boxed(
        SPAN,
        None,
        generator.random_option(),
        generator.random_option(),
        body,
        generator.ast(),
    ))
}

fn generate_for_in_statement<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Statement<'a> {
    let body = generate_loop_body(generator);
    let left = ForStatementLeft::AssignmentTargetIdentifier(IdentifierReference::boxed(
        SPAN,
        generator.random_ident(),
        generator.ast(),
    ));
    Statement::ForInStatement(ForInStatement::boxed(
        SPAN,
        left,
        generator.generate(),
        body,
        generator.ast(),
    ))
}

fn generate_for_of_statement<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Statement<'a> {
    let body = generate_loop_body(generator);
    let left = ForStatementLeft::AssignmentTargetIdentifier(IdentifierReference::boxed(
        SPAN,
        generator.random_ident(),
        generator.ast(),
    ));
    Statement::ForOfStatement(ForOfStatement::boxed(
        SPAN,
        false,
        left,
        generator.generate(),
        body,
        generator.ast(),
    ))
}

fn generate_try_statement<'a, R: Rng + ?Sized>(
    generator: &AstGenerator<'a, '_, R>,
) -> Statement<'a> {
    let block = BlockStatement::boxed(SPAN, ArenaVec::new_in(generator.ast()), generator.ast());
    let catch_body =
        BlockStatement::boxed(SPAN, ArenaVec::new_in(generator.ast()), generator.ast());
    let handler = CatchClause::boxed(SPAN, None, catch_body, generator.ast());
    Statement::TryStatement(TryStatement::boxed(SPAN, block, Some(handler), None, generator.ast()))
}

fn generate_if_statement<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Statement<'a> {
    let consequent = generator.generate();
    let alternate = generator.random_option();
    let test = generator.generate();
    Statement::IfStatement(IfStatement::boxed(SPAN, test, consequent, alternate, generator.ast()))
}

fn generate_while_statement<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Statement<'a> {
    let body = generate_loop_body(generator);
    let test = generator.generate();
    Statement::WhileStatement(WhileStatement::boxed(SPAN, test, body, generator.ast()))
}

fn generate_do_while_statement<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Statement<'a> {
    let body = generate_loop_body(generator);
    let test = generator.generate();
    Statement::DoWhileStatement(DoWhileStatement::boxed(SPAN, body, test, generator.ast()))
}

fn generate_with_statement<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Statement<'a> {
    let body = generator.generate();
    let object = generator.generate();
    Statement::WithStatement(WithStatement::boxed(SPAN, object, body, generator.ast()))
}

pub(crate) fn generate_statement<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Statement<'a> {
    if !generator.reserve_stmt() || generator.at_expr_limit() {
        return Statement::EmptyStatement(EmptyStatement::boxed(SPAN, generator.ast()));
    }
    let context = generator.context();
    if !generator.at_limit() {
        if context.in_function && generator.random_index(10) == 0 {
            return Statement::ReturnStatement(generator.random_box());
        }
        if (context.in_loop || context.in_switch) && generator.random_index(10) == 0 {
            return Statement::BreakStatement(BreakStatement::boxed(SPAN, None, generator.ast()));
        }
        if context.in_loop && generator.random_index(10) == 0 {
            return Statement::ContinueStatement(ContinueStatement::boxed(
                SPAN,
                None,
                generator.ast(),
            ));
        }
    }

    let choices = if generator.at_limit() { 2 } else { 15 };
    match generator.random_index(choices) {
        1 => Statement::ExpressionStatement(generator.random_box()),
        2 => Statement::BlockStatement(generator.random_box()),
        3 => generate_if_statement(generator),
        4 => generate_while_statement(generator),
        5 => Statement::ThrowStatement(generator.random_box()),
        6 => Statement::DebuggerStatement(generator.random_box()),
        7 => Statement::SwitchStatement(generator.random_box()),
        8 => generate_do_while_statement(generator),
        9 => generate_for_statement(generator),
        10 => generate_for_in_statement(generator),
        11 => generate_for_of_statement(generator),
        12 => Statement::LabeledStatement(generator.random_box()),
        13 => generate_try_statement(generator),
        14 if generator.source_type().is_script() && !generator.is_typescript() => {
            generate_with_statement(generator)
        }
        0 | 14 => Statement::EmptyStatement(generator.random_box()),
        _ => unreachable!(),
    }
}

pub(crate) fn generate_program_body<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> ArenaVec<'a, Statement<'a>> {
    if generator.is_typescript()
        && generator.source_type().is_module()
        && generator.random_index(16) == 0
    {
        let mut body = ArenaVec::with_capacity_in(1, generator.ast());
        assert!(generator.reserve_stmt());
        body.push(
            ModuleDeclaration::TSExportAssignment(TSExportAssignment::boxed(
                SPAN,
                Expression::new_identifier(SPAN, generator.random_ident(), generator.ast()),
                generator.ast(),
            ))
            .into(),
        );
        return body;
    }
    let len = generator.random_index(generator.remaining_stmts() + 1);
    let mut body = ArenaVec::with_capacity_in(len, generator.ast());
    let mut has_default_export = false;
    for _ in 0..len {
        if generator.at_stmt_limit() {
            break;
        }
        let choice_count = if generator.source_type().is_module() { 3 } else { 2 };
        match generator.random_index(choice_count) {
            0 => body.push(generator.generate()),
            1 => {
                assert!(generator.reserve_stmt());
                body.push(generator.generate::<Declaration<'a>>().into());
            }
            2 => {
                assert!(generator.reserve_stmt());
                let (declaration, is_default_export) =
                    generate_module_declaration(generator, !has_default_export);
                has_default_export |= is_default_export;
                body.push(declaration.into());
            }
            _ => unreachable!(),
        }
    }
    body
}

pub(crate) fn generate_loop_body<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Statement<'a> {
    generator.with_context(|context| context.in_loop = true, AstGenerator::generate)
}

pub(crate) fn generate_switch_consequent<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> ArenaVec<'a, Statement<'a>> {
    generator.with_context(
        |context| context.in_switch = true,
        |generator| generate_statement_list(generator, generator.remaining_stmts()),
    )
}

/// Generate statements contained by a braced block.
pub(crate) fn generate_block_statements<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> ArenaVec<'a, Statement<'a>> {
    let max = generator.max_stmts_per_block().min(generator.remaining_stmts());
    generate_statement_list(generator, max)
}

fn generate_statement_list<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
    max: usize,
) -> ArenaVec<'a, Statement<'a>> {
    let len = generator.random_index(max + 1);
    let mut statements = ArenaVec::with_capacity_in(len, generator.ast());
    for _ in 0..len {
        if generator.at_stmt_limit() {
            break;
        }
        statements.push(generator.generate());
    }
    statements
}

pub(crate) fn generate_switch_cases<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> ArenaVec<'a, SwitchCase<'a>> {
    let mut cases = ArenaVec::new_in(generator.ast());
    let mut has_default = false;
    while !generator.at_limit() && generator.random_bool() {
        let is_default = !has_default && generator.random_bool();
        has_default |= is_default;
        let test = if is_default { None } else { Some(generator.generate()) };
        let consequent = generate_switch_consequent(generator);
        cases.push(SwitchCase::new(SPAN, test, consequent, generator.ast()));
    }
    cases
}

pub(crate) fn generate_template_literal<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TemplateLiteral<'a> {
    let expression_count = if generator.at_limit() { 0 } else { generator.random_index(4) };
    let mut expressions = ArenaVec::with_capacity_in(expression_count, generator.ast());
    let mut quasis = ArenaVec::with_capacity_in(expression_count + 1, generator.ast());

    for index in 0..=expression_count {
        let value = TemplateElementValue {
            raw: generator.random_str(),
            cooked: Some(generator.random_str()),
        };
        quasis.push(TemplateElement::new(SPAN, value, index == expression_count, generator.ast()));
        if index < expression_count {
            expressions.push(generator.generate());
        }
    }

    TemplateLiteral::new(SPAN, quasis, expressions, generator.ast())
}

pub(crate) fn generate_function<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Function<'a> {
    generate_function_with_type(generator, FunctionType::FunctionExpression, None)
}

fn generate_function_with_type<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
    function_type: FunctionType,
    id: Option<BindingIdentifier<'a>>,
) -> Function<'a> {
    let is_async = generator.random_bool();
    let is_generator = generator.random_bool();
    let params = generate_parameters(generator, FormalParameterKind::FormalParameter);
    let body = generator.with_context(
        |context| {
            context.enter_function(is_async, is_generator);
        },
        AstGenerator::generate::<FunctionBody<'a>>,
    );

    Function::new(
        SPAN,
        function_type,
        id,
        is_generator,
        is_async,
        false,
        None,
        None,
        params,
        None,
        Some(ArenaBox::new_in(body, generator.ast())),
        generator.ast(),
    )
}

pub(crate) fn generate_declaration<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> Declaration<'a> {
    let choices = if generator.is_typescript() { 10 } else { 3 };
    match generator.random_index(choices) {
        0 => Declaration::VariableDeclaration(ArenaBox::new_in(
            generate_variable_declaration(generator),
            generator.ast(),
        )),
        1 => {
            let id = BindingIdentifier::new(SPAN, generator.random_ident(), generator.ast());
            let function =
                generate_function_with_type(generator, FunctionType::FunctionDeclaration, Some(id));
            Declaration::FunctionDeclaration(ArenaBox::new_in(function, generator.ast()))
        }
        2 => Declaration::ClassDeclaration(ArenaBox::new_in(
            generate_class_with_type(generator, ClassType::ClassDeclaration),
            generator.ast(),
        )),
        4 => Declaration::TSInterfaceDeclaration(ArenaBox::new_in(
            generate_ts_interface_declaration(generator),
            generator.ast(),
        )),
        5 => Declaration::TSEnumDeclaration(ArenaBox::new_in(
            generate_ts_enum_declaration(generator),
            generator.ast(),
        )),
        6 => Declaration::TSNamespaceDeclaration(ArenaBox::new_in(
            generate_ts_namespace_declaration(generator),
            generator.ast(),
        )),
        7 if generator.source_type().is_module() => Declaration::TSGlobalDeclaration(
            ArenaBox::new_in(generate_ts_global_declaration(generator), generator.ast()),
        ),
        3 | 7 => Declaration::TSTypeAliasDeclaration(generator.random_box()),
        8 => Declaration::TSImportEqualsDeclaration(ArenaBox::new_in(
            generate_ts_import_equals_declaration(generator),
            generator.ast(),
        )),
        9 => Declaration::TSExternalModuleDeclaration(ArenaBox::new_in(
            generate_ts_external_module_declaration(generator),
            generator.ast(),
        )),
        _ => unreachable!(),
    }
}

fn generate_ts_interface_declaration<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSInterfaceDeclaration<'a> {
    let id = BindingIdentifier::new(SPAN, generator.random_ident(), generator.ast());
    let body = TSInterfaceBody::boxed(SPAN, ArenaVec::new_in(generator.ast()), generator.ast());
    TSInterfaceDeclaration::new(
        SPAN,
        id,
        None,
        ArenaVec::new_in(generator.ast()),
        body,
        false,
        generator.ast(),
    )
}

fn generate_ts_enum_declaration<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSEnumDeclaration<'a> {
    let id = BindingIdentifier::new(SPAN, generator.random_ident(), generator.ast());
    let body = TSEnumBody::new(SPAN, ArenaVec::new_in(generator.ast()), generator.ast());
    TSEnumDeclaration::new(SPAN, id, body, false, false, generator.ast())
}

fn empty_ts_module_block<'a, R: Rng + ?Sized>(
    generator: &AstGenerator<'a, '_, R>,
) -> TSModuleBlock<'a> {
    TSModuleBlock::new(
        SPAN,
        ArenaVec::new_in(generator.ast()),
        ArenaVec::new_in(generator.ast()),
        generator.ast(),
    )
}

fn generate_ts_namespace_declaration<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSNamespaceDeclaration<'a> {
    let id = BindingIdentifier::new(SPAN, generator.random_ident(), generator.ast());
    let body = TSNamespaceDeclarationBody::TSModuleBlock(ArenaBox::new_in(
        empty_ts_module_block(generator),
        generator.ast(),
    ));
    TSNamespaceDeclaration::new(
        SPAN,
        id,
        body,
        TSNamespaceDeclarationKind::Namespace,
        false,
        generator.ast(),
    )
}

fn generate_ts_external_module_declaration<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSExternalModuleDeclaration<'a> {
    let id = StringLiteral::new(SPAN, generator.random_str(), None, generator.ast());
    let body = ArenaBox::new_in(empty_ts_module_block(generator), generator.ast());
    TSExternalModuleDeclaration::new(SPAN, id, Some(body), false, generator.ast())
}

fn generate_ts_global_declaration<'a, R: Rng + ?Sized>(
    generator: &AstGenerator<'a, '_, R>,
) -> TSGlobalDeclaration<'a> {
    TSGlobalDeclaration::new(SPAN, SPAN, empty_ts_module_block(generator), true, generator.ast())
}

fn generate_ts_import_equals_declaration<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSImportEqualsDeclaration<'a> {
    let id = BindingIdentifier::new(SPAN, generator.random_ident(), generator.ast());
    let source = StringLiteral::new(SPAN, generator.random_str(), None, generator.ast());
    let reference = TSModuleReference::ExternalModuleReference(TSExternalModuleReference::boxed(
        SPAN,
        source,
        generator.ast(),
    ));
    TSImportEqualsDeclaration::new(SPAN, id, reference, ImportOrExportKind::Value, generator.ast())
}

fn generate_variable_declaration<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> VariableDeclaration<'a> {
    let id = BindingPattern::BindingIdentifier(BindingIdentifier::boxed(
        SPAN,
        generator.random_ident(),
        generator.ast(),
    ));
    let declarator =
        VariableDeclarator::new(SPAN, id, None, Some(generator.generate()), false, generator.ast());
    let mut declarations = ArenaVec::with_capacity_in(1, generator.ast());
    declarations.push(declarator);
    VariableDeclaration::new(
        SPAN,
        VariableDeclarationKind::Let,
        declarations,
        false,
        generator.ast(),
    )
}

fn generate_module_declaration<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
    allow_default_export: bool,
) -> (ModuleDeclaration<'a>, bool) {
    let base_choices = if allow_default_export { 6 } else { 5 };
    let choices = base_choices + usize::from(generator.is_typescript());
    let choice = generator.random_index(choices);
    let choice = if allow_default_export {
        choice
    } else {
        match choice {
            0 | 1 => choice,
            2 => 3,
            3 => 4,
            4 => 5,
            5 => 6,
            _ => unreachable!(),
        }
    };
    let declaration = match choice {
        0 => ModuleDeclaration::ImportDeclaration(ArenaBox::new_in(
            ImportDeclaration::new(
                SPAN,
                None,
                StringLiteral::new(SPAN, generator.random_str(), None, generator.ast()),
                None,
                None,
                ImportOrExportKind::Value,
                generator.ast(),
            ),
            generator.ast(),
        )),
        1 => ModuleDeclaration::ExportAllDeclaration(ArenaBox::new_in(
            ExportAllDeclaration::new(
                SPAN,
                None,
                StringLiteral::new(SPAN, generator.random_str(), None, generator.ast()),
                None,
                ImportOrExportKind::Value,
                generator.ast(),
            ),
            generator.ast(),
        )),
        2 => ModuleDeclaration::ExportDefaultDeclaration(ArenaBox::new_in(
            ExportDefaultDeclaration::new(
                SPAN,
                generator.generate::<Expression<'a>>().into(),
                generator.ast(),
            ),
            generator.ast(),
        )),
        3 => ModuleDeclaration::ExportNamedDeclaration(ArenaBox::new_in(
            ExportNamedDeclaration::new(
                SPAN,
                ArenaVec::new_in(generator.ast()),
                ImportOrExportKind::Value,
                generator.ast(),
            ),
            generator.ast(),
        )),
        4 => ModuleDeclaration::ExportDeclaration(ArenaBox::new_in(
            ExportDeclaration::new(
                SPAN,
                Declaration::VariableDeclaration(ArenaBox::new_in(
                    generate_variable_declaration(generator),
                    generator.ast(),
                )),
                generator.ast(),
            ),
            generator.ast(),
        )),
        5 => ModuleDeclaration::ExportFromDeclaration(ArenaBox::new_in(
            ExportFromDeclaration::new(
                SPAN,
                ArenaVec::new_in(generator.ast()),
                StringLiteral::new(SPAN, generator.random_str(), None, generator.ast()),
                ImportOrExportKind::Value,
                None,
                generator.ast(),
            ),
            generator.ast(),
        )),
        6 => ModuleDeclaration::TSNamespaceExportDeclaration(ArenaBox::new_in(
            TSNamespaceExportDeclaration::new(
                SPAN,
                IdentifierName::new(SPAN, generator.random_ident(), generator.ast()),
                generator.ast(),
            ),
            generator.ast(),
        )),
        _ => unreachable!(),
    };
    (declaration, choice == 2)
}

pub(crate) fn generate_function_body<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> FunctionBody<'a> {
    let statements =
        generator.with_context(|context| context.in_function = true, generate_block_statements);
    FunctionBody::new(
        SPAN,
        ArenaVec::<Directive<'a>>::new_in(generator.ast()),
        statements,
        generator.ast(),
    )
}

pub(crate) fn generate_arrow_function<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> ArrowFunctionExpression<'a> {
    let is_async = generator.random_bool();
    let params = generate_parameters(generator, FormalParameterKind::ArrowFormalParameters);
    let body = generator.with_context(
        |context| {
            context.enter_arrow_function(is_async);
        },
        AstGenerator::generate::<FunctionBody<'a>>,
    );
    ArrowFunctionExpression::new(
        SPAN,
        is_async,
        None,
        params,
        None,
        ArrowFunctionBody::FunctionBody(ArenaBox::new_in(body, generator.ast())),
        generator.ast(),
    )
}

pub(crate) fn generate_yield_expression<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> YieldExpression<'a> {
    let delegate = generator.random_bool();
    let argument = if delegate { Some(generator.generate()) } else { generator.random_option() };
    YieldExpression::new(SPAN, delegate, argument, generator.ast())
}

pub(crate) fn generate_object_property<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> ObjectProperty<'a> {
    let key = PropertyKey::StaticIdentifier(IdentifierName::boxed(
        SPAN,
        generator.random_ident(),
        generator.ast(),
    ));
    let value = generator.generate();
    ObjectProperty::new(SPAN, PropertyKind::Init, key, value, false, false, false, generator.ast())
}

pub(crate) fn generate_function_type<R: Rng + ?Sized>(
    generator: &mut AstGenerator<'_, '_, R>,
) -> FunctionType {
    match generator.random_index(if generator.is_typescript() { 4 } else { 2 }) {
        0 => FunctionType::FunctionDeclaration,
        1 => FunctionType::FunctionExpression,
        2 => FunctionType::TSDeclareFunction,
        3 => FunctionType::TSEmptyBodyFunctionExpression,
        _ => unreachable!(),
    }
}

pub(crate) fn generate_formal_parameter_kind<R: Rng + ?Sized>(
    generator: &mut AstGenerator<'_, '_, R>,
) -> FormalParameterKind {
    match generator.random_index(if generator.is_typescript() { 4 } else { 3 }) {
        0 => FormalParameterKind::FormalParameter,
        1 => FormalParameterKind::UniqueFormalParameters,
        2 => FormalParameterKind::ArrowFormalParameters,
        3 => FormalParameterKind::Signature,
        _ => unreachable!(),
    }
}

pub(crate) fn generate_method_definition_type<R: Rng + ?Sized>(
    generator: &mut AstGenerator<'_, '_, R>,
) -> MethodDefinitionType {
    if generator.is_typescript() && generator.random_bool() {
        MethodDefinitionType::TSAbstractMethodDefinition
    } else {
        MethodDefinitionType::MethodDefinition
    }
}

pub(crate) fn generate_property_definition_type<R: Rng + ?Sized>(
    generator: &mut AstGenerator<'_, '_, R>,
) -> PropertyDefinitionType {
    if generator.is_typescript() && generator.random_bool() {
        PropertyDefinitionType::TSAbstractPropertyDefinition
    } else {
        PropertyDefinitionType::PropertyDefinition
    }
}

pub(crate) fn generate_accessor_property_type<R: Rng + ?Sized>(
    generator: &mut AstGenerator<'_, '_, R>,
) -> AccessorPropertyType {
    if generator.is_typescript() && generator.random_bool() {
        AccessorPropertyType::TSAbstractAccessorProperty
    } else {
        AccessorPropertyType::AccessorProperty
    }
}

pub(crate) fn generate_import_or_export_kind<R: Rng + ?Sized>(
    generator: &mut AstGenerator<'_, '_, R>,
) -> ImportOrExportKind {
    if generator.is_typescript() && generator.random_bool() {
        ImportOrExportKind::Type
    } else {
        ImportOrExportKind::Value
    }
}

pub(crate) fn generate_ts_type_parameter_instantiation<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSTypeParameterInstantiation<'a> {
    let mut params = ArenaVec::with_capacity_in(1, generator.ast());
    params.push(generator.generate());
    TSTypeParameterInstantiation::new(SPAN, params, generator.ast())
}

pub(crate) fn generate_ts_type<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSType<'a> {
    let choices = if generator.at_limit() { 14 } else { 28 };
    match generator.random_index(choices) {
        0 => TSType::TSAnyKeyword(generator.random_box()),
        1 => TSType::TSBigIntKeyword(generator.random_box()),
        2 => TSType::TSBooleanKeyword(generator.random_box()),
        3 | 13 => TSType::TSThisType(generator.random_box()),
        4 => TSType::TSNeverKeyword(generator.random_box()),
        5 => TSType::TSNullKeyword(generator.random_box()),
        6 => TSType::TSNumberKeyword(generator.random_box()),
        7 => TSType::TSObjectKeyword(generator.random_box()),
        8 => TSType::TSStringKeyword(generator.random_box()),
        9 => TSType::TSSymbolKeyword(generator.random_box()),
        10 => TSType::TSUndefinedKeyword(generator.random_box()),
        11 => TSType::TSUnknownKeyword(generator.random_box()),
        12 => TSType::TSVoidKeyword(generator.random_box()),
        14 => TSType::TSArrayType(generator.random_box()),
        15 => TSType::TSParenthesizedType(generator.random_box()),
        16 => TSType::TSConditionalType(generator.random_box()),
        17 => TSType::TSIndexedAccessType(generator.random_box()),
        18 => generate_ts_union_type(generator),
        19 => generate_ts_intersection_type(generator),
        20 => generate_ts_literal_type(generator),
        21 => generate_ts_tuple_type(generator),
        22 => TSType::TSTypeLiteral(TSTypeLiteral::boxed(
            SPAN,
            ArenaVec::new_in(generator.ast()),
            generator.ast(),
        )),
        23 => TSType::TSTypeOperatorType(TSTypeOperator::boxed(
            SPAN,
            TSTypeOperatorOperator::Keyof,
            generator.generate(),
            generator.ast(),
        )),
        24 => generate_ts_type_reference(generator),
        25 => generate_ts_type_query(generator),
        26 => generate_ts_import_type(generator),
        27 => TSType::TSMappedType(generator.random_box()),
        _ => unreachable!(),
    }
}

fn generate_ts_union_type<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSType<'a> {
    let mut types = ArenaVec::with_capacity_in(2, generator.ast());
    types.push(generator.generate());
    types.push(generator.generate());
    TSType::TSUnionType(TSUnionType::boxed(SPAN, types, generator.ast()))
}

fn generate_ts_intersection_type<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSType<'a> {
    let mut types = ArenaVec::with_capacity_in(2, generator.ast());
    types.push(generator.generate());
    types.push(generator.generate());
    TSType::TSIntersectionType(TSIntersectionType::boxed(SPAN, types, generator.ast()))
}

fn generate_ts_literal_type<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSType<'a> {
    let literal = TSLiteral::StringLiteral(StringLiteral::boxed(
        SPAN,
        generator.random_str(),
        None,
        generator.ast(),
    ));
    TSType::TSLiteralType(TSLiteralType::boxed(SPAN, literal, generator.ast()))
}

fn generate_ts_tuple_type<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSType<'a> {
    let mut elements = ArenaVec::with_capacity_in(2, generator.ast());
    elements.push(generator.generate::<TSType<'a>>().into());
    elements.push(generator.generate::<TSType<'a>>().into());
    TSType::TSTupleType(TSTupleType::boxed(SPAN, elements, generator.ast()))
}

fn generate_ts_type_reference<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSType<'a> {
    let name = TSTypeName::IdentifierReference(IdentifierReference::boxed(
        SPAN,
        generator.random_ident(),
        generator.ast(),
    ));
    TSType::TSTypeReference(TSTypeReference::boxed(SPAN, name, None, generator.ast()))
}

fn generate_ts_import_type<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSType<'a> {
    let argument = StringLiteral::new(SPAN, generator.random_str(), None, generator.ast());
    TSType::TSImportType(TSImportType::boxed(SPAN, argument, None, None, None, generator.ast()))
}

fn generate_ts_type_query<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSType<'a> {
    let name = TSTypeQueryExprName::IdentifierReference(IdentifierReference::boxed(
        SPAN,
        generator.random_ident(),
        generator.ast(),
    ));
    TSType::TSTypeQuery(TSTypeQuery::boxed(SPAN, name, None, generator.ast()))
}

pub(crate) fn generate_ts_instantiation_expression<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSInstantiationExpression<'a> {
    let expression = Expression::new_identifier(SPAN, generator.random_ident(), generator.ast());
    let type_arguments = generator.random_box();
    TSInstantiationExpression::new(SPAN, expression, type_arguments, generator.ast())
}

pub(crate) fn generate_ts_type_parameter_declaration<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSTypeParameterDeclaration<'a> {
    let mut params = ArenaVec::with_capacity_in(1, generator.ast());
    params.push(generator.generate());
    TSTypeParameterDeclaration::new(SPAN, params, generator.ast())
}

pub(crate) fn generate_ts_type_parameter<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
) -> TSTypeParameter<'a> {
    let name = BindingIdentifier::new(SPAN, generator.random_ident(), generator.ast());
    TSTypeParameter::new(SPAN, name, None, None, false, false, false, generator.ast())
}

fn generate_parameters<'a, R: Rng + ?Sized>(
    generator: &mut AstGenerator<'a, '_, R>,
    kind: FormalParameterKind,
) -> ArenaBox<'a, FormalParameters<'a>> {
    let mut items = ArenaVec::new_in(generator.ast());
    if !generator.at_limit() && generator.random_bool() {
        let pattern = BindingPattern::BindingIdentifier(BindingIdentifier::boxed(
            SPAN,
            generator.random_ident(),
            generator.ast(),
        ));
        let type_annotation = generator.is_typescript().then(|| {
            ArenaBox::new_in(
                TSTypeAnnotation::new(SPAN, generator.generate(), generator.ast()),
                generator.ast(),
            )
        });
        items.push(FormalParameter::new(
            SPAN,
            ArenaVec::new_in(generator.ast()),
            pattern,
            type_annotation,
            None,
            false,
            None,
            false,
            false,
            generator.ast(),
        ));
    }
    FormalParameters::boxed(SPAN, kind, items, None, generator.ast())
}
