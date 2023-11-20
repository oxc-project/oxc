use ezno_checker::{CheckingData, Environment, ReadFromFS, TypeId};
use oxc_ast::ast;

use crate::{oxc_span_to_source_map_span, OxcAST};

pub(crate) fn synthesise_type_annotation<T: ReadFromFS>(
    ta: &ast::TSType,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T, OxcAST>,
) -> TypeId {
    match ta {
        ast::TSType::TSAnyKeyword(_) => TypeId::ANY_TYPE,
        ast::TSType::TSBigIntKeyword(item) => {
            checking_data.raise_unimplemented_error(
                "big int keyword",
                oxc_span_to_source_map_span(item.span),
            );
            TypeId::ERROR_TYPE
        }
        ast::TSType::TSBooleanKeyword(_) => TypeId::BOOLEAN_TYPE,
        ast::TSType::TSNeverKeyword(_) => TypeId::NEVER_TYPE,
        ast::TSType::TSNullKeyword(_) => TypeId::NULL_TYPE,
        ast::TSType::TSNumberKeyword(_) => TypeId::NUMBER_TYPE,
        ast::TSType::TSObjectKeyword(_) => TypeId::OBJECT_TYPE,
        ast::TSType::TSStringKeyword(_) => TypeId::STRING_TYPE,
        ast::TSType::TSSymbolKeyword(item) => {
            checking_data.raise_unimplemented_error(
                "symbol keyword",
                oxc_span_to_source_map_span(item.span),
            );
            TypeId::ERROR_TYPE
        }
        ast::TSType::TSThisKeyword(item) => {
            checking_data
                .raise_unimplemented_error("this type", oxc_span_to_source_map_span(item.span));
            TypeId::ERROR_TYPE
        }
        // 🔥
        ast::TSType::TSVoidKeyword(_) | ast::TSType::TSUndefinedKeyword(_) => {
            TypeId::UNDEFINED_TYPE
        }
        ast::TSType::TSUnknownKeyword(item) => {
            checking_data
                .raise_unimplemented_error("unknown type", oxc_span_to_source_map_span(item.span));
            TypeId::ERROR_TYPE
        }
        ast::TSType::TSArrayType(item) => {
            checking_data
                .raise_unimplemented_error("array type", oxc_span_to_source_map_span(item.span));
            TypeId::ERROR_TYPE
        }
        ast::TSType::TSConditionalType(condition) => {
            let check_type =
                synthesise_type_annotation(&condition.check_type, environment, checking_data);
            let extends =
                synthesise_type_annotation(&condition.extends_type, environment, checking_data);
            let lhs = synthesise_type_annotation(&condition.true_type, environment, checking_data);
            let rhs = synthesise_type_annotation(&condition.false_type, environment, checking_data);

            checking_data.types.new_conditional_extends_type(check_type, extends, lhs, rhs)
        }
        ast::TSType::TSConstructorType(item) => {
            checking_data.raise_unimplemented_error(
                "constructor type",
                oxc_span_to_source_map_span(item.span),
            );
            TypeId::ERROR_TYPE
        }
        ast::TSType::TSFunctionType(item) => {
            checking_data
                .raise_unimplemented_error("function type", oxc_span_to_source_map_span(item.span));
            TypeId::ERROR_TYPE
        }
        ast::TSType::TSImportType(item) => {
            checking_data
                .raise_unimplemented_error("import type", oxc_span_to_source_map_span(item.span));
            TypeId::ERROR_TYPE
        }
        ast::TSType::TSIndexedAccessType(item) => {
            checking_data.raise_unimplemented_error(
                "index access type",
                oxc_span_to_source_map_span(item.span),
            );
            TypeId::ERROR_TYPE
        }
        ast::TSType::TSInferType(item) => {
            checking_data
                .raise_unimplemented_error("infer type", oxc_span_to_source_map_span(item.span));
            TypeId::ERROR_TYPE
        }
        ast::TSType::TSUnionType(r#union) => {
            // Borrow checker doesn't like :(
            // r#union
            // 	.types
            // 	.iter()
            // 	.map(|ta| synthesise_type_annotation(ta, environment, checking_data))
            // 	.reduce(|lhs, rhs| checking_data.types.new_or_type(lhs, rhs))
            // 	.unwrap(),
            let mut iter = r#union.types.iter();
            let mut acc =
                synthesise_type_annotation(iter.next().unwrap(), environment, checking_data);
            for next in iter {
                let rhs = synthesise_type_annotation(next, environment, checking_data);
                acc = checking_data.types.new_or_type(acc, rhs);
            }
            acc
        }
        ast::TSType::TSIntersectionType(intersection) => {
            let mut iter = intersection.types.iter();
            let mut acc =
                synthesise_type_annotation(iter.next().unwrap(), environment, checking_data);
            for next in iter {
                let rhs = synthesise_type_annotation(next, environment, checking_data);
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
            ast::TSLiteral::BigintLiteral(item) => {
                checking_data.raise_unimplemented_error(
                    "big int literal type",
                    oxc_span_to_source_map_span(item.span),
                );
                TypeId::ERROR_TYPE
            }
            ast::TSLiteral::RegExpLiteral(item) => {
                checking_data.raise_unimplemented_error(
                    "regexp literal type",
                    oxc_span_to_source_map_span(item.span),
                );
                TypeId::ERROR_TYPE
            }
            ast::TSLiteral::TemplateLiteral(item) => {
                checking_data.raise_unimplemented_error(
                    "template literal type",
                    oxc_span_to_source_map_span(item.span),
                );
                TypeId::ERROR_TYPE
            }
            ast::TSLiteral::UnaryExpression(item) => {
                checking_data.raise_unimplemented_error(
                    "unary expression type",
                    oxc_span_to_source_map_span(item.span),
                );
                TypeId::ERROR_TYPE
            }
        },
        ast::TSType::TSMappedType(item) => {
            checking_data.raise_unimplemented_error(
                "ts mapped type",
                oxc_span_to_source_map_span(item.span),
            );
            TypeId::ERROR_TYPE
        }
        ast::TSType::TSQualifiedName(item) => {
            checking_data.raise_unimplemented_error(
                "ts qualified name",
                oxc_span_to_source_map_span(item.span),
            );
            TypeId::ERROR_TYPE
        }
        ast::TSType::TSTemplateLiteralType(item) => {
            checking_data.raise_unimplemented_error(
                "ts template literal type",
                oxc_span_to_source_map_span(item.span),
            );
            TypeId::ERROR_TYPE
        }
        ast::TSType::TSTupleType(item) => {
            checking_data
                .raise_unimplemented_error("tuple type", oxc_span_to_source_map_span(item.span));
            TypeId::ERROR_TYPE
        }
        ast::TSType::TSTypeLiteral(anom_interface) => {
            let ty = checking_data.types.new_anonymous_interface_ty();
            crate::interfaces::synthesise_signatures(
                &anom_interface.members,
                environment,
                checking_data,
                ty,
            );
            ty
        }
        ast::TSType::TSTypeOperatorType(item) => {
            checking_data.raise_unimplemented_error(
                "ts operator type",
                oxc_span_to_source_map_span(item.span),
            );
            TypeId::ERROR_TYPE
        }
        ast::TSType::TSTypePredicate(item) => {
            checking_data.raise_unimplemented_error(
                "ts type predicate",
                oxc_span_to_source_map_span(item.span),
            );
            TypeId::ERROR_TYPE
        }
        ast::TSType::TSTypeQuery(item) => {
            checking_data
                .raise_unimplemented_error("ts type query", oxc_span_to_source_map_span(item.span));
            TypeId::ERROR_TYPE
        }
        ast::TSType::TSTypeReference(reference) => {
            if let Some(ref _args) = reference.type_parameters {
                checking_data.raise_unimplemented_error(
                    "reference with parameters",
                    oxc_span_to_source_map_span(reference.span),
                );
                return TypeId::ERROR_TYPE;
            }
            let tn = &reference.type_name;
            match tn {
                ast::TSTypeName::IdentifierReference(name) => environment
                    .get_type_by_name_handle_errors(
                        &name.name,
                        oxc_span_to_source_map_span(name.span),
                        checking_data,
                    ),
                ast::TSTypeName::QualifiedName(item) => {
                    checking_data.raise_unimplemented_error(
                        "qualified name",
                        oxc_span_to_source_map_span(item.span),
                    );
                    TypeId::ERROR_TYPE
                }
            }
        }
        ast::TSType::JSDocNullableType(item) => {
            checking_data.raise_unimplemented_error(
                "js doc nullable",
                oxc_span_to_source_map_span(item.span),
            );
            TypeId::ERROR_TYPE
        }
        ast::TSType::JSDocUnknownType(item) => {
            checking_data.raise_unimplemented_error(
                "js doc unknown",
                oxc_span_to_source_map_span(item.span),
            );
            TypeId::ERROR_TYPE
        }
    }
}
