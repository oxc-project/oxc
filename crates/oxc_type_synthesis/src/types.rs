use ezno_checker::{CheckingData, Environment, FSResolver, TypeId};
use oxc_ast::ast;

use crate::oxc_span_to_source_map_span;

pub(crate) fn synthesize_type_annotation<T: FSResolver>(
    ta: &ast::TSType,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T>,
) -> TypeId {
    match ta {
        ast::TSType::TSAnyKeyword(_) => TypeId::ANY_TYPE,
        ast::TSType::TSBigIntKeyword(_) => todo!(),
        ast::TSType::TSBooleanKeyword(_) => TypeId::BOOLEAN_TYPE,
        ast::TSType::TSNeverKeyword(_) => TypeId::NEVER_TYPE,
        ast::TSType::TSNullKeyword(_) => TypeId::NULL_TYPE,
        ast::TSType::TSNumberKeyword(_) => TypeId::NUMBER_TYPE,
        ast::TSType::TSObjectKeyword(_) => TypeId::OBJECT_TYPE,
        ast::TSType::TSStringKeyword(_) => TypeId::STRING_TYPE,
        ast::TSType::TSSymbolKeyword(_) => todo!(),
        ast::TSType::TSThisKeyword(_) => todo!(),
        ast::TSType::TSUndefinedKeyword(_) => TypeId::UNDEFINED_TYPE,
        ast::TSType::TSUnknownKeyword(_) => todo!(),
        ast::TSType::TSVoidKeyword(_) => TypeId::UNDEFINED_TYPE,
        ast::TSType::TSArrayType(_) => todo!(),
        ast::TSType::TSConditionalType(condition) => {
            let check_type =
                synthesize_type_annotation(&condition.check_type, environment, checking_data);
            let extends =
                synthesize_type_annotation(&condition.extends_type, environment, checking_data);
            let lhs = synthesize_type_annotation(&condition.true_type, environment, checking_data);
            let rhs = synthesize_type_annotation(&condition.false_type, environment, checking_data);

            checking_data.types.new_conditional_extends_type(check_type, extends, lhs, rhs)
        }
        ast::TSType::TSConstructorType(_) => todo!(),
        ast::TSType::TSFunctionType(_) => todo!(),
        ast::TSType::TSImportType(_) => todo!(),
        ast::TSType::TSIndexedAccessType(_) => todo!(),
        ast::TSType::TSInferType(_) => todo!(),
        ast::TSType::TSUnionType(r#union) => {
            // Borrow checker doesn't like :(
            // r#union
            // 	.types
            // 	.iter()
            // 	.map(|ta| synthesize_type_annotation(ta, environment, checking_data))
            // 	.reduce(|lhs, rhs| checking_data.types.new_or_type(lhs, rhs))
            // 	.unwrap(),
            let mut iter = r#union.types.iter();
            let mut acc =
                synthesize_type_annotation(iter.next().unwrap(), environment, checking_data);
            for next in iter {
                let rhs = synthesize_type_annotation(next, environment, checking_data);
                acc = checking_data.types.new_or_type(acc, rhs);
            }
            acc
        }
        ast::TSType::TSIntersectionType(intersection) => {
            let mut iter = intersection.types.iter();
            let mut acc =
                synthesize_type_annotation(iter.next().unwrap(), environment, checking_data);
            for next in iter {
                let rhs = synthesize_type_annotation(next, environment, checking_data);
                acc = checking_data.types.new_and_type(acc, rhs);
            }
            acc
        }
        ast::TSType::TSLiteralType(lit) => match &lit.literal {
            ast::TSLiteral::NullLiteral(_) => ezno_checker::TypeId::NULL_TYPE,
            ast::TSLiteral::BooleanLiteral(bool) => {
                checking_data.types.new_constant_type(ezno_checker::Constant::Boolean(bool.value))
            }
            ast::TSLiteral::NumberLiteral(number) => checking_data.types.new_constant_type(
                number
                    .value
                    .try_into()
                    .map(ezno_checker::Constant::Number)
                    .unwrap_or(ezno_checker::Constant::NaN),
            ),
            ast::TSLiteral::StringLiteral(string) => checking_data.types.new_constant_type(
                ezno_checker::Constant::String(string.value.as_str().to_owned()),
            ),
            ast::TSLiteral::BigintLiteral(_) => todo!(),
            ast::TSLiteral::RegExpLiteral(_) => todo!(),
            ast::TSLiteral::TemplateLiteral(_) => todo!(),
            ast::TSLiteral::UnaryExpression(_) => todo!(),
        },
        ast::TSType::TSMappedType(_) => todo!(),
        ast::TSType::TSQualifiedName(_name) => todo!(),
        ast::TSType::TSTemplateLiteralType(_) => todo!(),
        ast::TSType::TSTupleType(_) => todo!(),
        ast::TSType::TSTypeLiteral(anom_interface) => {
            let ty = checking_data.types.new_anonymous_interface_ty();
            crate::interfaces::synthesize_signatures(
                &anom_interface.members,
                environment,
                checking_data,
                ty,
            );
            ty
        }
        ast::TSType::TSTypeOperatorType(_) => todo!(),
        ast::TSType::TSTypePredicate(_) => todo!(),
        ast::TSType::TSTypeQuery(_) => todo!(),
        ast::TSType::TSTypeReference(reference) => {
            if let Some(ref _args) = reference.type_parameters {
                todo!()
            }
            let tn = &reference.type_name;
            match tn {
                ast::TSTypeName::IdentifierName(name) => environment
                    .get_type_by_name_handle_errors(
                        &name.name,
                        oxc_span_to_source_map_span(name.span),
                        checking_data,
                    ),
                ast::TSTypeName::QualifiedName(_) => todo!(),
            }
        }
        ast::TSType::JSDocNullableType(_) => todo!(),
        ast::TSType::JSDocUnknownType(_) => todo!(),
    }
}
