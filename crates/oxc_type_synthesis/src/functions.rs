use ezno_checker::{
    context::VariableRegisterBehavior,
    types::functions::{SynthesizedParameter, SynthesizedParameters},
    GenericTypeParameters, SynthesizableFunction, TypeId,
};
use oxc_ast::ast;

use crate::{
    oxc_span_to_source_map_span,
    statements_and_declarations::{register_variable, synthesize_statements},
    types::synthesize_type_annotation,
};

pub(crate) struct OxcFunction<'a, 'b>(pub &'a ast::Function<'b>, pub Option<ast::PropertyKind>);

impl<'a, 'b> SynthesizableFunction for OxcFunction<'a, 'b> {
    fn is_declare(&self) -> bool {
        self.0.is_ts_declare_function()
    }

    fn is_async(&self) -> bool {
        self.0.modifiers.contains(ast::ModifierKind::Async)
    }

    fn get_set_generator_or_none(&self) -> ezno_checker::GetSetGeneratorOrNone {
        if self.0.generator {
            ezno_checker::GetSetGeneratorOrNone::Generator
        } else if let Some(prop_kind) = self.1 {
            match prop_kind {
                ast::PropertyKind::Init => ezno_checker::GetSetGeneratorOrNone::None,
                ast::PropertyKind::Get => ezno_checker::GetSetGeneratorOrNone::Get,
                ast::PropertyKind::Set => ezno_checker::GetSetGeneratorOrNone::Set,
            }
        } else {
            ezno_checker::GetSetGeneratorOrNone::None
        }
    }

    fn id(&self) -> ezno_checker::context::FunctionId {
        ezno_checker::context::FunctionId(oxc_span_to_source_map_span(self.0.span))
    }

    fn this_constraint<T: ezno_checker::FSResolver>(
        &self,
        _environment: &mut ezno_checker::Environment,
        _checking_data: &mut ezno_checker::CheckingData<T>,
    ) -> Option<TypeId> {
        // TODO
        None
    }

    fn parameters<T: ezno_checker::FSResolver>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T>,
    ) -> SynthesizedParameters {
        synthesize_parameters(&self.0.params, environment, checking_data)
    }

    fn body<T: ezno_checker::FSResolver>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T>,
    ) {
        let body = self.0.body.as_ref().expect("trying to synthesize declare function");
        synthesize_statements(&body.statements, environment, checking_data);
    }

    fn type_parameters<T: ezno_checker::FSResolver>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T>,
    ) -> Option<GenericTypeParameters> {
        let type_parameters = self.0.type_parameters.as_deref();
        synthesize_type_parameters(type_parameters, environment, checking_data)
    }

    fn return_type_annotation<T: ezno_checker::FSResolver>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T>,
    ) -> Option<(TypeId, ezno_checker::Span)> {
        self.0.return_type.as_ref().map(|ta| {
            (
                synthesize_type_annotation(&ta.type_annotation, environment, checking_data),
                oxc_span_to_source_map_span(ta.span),
            )
        })
    }
}

pub(crate) fn synthesize_type_parameters<T: ezno_checker::FSResolver>(
    type_parameters: Option<&oxc_ast::ast::TSTypeParameterDeclaration>,
    environment: &mut ezno_checker::Environment,
    checking_data: &mut ezno_checker::CheckingData<T>,
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
                    .map(|ta| synthesize_type_annotation(ta, environment, checking_data));
                let default_type = ta
                    .default
                    .as_ref()
                    .map(|ta| synthesize_type_annotation(ta, environment, checking_data));
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

pub(crate) fn synthesize_parameters<T: ezno_checker::FSResolver>(
    params: &ast::FormalParameters,
    environment: &mut ezno_checker::Environment,
    checking_data: &mut ezno_checker::CheckingData<T>,
) -> SynthesizedParameters {
    let (mut parameters, optional_parameters, rest_parameter) = (Vec::new(), Vec::new(), None);

    for param in params.items.iter() {
        let annotation =
            param.pattern.type_annotation.as_ref().map(|ta| {
                synthesize_type_annotation(&ta.type_annotation, environment, checking_data)
            });

        let param_type = register_variable(
            &param.pattern.kind,
            &param.span,
            environment,
            checking_data,
            VariableRegisterBehavior::FunctionParameter { annotation },
        );

        match &param.pattern.kind {
            p @ ast::BindingPatternKind::BindingIdentifier(_)
            | p @ ast::BindingPatternKind::ObjectPattern(_)
            | p @ ast::BindingPatternKind::ArrayPattern(_) => {
                parameters.push(SynthesizedParameter {
                    name: param_to_string(p),
                    ty: param_type,
                    position: oxc_span_to_source_map_span(param.span),
                });
            }
            ast::BindingPatternKind::AssignmentPattern(_) => todo!(),
            // ast::BindingPatternKind::RestElement(element) => {
            // 	rest_parameter = Some(SynthesizedRestParameter {
            // 		name: param_to_string(&element.argument.kind),
            // 		item_type: param_type,
            // 		position: oxc_span_to_source_map_span(element.span),
            // 	})
            // }
        }
    }

    SynthesizedParameters { parameters, optional_parameters, rest_parameter }
}

fn param_to_string(binding: &ast::BindingPatternKind) -> String {
    match binding {
        ast::BindingPatternKind::BindingIdentifier(ident) => ident.name.as_str().to_owned(),
        ast::BindingPatternKind::ObjectPattern(_) => todo!(),
        ast::BindingPatternKind::ArrayPattern(_) => todo!(),
        // ast::BindingPatternKind::RestElement(_) => todo!(),
        ast::BindingPatternKind::AssignmentPattern(_) => todo!(),
    }
}

pub(crate) struct OxcArrowFunction<'a, 'b>(pub &'a ast::ArrowExpression<'b>);

impl<'a, 'b> SynthesizableFunction for OxcArrowFunction<'a, 'b> {
    fn is_declare(&self) -> bool {
        false
    }

    fn is_async(&self) -> bool {
        self.0.r#async
    }

    fn parameters<T: ezno_checker::FSResolver>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T>,
    ) -> SynthesizedParameters {
        synthesize_parameters(&self.0.params, environment, checking_data)
    }

    fn body<T: ezno_checker::FSResolver>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T>,
    ) {
        if self.0.expression {
            if let Some(ast::Statement::ExpressionStatement(expr)) = self.0.body.statements.get(0) {
                let returned = crate::expressions::synthesize_expression(
                    &expr.expression,
                    environment,
                    checking_data,
                );
                environment.return_value(returned)
            } else {
                unreachable!()
            }
        } else {
            synthesize_statements(&self.0.body.statements, environment, checking_data)
        }
    }

    fn type_parameters<T: ezno_checker::FSResolver>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T>,
    ) -> Option<GenericTypeParameters> {
        let type_parameters = self.0.type_parameters.as_deref();
        synthesize_type_parameters(type_parameters, environment, checking_data)
    }

    fn return_type_annotation<T: ezno_checker::FSResolver>(
        &self,
        environment: &mut ezno_checker::Environment,
        checking_data: &mut ezno_checker::CheckingData<T>,
    ) -> Option<(TypeId, ezno_checker::Span)> {
        self.0.return_type.as_ref().map(|ta| {
            (
                synthesize_type_annotation(&ta.type_annotation, environment, checking_data),
                oxc_span_to_source_map_span(ta.span),
            )
        })
    }

    fn get_set_generator_or_none(&self) -> ezno_checker::GetSetGeneratorOrNone {
        if self.0.generator {
            ezno_checker::GetSetGeneratorOrNone::Generator
        } else {
            ezno_checker::GetSetGeneratorOrNone::None
        }
    }

    fn id(&self) -> ezno_checker::context::FunctionId {
        ezno_checker::context::FunctionId(oxc_span_to_source_map_span(self.0.span))
    }

    fn this_constraint<T: ezno_checker::FSResolver>(
        &self,
        _environment: &mut ezno_checker::Environment,
        _checking_data: &mut ezno_checker::CheckingData<T>,
    ) -> Option<TypeId> {
        // TODO
        None
    }
}
