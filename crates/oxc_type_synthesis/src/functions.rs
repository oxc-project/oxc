use ezno_checker::{
    behavior::functions::SynthesisableFunction,
    context::VariableRegisterBehavior,
    types::{SynthesisedParameter, SynthesisedParameters},
    GenericTypeParameters, TypeId,
};
use oxc_ast::ast;

use crate::{
    oxc_span_to_source_map_span,
    statements_and_declarations::{register_variable, synthesise_statements},
    types::synthesise_type_annotation,
    OxcAST,
};

pub(crate) struct OxcFunction<'a, 'b>(pub &'a ast::Function<'b>, pub Option<ast::PropertyKind>);

impl<'a, 'b> SynthesisableFunction<OxcAST> for OxcFunction<'a, 'b> {
    fn has_body(&self) -> bool {
        !self.0.is_ts_declare_function()
    }

    fn this_constraint<T: ezno_checker::ReadFromFS>(
        &self,
        _environment: &mut ezno_checker::Environment,
        _checking_data: &mut ezno_checker::CheckingData<T, OxcAST>,
    ) -> Option<TypeId> {
        // TODO
        None
    }

    fn parameters<T: ezno_checker::ReadFromFS>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T, OxcAST>,
    ) -> SynthesisedParameters {
        synthesise_parameters(&self.0.params, environment, checking_data)
    }

    fn body<T: ezno_checker::ReadFromFS>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T, OxcAST>,
    ) {
        let body = self.0.body.as_ref().expect("trying to synthesise declare function");
        synthesise_statements(&body.statements, environment, checking_data);
    }

    fn type_parameters<T: ezno_checker::ReadFromFS>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T, OxcAST>,
    ) -> Option<GenericTypeParameters> {
        let type_parameters = self.0.type_parameters.as_deref();
        synthesise_type_parameters(type_parameters, environment, checking_data)
    }

    fn return_type_annotation<T: ezno_checker::ReadFromFS>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T, OxcAST>,
    ) -> Option<(TypeId, ezno_checker::Span)> {
        self.0.return_type.as_ref().map(|ta| {
            (
                synthesise_type_annotation(&ta.type_annotation, environment, checking_data),
                oxc_span_to_source_map_span(ta.span),
            )
        })
    }

    fn id(&self, source_id: ezno_checker::SourceId) -> ezno_checker::FunctionId {
        ezno_checker::FunctionId(source_id, self.0.span.start)
    }

    fn super_constraint<T: ezno_checker::ReadFromFS>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T, OxcAST>,
    ) -> Option<TypeId> {
        None
    }
}

pub(crate) fn synthesise_type_parameters<T: ezno_checker::ReadFromFS>(
    type_parameters: Option<&oxc_ast::ast::TSTypeParameterDeclaration>,
    environment: &mut ezno_checker::Environment,
    checking_data: &mut ezno_checker::CheckingData<T, OxcAST>,
) -> Option<GenericTypeParameters> {
    type_parameters.as_ref().map(|params| {
        params
            .params
            .iter()
            .map(|ta| {
                // TODO effects in a map :/
                let constraint_type = ta
                    .constraint
                    .as_ref()
                    .map(|ta| synthesise_type_annotation(ta, environment, checking_data));
                let default_type = ta
                    .default
                    .as_ref()
                    .map(|ta| synthesise_type_annotation(ta, environment, checking_data));
                environment.new_explicit_type_parameter(
                    ta.name.name.as_str(),
                    constraint_type,
                    default_type,
                    &mut checking_data.types,
                )
            })
            .collect()
    })
}

pub(crate) fn synthesise_parameters<T: ezno_checker::ReadFromFS>(
    params: &ast::FormalParameters,
    environment: &mut ezno_checker::Environment,
    checking_data: &mut ezno_checker::CheckingData<T, OxcAST>,
) -> SynthesisedParameters {
    let (mut parameters, rest_parameter) = (Vec::new(), None);

    for param in params.items.iter() {
        let annotation =
            param.pattern.type_annotation.as_ref().map(|ta| {
                synthesise_type_annotation(&ta.type_annotation, environment, checking_data)
            });

        let param_type = register_variable(
            &param.pattern.kind,
            &param.span,
            environment,
            checking_data,
            VariableRegisterBehavior::FunctionParameter { annotation },
        );

        match &param.pattern.kind {
            p @ (ast::BindingPatternKind::BindingIdentifier(_)
            | ast::BindingPatternKind::ObjectPattern(_)
            | ast::BindingPatternKind::ArrayPattern(_)) => {
                parameters.push(SynthesisedParameter {
                    name: match param_to_string(p) {
                        crate::PartiallyImplemented::Ok(name) => name,
                        crate::PartiallyImplemented::NotImplemented(item, span) => {
                            checking_data.raise_unimplemented_error(item, span);
                            "temp".into()
                        }
                    },
                    ty: param_type,
                    position: oxc_span_to_source_map_span(param.span),
                    // TODO param.pattern.
                    missing_value: None,
                });
            }
            ast::BindingPatternKind::AssignmentPattern(item) => checking_data
                .raise_unimplemented_error(
                    "parameter with default value",
                    oxc_span_to_source_map_span(item.span),
                ),
            // ast::BindingPatternKind::RestElement(element) => {
            // 	rest_parameter = Some(synthesisedRestParameter {
            // 		name: param_to_string(&element.argument.kind),
            // 		item_type: param_type,
            // 		position: oxc_span_to_source_map_span(element.span),
            // 	})
            // }
        }
    }

    SynthesisedParameters { parameters, rest_parameter }
}

fn param_to_string(binding: &ast::BindingPatternKind) -> crate::PartiallyImplemented<String> {
    match binding {
        ast::BindingPatternKind::BindingIdentifier(ident) => {
            crate::PartiallyImplemented::Ok(ident.name.as_str().to_owned())
        }
        ast::BindingPatternKind::ObjectPattern(param) => {
            crate::PartiallyImplemented::NotImplemented(
                "stringing complex parameters",
                oxc_span_to_source_map_span(param.span),
            )
        }
        ast::BindingPatternKind::ArrayPattern(param) => {
            crate::PartiallyImplemented::NotImplemented(
                "stringing complex parameters",
                oxc_span_to_source_map_span(param.span),
            )
        }
        ast::BindingPatternKind::AssignmentPattern(param) => {
            crate::PartiallyImplemented::NotImplemented(
                "stringing complex parameters",
                oxc_span_to_source_map_span(param.span),
            )
        }
    }
}

pub(crate) struct OxcArrowFunction<'a, 'b>(pub &'a ast::ArrowExpression<'b>);

impl<'a, 'b> SynthesisableFunction<OxcAST> for OxcArrowFunction<'a, 'b> {
    fn parameters<T: ezno_checker::ReadFromFS>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T, OxcAST>,
    ) -> SynthesisedParameters {
        synthesise_parameters(&self.0.params, environment, checking_data)
    }

    fn body<T: ezno_checker::ReadFromFS>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T, OxcAST>,
    ) {
        if self.0.expression {
            if let Some(ast::Statement::ExpressionStatement(expr)) = self.0.body.statements.get(0) {
                let returned = crate::expressions::synthesise_expression(
                    &expr.expression,
                    // TODO
                    TypeId::ANY_TYPE,
                    environment,
                    checking_data,
                );
                environment.return_value(
                    returned,
                    oxc_span_to_source_map_span(self.0.span)
                        .with_source(environment.get_environment_id()),
                )
            } else {
                unreachable!()
            }
        } else {
            synthesise_statements(&self.0.body.statements, environment, checking_data)
        }
    }

    fn type_parameters<T: ezno_checker::ReadFromFS>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T, OxcAST>,
    ) -> Option<GenericTypeParameters> {
        let type_parameters = self.0.type_parameters.as_deref();
        synthesise_type_parameters(type_parameters, environment, checking_data)
    }

    fn return_type_annotation<T: ezno_checker::ReadFromFS>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T, OxcAST>,
    ) -> Option<(TypeId, ezno_checker::Span)> {
        self.0.return_type.as_ref().map(|ta| {
            (
                synthesise_type_annotation(&ta.type_annotation, environment, checking_data),
                oxc_span_to_source_map_span(ta.span),
            )
        })
    }

    fn this_constraint<T: ezno_checker::ReadFromFS>(
        &self,
        _environment: &mut ezno_checker::Environment,
        _checking_data: &mut ezno_checker::CheckingData<T, OxcAST>,
    ) -> Option<TypeId> {
        // TODO
        None
    }

    fn id(&self, source_id: ezno_checker::SourceId) -> ezno_checker::FunctionId {
        ezno_checker::FunctionId(source_id, self.0.span.start)
    }

    fn has_body(&self) -> bool {
        todo!()
    }

    fn super_constraint<T: ezno_checker::ReadFromFS>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T, OxcAST>,
    ) -> Option<TypeId> {
        todo!()
    }
}
