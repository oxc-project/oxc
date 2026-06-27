use oxc_allocator::{ArenaBox, ArenaVec, CloneIn, GetAllocator};
use oxc_ast::{NONE, ast::*};
use oxc_span::{SPAN, Span};

use crate::{
    IsolatedDeclarations,
    diagnostics::{
        function_must_have_explicit_return_type, implicitly_adding_undefined_to_type,
        parameter_must_have_explicit_type,
    },
    formal_parameter_binding_pattern::FormalParameterBindingPattern,
};

impl<'a> IsolatedDeclarations<'a> {
    pub(crate) fn transform_function(
        &self,
        func: &Function<'a>,
        declare: Option<bool>,
    ) -> ArenaBox<'a, Function<'a>> {
        let return_type = self.infer_function_return_type(func);
        if return_type.is_none() {
            self.error(function_must_have_explicit_return_type(get_function_span(func)));
        }
        let params = self.transform_formal_parameters(&func.params, false);
        Function::boxed(
            func.span,
            func.r#type,
            func.id.clone_in(self.allocator()),
            false,
            false,
            declare.unwrap_or_else(|| self.is_declare()),
            func.type_parameters.clone_in(self.allocator()),
            func.this_param.clone_in(self.allocator()),
            params,
            return_type,
            NONE,
            self,
        )
    }

    pub(crate) fn transform_formal_parameter(
        &self,
        param: &FormalParameter<'a>,
        is_remaining_params_have_required: bool,
        in_private_constructor: bool,
    ) -> Option<FormalParameter<'a>> {
        let pattern = &param.pattern;
        if param.initializer.is_some()
            && pattern.is_destructuring_pattern()
            && param.type_annotation.is_none()
        {
            self.error(parameter_must_have_explicit_type(param.span));
            return None;
        }

        let is_assignment_pattern = param.initializer.is_some();
        let mut pattern = if let BindingPattern::AssignmentPattern(pattern) = &param.pattern {
            pattern.left.clone_in(self.allocator())
        } else {
            param.pattern.clone_in(self.allocator())
        };

        FormalParameterBindingPattern::remove_assignments_from_kind(self, &mut pattern);

        if is_assignment_pattern
            || param.type_annotation.is_none()
            || (param.optional && param.has_modifier())
        {
            let type_annotation = param
                .type_annotation
                .as_ref()
                .map(|type_annotation| type_annotation.type_annotation.clone_in(self.allocator()))
                .or_else(|| {
                    let new_type = self.infer_type_from_formal_parameter(param);
                    // A private parameter property on a private constructor needs no
                    // explicit type: the constructor signature is collapsed to
                    // `private constructor();` and the class member is emitted as
                    // `private readonly name;` with no type annotation.
                    let is_elided_private_param = in_private_constructor
                        && param.accessibility.is_some_and(TSAccessibility::is_private);
                    if new_type.is_none() && !is_elided_private_param {
                        self.error(parameter_must_have_explicit_type(param.span));
                    }
                    new_type
                })
                .map(|ts_type| {
                    // jf next param is not optional and current param is assignment pattern
                    // we need to add undefined to it's type
                    if is_remaining_params_have_required || (param.optional && param.has_modifier())
                    {
                        if matches!(ts_type, TSType::TSTypeReference(_)) {
                            self.error(implicitly_adding_undefined_to_type(param.span));
                        } else if !ts_type.is_maybe_undefined() {
                            // union with `undefined`
                            return TSTypeAnnotation::new(
                                SPAN,
                                TSType::new_ts_union_type(
                                    SPAN,
                                    ArenaVec::from_array_in(
                                        [ts_type, TSType::new_ts_undefined_keyword(SPAN, self)],
                                        self,
                                    ),
                                    self,
                                ),
                                self,
                            );
                        }
                    }

                    TSTypeAnnotation::new(SPAN, ts_type, self)
                });

            let optional =
                param.optional || (!is_remaining_params_have_required && is_assignment_pattern);
            return Some(FormalParameter::new(
                param.span,
                ArenaVec::new_in(self),
                pattern.clone_in(self.allocator()),
                type_annotation,
                NONE,
                optional,
                None,
                false,
                false,
                self,
            ));
        }

        Some(FormalParameter::new(
            param.span,
            ArenaVec::new_in(self),
            pattern,
            param.type_annotation.clone_in(self.allocator()),
            NONE,
            param.optional,
            None,
            false,
            false,
            self,
        ))
    }

    pub(crate) fn transform_formal_parameters(
        &self,
        params: &ArenaBox<'a, FormalParameters<'a>>,
        in_private_constructor: bool,
    ) -> ArenaBox<'a, FormalParameters<'a>> {
        if params.kind.is_signature() || (params.rest.is_none() && params.items.is_empty()) {
            return params.clone_in(self.allocator());
        }

        let items = ArenaVec::from_iter_in(
            params
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| !in_private_constructor || item.has_modifier())
                .filter_map(|(index, item)| {
                    let is_remaining_params_have_required = params
                        .items
                        .iter()
                        .skip(index)
                        .any(|item| !(item.optional || item.initializer.is_some()));
                    self.transform_formal_parameter(
                        item,
                        is_remaining_params_have_required,
                        in_private_constructor,
                    )
                }),
            self,
        );

        if let Some(rest) = &params.rest
            && rest.type_annotation.is_none()
        {
            self.error(parameter_must_have_explicit_type(rest.span));
        }

        let rest = params.rest.as_ref().map(|rest| {
            let mut rest = rest.clone_in(self.allocator());
            FormalParameterBindingPattern::remove_assignments_from_kind(
                self,
                &mut rest.rest.argument,
            );
            rest
        });

        FormalParameters::boxed(params.span, FormalParameterKind::Signature, items, rest, self)
    }
}

pub fn get_function_span(func: &Function<'_>) -> Span {
    func.id.as_ref().map_or_else(|| Span::empty(func.params.span.start), |id| id.span)
}
