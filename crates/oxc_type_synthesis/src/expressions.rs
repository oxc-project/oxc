use ezno_checker::{
    self, structures::functions::SynthesizedArgument, CheckingData, Environment, FSResolver, TypeId,
};
use oxc_ast::ast;
use oxc_span::GetSpan;

use super::property_key_to_type;
use crate::{oxc_span_to_source_map_span, types::synthesize_type_annotation};

pub(crate) fn synthesize_expression<T: FSResolver>(
    expr: &ast::Expression,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T>,
) -> TypeId {
    match expr {
        ast::Expression::BooleanLiteral(boolean) => {
            checking_data.types.new_constant_type(ezno_checker::Constant::Boolean(boolean.value))
        }
        ast::Expression::NullLiteral(_) => TypeId::NULL_TYPE,
        ast::Expression::BigintLiteral(_) => todo!(),
        ast::Expression::RegExpLiteral(_) => todo!(),
        ast::Expression::NumberLiteral(number) => checking_data.types.new_constant_type(
            number
                .value
                .try_into()
                .map(ezno_checker::Constant::Number)
                .unwrap_or(ezno_checker::Constant::NaN),
        ),
        ast::Expression::StringLiteral(string) => {
            // TODO could be better here :)
            checking_data
                .types
                .new_constant_type(ezno_checker::Constant::String(string.value.as_str().to_owned()))
        }
        ast::Expression::TemplateLiteral(_) => todo!(),
        ast::Expression::Identifier(identifier) => {
            let result = environment.get_variable_or_error(
                &identifier.name,
                &oxc_span_to_source_map_span(identifier.span),
                checking_data,
            );

            match result {
                Ok(ok) => ok.1,
                Err(err) => err,
            }
        }
        ast::Expression::MetaProperty(_) => todo!(),
        ast::Expression::Super(_) => todo!(),
        ast::Expression::ArrayExpression(_) => todo!(),
        ast::Expression::AssignmentExpression(assignment) => {
            synthesize_assignment(assignment, environment, checking_data)
        }
        ast::Expression::AwaitExpression(_) => todo!(),
        ast::Expression::BinaryExpression(bin_expr) => synthesize_binary_expression(
            &bin_expr.left,
            bin_expr.operator,
            &bin_expr.right,
            checking_data,
            environment,
        ),
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
                    ast::Argument::SpreadElement(_) => todo!(),
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

            ezno_checker::call_type_handle_errors(
                parent,
                arguments,
                this_argument,
                type_arguments,
                environment,
                checking_data,
                ezno_checker::events::CalledWithNew::None,
                oxc_span_to_source_map_span(expr.span),
            )
        }
        ast::Expression::ChainExpression(_) => todo!(),
        ast::Expression::ClassExpression(_) => todo!(),
        ast::Expression::ConditionalExpression(_) => todo!(),
        ast::Expression::FunctionExpression(_) => todo!(),
        ast::Expression::ImportExpression(_) => todo!(),
        ast::Expression::LogicalExpression(_) => todo!(),
        ast::Expression::MemberExpression(expr) => match &**expr {
            ast::MemberExpression::ComputedMemberExpression(comp) => {
                let property = synthesize_expression(&comp.expression, environment, checking_data);
                let parent = synthesize_expression(&comp.object, environment, checking_data);
                environment.get_property_handle_errors(
                    parent,
                    property,
                    checking_data,
                    oxc_span_to_source_map_span(comp.span),
                )
            }
            ast::MemberExpression::StaticMemberExpression(expr) => {
                let parent = synthesize_expression(&expr.object, environment, checking_data);
                let property = checking_data.types.new_constant_type(
                    ezno_checker::Constant::String(expr.property.name.as_str().to_owned()),
                );

                environment.get_property_handle_errors(
                    parent,
                    property,
                    checking_data,
                    oxc_span_to_source_map_span(expr.span),
                )
            }
            ast::MemberExpression::PrivateFieldExpression(_) => todo!(),
        },
        ast::Expression::NewExpression(_) => todo!(),
        ast::Expression::ObjectExpression(object) => {
            synthesize_object(object, environment, checking_data)
        }
        ast::Expression::ParenthesizedExpression(inner) => {
            synthesize_expression(&inner.expression, environment, checking_data)
        }
        ast::Expression::SequenceExpression(_) => todo!(),
        ast::Expression::TaggedTemplateExpression(_) => todo!(),
        ast::Expression::ThisExpression(_) => todo!(),
        ast::Expression::UnaryExpression(_) => todo!(),
        ast::Expression::UpdateExpression(_update_expr) => {
            todo!("update_expr")
        }
        ast::Expression::YieldExpression(_) => todo!(),
        ast::Expression::PrivateInExpression(_) => todo!(),
        ast::Expression::JSXElement(_) => todo!(),
        ast::Expression::JSXFragment(_) => todo!(),
        ast::Expression::TSAsExpression(_) => todo!(),
        ast::Expression::TSSatisfiesExpression(_) => todo!(),
        ast::Expression::TSTypeAssertion(_) => todo!(),
        ast::Expression::TSNonNullExpression(_) => todo!(),
        ast::Expression::TSInstantiationExpression(_) => todo!(),
        ast::Expression::ArrowExpression(_) => todo!(),
    }
}

// TODO others need to be built into helper methods in the checker
fn synthesize_assignment<T: FSResolver>(
    assignment: &ast::AssignmentExpression,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T>,
) -> TypeId {
    match &assignment.left {
        ast::AssignmentTarget::SimpleAssignmentTarget(simple) => match simple {
            ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => {
                let rhs = synthesize_expression(&assignment.right, environment, checking_data);
                environment.assign_variable_handle_errors(
                    identifier.name.as_str(),
                    oxc_span_to_source_map_span(assignment.span),
                    rhs,
                    checking_data,
                )
            }
            ast::SimpleAssignmentTarget::MemberAssignmentTarget(_) => todo!(),
            ast::SimpleAssignmentTarget::TSAsExpression(_) => todo!(),
            ast::SimpleAssignmentTarget::TSSatisfiesExpression(_) => todo!(),
            ast::SimpleAssignmentTarget::TSNonNullExpression(_) => todo!(),
            ast::SimpleAssignmentTarget::TSTypeAssertion(_) => todo!(),
        },
        ast::AssignmentTarget::AssignmentTargetPattern(_) => todo!(),
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

                let value_ty = synthesize_expression(&property.value, environment, checking_data);
                environment.register_property_on_object(ty, key_ty, value_ty);
            }
            ast::ObjectPropertyKind::SpreadProperty(_) => todo!(),
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
        BinaryOperator::Equality => {
            todo!()
        }
        BinaryOperator::Inequality => todo!(),
        BinaryOperator::StrictEquality => {
            ezno_checker::structures::operators::BinaryOperator::RelationOperator(
                ezno_checker::structures::operators::RelationOperator::Equal,
            )
        }
        BinaryOperator::StrictInequality => todo!(),
        BinaryOperator::LessThan => todo!(),
        BinaryOperator::LessEqualThan => todo!(),
        BinaryOperator::GreaterThan => todo!(),
        BinaryOperator::GreaterEqualThan => todo!(),
        BinaryOperator::ShiftLeft => todo!(),
        BinaryOperator::ShiftRight => todo!(),
        BinaryOperator::ShiftRightZeroFill => todo!(),
        BinaryOperator::Addition => ezno_checker::structures::operators::BinaryOperator::Add,
        BinaryOperator::Subtraction => todo!(),
        BinaryOperator::Multiplication => {
            ezno_checker::structures::operators::BinaryOperator::Multiply
        }
        BinaryOperator::Division => todo!(),
        BinaryOperator::Remainder => todo!(),
        BinaryOperator::BitwiseOR => todo!(),
        BinaryOperator::BitwiseXOR => todo!(),
        BinaryOperator::BitwiseAnd => todo!(),
        BinaryOperator::In => todo!(),
        BinaryOperator::Instanceof => todo!(),
        BinaryOperator::Exponential => todo!(),
    };
    ezno_checker::evaluate_binary_operator_handle_errors(
        op,
        (lhs_ty, oxc_span_to_source_map_span(GetSpan::span(lhs))),
        (rhs_ty, oxc_span_to_source_map_span(GetSpan::span(rhs))),
        environment,
        checking_data,
    )
}
