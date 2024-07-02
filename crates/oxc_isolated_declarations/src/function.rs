use oxc_allocator::Box;
use oxc_ast::ast::Function;
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_span::{Span, SPAN};

use crate::{
    diagnostics::{
        function_must_have_explicit_return_type, implicitly_adding_undefined_to_type,
        parameter_must_have_explicit_type,
    },
    IsolatedDeclarations,
};

impl<'a> IsolatedDeclarations<'a> {
    pub fn transform_function(
        &mut self,
        func: &Function<'a>,
        declare: Option<bool>,
    ) -> Option<Box<'a, Function<'a>>> {
        if func.declare {
            None
        } else {
            let return_type = self.infer_function_return_type(func);
            if return_type.is_none() {
                self.error(function_must_have_explicit_return_type(get_function_span(func)));
            }
            let params = self.transform_formal_parameters(&func.params);
            Some(self.ast.function(
                func.r#type,
                func.span,
                self.ast.copy(&func.id),
                false,
                false,
                declare.unwrap_or_else(|| self.is_declare()),
                self.ast.copy(&func.this_param),
                params,
                None,
                self.ast.copy(&func.type_parameters),
                return_type,
            ))
        }
    }

    pub fn transform_formal_parameter(
        &self,
        param: &FormalParameter<'a>,
        is_remaining_params_have_required: bool,
    ) -> Option<FormalParameter<'a>> {
        let pattern = &param.pattern;
        if pattern.type_annotation.is_none() && pattern.kind.is_destructuring_pattern() {
            self.error(parameter_must_have_explicit_type(param.span));
            return None;
        }

        let is_assignment_pattern = pattern.kind.is_assignment_pattern();
        let mut pattern =
            if let BindingPatternKind::AssignmentPattern(pattern) = &param.pattern.kind {
                self.ast.copy(&pattern.left)
            } else {
                self.ast.copy(&param.pattern)
            };

        if is_assignment_pattern || pattern.type_annotation.is_none() {
            let type_annotation = pattern
                .type_annotation
                .as_ref()
                .map(|type_annotation| self.ast.copy(&type_annotation.type_annotation))
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
                    if is_remaining_params_have_required {
                        if matches!(ts_type, TSType::TSTypeReference(_)) {
                            self.error(implicitly_adding_undefined_to_type(param.span));
                        } else if !ts_type.is_maybe_undefined() {
                            // union with undefined
                            return self.ast.ts_type_annotation(
                                SPAN,
                                self.ast.ts_union_type(
                                    SPAN,
                                    self.ast.new_vec_from_iter([
                                        ts_type,
                                        self.ast.ts_undefined_keyword(SPAN),
                                    ]),
                                ),
                            );
                        }
                    }

                    self.ast.ts_type_annotation(SPAN, ts_type)
                });

            pattern = self.ast.binding_pattern(
                self.ast.copy(&pattern.kind),
                type_annotation,
                // if it's assignment pattern, it's optional
                pattern.optional || (!is_remaining_params_have_required && is_assignment_pattern),
            );
        }

        Some(self.ast.formal_parameter(
            param.span,
            pattern,
            None,
            false,
            false,
            self.ast.new_vec(),
        ))
    }

    pub fn transform_formal_parameters(
        &self,
        params: &FormalParameters<'a>,
    ) -> Box<'a, FormalParameters<'a>> {
        if params.kind.is_signature() || (params.rest.is_none() && params.items.is_empty()) {
            return self.ast.alloc(self.ast.copy(params));
        }

        let items = self.ast.new_vec_from_iter(params.items.iter().enumerate().filter_map(
            |(index, item)| {
                let is_remaining_params_have_required =
                    params.items.iter().skip(index).any(|item| {
                        !(item.pattern.optional || item.pattern.kind.is_assignment_pattern())
                    });
                self.transform_formal_parameter(item, is_remaining_params_have_required)
            },
        ));

        if let Some(rest) = &params.rest {
            if rest.argument.type_annotation.is_none() {
                self.error(parameter_must_have_explicit_type(rest.span));
            }
        }

        self.ast.formal_parameters(
            params.span,
            FormalParameterKind::Signature,
            items,
            self.ast.copy(&params.rest),
        )
    }
}

pub fn get_function_span(func: &Function<'_>) -> Span {
    func.id.as_ref().map_or_else(
        || {
            let start = func.params.span.start;
            Span::new(start, start)
        },
        |id| id.span,
    )
}
