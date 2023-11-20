use ezno_checker::{
    self,
    behavior::{
        assignments::{
            Assignable, AssignmentKind, AssignmentReturnStatus, IncrementOrDecrement, Reference,
        },
        operations::{evaluate_pure_binary_operation_handle_errors, PureBinaryOperation},
    },
    context::facts::Publicity,
    types::{calling::CalledWithNew, SynthesisedArgument},
    CheckingData, Environment, Instance, PropertyValue, ReadFromFS, TypeId,
};
use oxc_ast::ast;
use oxc_span::GetSpan;
use oxc_syntax::operator::AssignmentOperator;

use super::property_key_to_type;
use crate::{
    functions::{OxcArrowFunction, OxcFunction},
    oxc_span_to_source_map_span,
    types::synthesise_type_annotation,
    OxcAST,
};

pub(crate) fn synthesise_expression<T: ReadFromFS>(
    expr: &ast::Expression,
    expected: TypeId,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T, OxcAST>,
) -> TypeId {
    let instance = match expr {
        ast::Expression::BooleanLiteral(boolean) => {
            return checking_data
                .types
                .new_constant_type(ezno_checker::Constant::Boolean(boolean.value));
        }
        ast::Expression::NullLiteral(_) => return TypeId::NULL_TYPE,
        ast::Expression::BigintLiteral(big_int) => {
            checking_data.raise_unimplemented_error(
                "big int literal",
                oxc_span_to_source_map_span(big_int.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::RegExpLiteral(regexp) => {
            return checking_data.types.new_constant_type(ezno_checker::Constant::Regexp(
                regexp.regex.pattern.to_string(),
            ));
        }
        ast::Expression::NumberLiteral(number) => {
            return checking_data.types.new_constant_type(
                number
                    .value
                    .try_into()
                    .map(ezno_checker::Constant::Number)
                    .unwrap_or(ezno_checker::Constant::NaN),
            );
        }
        ast::Expression::StringLiteral(string) => {
            return checking_data.types.new_constant_type(ezno_checker::Constant::String(
                string.value.as_str().to_owned(),
            ));
        }
        ast::Expression::TemplateLiteral(tl) => {
            checking_data.raise_unimplemented_error(
                "template literals",
                oxc_span_to_source_map_span(tl.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::Identifier(identifier) => {
            let result = environment.get_variable_or_error(
                &identifier.name,
                &oxc_span_to_source_map_span(identifier.span),
                checking_data,
            );

            match result {
                Ok(ok) => Instance::LValue(ok),
                Err(err) => return err,
            }
        }
        ast::Expression::MetaProperty(meta_prop) => {
            checking_data.raise_unimplemented_error(
                "meta_prop",
                oxc_span_to_source_map_span(meta_prop.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::Super(super_item) => {
            checking_data
                .raise_unimplemented_error("super", oxc_span_to_source_map_span(super_item.span));
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::ArrayExpression(array_expr) => {
            checking_data.raise_unimplemented_error(
                "array expression",
                oxc_span_to_source_map_span(array_expr.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::AssignmentExpression(assignment) => {
            Instance::RValue(synthesise_assignment(assignment, environment, checking_data))
        }
        ast::Expression::AwaitExpression(r#await) => {
            checking_data.raise_unimplemented_error(
                "await expression",
                oxc_span_to_source_map_span(r#await.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::BinaryExpression(bin_expr) => {
            let value = synthesise_binary_expression(
                &bin_expr.left,
                bin_expr.operator,
                &bin_expr.right,
                checking_data,
                environment,
            );
            Instance::RValue(value)
        }
        ast::Expression::CallExpression(expr) => {
            let parent =
                synthesise_expression(&expr.callee, TypeId::ANY_TYPE, environment, checking_data);

            let type_arguments = expr.type_parameters.as_ref().map(|tp| {
                tp.params
                    .iter()
                    .map(|ta| {
                        (
                            oxc_span_to_source_map_span(ta.span()),
                            synthesise_type_annotation(ta, environment, checking_data),
                        )
                    })
                    .collect()
            });

            // TODO this will be abstracted in the future
            let arguments: Vec<SynthesisedArgument> = expr
                .arguments
                .iter()
                .map(|arg| match arg {
                    ast::Argument::SpreadElement(expr) => {
                        checking_data.raise_unimplemented_error(
                            "spread argument",
                            oxc_span_to_source_map_span(expr.span),
                        );

                        SynthesisedArgument::NonSpread {
                            ty: TypeId::ERROR_TYPE,
                            position: oxc_span_to_source_map_span(expr.span),
                        }
                    }
                    ast::Argument::Expression(expr) => {
                        let ty = synthesise_expression(
                            expr,
                            TypeId::ANY_TYPE,
                            environment,
                            checking_data,
                        );
                        SynthesisedArgument::NonSpread {
                            ty,
                            position: oxc_span_to_source_map_span(GetSpan::span(expr)),
                        }
                    }
                })
                .collect();

            // TODO
            let this_argument = None;

            let result = ezno_checker::call_type_handle_errors(
                parent,
                arguments,
                this_argument,
                type_arguments,
                environment,
                checking_data,
                CalledWithNew::None,
                oxc_span_to_source_map_span(expr.span),
            );
            Instance::RValue(result)
        }
        ast::Expression::ChainExpression(item) => {
            checking_data.raise_unimplemented_error(
                "chain expression",
                oxc_span_to_source_map_span(item.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::ClassExpression(item) => {
            checking_data.raise_unimplemented_error(
                "class expression",
                oxc_span_to_source_map_span(item.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::ConditionalExpression(item) => {
            checking_data.raise_unimplemented_error(
                "conditional expression",
                oxc_span_to_source_map_span(item.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::ArrowExpression(func) => {
            Instance::RValue(ezno_checker::behavior::functions::register_arrow_function(
                expected,
                func.r#async,
                &OxcFunction(&**func, None),
                environment,
                checking_data,
            ))
        }
        ast::Expression::FunctionExpression(func) => {
            Instance::RValue(ezno_checker::behavior::functions::register_expression_function(
                expected,
                func.r#async,
                func.generator,
                None,
                &OxcFunction(&**func, None),
                environment,
                checking_data,
            ))
        }
        ast::Expression::ImportExpression(item) => {
            checking_data.raise_unimplemented_error(
                "import expression",
                oxc_span_to_source_map_span(item.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::LogicalExpression(item) => {
            checking_data.raise_unimplemented_error(
                "logical expression",
                oxc_span_to_source_map_span(item.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::MemberExpression(expr) => match &**expr {
            ast::MemberExpression::ComputedMemberExpression(comp) => {
                // TODO
                let expected = TypeId::ANY_TYPE;
                let parent =
                    synthesise_expression(&comp.object, expected, environment, checking_data);
                let property = synthesise_expression(
                    &comp.expression,
                    TypeId::ANY_TYPE,
                    environment,
                    checking_data,
                );
                Instance::RValue(environment.get_property_handle_errors(
                    parent,
                    property,
                    ezno_checker::context::facts::Publicity::Public,
                    checking_data,
                    oxc_span_to_source_map_span(comp.span),
                ))
            }
            ast::MemberExpression::StaticMemberExpression(expr) => {
                let parent = synthesise_expression(
                    &expr.object,
                    TypeId::ANY_TYPE,
                    environment,
                    checking_data,
                );
                let property = checking_data.types.new_constant_type(
                    ezno_checker::Constant::String(expr.property.name.as_str().to_owned()),
                );

                Instance::RValue(environment.get_property_handle_errors(
                    parent,
                    property,
                    ezno_checker::context::facts::Publicity::Public,
                    checking_data,
                    oxc_span_to_source_map_span(expr.span),
                ))
            }
            ast::MemberExpression::PrivateFieldExpression(item) => {
                checking_data.raise_unimplemented_error(
                    "private field expression",
                    oxc_span_to_source_map_span(item.span),
                );
                return TypeId::ERROR_TYPE;
            }
        },
        ast::Expression::NewExpression(item) => {
            checking_data.raise_unimplemented_error(
                "new expression",
                oxc_span_to_source_map_span(item.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::ObjectExpression(object) => {
            Instance::RValue(synthesise_object(object, environment, checking_data))
        }
        ast::Expression::ParenthesizedExpression(inner) => {
            return synthesise_expression(&inner.expression, expected, environment, checking_data);
        }
        ast::Expression::SequenceExpression(item) => {
            checking_data.raise_unimplemented_error(
                "sequence expression",
                oxc_span_to_source_map_span(item.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::TaggedTemplateExpression(item) => {
            checking_data.raise_unimplemented_error(
                "tagged template literal expression",
                oxc_span_to_source_map_span(item.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::ThisExpression(this) => {
            checking_data.raise_unimplemented_error(
                "this expression",
                oxc_span_to_source_map_span(this.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::UnaryExpression(unary) => {
            checking_data.raise_unimplemented_error(
                "unary expression",
                oxc_span_to_source_map_span(unary.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::UpdateExpression(update_expr) => {
            let target = synthesise_simple_assignment_target_to_reference(
                &update_expr.argument,
                environment,
                checking_data,
            );
            let target = match target {
                crate::PartiallyImplemented::Ok(target) => target,
                crate::PartiallyImplemented::NotImplemented(item, span) => {
                    checking_data.raise_unimplemented_error(item, span);
                    return TypeId::ERROR_TYPE;
                }
            };
            // TODO need to cast as number...
            let result = environment.assign_to_assignable_handle_errors::<_, OxcAST>(
                Assignable::Reference(target),
                AssignmentKind::IncrementOrDecrement(
                    match update_expr.operator {
                        oxc_syntax::operator::UpdateOperator::Increment => {
                            IncrementOrDecrement::Increment
                        }
                        oxc_syntax::operator::UpdateOperator::Decrement => {
                            IncrementOrDecrement::Decrement
                        }
                    },
                    match update_expr.prefix {
                        true => AssignmentReturnStatus::New,
                        false => AssignmentReturnStatus::Previous,
                    },
                ),
                None,
                oxc_span_to_source_map_span(update_expr.span),
                checking_data,
            );
            Instance::RValue(result)
        }
        ast::Expression::YieldExpression(item) => {
            checking_data.raise_unimplemented_error(
                "yield expression",
                oxc_span_to_source_map_span(item.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::PrivateInExpression(item) => {
            checking_data.raise_unimplemented_error(
                "PrivateInExpression expression",
                oxc_span_to_source_map_span(item.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::JSXElement(item) => {
            checking_data.raise_unimplemented_error(
                "JSXElement expression",
                oxc_span_to_source_map_span(item.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::JSXFragment(item) => {
            checking_data.raise_unimplemented_error(
                "JSXFragment expression",
                oxc_span_to_source_map_span(item.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::TSAsExpression(item) => {
            checking_data.raise_unimplemented_error(
                "TSAsExpression expression",
                oxc_span_to_source_map_span(item.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::TSSatisfiesExpression(item) => {
            let item_ty =
                synthesise_expression(&item.expression, expected, environment, checking_data);
            let to_satisfy =
                synthesise_type_annotation(&item.type_annotation, environment, checking_data);
            checking_data.check_satisfies(
                item_ty,
                to_satisfy,
                oxc_span_to_source_map_span(item.span)
                    .with_source(environment.get_environment_id()),
                environment,
            );
            return item_ty;
        }
        ast::Expression::TSTypeAssertion(item) => {
            checking_data.raise_unimplemented_error(
                "TSTypeAssertion expression",
                oxc_span_to_source_map_span(item.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::TSNonNullExpression(item) => {
            checking_data.raise_unimplemented_error(
                "TSNonNullExpression expression",
                oxc_span_to_source_map_span(item.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::TSInstantiationExpression(item) => {
            checking_data.raise_unimplemented_error(
                "TSInstantiationExpression expression",
                oxc_span_to_source_map_span(item.span),
            );
            return TypeId::ERROR_TYPE;
        }
    };

    checking_data
        .add_expression_mapping(oxc_span_to_source_map_span(expr.span()), instance.clone());

    instance.get_value()
}

fn synthesise_assignment<T: ReadFromFS>(
    expr: &ast::AssignmentExpression,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T, OxcAST>,
) -> TypeId {
    let lhs = synthesise_assignment_target_to_assignable(&expr.left, environment, checking_data);
    let lhs = match lhs {
        crate::PartiallyImplemented::Ok(lhs) => lhs,
        crate::PartiallyImplemented::NotImplemented(item, span) => {
            checking_data.raise_unimplemented_error(item, span);
            return TypeId::ERROR_TYPE;
        }
    };
    use ezno_checker::behavior::operations::MathematicalAndBitwise;
    let operator: AssignmentKind = match expr.operator {
        AssignmentOperator::Assign => AssignmentKind::Assign,
        AssignmentOperator::Addition => AssignmentKind::PureUpdate(MathematicalAndBitwise::Add),
        AssignmentOperator::Subtraction => {
            AssignmentKind::PureUpdate(MathematicalAndBitwise::Subtract)
        }
        AssignmentOperator::Multiplication => {
            AssignmentKind::PureUpdate(MathematicalAndBitwise::Multiply)
        }
        AssignmentOperator::Division => AssignmentKind::PureUpdate(MathematicalAndBitwise::Divide),
        AssignmentOperator::Remainder => AssignmentKind::PureUpdate(MathematicalAndBitwise::Modulo),
        AssignmentOperator::ShiftLeft => {
            AssignmentKind::PureUpdate(MathematicalAndBitwise::BitwiseShiftLeft)
        }
        AssignmentOperator::ShiftRight => {
            AssignmentKind::PureUpdate(MathematicalAndBitwise::BitwiseShiftRight)
        }
        AssignmentOperator::ShiftRightZeroFill => {
            AssignmentKind::PureUpdate(MathematicalAndBitwise::BitwiseShiftRightUnsigned)
        }
        AssignmentOperator::BitwiseOR => {
            AssignmentKind::PureUpdate(MathematicalAndBitwise::BitwiseOr)
        }
        AssignmentOperator::BitwiseXOR => {
            AssignmentKind::PureUpdate(MathematicalAndBitwise::BitwiseXOr)
        }
        AssignmentOperator::BitwiseAnd => {
            AssignmentKind::PureUpdate(MathematicalAndBitwise::BitwiseAnd)
        }
        AssignmentOperator::Exponential => {
            AssignmentKind::PureUpdate(MathematicalAndBitwise::Exponent)
        }
        AssignmentOperator::LogicalAnd => {
            AssignmentKind::ConditionalUpdate(ezno_checker::behavior::operations::Logical::And)
        }
        AssignmentOperator::LogicalOr => {
            AssignmentKind::ConditionalUpdate(ezno_checker::behavior::operations::Logical::Or)
        }
        AssignmentOperator::LogicalNullish => AssignmentKind::ConditionalUpdate(
            ezno_checker::behavior::operations::Logical::NullCoalescing,
        ),
    };

    environment.assign_to_assignable_handle_errors(
        lhs,
        operator,
        Some(&expr.right),
        oxc_span_to_source_map_span(expr.span).with_source(environment.get_environment_id()),
        checking_data,
    )
}

// TODO others need to be built into helper methods in the checker
fn synthesise_assignment_target_to_assignable<T: ReadFromFS>(
    target: &ast::AssignmentTarget,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T, OxcAST>,
) -> crate::PartiallyImplemented<Assignable> {
    match target {
        ast::AssignmentTarget::SimpleAssignmentTarget(simple) => {
            match synthesise_simple_assignment_target_to_reference(
                simple,
                environment,
                checking_data,
            ) {
                crate::PartiallyImplemented::Ok(reference) => {
                    crate::PartiallyImplemented::Ok(Assignable::Reference(reference))
                }
                crate::PartiallyImplemented::NotImplemented(item, span) => {
                    crate::PartiallyImplemented::NotImplemented(item, span)
                }
            }
        }
        ast::AssignmentTarget::AssignmentTargetPattern(pattern) => match pattern {
            ast::AssignmentTargetPattern::ArrayAssignmentTarget(array) => {
                crate::PartiallyImplemented::NotImplemented(
                    "array assignment pattern",
                    oxc_span_to_source_map_span(array.span),
                )
            }
            ast::AssignmentTargetPattern::ObjectAssignmentTarget(object) => {
                crate::PartiallyImplemented::NotImplemented(
                    "object assignment pattern",
                    oxc_span_to_source_map_span(object.span),
                )
            }
        },
    }
}

fn synthesise_simple_assignment_target_to_reference<T: ReadFromFS>(
    simple: &ast::SimpleAssignmentTarget,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T, OxcAST>,
) -> crate::PartiallyImplemented<Reference> {
    match simple {
        ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            crate::PartiallyImplemented::Ok(Reference::Variable(
                identifier.name.as_str().to_owned(),
                oxc_span_to_source_map_span(identifier.span),
            ))
        }
        ast::SimpleAssignmentTarget::MemberAssignmentTarget(expr) => {
            let (parent_ty, key_ty, publicity) = match &**expr {
                ast::MemberExpression::ComputedMemberExpression(comp) => {
                    let property = synthesise_expression(
                        &comp.expression,
                        TypeId::ANY_TYPE,
                        environment,
                        checking_data,
                    );
                    let parent = synthesise_expression(
                        &comp.object,
                        TypeId::ANY_TYPE,
                        environment,
                        checking_data,
                    );
                    (parent, property)
                }
                ast::MemberExpression::StaticMemberExpression(expr) => {
                    let parent = synthesise_expression(
                        &expr.object,
                        TypeId::ANY_TYPE,
                        environment,
                        checking_data,
                    );
                    let property = checking_data.types.new_constant_type(
                        ezno_checker::Constant::String(expr.property.name.as_str().to_owned()),
                    );
                    (parent, property)
                }
                ast::MemberExpression::PrivateFieldExpression(item) => {
                    return crate::PartiallyImplemented::NotImplemented(
                        "private field expression",
                        oxc_span_to_source_map_span(item.span),
                    );
                }
            };
            crate::PartiallyImplemented::Ok(Reference::Property {
                on: parent_ty,
                with: key_ty,
                publicity,
                span: oxc_span_to_source_map_span(expr.span()),
            })
        }
        // TODO not sure if these exist...?
        ast::SimpleAssignmentTarget::TSAsExpression(_)
        | ast::SimpleAssignmentTarget::TSSatisfiesExpression(_)
        | ast::SimpleAssignmentTarget::TSNonNullExpression(_)
        | ast::SimpleAssignmentTarget::TSTypeAssertion(_) => {
            crate::PartiallyImplemented::NotImplemented(
                "left hand side typescript",
                oxc_span_to_source_map_span(GetSpan::span(simple)),
            )
        }
    }
}

/// TODO this logic needs to be moved to ezno-checker and
/// abstracted to use a builder pattern, which can be reused for array literals
pub(crate) fn synthesise_object<T: ReadFromFS>(
    object: &ast::ObjectExpression,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T, OxcAST>,
) -> TypeId {
    let ty = environment.new_object(&mut checking_data.types, None);
    for property in object.properties.iter() {
        match property {
            ast::ObjectPropertyKind::ObjectProperty(property) => {
                let key_ty = property_key_to_type(&property.key, environment, checking_data);
                let value = if let ast::Expression::FunctionExpression(func) = &property.value {
                    todo!()
                    // environment.new_function(
                    //     checking_data,
                    //     &OxcFunction(&**func, Some(property.kind)),
                    //     RegisterOnExistingObject,
                    // )
                } else {
                    PropertyValue::Value(synthesise_expression(
                        &property.value,
                        TypeId::ANY_TYPE,
                        environment,
                        checking_data,
                    ))
                };

                let position = oxc_span_to_source_map_span(property.span)
                    .with_source(environment.get_environment_id());

                environment.facts.register_property(
                    ty,
                    Publicity::Public,
                    key_ty,
                    value,
                    true,
                    Some(position),
                );
            }
            ast::ObjectPropertyKind::SpreadProperty(spread) => checking_data
                .raise_unimplemented_error(
                    "spread object property",
                    oxc_span_to_source_map_span(spread.span),
                ),
        }
    }
    ty
}

fn synthesise_binary_expression<T: ReadFromFS>(
    lhs: &ast::Expression,
    operator: oxc_syntax::operator::BinaryOperator,
    rhs: &ast::Expression,
    checking_data: &mut CheckingData<T, OxcAST>,
    environment: &mut Environment,
) -> TypeId {
    let lhs_ty = synthesise_expression(lhs, TypeId::ANY_TYPE, environment, checking_data);
    let rhs_ty = synthesise_expression(rhs, TypeId::ANY_TYPE, environment, checking_data);

    let op = match operator {
        oxc_syntax::operator::BinaryOperator::Equality => PureBinaryOperation::EqualityAndInequality(ezno_checker::behavior::operations::EqualityAndInequality::Equal),
        oxc_syntax::operator::BinaryOperator::Inequality => PureBinaryOperation::EqualityAndInequality(ezno_checker::behavior::operations::EqualityAndInequality::NotEqual),
        oxc_syntax::operator::BinaryOperator::StrictEquality => PureBinaryOperation::EqualityAndInequality(ezno_checker::behavior::operations::EqualityAndInequality::StrictEqual),
        oxc_syntax::operator::BinaryOperator::StrictInequality => PureBinaryOperation::EqualityAndInequality(ezno_checker::behavior::operations::EqualityAndInequality::StrictNotEqual),
        oxc_syntax::operator::BinaryOperator::LessThan => PureBinaryOperation::EqualityAndInequality(ezno_checker::behavior::operations::EqualityAndInequality::LessThan),
        oxc_syntax::operator::BinaryOperator::LessEqualThan => PureBinaryOperation::EqualityAndInequality(ezno_checker::behavior::operations::EqualityAndInequality::LessThanEqual),
        oxc_syntax::operator::BinaryOperator::GreaterThan => PureBinaryOperation::EqualityAndInequality(ezno_checker::behavior::operations::EqualityAndInequality::GreaterThan),
        oxc_syntax::operator::BinaryOperator::GreaterEqualThan => PureBinaryOperation::EqualityAndInequality(ezno_checker::behavior::operations::EqualityAndInequality::GreaterThanEqual),

        oxc_syntax::operator::BinaryOperator::ShiftLeft => PureBinaryOperation::MathematicalAndBitwise(ezno_checker::behavior::operations::MathematicalAndBitwise::BitwiseShiftLeft),
        oxc_syntax::operator::BinaryOperator::ShiftRight => PureBinaryOperation::MathematicalAndBitwise(ezno_checker::behavior::operations::MathematicalAndBitwise::BitwiseShiftRight),
        oxc_syntax::operator::BinaryOperator::ShiftRightZeroFill => PureBinaryOperation::MathematicalAndBitwise(ezno_checker::behavior::operations::MathematicalAndBitwise::BitwiseShiftRightUnsigned),
        oxc_syntax::operator::BinaryOperator::Addition => PureBinaryOperation::MathematicalAndBitwise(ezno_checker::behavior::operations::MathematicalAndBitwise::Add),
        oxc_syntax::operator::BinaryOperator::Subtraction => PureBinaryOperation::MathematicalAndBitwise(ezno_checker::behavior::operations::MathematicalAndBitwise::Subtract),
        oxc_syntax::operator::BinaryOperator::Multiplication => PureBinaryOperation::MathematicalAndBitwise(ezno_checker::behavior::operations::MathematicalAndBitwise::Multiply),
        oxc_syntax::operator::BinaryOperator::Division => PureBinaryOperation::MathematicalAndBitwise(ezno_checker::behavior::operations::MathematicalAndBitwise::Divide),
        oxc_syntax::operator::BinaryOperator::Remainder => PureBinaryOperation::MathematicalAndBitwise(ezno_checker::behavior::operations::MathematicalAndBitwise::Modulo),
        oxc_syntax::operator::BinaryOperator::BitwiseOR => PureBinaryOperation::MathematicalAndBitwise(ezno_checker::behavior::operations::MathematicalAndBitwise::BitwiseOr),
        oxc_syntax::operator::BinaryOperator::BitwiseXOR => PureBinaryOperation::MathematicalAndBitwise(ezno_checker::behavior::operations::MathematicalAndBitwise::BitwiseXOr),
        oxc_syntax::operator::BinaryOperator::BitwiseAnd => PureBinaryOperation::MathematicalAndBitwise(ezno_checker::behavior::operations::MathematicalAndBitwise::BitwiseAnd),
        oxc_syntax::operator::BinaryOperator::Exponential => PureBinaryOperation::MathematicalAndBitwise(ezno_checker::behavior::operations::MathematicalAndBitwise::Exponent),
        oxc_syntax::operator::BinaryOperator::In => todo!("use different environment methods"),
        oxc_syntax::operator::BinaryOperator::Instanceof => todo!("use different environment methods"),
    };
    evaluate_pure_binary_operation_handle_errors(
        op,
        (lhs_ty, oxc_span_to_source_map_span(GetSpan::span(lhs))),
        (rhs_ty, oxc_span_to_source_map_span(GetSpan::span(rhs))),
        environment,
        checking_data,
    )
}
