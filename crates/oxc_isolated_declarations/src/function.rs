use oxc_allocator::Box;
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, NONE};
use oxc_span::{Span, SPAN};

use crate::{
    diagnostics::{
        function_must_have_explicit_return_type, implicitly_adding_undefined_to_type,
        parameter_must_have_explicit_type,
    },
    formal_parameter_binding_pattern::FormalParameterBindingPattern,
    IsolatedDeclarations,
};

impl<'a> IsolatedDeclarations<'a> {
    pub(crate) fn transform_function(
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
            Some(self.ast.alloc_function(
                func.r#type,
                func.span,
                // SAFETY: `ast.copy` is unsound! We need to fix.
                unsafe { self.ast.copy(&func.id) },
                false,
                false,
                declare.unwrap_or_else(|| self.is_declare()),
                // SAFETY: `ast.copy` is unsound! We need to fix.
                unsafe { self.ast.copy(&func.type_parameters) },
                // SAFETY: `ast.copy` is unsound! We need to fix.
                unsafe { self.ast.copy(&func.this_param) },
                params,
                return_type,
                NONE,
            ))
        }
    }

    pub(crate) fn transform_formal_parameter(
        &self,
        param: &FormalParameter<'a>,
        is_remaining_params_have_required: bool,
    ) -> Option<FormalParameter<'a>> {
        let pattern = &param.pattern;
        if let BindingPatternKind::AssignmentPattern(pattern) = &pattern.kind {
            if pattern.left.kind.is_destructuring_pattern()
                && pattern.left.type_annotation.is_none()
            {
                self.error(parameter_must_have_explicit_type(param.span));
                return None;
            }
        }

        let is_assignment_pattern = pattern.kind.is_assignment_pattern();
        let mut pattern =
            if let BindingPatternKind::AssignmentPattern(pattern) = &param.pattern.kind {
                // SAFETY: `ast.copy` is unsound! We need to fix.
                unsafe { self.ast.copy(&pattern.left) }
            } else {
                // SAFETY: `ast.copy` is unsound! We need to fix.
                unsafe { self.ast.copy(&param.pattern) }
            };

        FormalParameterBindingPattern::remove_assignments_from_kind(self.ast, &mut pattern.kind);

        if is_assignment_pattern || pattern.type_annotation.is_none() {
            let type_annotation = pattern
                .type_annotation
                .as_ref()
                .map(|type_annotation| {
                    // SAFETY: `ast.copy` is unsound! We need to fix.
                    unsafe { self.ast.copy(&type_annotation.type_annotation) }
                })
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
                            // union with `undefined`
                            return self.ast.ts_type_annotation(
                                SPAN,
                                self.ast.ts_type_union_type(
                                    SPAN,
                                    self.ast.vec_from_iter([
                                        ts_type,
                                        self.ast.ts_type_undefined_keyword(SPAN),
                                    ]),
                                ),
                            );
                        }
                    }

                    self.ast.ts_type_annotation(SPAN, ts_type)
                });

            pattern = self.ast.binding_pattern(
                // SAFETY: `ast.copy` is unsound! We need to fix.
                unsafe { self.ast.copy(&pattern.kind) },
                type_annotation,
                // if it's assignment pattern, it's optional
                pattern.optional || (!is_remaining_params_have_required && is_assignment_pattern),
            );
        }

        Some(self.ast.formal_parameter(param.span, self.ast.vec(), pattern, None, false, false))
    }

    pub(crate) fn transform_formal_parameters(
        &self,
        params: &FormalParameters<'a>,
    ) -> Box<'a, FormalParameters<'a>> {
        if params.kind.is_signature() || (params.rest.is_none() && params.items.is_empty()) {
            // SAFETY: `ast.copy` is unsound! We need to fix.
            return self.ast.alloc(unsafe { self.ast.copy(params) });
        }

        let items =
            self.ast.vec_from_iter(params.items.iter().enumerate().filter_map(|(index, item)| {
                let is_remaining_params_have_required =
                    params.items.iter().skip(index).any(|item| {
                        !(item.pattern.optional || item.pattern.kind.is_assignment_pattern())
                    });
                self.transform_formal_parameter(item, is_remaining_params_have_required)
            }));

        if let Some(rest) = &params.rest {
            if rest.argument.type_annotation.is_none() {
                self.error(parameter_must_have_explicit_type(rest.span));
            }
        }

        self.ast.alloc_formal_parameters(
            params.span,
            FormalParameterKind::Signature,
            items,
            // SAFETY: `ast.copy` is unsound! We need to fix.
            unsafe { self.ast.copy(&params.rest) },
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
