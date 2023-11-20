use ezno_checker::{CheckingData, Environment, TypeId};
use oxc_ast::ast;

use crate::{
    oxc_span_to_source_map_span, property_key_to_type,
    statements_and_declarations::synthesise_statement, types::synthesise_type_annotation, OxcAST,
};

pub(crate) fn synthesise_interface<T: ezno_checker::ReadFromFS>(
    interface: &ast::TSInterfaceDeclaration,
    interface_id: TypeId,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T, OxcAST>,
) {
    synthesise_signatures(&interface.body.body, environment, checking_data, interface_id);
}

pub(crate) fn synthesise_signatures<T: ezno_checker::ReadFromFS>(
    signatures: &[ast::TSSignature],
    environment: &mut Environment,
    checking_data: &mut CheckingData<T, OxcAST>,
    onto: TypeId,
) {
    for declaration in signatures.iter() {
        match declaration {
            ast::TSSignature::TSPropertySignature(property) => {
                let key_ty = property_key_to_type(&property.key, environment, checking_data);
                let value_ty = synthesise_type_annotation(
                    &property.type_annotation.as_ref().unwrap().type_annotation,
                    environment,
                    checking_data,
                );
                environment.facts.register_property(
                    onto,
                    ezno_checker::context::facts::Publicity::Public,
                    key_ty,
                    ezno_checker::PropertyValue::Value(value_ty),
                    false,
                    None,
                );
            }
            ast::TSSignature::TSIndexSignature(item) => {
                checking_data.raise_unimplemented_error(
                    "ts index signature",
                    oxc_span_to_source_map_span(item.span),
                );
            }
            ast::TSSignature::TSCallSignatureDeclaration(item) => {
                checking_data.raise_unimplemented_error(
                    "ts call signature",
                    oxc_span_to_source_map_span(item.span),
                );
            }
            ast::TSSignature::TSConstructSignatureDeclaration(item) => {
                checking_data.raise_unimplemented_error(
                    "ts construct signature",
                    oxc_span_to_source_map_span(item.span),
                );
            }
            ast::TSSignature::TSMethodSignature(method) => {
                // TODO reuse more functions
                let key_ty = property_key_to_type(&method.key, environment, checking_data);

                let ((type_parameters, parameters, returned, constant_fn), stuff, _) = environment
                    .new_lexical_environment_fold_into_parent(
                        ezno_checker::Scope::FunctionAnnotation {},
                        checking_data,
                        |environment, checking_data| {
                            let type_parameters = crate::functions::synthesise_type_parameters(
                                method.type_parameters.as_deref(),
                                environment,
                                checking_data,
                            );

                            let parameters = crate::functions::synthesise_parameters(
                                &method.params,
                                environment,
                                checking_data,
                            );

                            let (returned, constant_fn) =
                                if let Some(ta) = method.return_type.as_ref() {
                                    let mut ta = &ta.type_annotation;
                                    let mut constant_fn = None;

                                    get_constant_function(
                                        ta,
                                        environment,
                                        checking_data,
                                        &mut constant_fn,
                                    );

                                    (
                                        synthesise_type_annotation(ta, environment, checking_data),
                                        constant_fn,
                                    )
                                } else {
                                    (TypeId::UNDEFINED_TYPE, None)
                                };
                            (type_parameters, parameters, returned, constant_fn)
                        },
                    );

                let func_ty = checking_data.types.new_function_type_reference(
                    type_parameters,
                    parameters,
                    returned,
                    oxc_span_to_source_map_span(method.span),
                    stuff.unwrap().0,
                    constant_fn,
                );

                environment.register_property(
                    onto,
                    key_ty,
                    ezno_checker::PropertyValue::Value(func_ty),
                );
            }
        }
    }
}

fn get_constant_function<T: crate::ReadFromFS>(
    mut ta: &ast::TSType<'_>,
    environment: &mut ezno_checker::Environment,
    checking_data: &mut CheckingData<'_, T, OxcAST>,
    constant_fn: &mut Option<String>,
) {
    if let ast::TSType::TSIntersectionType(intersection) = ta {
        if let ast::TSType::TSTypeReference(type_ref) = intersection.types.last().unwrap() {
            if let (ast::TSTypeName::QualifiedName(ref qual), Some(tp)) =
                (&type_ref.type_name, &type_ref.type_parameters)
            {
                if let ast::TSTypeName::IdentifierReference(ref parent_name) = qual.left {
                    if parent_name.name == "Ezno" {
                        // *remove* the right annotation
                        // TODO discards middle ones
                        ta = intersection.types.first().unwrap();

                        match qual.right.name.as_str() {
                            "Performs" => register_internal_events(tp, environment, checking_data),
                            "ConstantFunction" => {
                                if let Some(ast::TSType::TSLiteralType(lit)) = tp.params.first() {
                                    if let ast::TSLiteral::StringLiteral(string) = &lit.literal {
                                        *constant_fn = Some(string.value.as_str().to_owned());
                                    }
                                }
                            }
                            unknown => panic!("Ezno.{}", unknown),
                        }
                    }
                }
            }
        }
    }
}

fn register_internal_events<T: ezno_checker::ReadFromFS>(
    tp: &ast::TSTypeParameterInstantiation,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T, OxcAST>,
) {
    if let Some(ast::TSType::TSLiteralType(lit)) = tp.params.first() {
        if let ast::TSLiteral::StringLiteral(string) = &lit.literal {
            let source = string.value.as_str().to_owned();

            let allocator = oxc_allocator::Allocator::default();
            let ret = oxc_parser::Parser::new(&allocator, &source, oxc_span::SourceType::default())
                .parse();

            for statement in ret.program.body.iter() {
                synthesise_statement(statement, environment, checking_data)
            }
        }
    }
}
