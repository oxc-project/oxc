use oxc_allocator::{Box as ArenaBox, CloneIn};
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
        self.ast.alloc_function(
            func.span,
            func.r#type,
            func.id.clone_in(self.ast.allocator),
            false,
            false,
            declare.unwrap_or_else(|| self.is_declare()),
            func.type_parameters.clone_in(self.ast.allocator),
            func.this_param.clone_in(self.ast.allocator),
            params,
            return_type,
            NONE,
        )
    }

    pub(crate) fn transform_formal_parameter(
        &self,
        param: &FormalParameter<'a>,
        is_remaining_params_have_required: bool,
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
            pattern.left.clone_in(self.ast.allocator)
        } else {
            param.pattern.clone_in(self.ast.allocator)
        };

        FormalParameterBindingPattern::remove_assignments_from_kind(self.ast, &mut pattern);

        if param.initializer.is_some()
            || param.type_annotation.is_none()
            || (param.optional && param.has_modifier())
        {
            let type_annotation = param
                .type_annotation
                .as_ref()
                .map(|type_annotation| type_annotation.type_annotation.clone_in(self.ast.allocator))
                .or_else(|| {
                    // report error for has no type annotation
                    let new_type = self.infer_type_from_formal_parameter(param);
                    if new_type.is_none() {
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
                            return self.ast.ts_type_annotation(
                                SPAN,
                                self.ast.ts_type_union_type(
                                    SPAN,
                                    self.ast.vec_from_array([
                                        ts_type,
                                        self.ast.ts_type_undefined_keyword(SPAN),
                                    ]),
                                ),
                            );
                        }
                    }

                    self.ast.ts_type_annotation(SPAN, ts_type)
                });

            let optional =
                param.optional || (!is_remaining_params_have_required && is_assignment_pattern);
            return Some(self.ast.formal_parameter(
                param.span,
                self.ast.vec(),
                pattern.clone_in(self.ast.allocator),
                type_annotation,
                NONE,
                optional,
                None,
                false,
                false,
            ));
        }

        Some(self.ast.formal_parameter(
            param.span,
            self.ast.vec(),
            pattern,
            param.type_annotation.clone_in(self.ast.allocator),
            NONE,
            param.optional,
            None,
            false,
            false,
        ))
    }

    pub(crate) fn transform_formal_parameters(
        &self,
        params: &FormalParameters<'a>,
        skip_no_accessibility_param: bool,
    ) -> ArenaBox<'a, FormalParameters<'a>> {
        if params.kind.is_signature() || (params.rest.is_none() && params.items.is_empty()) {
            return self.ast.alloc(params.clone_in(self.ast.allocator));
        }

        let items = self.ast.vec_from_iter(
            params
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| !skip_no_accessibility_param || item.has_modifier())
                .filter_map(|(index, item)| {
                    let is_remaining_params_have_required = params
                        .items
                        .iter()
                        .skip(index)
                        .any(|item| !(item.optional || item.initializer.is_some()));
                    self.transform_formal_parameter(item, is_remaining_params_have_required)
                }),
        );

        if let Some(rest) = &params.rest
            && rest.type_annotation.is_none()
        {
            self.error(parameter_must_have_explicit_type(rest.span));
        }

        self.ast.alloc_formal_parameters(
            params.span,
            FormalParameterKind::Signature,
            items,
            params.rest.clone_in(self.ast.allocator),
        )
    }
}

pub fn get_function_span(func: &Function<'_>) -> Span {
    func.id.as_ref().map_or_else(|| Span::empty(func.params.span.start), |id| id.span)
}
