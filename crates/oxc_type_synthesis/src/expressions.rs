use ezno_checker::{
    self, structures::functions::SynthesizedArgument, Assignable, CheckingData, Environment,
    FSResolver, Instance, Property, Reference, RegisterAsType, RegisterOnExistingObject, TypeId,
};
use oxc_ast::ast;
use oxc_span::GetSpan;
use oxc_syntax::operator::AssignmentOperator;

use super::property_key_to_type;
use crate::{
    functions::{OxcArrowFunction, OxcFunction},
    oxc_span_to_source_map_span,
    types::synthesize_type_annotation,
};

pub(crate) fn synthesize_expression<T: FSResolver>(
    expr: &ast::Expression,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T>,
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
            Instance::RValue(synthesize_assignment(assignment, environment, checking_data))
        }
        ast::Expression::AwaitExpression(r#await) => {
            checking_data.raise_unimplemented_error(
                "await expression",
                oxc_span_to_source_map_span(r#await.span),
            );
            return TypeId::ERROR_TYPE;
        }
        ast::Expression::BinaryExpression(bin_expr) => {
            let value = synthesize_binary_expression(
                &bin_expr.left,
                bin_expr.operator,
                &bin_expr.right,
                checking_data,
                environment,
            );
            Instance::RValue(value)
        }
        ast::Expression::CallExpression(expr) => {
            let parent = synthesize_expression(&expr.callee, environment, checking_data);

            let type_arguments = expr.type_parameters.as_ref().map(|tp| {
                tp.params
                    .iter()
                    .map(|ta| {
                        (
                            oxc_span_to_source_map_span(ta.span()),
                            synthesize_type_annotation(ta, environment, checking_data),
                        )
                    })
                    .collect()
            });

            let arguments: Vec<SynthesizedArgument> = expr
                .arguments
                .iter()
                .map(|arg| match arg {
                    ast::Argument::SpreadElement(expr) => {
                        checking_data.raise_unimplemented_error(
                            "spread argument",
                            oxc_span_to_source_map_span(expr.span),
                        );

                        SynthesizedArgument::NonSpread {
                            ty: TypeId::ERROR_TYPE,
                            position: oxc_span_to_source_map_span(expr.span),
                        }
                    }
                    ast::Argument::Expression(expr) => {
                        let ty = synthesize_expression(expr, environment, checking_data);
                        SynthesizedArgument::NonSpread {
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
                ezno_checker::events::CalledWithNew::None,
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
        ast::Expression::FunctionExpression(func) => Instance::RValue(environment.new_function(
            checking_data,
            &OxcFunction(&**func, None),
            RegisterAsType,
        )),
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
                let parent = synthesize_expression(&comp.object, environment, checking_data);
                let property = synthesize_expression(&comp.expression, environment, checking_data);
                Instance::RValue(environment.get_property_handle_errors(
                    parent,
                    property,
                    checking_data,
                    oxc_span_to_source_map_span(comp.span),
                ))
            }
            ast::MemberExpression::StaticMemberExpression(expr) => {
                let parent = synthesize_expression(&expr.object, environment, checking_data);
                let property = checking_data.types.new_constant_type(
                    ezno_checker::Constant::String(expr.property.name.as_str().to_owned()),
                );

                Instance::RValue(environment.get_property_handle_errors(
                    parent,
                    property,
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
            Instance::RValue(synthesize_object(object, environment, checking_data))
        }
        ast::Expression::ParenthesizedExpression(inner) => {
            return synthesize_expression(&inner.expression, environment, checking_data);
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
            let target = synthesize_simple_assignment_target_to_reference(
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
            let result = environment.assign_to_assignable_handle_errors(
                Assignable::Reference(target),
                ezno_checker::AssignmentKind::IncrementOrDecrement(
                    match update_expr.operator {
                        oxc_syntax::operator::UpdateOperator::Increment => {
                            ezno_checker::IncrementOrDecrement::Increment
                        }
                        oxc_syntax::operator::UpdateOperator::Decrement => {
                            ezno_checker::IncrementOrDecrement::Decrement
                        }
                    },
                    match update_expr.prefix {
                        true => ezno_checker::AssignmentReturnStatus::New,
                        false => ezno_checker::AssignmentReturnStatus::Previous,
                    },
                ),
                None::<&OxcExpression>,
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
            let item_ty = synthesize_expression(&item.expression, environment, checking_data);
            let to_satisfy =
                synthesize_type_annotation(&item.type_annotation, environment, checking_data);
            checking_data.check_satisfies(
                item_ty,
                to_satisfy,
                oxc_span_to_source_map_span(item.span),
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
        ast::Expression::ArrowExpression(func) => Instance::RValue(environment.new_function(
            checking_data,
            &OxcArrowFunction(&**func),
            RegisterAsType,
        )),
    };

    checking_data
        .add_expression_mapping(oxc_span_to_source_map_span(expr.span()), instance.clone());

    instance.get_value()
}

fn synthesize_assignment<T: FSResolver>(
    expr: &ast::AssignmentExpression,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T>,
) -> TypeId {
    let lhs = synthesize_assignment_target_to_assignable(&expr.left, environment, checking_data);
    let lhs = match lhs {
        crate::PartiallyImplemented::Ok(lhs) => lhs,
        crate::PartiallyImplemented::NotImplemented(item, span) => {
            checking_data.raise_unimplemented_error(item, span);
            return TypeId::ERROR_TYPE;
        }
    };
    let operator = match expr.operator {
        AssignmentOperator::Assign => ezno_checker::AssignmentKind::Assign,
        AssignmentOperator::Addition => ezno_checker::AssignmentKind::Update(
            ezno_checker::structures::operators::BinaryOperator::Add,
        ),
        AssignmentOperator::Multiplication => ezno_checker::AssignmentKind::Update(
            ezno_checker::structures::operators::BinaryOperator::Multiply,
        ),
        AssignmentOperator::Subtraction
        | AssignmentOperator::Division
        | AssignmentOperator::Remainder
        | AssignmentOperator::ShiftLeft
        | AssignmentOperator::ShiftRight
        | AssignmentOperator::ShiftRightZeroFill
        | AssignmentOperator::BitwiseOR
        | AssignmentOperator::BitwiseXOR
        | AssignmentOperator::BitwiseAnd
        | AssignmentOperator::LogicalAnd
        | AssignmentOperator::LogicalOr
        | AssignmentOperator::LogicalNullish
        | AssignmentOperator::Exponential => {
            checking_data.raise_unimplemented_error(
                "this assignment operator",
                oxc_span_to_source_map_span(expr.span),
            );
            return TypeId::ERROR_TYPE;
        }
    };

    environment.assign_to_assignable_handle_errors(
        lhs,
        operator,
        Some(&OxcExpression(&expr.right)),
        oxc_span_to_source_map_span(expr.span),
        checking_data,
    )
}

// TODO others need to be built into helper methods in the checker
fn synthesize_assignment_target_to_assignable<T: FSResolver>(
    target: &ast::AssignmentTarget,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T>,
) -> crate::PartiallyImplemented<Assignable> {
    match target {
        ast::AssignmentTarget::SimpleAssignmentTarget(simple) => {
            match synthesize_simple_assignment_target_to_reference(
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

fn synthesize_simple_assignment_target_to_reference<T: FSResolver>(
    simple: &ast::SimpleAssignmentTarget,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T>,
) -> crate::PartiallyImplemented<Reference> {
    match simple {
        ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            crate::PartiallyImplemented::Ok(Reference::Variable(
                identifier.name.as_str().to_owned(),
                oxc_span_to_source_map_span(identifier.span),
            ))
        }
        ast::SimpleAssignmentTarget::MemberAssignmentTarget(expr) => {
            let (parent_ty, key_ty) = match &**expr {
                ast::MemberExpression::ComputedMemberExpression(comp) => {
                    let property =
                        synthesize_expression(&comp.expression, environment, checking_data);
                    let parent = synthesize_expression(&comp.object, environment, checking_data);
                    (parent, property)
                }
                ast::MemberExpression::StaticMemberExpression(expr) => {
                    let parent = synthesize_expression(&expr.object, environment, checking_data);
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
pub(crate) fn synthesize_object<T: FSResolver>(
    object: &ast::ObjectExpression,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T>,
) -> TypeId {
    let ty = environment.new_object(&mut checking_data.types, None);
    for property in object.properties.iter() {
        match property {
            ast::ObjectPropertyKind::ObjectProperty(property) => {
                let key_ty = property_key_to_type(&property.key, environment, checking_data);
                let property = if let ast::Expression::FunctionExpression(func) = &property.value {
                    environment.new_function(
                        checking_data,
                        &OxcFunction(&**func, Some(property.kind)),
                        RegisterOnExistingObject,
                    )
                } else {
                    Property::Value(synthesize_expression(
                        &property.value,
                        environment,
                        checking_data,
                    ))
                };
                environment.register_property(ty, key_ty, property);
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

fn synthesize_binary_expression<T: FSResolver>(
    lhs: &ast::Expression,
    operator: oxc_syntax::operator::BinaryOperator,
    rhs: &ast::Expression,
    checking_data: &mut CheckingData<T>,
    environment: &mut Environment,
) -> TypeId {
    let lhs_ty = synthesize_expression(lhs, environment, checking_data);
    let rhs_ty = synthesize_expression(rhs, environment, checking_data);
    use oxc_syntax::operator::BinaryOperator;

    let op = match operator {
        BinaryOperator::StrictEquality => {
            ezno_checker::structures::operators::BinaryOperator::RelationOperator(
                ezno_checker::structures::operators::RelationOperator::Equal,
            )
        }
        BinaryOperator::Addition => ezno_checker::structures::operators::BinaryOperator::Add,
        BinaryOperator::Multiplication => {
            ezno_checker::structures::operators::BinaryOperator::Multiply
        }
        BinaryOperator::Equality
        | BinaryOperator::Inequality
        | BinaryOperator::StrictInequality
        | BinaryOperator::LessThan
        | BinaryOperator::LessEqualThan
        | BinaryOperator::GreaterThan
        | BinaryOperator::GreaterEqualThan
        | BinaryOperator::ShiftLeft
        | BinaryOperator::ShiftRight
        | BinaryOperator::ShiftRightZeroFill
        | BinaryOperator::Subtraction
        | BinaryOperator::Division
        | BinaryOperator::Remainder
        | BinaryOperator::BitwiseOR
        | BinaryOperator::BitwiseXOR
        | BinaryOperator::BitwiseAnd
        | BinaryOperator::In
        | BinaryOperator::Instanceof
        | BinaryOperator::Exponential => {
            checking_data.raise_unimplemented_error(
                "this operator",
                oxc_span_to_source_map_span(lhs.span()),
            );

            return TypeId::ERROR_TYPE;
        }
    };
    ezno_checker::evaluate_binary_operator_handle_errors(
        op,
        (lhs_ty, oxc_span_to_source_map_span(GetSpan::span(lhs))),
        (rhs_ty, oxc_span_to_source_map_span(GetSpan::span(rhs))),
        environment,
        checking_data,
    )
}

struct OxcExpression<'a, 'b>(pub(crate) &'a ast::Expression<'b>);

impl ezno_checker::SynthesizableExpression for OxcExpression<'_, '_> {
    fn synthesize_expression<U: ezno_checker::FSResolver>(
        &self,
        environment: &mut Environment,
        checking_data: &mut CheckingData<U>,
    ) -> TypeId {
        synthesize_expression(self.0, environment, checking_data)
    }

    fn get_position(&self) -> ezno_checker::Span {
        oxc_span_to_source_map_span(self.0.span())
    }
}
